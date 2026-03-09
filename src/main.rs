mod db;
mod models;
mod handlers;

use handlers::{get_product::get_all_products, upload_product::create_product, edit_product::edit_product, delete_product::delete_product};

use axum::{
    Router,
    routing::{delete, get},
};
use std::{ net::SocketAddr };
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
        .route(
            "/products", get(get_all_products).post(create_product).put(edit_product)
        )
        .route("/products/{id}", delete(delete_product))
        .with_state(pool);

    // port
    let addr = SocketAddr::from(([127, 0, 0, 1], 8002));

    // open the door    
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("Server running on http://{}", addr);

    // run server
    axum::serve(listener, app)
        .await
        .unwrap();
}

