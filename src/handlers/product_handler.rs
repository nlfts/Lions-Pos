use axum::{
    extract::{Path, State},
    Json,
};
use chrono::Utc;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    errors::AppError,
    models::{
        catalog::{
            CategorySummary, CreateProductPhotoRequest, CreateProductRequest, ProductPhoto,
            ProductRaw, ProductResponse, UpdateProductPhotoRequest, UpdateProductRequest,
        },
        user::StandardResponse,
    },
    state::AppState,
};

fn map_to_response(raw: ProductRaw) -> ProductResponse {
    let category = match (raw.category_id, raw.category_name) {
        (Some(id), Some(name)) => Some(CategorySummary { id, name }),
        _ => None,
    };

    ProductResponse {
        id: raw.id,
        name: raw.name,
        sku: raw.sku,
        barcode: raw.barcode,
        base_price: raw.base_price,
        cost_price: raw.cost_price,
        category_id: category,
        track_stock: raw.track_stock,
        is_active: raw.is_active,
        image_url: raw.image_url,
        created_at: raw.created_at,
        updated_at: raw.updated_at,
    }
}

pub async fn list_products(
    State(state): State<AppState>,
) -> Result<Json<StandardResponse<Vec<ProductResponse>>>, AppError> {
    let rows = sqlx::query_as::<_, ProductRaw>(
        r#"
        SELECT p.id, p.name, p.sku, p.barcode, p.base_price, p.cost_price,
               p.category_id, c.name as category_name, p.track_stock, p.is_active,
               p.image_url, p.created_at, p.updated_at
        FROM products p
        LEFT JOIN categories c ON c.id = p.category_id
        WHERE p.is_active = 1
        ORDER BY p.created_at DESC
        "#,
    )
    .fetch_all(&state.pool)
    .await?;

    let response: Vec<ProductResponse> = rows.into_iter().map(map_to_response).collect();
    Ok(Json(StandardResponse::success(response, None)))
}

pub async fn list_products_admin(
    State(state): State<AppState>,
) -> Result<Json<StandardResponse<Vec<ProductResponse>>>, AppError> {
    let rows = sqlx::query_as::<_, ProductRaw>(
        r#"
        SELECT p.id, p.name, p.sku, p.barcode, p.base_price, p.cost_price,
               p.category_id, c.name as category_name, p.track_stock, p.is_active,
               p.image_url, p.created_at, p.updated_at
        FROM products p
        LEFT JOIN categories c ON c.id = p.category_id
        ORDER BY p.created_at DESC
        "#,
    )
    .fetch_all(&state.pool)
    .await?;

    let response: Vec<ProductResponse> = rows.into_iter().map(map_to_response).collect();
    Ok(Json(StandardResponse::success(response, None)))
}

pub async fn get_product(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<StandardResponse<ProductResponse>>, AppError> {
    let raw = sqlx::query_as::<_, ProductRaw>(
        r#"
        SELECT p.id, p.name, p.sku, p.barcode, p.base_price, p.cost_price,
               p.category_id, c.name as category_name, p.track_stock, p.is_active,
               p.image_url, p.created_at, p.updated_at
        FROM products p
        LEFT JOIN categories c ON c.id = p.category_id
        WHERE p.id = ?
        "#,
    )
    .bind(&id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Produk tidak ditemukan".to_string()))?;

    Ok(Json(StandardResponse::success(map_to_response(raw), None)))
}

pub async fn create_product(
    State(state): State<AppState>,
    Json(payload): Json<CreateProductRequest>,
) -> Result<Json<StandardResponse<ProductResponse>>, AppError> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let track_stock = payload.track_stock.unwrap_or(true);
    let is_active = payload.is_active.unwrap_or(true);

    sqlx::query(
        r#"
        INSERT INTO products (id, name, sku, barcode, base_price, cost_price, category_id, track_stock, is_active, image_url, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?);
        "#,
    )
    .bind(&id)
    .bind(&payload.name)
    .bind(&payload.sku)
    .bind(&payload.barcode)
    .bind(payload.base_price)
    .bind(payload.cost_price)
    .bind(&payload.category_id)
    .bind(track_stock)
    .bind(is_active)
    .bind(&payload.image_url)
    .bind(&now)
    .bind(&now)
    .execute(&state.pool)
    .await?;

    let raw = sqlx::query_as::<_, ProductRaw>(
        r#"
        SELECT p.id, p.name, p.sku, p.barcode, p.base_price, p.cost_price,
               p.category_id, c.name as category_name, p.track_stock, p.is_active,
               p.image_url, p.created_at, p.updated_at
        FROM products p
        LEFT JOIN categories c ON c.id = p.category_id
        WHERE p.id = ?
        "#,
    )
    .bind(&id)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(StandardResponse::success(map_to_response(raw), Some("Produk berhasil dibuat"))))
}

