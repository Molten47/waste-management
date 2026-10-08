use crate::auth::{AuthUser, Role};
use axum::{
    Json, Router,
    extract::{Query, State},
    routing::{get, post},
};

use serde::Deserialize;
use sqlx::PgPool;

use crate::data;
use crate::errors::TruckError;
use crate::models::RouteInfo;

#[derive(Deserialize)]
pub struct LookupParams {
    street: String,
    house: i32,
}

async fn health() -> &'static str {
    "ok:200"
}

async fn lookup(
    State(pool): State<PgPool>,
    Query(params): Query<LookupParams>,
) -> Result<Json<RouteInfo>, TruckError> {
    match data::find_route(&pool, &params.street, params.house).await? {
        Some(route) => Ok(Json(route)),
        None => Err(TruckError::NotFound),
    }
}

pub fn router(pool: PgPool) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/lookup", get(lookup))
        .route("/login", post(crate::auth::login))
        .route("/me", get(me))
        .route("/admin/ping", get(admin_ping))
        .route("/users", post(crate::users::create_user))
        .route("/activate", post(crate::users::activate))
        .with_state(pool)
}

#[derive(serde::Serialize)]
struct MeResponse {
    id: String,
    role: Role,
}

async fn me(user: AuthUser) -> Json<MeResponse> {
    Json(MeResponse {
        id: user.id,
        role: user.role,
    })
}

async fn admin_ping(user: AuthUser) -> Result<&'static str, TruckError> {
    user.require_any(&[Role::Owner, Role::Admin])?;
    Ok("admin ok")
}
