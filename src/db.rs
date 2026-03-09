use sqlx::postgres::{PgPool, PgPoolOptions};
use std::env;

// func for connection
pub async fn connect_db() -> PgPool {
    // take address database
    let database_url = env::var("DATABASE_URL")
        .expect("Variable DATABASE_URL is Not Found!");

    // create cashier 
    PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to Connection to database")
}