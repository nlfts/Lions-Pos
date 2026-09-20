use axum::{
    extract::{Path, State},
    Json,
};
use chrono::Utc;
use uuid::Uuid;

use crate::{
    errors::AppError,
    models::{
        inventory::{
            Branch, CreateBranchRequest, CreateWarehouseRequest, UpdateBranchRequest, Warehouse,
        },
        user::StandardResponse,
    },
    state::AppState,
};

pub async fn list_branches(
    State(state): State<AppState>,
) -> Result<Json<StandardResponse<Vec<Branch>>>, AppError> {
    let branches = sqlx::query_as::<_, Branch>(
        "SELECT * FROM branches ORDER BY name ASC",
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(StandardResponse::success(branches, None)))
}

pub async fn get_branch(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<StandardResponse<Branch>>, AppError> {
    let branch = sqlx::query_as::<_, Branch>("SELECT * FROM branches WHERE id = ?")
        .bind(&id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Cabang tidak ditemukan".to_string()))?;

    Ok(Json(StandardResponse::success(branch, None)))
}

pub async fn create_branch(
    State(state): State<AppState>,
    Json(payload): Json<CreateBranchRequest>,
) -> Result<Json<StandardResponse<Branch>>, AppError> {
    let id = Uuid::new_v4().to_string();
    let is_active = payload.is_active.unwrap_or(true);
    let now = Utc::now().to_rfc3339();

    sqlx::query(
        r#"
        INSERT INTO branches (id, name, code, address, phone, is_active, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?);
        "#,
    )
    .bind(&id)
    .bind(&payload.name)
    .bind(&payload.code)
    .bind(&payload.address)
    .bind(&payload.phone)
    .bind(is_active)
    .bind(&now)
    .bind(&now)
    .execute(&state.pool)
    .await?;

    let branch = sqlx::query_as::<_, Branch>("SELECT * FROM branches WHERE id = ?")
        .bind(&id)
        .fetch_one(&state.pool)
        .await?;

    Ok(Json(StandardResponse::success(branch, Some("Cabang berhasil dibuat"))))
}

pub async fn update_branch(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateBranchRequest>,
) -> Result<Json<StandardResponse<Branch>>, AppError> {
    let existing = sqlx::query_as::<_, Branch>("SELECT * FROM branches WHERE id = ?")
        .bind(&id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Cabang tidak ditemukan".to_string()))?;

    let name = payload.name.unwrap_or(existing.name);
    let code = payload.code.unwrap_or(existing.code);
    let address = payload.address.or(existing.address);
    let phone = payload.phone.or(existing.phone);
    let is_active = payload.is_active.unwrap_or(existing.is_active);
    let now = Utc::now().to_rfc3339();

    sqlx::query(
        r#"
        UPDATE branches
        SET name = ?, code = ?, address = ?, phone = ?, is_active = ?, updated_at = ?
        WHERE id = ?;
        "#,
    )
    .bind(&name)
    .bind(&code)
    .bind(&address)
    .bind(&phone)
    .bind(is_active)
    .bind(&now)
    .bind(&id)
    .execute(&state.pool)
    .await?;

    let updated = sqlx::query_as::<_, Branch>("SELECT * FROM branches WHERE id = ?")
        .bind(&id)
        .fetch_one(&state.pool)
        .await?;

    Ok(Json(StandardResponse::success(updated, Some("Cabang berhasil diperbarui"))))
}

pub async fn delete_branch(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<StandardResponse<bool>>, AppError> {
    let res = sqlx::query("DELETE FROM branches WHERE id = ?")
        .bind(&id)
        .execute(&state.pool)
        .await?;

    if res.rows_affected() == 0 {
        return Err(AppError::NotFound("Cabang tidak ditemukan".to_string()));
    }

    Ok(Json(StandardResponse::success(true, Some("Cabang berhasil dihapus"))))
}

// ─── Warehouses ─────────────────────────────────────────────────────────────
pub async fn list_warehouses(
    State(state): State<AppState>,
) -> Result<Json<StandardResponse<Vec<Warehouse>>>, AppError> {
    let warehouses = sqlx::query_as::<_, Warehouse>(
        "SELECT * FROM warehouses ORDER BY name ASC",
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(StandardResponse::success(warehouses, None)))
}

pub async fn create_warehouse(
    State(state): State<AppState>,
    Json(payload): Json<CreateWarehouseRequest>,
) -> Result<Json<StandardResponse<Warehouse>>, AppError> {
    let id = Uuid::new_v4().to_string();
    let is_active = payload.is_active.unwrap_or(true);
    let now = Utc::now().to_rfc3339();

    sqlx::query(
        r#"
        INSERT INTO warehouses (id, branch_id, name, code, is_active, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?);
        "#,
    )
    .bind(&id)
    .bind(&payload.branch_id)
    .bind(&payload.name)
    .bind(&payload.code)
    .bind(is_active)
    .bind(&now)
    .bind(&now)
    .execute(&state.pool)
    .await?;

    let wh = sqlx::query_as::<_, Warehouse>("SELECT * FROM warehouses WHERE id = ?")
        .bind(&id)
        .fetch_one(&state.pool)
        .await?;

    Ok(Json(StandardResponse::success(wh, Some("Gudang berhasil dibuat"))))
}