pub async fn update_product(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateProductRequest>,
) -> Result<Json<StandardResponse<ProductResponse>>, AppError> {
    let existing = sqlx::query_as::<_, ProductRaw>(
        "SELECT p.*, '' as category_name FROM products p WHERE p.id = ?",
    )
    .bind(&id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Produk tidak ditemukan".to_string()))?;

    let name = payload.name.unwrap_or(existing.name);
    let sku = payload.sku.or(existing.sku);
    let barcode = payload.barcode.or(existing.barcode);
    let base_price = payload.base_price.unwrap_or(existing.base_price);
    let cost_price = payload.cost_price.or(existing.cost_price);
    let category_id = payload.category_id.or(existing.category_id);
    let track_stock = payload.track_stock.unwrap_or(existing.track_stock);
    let is_active = payload.is_active.unwrap_or(existing.is_active);
    let image_url = payload.image_url.or(existing.image_url);
    let now = Utc::now().to_rfc3339();

    sqlx::query(
        r#"
        UPDATE products
        SET name = ?, sku = ?, barcode = ?, base_price = ?, cost_price = ?,
            category_id = ?, track_stock = ?, is_active = ?, image_url = ?, updated_at = ?
        WHERE id = ?;
        "#,
    )
    .bind(&name)
    .bind(&sku)
    .bind(&barcode)
    .bind(base_price)
    .bind(cost_price)
    .bind(&category_id)
    .bind(track_stock)
    .bind(is_active)
    .bind(&image_url)
    .bind(&now)
    .bind(&id)
    .execute(&state.pool)
    .await?;

    let raw = sqlx::query_as::<_, ProductRaw>(
        r#"
        SELECT p.id, p.name, p.sku, p.barcode, p.base_price, p.cost_price,
               p.category_id, c.name as category_name, p.track_stock, p.is_active,
               p.image_url, p.created_at, p.updated_at
        FROM products p
        LEFT JOIN categories c ON c.id = p.category_id
        WHERE p.id = ?
        "#,
    )
    .bind(&id)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(StandardResponse::success(map_to_response(raw), Some("Produk berhasil diperbarui"))))
}

#[derive(Deserialize)]
pub struct PatchProductRequest {
    #[serde(rename = "is_active")]
    pub is_active: Option<bool>,
    #[serde(rename = "track_stock")]
    pub track_stock: Option<bool>,
}

pub async fn patch_product(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<PatchProductRequest>,
) -> Result<Json<StandardResponse<bool>>, AppError> {
    let now = Utc::now().to_rfc3339();

    if let Some(active) = payload.is_active {
        sqlx::query("UPDATE products SET is_active = ?, updated_at = ? WHERE id = ?")
            .bind(active)
            .bind(&now)
            .bind(&id)
            .execute(&state.pool)
            .await?;
    }

    if let Some(track) = payload.track_stock {
        sqlx::query("UPDATE products SET track_stock = ?, updated_at = ? WHERE id = ?")
            .bind(track)
            .bind(&now)
            .bind(&id)
            .execute(&state.pool)
            .await?;
    }

    Ok(Json(StandardResponse::success(true, Some("Status produk berhasil diubah"))))
}

