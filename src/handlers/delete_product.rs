use crate::models::{DeleteImage};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use reqwest;
use sqlx::PgPool;

pub async fn delete_product(Path(id): Path<i32>, State(pool): State<PgPool>) -> impl IntoResponse {
    
    let supabase_url = std::env::var("SUPABASE_URL").expect("SUPABASE_URL Not Found!");
    let supabase_key = std::env::var("SUPABASE_KEY").expect("SUPABASE_KEY Not Found!");

    // hapus gambarnya
    let data = sqlx::query_as::<_, DeleteImage>("SELECT image_url FROM products_store WHERE id = $1")
        .bind(id)
        .fetch_one(&pool)
        .await
        .expect("Failed to get products");

    // 2. Proses penghapusan di Supabase Storage
    if let Some(old_url) = data.image_url {
        // Kita ambil nama filenya saja (misal: "kopi.jpg")
        if let Some(file_name) = old_url.split('/').last() {
            let client = reqwest::Client::new();
            // Alamat storage (Ganti 'uploads' sesuai nama bucket-mu)
            let storage_url = format!("{}/storage/v1/object/products/{}", supabase_url, file_name);

            println!("Mandor: Sedang membuang foto lama: {}", file_name);

            let res = client.request(reqwest::Method::DELETE, &storage_url)
                .header("Authorization", format!("Bearer {}", supabase_key))
                .header("apiKey", &supabase_key)
                .header("Content-Length", "0")
                .body("")
                .send()
                .await;

            // Cek apakah hapusnya sukses atau tidak
            match res {
                Ok(resp) if resp.status().is_success() => {
                    println!("Mandor: Foto lama sukses dihapus!")
                }
                Ok(resp) => {
                    println!("Mandor: Supabase nolak hapus (Status: {})", resp.status())
                }
                Err(e) => println!("Mandor: Gagal kirim perintah hapus: {}", e),
            }
        }
    }

    // hapus yang berkaitan
    let delete = sqlx::query("DELETE FROM order_items_store WHERE product_id = $1")
        .bind(id)
        .execute(&pool)
        .await;

    match delete {
        Err(e) => {
            println!("Error While Delete: {}", e);
        }
        _ => {}
    }

    let result = sqlx::query("DELETE FROM products_store WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await;

    match result {
        Ok(_) => StatusCode::NO_CONTENT,
        Err(e) => {
            println!("Error While Delete: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
