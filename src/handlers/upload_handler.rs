use axum::{
    extract::{Multipart, State},
    Json,
};
use serde_json::json;
use tokio::{fs, io::AsyncWriteExt};
use uuid::Uuid;

use crate::{errors::AppError, models::user::StandardResponse, state::AppState};

pub async fn upload_product_image(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<StandardResponse<serde_json::Value>>, AppError> {
    let upload_base = &state.config.upload_dir;
    let products_dir = format!("{}/products", upload_base);

    // Ensure dir exists
    fs::create_dir_all(&products_dir).await.map_err(|e| {
        AppError::Internal(format!("Gagal membuat direktori upload: {}", e))
    })?;

    let mut file_url = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("Gagal membaca multipart upload: {}", e)))?
    {
        let file_name = field
            .file_name()
            .unwrap_or("image.jpg")
            .to_string();

        let ext = file_name
            .rsplit('.')
            .next()
            .unwrap_or("jpg")
            .to_lowercase();

        let new_file_name = format!("{}.{}", Uuid::new_v4(), ext);
        let save_path = format!("{}/{}", products_dir, new_file_name);

        let data = field
            .bytes()
            .await
            .map_err(|e| AppError::BadRequest(format!("Gagal membaca data file: {}", e)))?;

        let mut file = fs::File::create(&save_path)
            .await
            .map_err(|e| AppError::Internal(format!("Gagal menyimpan file: {}", e)))?;

        file.write_all(&data)
            .await
            .map_err(|e| AppError::Internal(format!("Gagal menulis data file: {}", e)))?;

        file_url = Some(format!("/uploads/products/{}", new_file_name));
        break;
    }

    let url = file_url.ok_or_else(|| AppError::BadRequest("File tidak ditemukan dalam form upload".to_string()))?;

    Ok(Json(StandardResponse::success(
        json!({ "url": url }),
        Some("File berhasil diupload"),
    )))
}
