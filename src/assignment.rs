use axum::{
    Json,
    extract::{Path, State},
};
use serde::Deserialize;
use sqlx::PgPool;

use crate::auth::{AuthUser, Role};
use crate::errors::TruckError;
use crate::truck::{TruckSummary, scope_for};

#[derive(Deserialize)]
pub struct AssignBody {
    email: String,
}

#[derive(sqlx::FromRow)]
struct Candidate {
    id: String,
    role: String,
}

fn check_role(actual: &str, wanted: Role) -> Result<(), TruckError> {
    if Role::parse(actual) == Some(wanted) {
        Ok(())
    } else {
        Err(TruckError::WrongRole)
    }
}

/// Finds an active user by email and checks they hold the wanted role.
/// Returns the user's id as text.
async fn find_active_user(pool: &PgPool, email: &str, wanted: Role) -> Result<String, TruckError> {
    let row: Option<Candidate> = sqlx::query_as(
        "SELECT id::text AS id, role::text AS role
         FROM users
         WHERE LOWER(email) = LOWER($1) AND is_active",
    )
    .bind(email)
    .fetch_optional(pool)
    .await?;

    let row = row.ok_or(TruckError::NoSuchUser)?;
    check_role(&row.role, wanted)?;
    Ok(row.id)
}

const SET_DRIVER_SQL: &str = "WITH t AS (
    UPDATE trucks SET driver_id = $1::uuid
    WHERE fleet_code = $2 AND ($3::text IS NULL OR supervisor_id::text = $3)
    RETURNING *
)
SELECT t.fleet_code, d.full_name AS driver, s.full_name AS supervisor,
       t.shift_start, t.shift_end
FROM t
LEFT JOIN users d ON d.id = t.driver_id
LEFT JOIN users s ON s.id = t.supervisor_id";

const SET_SUPERVISOR_SQL: &str = "WITH t AS (
    UPDATE trucks SET supervisor_id = $1::uuid
    WHERE fleet_code = $2 AND ($3::text IS NULL OR supervisor_id::text = $3)
    RETURNING *
)
SELECT t.fleet_code, d.full_name AS driver, s.full_name AS supervisor,
       t.shift_start, t.shift_end
FROM t
LEFT JOIN users d ON d.id = t.driver_id
LEFT JOIN users s ON s.id = t.supervisor_id";

/// Runs one of the two assignment queries above. `scope` is the same supervisor
/// scoping used in `update_shift`: a supervisor touching a truck they don't
/// supervise updates zero rows and gets a 404.
async fn set_assignment(
    pool: &PgPool,
    sql: &'static str,
    new_id: Option<String>,
    fleet_code: &str,
    scope: Option<String>,
) -> Result<Json<TruckSummary>, TruckError> {
    let updated: Option<TruckSummary> = sqlx::query_as(sql)
        .bind(new_id)
        .bind(fleet_code)
        .bind(scope)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(db) = &e {
                if db.is_unique_violation() {
                    return TruckError::DriverAlreadyAssigned;
                }
            }
            TruckError::from(e)
        })?;

    updated.map(Json).ok_or(TruckError::NoSuchTruck)
}

pub async fn assign_supervisor(
    user: AuthUser,
    State(pool): State<PgPool>,
    Path(fleet_code): Path<String>,
    Json(body): Json<AssignBody>,
) -> Result<Json<TruckSummary>, TruckError> {
    user.require_any(&[Role::Owner, Role::Admin])?;
    let id = find_active_user(&pool, &body.email, Role::Supervisor).await?;
    set_assignment(
        &pool,
        SET_SUPERVISOR_SQL,
        Some(id),
        &fleet_code,
        scope_for(&user),
    )
    .await
}

pub async fn assign_driver(
    user: AuthUser,
    State(pool): State<PgPool>,
    Path(fleet_code): Path<String>,
    Json(body): Json<AssignBody>,
) -> Result<Json<TruckSummary>, TruckError> {
    user.require_any(&[Role::Owner, Role::Admin, Role::Supervisor])?;
    let id = find_active_user(&pool, &body.email, Role::Driver).await?;
    set_assignment(
        &pool,
        SET_DRIVER_SQL,
        Some(id),
        &fleet_code,
        scope_for(&user),
    )
    .await
}

pub async fn unassign_driver(
    user: AuthUser,
    State(pool): State<PgPool>,
    Path(fleet_code): Path<String>,
) -> Result<Json<TruckSummary>, TruckError> {
    user.require_any(&[Role::Owner, Role::Admin, Role::Supervisor])?;
    set_assignment(&pool, SET_DRIVER_SQL, None, &fleet_code, scope_for(&user)).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matching_role_passes() {
        assert!(check_role("driver", Role::Driver).is_ok());
        assert!(check_role("supervisor", Role::Supervisor).is_ok());
    }

    #[test]
    fn different_role_is_rejected() {
        assert!(matches!(
            check_role("supervisor", Role::Driver),
            Err(TruckError::WrongRole)
        ));
    }

    #[test]
    fn unknown_role_string_is_rejected() {
        assert!(matches!(
            check_role("superuser", Role::Driver),
            Err(TruckError::WrongRole)
        ));
    }
}
