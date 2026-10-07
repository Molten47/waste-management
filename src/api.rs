use axum::{
    Json, Router,
    extract::{Query, State},
    routing::get,
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
        .with_state(pool)
}
