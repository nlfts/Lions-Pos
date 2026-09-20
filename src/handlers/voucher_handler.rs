use axum::{
    extract::{Path, State},
    Json,
};
use chrono::Utc;
use uuid::Uuid;

use crate::{
    errors::AppError,
    models::{
        user::StandardResponse,
        voucher::{CreateVoucherRequest, UpdateVoucherRequest, Voucher},
    },
    state::AppState,
};

pub async fn list_vouchers(
    State(state): State<AppState>,
) -> Result<Json<StandardResponse<Vec<Voucher>>>, AppError> {
    let vouchers = sqlx::query_as::<_, Voucher>(
        "SELECT * FROM vouchers WHERE is_active = 1 ORDER BY created_at DESC",
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(StandardResponse::success(vouchers, None)))
}

pub async fn get_voucher_by_code(
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> Result<Json<StandardResponse<Voucher>>, AppError> {
    let voucher = sqlx::query_as::<_, Voucher>(
        "SELECT * FROM vouchers WHERE UPPER(code) = UPPER(?) AND is_active = 1",
    )
    .bind(&code)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Voucher tidak valid atau sudah tidak aktif".to_string()))?;

    // Check quota
    if let Some(quota) = voucher.quota {
        if voucher.used_count >= quota {
            return Err(AppError::BadRequest("Kuota voucher telah habis".to_string()));
        }
    }

    Ok(Json(StandardResponse::success(voucher, None)))
}

pub async fn create_voucher(
    State(state): State<AppState>,
    Json(payload): Json<CreateVoucherRequest>,
) -> Result<Json<StandardResponse<Voucher>>, AppError> {
    let id = Uuid::new_v4().to_string();
    let is_active = payload.is_active.unwrap_or(true);
    let now = Utc::now().to_rfc3339();

    sqlx::query(
        r#"
        INSERT INTO vouchers (id, code, description, discount_type, discount_value, min_spend, max_discount, quota, used_count, is_active, start_date, end_date, created_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, 0, ?, ?, ?, ?);
        "#,
    )
    .bind(&id)
    .bind(&payload.code)
    .bind(&payload.description)
    .bind(&payload.discount_type)
    .bind(payload.discount_value)
    .bind(payload.min_spend)
    .bind(payload.max_discount)
    .bind(payload.quota)
    .bind(is_active)
    .bind(&payload.start_date)
    .bind(&payload.end_date)
    .bind(&now)
    .execute(&state.pool)
    .await?;

    let voucher = sqlx::query_as::<_, Voucher>("SELECT * FROM vouchers WHERE id = ?")
        .bind(&id)
        .fetch_one(&state.pool)
        .await?;

    Ok(Json(StandardResponse::success(voucher, Some("Voucher berhasil dibuat"))))
}

pub async fn update_voucher(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateVoucherRequest>,
) -> Result<Json<StandardResponse<Voucher>>, AppError> {
    let existing = sqlx::query_as::<_, Voucher>("SELECT * FROM vouchers WHERE id = ?")
        .bind(&id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Voucher tidak ditemukan".to_string()))?;

    let code = payload.code.unwrap_or(existing.code);
    let description = payload.description.or(existing.description);
    let discount_type = payload.discount_type.unwrap_or(existing.discount_type);
    let discount_value = payload.discount_value.unwrap_or(existing.discount_value);
    let min_spend = payload.min_spend.or(existing.min_spend);
    let max_discount = payload.max_discount.or(existing.max_discount);
    let quota = payload.quota.or(existing.quota);
    let is_active = payload.is_active.unwrap_or(existing.is_active);
    let start_date = payload.start_date.or(existing.start_date);
    let end_date = payload.end_date.or(existing.end_date);

    sqlx::query(
        r#"
        UPDATE vouchers
        SET code = ?, description = ?, discount_type = ?, discount_value = ?,
            min_spend = ?, max_discount = ?, quota = ?, is_active = ?, start_date = ?, end_date = ?
        WHERE id = ?;
        "#,
    )
    .bind(&code)
    .bind(&description)
    .bind(&discount_type)
    .bind(discount_value)
    .bind(min_spend)
    .bind(max_discount)
    .bind(quota)
    .bind(is_active)
    .bind(&start_date)
    .bind(&end_date)
    .bind(&id)
    .execute(&state.pool)
    .await?;

    let updated = sqlx::query_as::<_, Voucher>("SELECT * FROM vouchers WHERE id = ?")
        .bind(&id)
        .fetch_one(&state.pool)
        .await?;

    Ok(Json(StandardResponse::success(updated, Some("Voucher berhasil diperbarui"))))
}

pub async fn delete_voucher(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<StandardResponse<bool>>, AppError> {
    let res = sqlx::query("DELETE FROM vouchers WHERE id = ?")
        .bind(&id)
        .execute(&state.pool)
        .await?;

    if res.rows_affected() == 0 {
        return Err(AppError::NotFound("Voucher tidak ditemukan".to_string()));
    }

    Ok(Json(StandardResponse::success(true, Some("Voucher berhasil dihapus"))))
}
