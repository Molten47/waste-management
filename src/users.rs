use argon2::password_hash::rand_core::{OsRng, RngCore};
use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::auth::{AuthUser, Role};
use crate::errors::TruckError;
use crate::password::{hash_password, verify_password};

const OTP_HOURS: u64 = 24;
const MAX_OTP_ATTEMPTS: i16 = 5;
// 32 symbols (divides 2^32 evenly, so no modulo bias); no 0/O/1/I to avoid misreading.
const OTP_ALPHABET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";

pub fn generate_otp() -> String {
    (0..8)
        .map(|_| OTP_ALPHABET[(OsRng.next_u32() % 32) as usize] as char)
        .collect()
}

/// Owner creates anyone except another owner; admins create everyone except admins and owners.
pub fn can_create(caller: Role, target: Role) -> bool {
    match (caller, target) {
        (_, Role::Owner) => false,
        (Role::Owner, _) => true,
        (Role::Admin, Role::Admin) => false,
        (Role::Admin, _) => true,
        _ => false,
    }
}

#[derive(Deserialize)]
pub struct CreateUser {
    full_name: String,
    email: String,
    phone: Option<String>,
    role: String,
}

#[derive(Serialize)]
pub struct CreateUserResponse {
    id: String,
    email: String,
    role: String,
    otp: String,
    otp_valid_hours: u64,
}

pub async fn create_user(
    caller: AuthUser,
    State(pool): State<PgPool>,
    Json(body): Json<CreateUser>,
) -> Result<Json<CreateUserResponse>, TruckError> {
    let target =
        Role::parse(&body.role).ok_or_else(|| TruckError::InvalidInput("unknown role".into()))?;
    if !can_create(caller.role, target) {
        return Err(TruckError::Forbidden);
    }

    let email = body.email.trim().to_lowercase();
    if !email.contains('@') || body.full_name.trim().is_empty() {
        return Err(TruckError::InvalidInput(
            "name and a valid email are required".into(),
        ));
    }

    let otp = generate_otp();
    let otp_for_hash = otp.clone();
    let otp_hash = tokio::task::spawn_blocking(move || hash_password(&otp_for_hash))
        .await
        .map_err(|e| TruckError::Internal(e.to_string()))?
        .map_err(|e| TruckError::Internal(e.to_string()))?;

    let id: String = sqlx::query_scalar(
        "INSERT INTO users (full_name, email, phone, role, otp_hash, otp_expires_at)
         VALUES ($1, $2, $3, $4::text::user_role, $5, now() + make_interval(hours => $6))
         RETURNING id::text",
    )
    .bind(body.full_name.trim())
    .bind(&email)
    .bind(&body.phone)
    .bind(&body.role)
    .bind(&otp_hash)
    .bind(OTP_HOURS as i32)
    .fetch_one(&pool)
    .await
    .map_err(|e| match &e {
        sqlx::Error::Database(db) if db.is_unique_violation() => {
            TruckError::Conflict("a user with that email already exists".into())
        }
        _ => TruckError::from(e),
    })?;

    Ok(Json(CreateUserResponse {
        id,
        email,
        role: body.role,
        otp,
        otp_valid_hours: OTP_HOURS,
    }))
}

#[derive(Deserialize)]
pub struct ActivateRequest {
    email: String,
    otp: String,
    new_password: String,
}

#[derive(sqlx::FromRow)]
struct PendingUser {
    id: String,
    otp_hash: String,
    otp_valid: bool,
    otp_attempts: i16,
}

pub async fn activate(
    State(pool): State<PgPool>,
    Json(body): Json<ActivateRequest>,
) -> Result<&'static str, TruckError> {
    if body.new_password.len() < 12 {
        return Err(TruckError::InvalidInput(
            "password must be at least 12 characters".into(),
        ));
    }

    let pending: Option<PendingUser> = sqlx::query_as(
        "SELECT id::text AS id, otp_hash, (otp_expires_at > now()) AS otp_valid, otp_attempts
         FROM users
         WHERE LOWER(email) = LOWER($1) AND is_active AND otp_hash IS NOT NULL",
    )
    .bind(body.email.trim())
    .fetch_optional(&pool)
    .await?;

    // One generic error for every failure, so nothing reveals which emails exist.
    let Some(user) = pending else {
        return Err(TruckError::Unauthorized);
    };
    if !user.otp_valid || user.otp_attempts >= MAX_OTP_ATTEMPTS {
        return Err(TruckError::Unauthorized);
    }

    let otp = body.otp.trim().to_uppercase();
    let stored = user.otp_hash.clone();
    let ok = tokio::task::spawn_blocking(move || verify_password(&otp, &stored))
        .await
        .map_err(|e| TruckError::Internal(e.to_string()))?;

    if !ok {
        sqlx::query("UPDATE users SET otp_attempts = otp_attempts + 1 WHERE id::text = $1")
            .bind(&user.id)
            .execute(&pool)
            .await?;
        return Err(TruckError::Unauthorized);
    }

    let new_password = body.new_password;
    let hash = tokio::task::spawn_blocking(move || hash_password(&new_password))
        .await
        .map_err(|e| TruckError::Internal(e.to_string()))?
        .map_err(|e| TruckError::Internal(e.to_string()))?;

    sqlx::query(
        "UPDATE users
         SET password_hash = $1, otp_hash = NULL, otp_expires_at = NULL, otp_attempts = 0
         WHERE id::text = $2",
    )
    .bind(&hash)
    .bind(&user.id)
    .execute(&pool)
    .await?;

    Ok("account activated")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn otp_has_expected_shape() {
        let otp = generate_otp();
        assert_eq!(otp.len(), 8);
        assert!(otp.bytes().all(|b| OTP_ALPHABET.contains(&b)));
    }

    #[test]
    fn otps_differ() {
        assert_ne!(generate_otp(), generate_otp());
    }

    #[test]
    fn owner_can_create_admin_but_not_owner() {
        assert!(can_create(Role::Owner, Role::Admin));
        assert!(!can_create(Role::Owner, Role::Owner));
    }

    #[test]
    fn admin_creates_staff_but_not_admins() {
        assert!(can_create(Role::Admin, Role::Supervisor));
        assert!(can_create(Role::Admin, Role::Driver));
        assert!(!can_create(Role::Admin, Role::Admin));
        assert!(!can_create(Role::Admin, Role::Owner));
    }

    #[test]
    fn other_roles_cannot_create_anyone() {
        assert!(!can_create(Role::Supervisor, Role::Driver));
        assert!(!can_create(Role::Hr, Role::Driver));
        assert!(!can_create(Role::Driver, Role::Driver));
    }
}
