mod db;
mod handlers;
mod models;

use handlers::{
    delete_product::delete_product, edit_product::edit_product, get_product::get_all_products,
    upload_product::create_product,
};

use axum::{
    Router,
    routing::{delete, get},
};
use std::net::SocketAddr;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    // read env
    dotenvy::dotenv().ok();

    // run connection
    let pool = db::connect_db().await;

    println!("✅ Connection succefully.");

    // Router
    let app = Router::new()
        .route("/", get(health_check))
        .route(
            "/products",
            get(get_all_products).post(create_product).put(edit_product),
        )
        .route("/products/{id}", delete(delete_product))
        .with_state(pool);

    // port
    // Baca port dari environment (buat Render nanti)
    let port = std::env::var("PORT").unwrap_or_else(|_| "8002".to_string());
    let addr = format!("0.0.0.0:{}", port);

    println!("🚀 Mandor standby di http://{}", addr);

    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();

}

// Ganti bagian bawah main.rs kamu jadi begini:
async fn health_check() -> &'static str {
    "Mandor Siap Kerja!"
}
