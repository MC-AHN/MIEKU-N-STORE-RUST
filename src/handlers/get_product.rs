use axum::{extract::State, Json};
use sqlx::PgPool;
use crate::models::Product;

// func for get all product
pub async fn get_all_products (
    State(pool): State<PgPool>, // Casier 
) -> Json<Vec<Product>> {
    // casier take a data from database
    let products = sqlx::query_as::<_, Product>("SELECT * FROM products_store")
        .fetch_all(&pool)
        .await
        .expect("Failed to get products");

    Json(products)
}