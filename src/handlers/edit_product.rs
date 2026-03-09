use crate::models::{DeleteImage, Product};
use axum::{
    Json,
    extract::{Multipart, State},
    http::StatusCode,
};
use bigdecimal::BigDecimal;
use reqwest;
use sqlx::PgPool;
use std::str::FromStr;

pub async fn edit_product(
    State(pool): State<PgPool>,
    mut multipart: Multipart,
) -> Result<Json<Product>, (StatusCode, String)> {
    // prepare variable
    let mut id: i32 = 0;
    let mut name = String::new();
    let mut description: Option<String> = None; // Pakai Option supaya kalau kosong jadi NULL di DB
    let mut price = BigDecimal::from(0);
    let mut stock: i32 = 0;
    let mut image_url = String::new();
    let mut category_id: Option<i32> = None; // Pakai Option karena di DB bisa kosong (null)

    // supabase
    let supabase_url = std::env::var("SUPABASE_URL").expect("SUPABASE_URL Not Found!");
    let supabase_key = std::env::var("SUPABASE_KEY").expect("SUPABASE_KEY Not Found!");

    // open the package
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?
    {
        let name_label = field.name().unwrap_or("").to_string();

        match name_label.as_str() {
            "id" => {
                let i = field
                    .text()
                    .await
                    .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
                id = i.parse().unwrap_or(0);
            }
            "name" => {
                name = field
                    .text()
                    .await
                    .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
            }
            "description" => {
                let d_text = field.text().await.unwrap_or_default();
                description = Some(d_text); // Kita masukkan teksnya ke dalam kotak Option
            }
            "price" => {
                let p = field
                    .text()
                    .await
                    .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
                price = BigDecimal::from_str(&p).unwrap_or_default();
            }
            "stock" => {
                let s = field
                    .text()
                    .await
                    .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
                stock = s.parse().unwrap_or(0);
            }
            "image_url" => {
                let data = sqlx::query_as::<_, DeleteImage>(
                    "SELECT image_url FROM products_store WHERE id = $1",
                )
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
                        let storage_url =
                            format!("{}/storage/v1/object/products/{}", supabase_url, file_name);

                        println!("Mandor: Sedang membuang foto lama: {}", file_name);

                        let res = client
                            .request(reqwest::Method::DELETE, &storage_url)
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

                let file_name = field.file_name().unwrap_or("image.jpeg").to_string();

                // create name for file photo
                let content_type = match file_name.split(".").last() {
                    Some("png") => "image/png",
                    Some("git") => "image/gif",
                    Some("webp") => "image/webp",
                    Some("svg") => "image/svg+xml",
                    _ => "image/jpeg",
                };
                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                let name_file = format!("prod_{}_{}", timestamp, file_name.replace(" ", "_"));

                // url format
                let upload_url =
                    format!("{}/storage/v1/object/products/{}", supabase_url, name_file);
                image_url = format!(
                    "{}/storage/v1/object/public/products/{}",
                    supabase_url, name_file
                );

                let data_photo = field.bytes().await.unwrap().to_vec();

                // send data
                let client = reqwest::Client::new();
                client
                    .post(&upload_url)
                    .header("Authorization", format!("Bearer {}", supabase_key))
                    .header("Content-Type", content_type)
                    .body(data_photo)
                    .send()
                    .await
                    .map_err(|e| {
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            format!("Failed send data: {}", e),
                        )
                    })?;
            }
            "category_id" => {
                let c_text = field.text().await.unwrap_or_default();
                category_id = c_text.parse().ok(); // .ok() akan mengubahnya jadi Some(angka) atau None jika gagal
            }
            _ => {}
        }
    }

    // save into database
    let new_product = sqlx::query_as::<_, Product>(
        "UPDATE products_store
        SET name = $1, description = $2, price = $3, stock = $4, image_url = $5, category_id = $6
        WHERE id = $7
        RETURNING *",
    )
    .bind(name) // $1
    .bind(description) // $2
    .bind(price) // $3
    .bind(stock) // $4
    .bind(image_url) // $5
    .bind(category_id) // $6
    .bind(id)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database mogok: {}", e),
        )
    })?;

    Ok(Json(new_product))
}
