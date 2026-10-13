mod api;
mod assignment;
mod auth;
mod cli;
mod data;
mod db;
mod errors;
mod logic;
mod models;
mod password;
mod truck;
mod users;

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
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".into());
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .unwrap();
    tracing::info!("listening on port {port}");
    axum::serve(listener, app).await.unwrap();
}
