use serde::{Serialize, Deserialize};
use sqlx::FromRow;
use bigdecimal::BigDecimal;

#[derive(Serialize, Deserialize, FromRow)]
pub struct Product {
    pub id: i32,
    pub name: String,
    pub description: Option<String>, // Because can be null
    pub price: BigDecimal, // Numeric -> f64
    pub stock: i32,
    pub image_url: Option<String>,
    pub category_id: Option<i32>,
}

#[derive(Serialize, Deserialize, FromRow)]
pub struct DeleteImage {
    pub image_url: Option<String>,
}