pub async fn delete_product(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<StandardResponse<bool>>, AppError> {
    let res = sqlx::query("DELETE FROM products WHERE id = ?")
        .bind(&id)
        .execute(&state.pool)
        .await?;

    if res.rows_affected() == 0 {
        return Err(AppError::NotFound("Produk tidak ditemukan".to_string()));
    }

    Ok(Json(StandardResponse::success(true, Some("Produk berhasil dihapus"))))
}

// ─── Product Photos ─────────────────────────────────────────────────────────
pub async fn list_photos_by_product(
    State(state): State<AppState>,
    Path(product_id): Path<String>,
) -> Result<Json<Vec<ProductPhoto>>, AppError> {
    let photos = sqlx::query_as::<_, ProductPhoto>(
        "SELECT * FROM product_photos WHERE product_id = ? ORDER BY sort_order ASC",
    )
    .bind(&product_id)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(photos))
}

pub async fn create_photo(
    State(state): State<AppState>,
    Json(payload): Json<CreateProductPhotoRequest>,
) -> Result<Json<StandardResponse<ProductPhoto>>, AppError> {
    let id = Uuid::new_v4().to_string();
    let is_primary = payload.is_primary.unwrap_or(false);
    let sort_order = payload.sort_order.unwrap_or(0);
    let now = Utc::now().to_rfc3339();

    if is_primary {
        sqlx::query("UPDATE product_photos SET is_primary = 0 WHERE product_id = ?")
            .bind(&payload.product_id)
            .execute(&state.pool)
            .await?;
    }

    sqlx::query(
        "INSERT INTO product_photos (id, product_id, url, is_primary, sort_order, created_at) VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&payload.product_id)
    .bind(&payload.url)
    .bind(is_primary)
    .bind(sort_order)
    .bind(&now)
    .execute(&state.pool)
    .await?;

    let photo = sqlx::query_as::<_, ProductPhoto>("SELECT * FROM product_photos WHERE id = ?")
        .bind(&id)
        .fetch_one(&state.pool)
        .await?;

    Ok(Json(StandardResponse::success(photo, None)))
}

pub async fn update_photo(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateProductPhotoRequest>,
) -> Result<Json<StandardResponse<ProductPhoto>>, AppError> {
    let existing = sqlx::query_as::<_, ProductPhoto>("SELECT * FROM product_photos WHERE id = ?")
        .bind(&id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Foto produk tidak ditemukan".to_string()))?;

    let product_id = payload.product_id.unwrap_or(existing.product_id);
    let url = payload.url.unwrap_or(existing.url);
    let is_primary = payload.is_primary.unwrap_or(existing.is_primary);
    let sort_order = payload.sort_order.unwrap_or(existing.sort_order);

    if is_primary {
        sqlx::query("UPDATE product_photos SET is_primary = 0 WHERE product_id = ?")
            .bind(&product_id)
            .execute(&state.pool)
            .await?;
    }

    sqlx::query(
        "UPDATE product_photos SET product_id = ?, url = ?, is_primary = ?, sort_order = ? WHERE id = ?",
    )
    .bind(&product_id)
    .bind(&url)
    .bind(is_primary)
    .bind(sort_order)
    .bind(&id)
    .execute(&state.pool)
    .await?;

    let updated = sqlx::query_as::<_, ProductPhoto>("SELECT * FROM product_photos WHERE id = ?")
        .bind(&id)
        .fetch_one(&state.pool)
        .await?;

    Ok(Json(StandardResponse::success(updated, None)))
}

pub async fn delete_photo(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<StandardResponse<bool>>, AppError> {
    sqlx::query("DELETE FROM product_photos WHERE id = ?")
        .bind(&id)
        .execute(&state.pool)
        .await?;

    Ok(Json(StandardResponse::success(true, None)))
}
