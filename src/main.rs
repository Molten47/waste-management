mod cli;
mod data;
mod db;
mod errors;
mod logic;
mod models;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let pool = match db::connect().await {
        Ok(pool) => pool,
        Err(error) => {
            eprintln!("Could not connect to the database: {error}");
            return;
        }
    };

    println!("Connected. Migrations applied.");

    if let Err(error) = cli::run(&pool).await {
        eprintln!("Fatal error: {error}");
    }
}
