use axum::{
    extract::{Path, State},
    Json,
};
use chrono::Utc;
use uuid::Uuid;

use crate::{
    errors::AppError,
    models::{
        catalog::{Category, CreateCategoryRequest, UpdateCategoryRequest},
        user::StandardResponse,
    },
    state::AppState,
};

pub async fn list_categories(
    State(state): State<AppState>,
) -> Result<Json<StandardResponse<Vec<Category>>>, AppError> {
    let categories = sqlx::query_as::<_, Category>(
        "SELECT * FROM categories WHERE is_active = 1 ORDER BY name ASC",
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(StandardResponse::success(categories, None)))
}

pub async fn list_categories_admin(
    State(state): State<AppState>,
) -> Result<Json<StandardResponse<Vec<Category>>>, AppError> {
    let categories = sqlx::query_as::<_, Category>(
        "SELECT * FROM categories ORDER BY created_at DESC",
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(StandardResponse::success(categories, None)))
}

pub async fn get_category(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<StandardResponse<Category>>, AppError> {
    let category = sqlx::query_as::<_, Category>("SELECT * FROM categories WHERE id = ?")
        .bind(&id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Kategori tidak ditemukan".to_string()))?;

    Ok(Json(StandardResponse::success(category, None)))
}

pub async fn create_category(
    State(state): State<AppState>,
    Json(payload): Json<CreateCategoryRequest>,
) -> Result<Json<StandardResponse<Category>>, AppError> {
    let id = Uuid::new_v4().to_string();
    let slug = payload
        .slug
        .unwrap_or_else(|| payload.name.to_lowercase().replace(' ', "-"));
    let is_active = payload.is_active.unwrap_or(true);
    let now = Utc::now().to_rfc3339();

    sqlx::query(
        r#"
        INSERT INTO categories (id, name, slug, icon, is_active, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?);
        "#,
    )
    .bind(&id)
    .bind(&payload.name)
    .bind(&slug)
    .bind(&payload.icon)
    .bind(is_active)
    .bind(&now)
    .bind(&now)
    .execute(&state.pool)
    .await?;

    let category = sqlx::query_as::<_, Category>("SELECT * FROM categories WHERE id = ?")
        .bind(&id)
        .fetch_one(&state.pool)
        .await?;

    Ok(Json(StandardResponse::success(category, Some("Kategori berhasil dibuat"))))
}

pub async fn update_category(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateCategoryRequest>,
) -> Result<Json<StandardResponse<Category>>, AppError> {
    let existing = sqlx::query_as::<_, Category>("SELECT * FROM categories WHERE id = ?")
        .bind(&id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Kategori tidak ditemukan".to_string()))?;

    let name = payload.name.unwrap_or(existing.name);
    let slug = payload.slug.unwrap_or(existing.slug);
    let icon = payload.icon.or(existing.icon);
    let is_active = payload.is_active.unwrap_or(existing.is_active);
    let now = Utc::now().to_rfc3339();

    sqlx::query(
        r#"
        UPDATE categories
        SET name = ?, slug = ?, icon = ?, is_active = ?, updated_at = ?
        WHERE id = ?;
        "#,
    )
    .bind(&name)
    .bind(&slug)
    .bind(&icon)
    .bind(is_active)
    .bind(&now)
    .bind(&id)
    .execute(&state.pool)
    .await?;

    let updated = sqlx::query_as::<_, Category>("SELECT * FROM categories WHERE id = ?")
        .bind(&id)
        .fetch_one(&state.pool)
        .await?;

    Ok(Json(StandardResponse::success(updated, Some("Kategori berhasil diperbarui"))))
}

pub async fn delete_category(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<StandardResponse<bool>>, AppError> {
    let res = sqlx::query("DELETE FROM categories WHERE id = ?")
        .bind(&id)
        .execute(&state.pool)
        .await?;

    if res.rows_affected() == 0 {
        return Err(AppError::NotFound("Kategori tidak ditemukan".to_string()));
    }

    Ok(Json(StandardResponse::success(true, Some("Kategori berhasil dihapus"))))
}
