mod api;
mod auth;
mod cli;
mod data;
mod db;
mod errors;
mod logic;
mod models;
mod password;

use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    auth::jwt_secret(); // fail fast if JWT_SECRET is missing or too short

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,tower_http=debug".into()),
        )
        .init();

    let pool = match db::connect().await {
        Ok(pool) => pool,
        Err(error) => {
            eprintln!("Could not connect to the database: {error}");
            return;
        }
    };
    if let Err(error) = auth::seed_owner(&pool).await {
        eprintln!("Could not seed owner: {error}");
        return;
    }

    let app = api::router(pool).layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::info!("listening on http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}
