use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::{
    Json,
    extract::{FromRequestParts, State},
    http::{header, request::Parts},
};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::errors::TruckError;
use crate::password::{hash_password, verify_password};

type BoxError = Box<dyn std::error::Error + Send + Sync>;

/// Creates the first owner if none exists. Safe to run on every startup.
pub async fn seed_owner(pool: &PgPool) -> Result<(), BoxError> {
    let (Ok(email), Ok(password)) = (
        std::env::var("OWNER_EMAIL"),
        std::env::var("OWNER_PASSWORD"),
    ) else {
        tracing::warn!("OWNER_EMAIL / OWNER_PASSWORD not set, skipping owner seed");
        return Ok(());
    };

    let owner_exists: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM users WHERE role = 'owner')")
            .fetch_one(pool)
            .await?;

    if owner_exists {
        return Ok(());
    }

    if password.len() < 12 {
        return Err("OWNER_PASSWORD must be at least 12 characters".into());
    }

    let hash = tokio::task::spawn_blocking(move || hash_password(&password))
        .await?
        .map_err(|e| format!("could not hash password: {e}"))?;

    sqlx::query(
        "INSERT INTO users (full_name, email, role, password_hash)
         VALUES ('Owner', $1, 'owner', $2)",
    )
    .bind(&email)
    .bind(&hash)
    .execute(pool)
    .await?;

    tracing::info!("created initial owner account for {email}");
    Ok(())
}

const TOKEN_LIFETIME_SECS: u64 = 8 * 60 * 60;

/// Read once from the environment. Call it at startup so a bad config fails fast.
pub fn jwt_secret() -> &'static str {
    static SECRET: OnceLock<String> = OnceLock::new();
    SECRET.get_or_init(|| {
        let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
        assert!(
            secret.len() >= 32,
            "JWT_SECRET must be at least 32 characters"
        );
        secret
    })
}

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user id
    pub role: String,
    pub exp: u64, // expiry, unix seconds
}

#[derive(Deserialize)]
pub struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    token: String,
    role: String,
}

#[derive(sqlx::FromRow)]
struct LoginRow {
    id: String,
    role: String,
    password_hash: Option<String>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Owner,
    Admin,
    Hr,
    Finance,
    Supervisor,
    Driver,
}

impl Role {
    fn parse(s: &str) -> Option<Role> {
        match s {
            "owner" => Some(Role::Owner),
            "admin" => Some(Role::Admin),
            "hr" => Some(Role::Hr),
            "finance" => Some(Role::Finance),
            "supervisor" => Some(Role::Supervisor),
            "driver" => Some(Role::Driver),
            _ => None,
        }
    }
}

/// Add this as a handler argument and the route requires a valid token.
pub struct AuthUser {
    pub id: String,
    pub role: Role,
}

impl AuthUser {
    pub fn require_any(&self, allowed: &[Role]) -> Result<(), TruckError> {
        if allowed.contains(&self.role) {
            Ok(())
        } else {
            Err(TruckError::Forbidden)
        }
    }
}

impl<S: Send + Sync> FromRequestParts<S> for AuthUser {
    type Rejection = TruckError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, TruckError> {
        let header = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or(TruckError::Unauthorized)?;
        let token = header
            .strip_prefix("Bearer ")
            .ok_or(TruckError::Unauthorized)?;

        // Validation::default() is HS256 and checks the `exp` claim for us.
        let data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(jwt_secret().as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| TruckError::Unauthorized)?;

        let role = Role::parse(&data.claims.role).ok_or(TruckError::Unauthorized)?;
        Ok(AuthUser {
            id: data.claims.sub,
            role,
        })
    }
}

pub async fn login(
    State(pool): State<sqlx::PgPool>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, TruckError> {
    let row: Option<LoginRow> = sqlx::query_as(
        "SELECT id::text AS id, role::text AS role, password_hash
         FROM users
         WHERE LOWER(email) = LOWER($1) AND is_active",
    )
    .bind(&body.email)
    .fetch_optional(&pool)
    .await?;

    // Same error for "no such user", "no password set" and "wrong password",
    // so the API never reveals which emails exist.
    let Some(LoginRow {
        id,
        role,
        password_hash: Some(hash),
    }) = row
    else {
        return Err(TruckError::Unauthorized);
    };

    let password = body.password;
    let ok = tokio::task::spawn_blocking(move || verify_password(&password, &hash))
        .await
        .map_err(|e| TruckError::Internal(e.to_string()))?;
    if !ok {
        return Err(TruckError::Unauthorized);
    }

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| TruckError::Internal(e.to_string()))?
        .as_secs();

    let claims = Claims {
        sub: id,
        role: role.clone(),
        exp: now + TOKEN_LIFETIME_SECS,
    };
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret().as_bytes()),
    )
    .map_err(|e| TruckError::Internal(e.to_string()))?;

    Ok(Json(LoginResponse { token, role }))
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_known_roles() {
        assert_eq!(Role::parse("owner"), Some(Role::Owner));
        assert_eq!(Role::parse("supervisor"), Some(Role::Supervisor));
    }

    #[test]
    fn unknown_role_is_rejected() {
        assert_eq!(Role::parse("superuser"), None);
    }

    #[test]
    fn require_any_checks_membership() {
        let user = AuthUser {
            id: "1".into(),
            role: Role::Supervisor,
        };
        assert!(user.require_any(&[Role::Owner, Role::Supervisor]).is_ok());
        assert!(matches!(
            user.require_any(&[Role::Owner, Role::Admin]),
            Err(TruckError::Forbidden)
        ));
    }
}
