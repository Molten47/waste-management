use axum::{
    Json,
    extract::{Path, State},
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::auth::{AuthUser, Role};
use crate::errors::TruckError;

#[derive(Serialize, sqlx::FromRow)]
pub struct TruckSummary {
    fleet_code: String,
    driver: Option<String>,
    supervisor: Option<String>,
    shift_start: i16,
    shift_end: i16,
}

/// None = may see every truck; Some(id) = only trucks supervised by that user.
fn scope_for(user: &AuthUser) -> Option<String> {
    if user.role == Role::Supervisor {
        Some(user.id.clone())
    } else {
        None
    }
}

fn validate_shift(start: i16, end: i16) -> Result<(), TruckError> {
    if !(0..=23).contains(&start) || !(0..=23).contains(&end) {
        return Err(TruckError::HourOutOfRange);
    }
    if start == end {
        return Err(TruckError::InvalidShift);
    }
    Ok(())
}

pub async fn list_trucks(
    user: AuthUser,
    State(pool): State<PgPool>,
) -> Result<Json<Vec<TruckSummary>>, TruckError> {
    user.require_any(&[Role::Owner, Role::Admin, Role::Supervisor])?;

    let rows: Vec<TruckSummary> = sqlx::query_as(
        "SELECT t.fleet_code, d.full_name AS driver, s.full_name AS supervisor,
                t.shift_start, t.shift_end
         FROM trucks t
         LEFT JOIN users d ON d.id = t.driver_id
         LEFT JOIN users s ON s.id = t.supervisor_id
         WHERE ($1::text IS NULL OR t.supervisor_id::text = $1)
         ORDER BY t.fleet_code",
    )
    .bind(scope_for(&user))
    .fetch_all(&pool)
    .await?;

    Ok(Json(rows))
}

#[derive(Deserialize)]
pub struct UpdateShift {
    shift_start: i16,
    shift_end: i16,
}

pub async fn update_shift(
    user: AuthUser,
    State(pool): State<PgPool>,
    Path(fleet_code): Path<String>,
    Json(body): Json<UpdateShift>,
) -> Result<Json<TruckSummary>, TruckError> {
    user.require_any(&[Role::Owner, Role::Admin, Role::Supervisor])?;
    validate_shift(body.shift_start, body.shift_end)?;

    let updated: Option<TruckSummary> = sqlx::query_as(
        "WITH t AS (
             UPDATE trucks SET shift_start = $1, shift_end = $2
             WHERE fleet_code = $3 AND ($4::text IS NULL OR supervisor_id::text = $4)
             RETURNING *
         )
         SELECT t.fleet_code, d.full_name AS driver, s.full_name AS supervisor,
                t.shift_start, t.shift_end
         FROM t
         LEFT JOIN users d ON d.id = t.driver_id
         LEFT JOIN users s ON s.id = t.supervisor_id",
    )
    .bind(body.shift_start)
    .bind(body.shift_end)
    .bind(&fleet_code)
    .bind(scope_for(&user))
    .fetch_optional(&pool)
    .await?;

    updated.map(Json).ok_or(TruckError::NoSuchTruck)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn user(role: Role) -> AuthUser {
        AuthUser {
            id: "u-1".into(),
            role,
        }
    }

    #[test]
    fn owner_and_admin_are_unscoped() {
        assert_eq!(scope_for(&user(Role::Owner)), None);
        assert_eq!(scope_for(&user(Role::Admin)), None);
    }

    #[test]
    fn supervisor_is_scoped_to_own_id() {
        assert_eq!(scope_for(&user(Role::Supervisor)), Some("u-1".to_string()));
    }

    #[test]
    fn valid_shift_passes() {
        assert!(validate_shift(6, 14).is_ok());
        assert!(validate_shift(22, 6).is_ok());
    }

    #[test]
    fn shift_hours_must_be_0_to_23() {
        assert!(matches!(
            validate_shift(6, 24),
            Err(TruckError::HourOutOfRange)
        ));
        assert!(matches!(
            validate_shift(-1, 6),
            Err(TruckError::HourOutOfRange)
        ));
    }

    #[test]
    fn shift_start_and_end_must_differ() {
        assert!(matches!(
            validate_shift(8, 8),
            Err(TruckError::InvalidShift)
        ));
    }
}
