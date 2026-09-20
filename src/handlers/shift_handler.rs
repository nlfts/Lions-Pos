use axum::{
    extract::{Query, State},
    Json,
};
use chrono::Utc;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    errors::AppError,
    middleware::auth::AuthenticatedUser,
    models::{
        order::{CashierShift, CloseShiftRequest, OpenShiftRequest},
        user::StandardResponse,
    },
    state::AppState,
};

#[derive(Deserialize)]
pub struct ActiveShiftQuery {
    #[serde(rename = "branchId")]
    pub branch_id: Option<String>,
}

pub async fn get_active_shift(
    State(state): State<AppState>,
    Query(query): Query<ActiveShiftQuery>,
    AuthenticatedUser(claims): AuthenticatedUser,
) -> Result<Json<StandardResponse<Option<CashierShift>>>, AppError> {
    let shift = if let Some(ref bid) = query.branch_id {
        sqlx::query_as::<_, CashierShift>(
            r#"
            SELECT cs.*, u.fullname as user_name, b.name as branch_name
            FROM cashier_shifts cs
            JOIN users u ON u.id = cs.user_id
            JOIN branches b ON b.id = cs.branch_id
            WHERE cs.branch_id = ? AND cs.user_id = ? AND cs.status = 'OPEN'
            ORDER BY cs.start_time DESC LIMIT 1
            "#,
        )
        .bind(bid)
        .bind(&claims.sub)
        .fetch_optional(&state.pool)
        .await?
    } else {
        sqlx::query_as::<_, CashierShift>(
            r#"
            SELECT cs.*, u.fullname as user_name, b.name as branch_name
            FROM cashier_shifts cs
            JOIN users u ON u.id = cs.user_id
            JOIN branches b ON b.id = cs.branch_id
            WHERE cs.user_id = ? AND cs.status = 'OPEN'
            ORDER BY cs.start_time DESC LIMIT 1
            "#,
        )
        .bind(&claims.sub)
        .fetch_optional(&state.pool)
        .await?
    };

    Ok(Json(StandardResponse::success(shift, None)))
}

pub async fn open_shift(
    State(state): State<AppState>,
    AuthenticatedUser(claims): AuthenticatedUser,
    Json(payload): Json<OpenShiftRequest>,
) -> Result<Json<StandardResponse<CashierShift>>, AppError> {
    // Check if user already has an open shift in this branch
    let existing_open: Option<String> = sqlx::query_scalar(
        "SELECT id FROM cashier_shifts WHERE user_id = ? AND branch_id = ? AND status = 'OPEN'",
    )
    .bind(&claims.sub)
    .bind(&payload.branch_id)
    .fetch_optional(&state.pool)
    .await?;

    if existing_open.is_some() {
        return Err(AppError::BadRequest("Anda masih memiliki shift aktif di cabang ini".to_string()));
    }

    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    sqlx::query(
        r#"
        INSERT INTO cashier_shifts (id, user_id, branch_id, start_time, starting_cash, notes, status, created_at)
        VALUES (?, ?, ?, ?, ?, ?, 'OPEN', ?);
        "#,
    )
    .bind(&id)
    .bind(&claims.sub)
    .bind(&payload.branch_id)
    .bind(&now)
    .bind(payload.starting_cash)
    .bind(&payload.notes)
    .bind(&now)
    .execute(&state.pool)
    .await?;

    let shift = sqlx::query_as::<_, CashierShift>(
        r#"
        SELECT cs.*, u.fullname as user_name, b.name as branch_name
        FROM cashier_shifts cs
        JOIN users u ON u.id = cs.user_id
        JOIN branches b ON b.id = cs.branch_id
        WHERE cs.id = ?
        "#,
    )
    .bind(&id)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(StandardResponse::success(shift, Some("Shift kasir berhasil dibuka"))))
}

pub async fn close_shift(
    State(state): State<AppState>,
    Json(payload): Json<CloseShiftRequest>,
) -> Result<Json<StandardResponse<CashierShift>>, AppError> {
    let shift = sqlx::query_as::<_, CashierShift>("SELECT cs.*, '' as user_name, '' as branch_name FROM cashier_shifts cs WHERE cs.id = ?")
        .bind(&payload.shift_id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Shift tidak ditemukan".to_string()))?;

    if shift.status == "CLOSED" {
        return Err(AppError::BadRequest("Shift ini sudah ditutup sebelumnya".to_string()));
    }

    let now = Utc::now().to_rfc3339();

    // Calculate total cash payments during this shift
    let cash_sales: f64 = sqlx::query_scalar(
        r#"
        SELECT COALESCE(SUM(p.amount), 0.0)
        FROM payments p
        JOIN orders o ON o.id = p.order_id
        WHERE o.shift_id = ? AND p.method = 'CASH' AND o.status = 'PAID'
        "#,
    )
    .bind(&payload.shift_id)
    .fetch_one(&state.pool)
    .await?;

    let expected_cash = shift.starting_cash + cash_sales;
    let difference = payload.actual_cash - expected_cash;

    sqlx::query(
        r#"
        UPDATE cashier_shifts
        SET end_time = ?, actual_cash = ?, expected_cash = ?, difference = ?, notes = ?, status = 'CLOSED'
        WHERE id = ?;
        "#,
    )
    .bind(&now)
    .bind(payload.actual_cash)
    .bind(expected_cash)
    .bind(difference)
    .bind(&payload.notes)
    .bind(&payload.shift_id)
    .execute(&state.pool)
    .await?;

    let closed = sqlx::query_as::<_, CashierShift>(
        r#"
        SELECT cs.*, u.fullname as user_name, b.name as branch_name
        FROM cashier_shifts cs
        JOIN users u ON u.id = cs.user_id
        JOIN branches b ON b.id = cs.branch_id
        WHERE cs.id = ?
        "#,
    )
    .bind(&payload.shift_id)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(StandardResponse::success(closed, Some("Shift kasir berhasil ditutup"))))
}

pub async fn list_shifts(
    State(state): State<AppState>,
) -> Result<Json<StandardResponse<Vec<CashierShift>>>, AppError> {
    let shifts = sqlx::query_as::<_, CashierShift>(
        r#"
        SELECT cs.*, u.fullname as user_name, b.name as branch_name
        FROM cashier_shifts cs
        JOIN users u ON u.id = cs.user_id
        JOIN branches b ON b.id = cs.branch_id
        ORDER BY cs.start_time DESC LIMIT 100
        "#,
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(StandardResponse::success(shifts, None)))
}
