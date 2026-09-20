use axum::{
    extract::{Query, State},
    Json,
};
use chrono::Utc;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    errors::AppError,
    models::{
        inventory::{StockBalanceResponse, StockMutation, UpdateStockRequest},
        user::StandardResponse,
    },
    state::AppState,
};

#[derive(Deserialize)]
pub struct StockQuery {
    #[serde(rename = "branchId")]
    pub branch_id: Option<String>,
}

#[derive(sqlx::FromRow)]
struct StockJoinRow {
    id: String,
    branch_id: String,
    product_id: String,
    product_name: Option<String>,
    product_sku: Option<String>,
    quantity: i64,
    min_stock: i64,
    updated_at: String,
}

pub async fn list_stock_balances(
    State(state): State<AppState>,
    Query(query): Query<StockQuery>,
) -> Result<Json<StandardResponse<Vec<StockBalanceResponse>>>, AppError> {
    let rows = if let Some(ref bid) = query.branch_id {
        sqlx::query_as::<_, StockJoinRow>(
            r#"
            SELECT sb.id, sb.branch_id, sb.product_id, p.name as product_name, p.sku as product_sku,
                   sb.quantity, sb.min_stock, sb.updated_at
            FROM stock_balances sb
            JOIN products p ON p.id = sb.product_id
            WHERE sb.branch_id = ?
            ORDER BY p.name ASC
            "#,
        )
        .bind(bid)
        .fetch_all(&state.pool)
        .await?
    } else {
        sqlx::query_as::<_, StockJoinRow>(
            r#"
            SELECT sb.id, sb.branch_id, sb.product_id, p.name as product_name, p.sku as product_sku,
                   sb.quantity, sb.min_stock, sb.updated_at
            FROM stock_balances sb
            JOIN products p ON p.id = sb.product_id
            ORDER BY p.name ASC
            "#,
        )
        .fetch_all(&state.pool)
        .await?
    };

    let result = rows
        .into_iter()
        .map(|r| StockBalanceResponse {
            id: r.id,
            branch_id: r.branch_id,
            product_id: r.product_id,
            product_name: r.product_name,
            product_sku: r.product_sku,
            quantity: r.quantity,
            min_stock: r.min_stock,
            updated_at: r.updated_at,
        })
        .collect();

    Ok(Json(StandardResponse::success(result, None)))
}

pub async fn update_stock_balance(
    State(state): State<AppState>,
    Json(payload): Json<UpdateStockRequest>,
) -> Result<Json<StandardResponse<StockBalanceResponse>>, AppError> {
    let now = Utc::now().to_rfc3339();
    let min_stock = payload.min_stock.unwrap_or(5);

    let existing_stock: Option<(String, i64)> = sqlx::query_as::<_, (String, i64)>(
        "SELECT id, quantity FROM stock_balances WHERE branch_id = ? AND product_id = ?",
    )
    .bind(&payload.branch_id)
    .bind(&payload.product_id)
    .fetch_optional(&state.pool)
    .await?;

    let (stock_id, old_qty) = match existing_stock {
        Some((id, qty)) => {
            sqlx::query(
                "UPDATE stock_balances SET quantity = ?, min_stock = ?, updated_at = ? WHERE id = ?",
            )
            .bind(payload.quantity)
            .bind(min_stock)
            .bind(&now)
            .bind(&id)
            .execute(&state.pool)
            .await?;
            (id, qty)
        }
        None => {
            let id = Uuid::new_v4().to_string();
            sqlx::query(
                "INSERT INTO stock_balances (id, branch_id, product_id, quantity, min_stock, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
            )
            .bind(&id)
            .bind(&payload.branch_id)
            .bind(&payload.product_id)
            .bind(payload.quantity)
            .bind(min_stock)
            .bind(&now)
            .execute(&state.pool)
            .await?;
            (id, 0)
        }
    };

    // Log mutation
    let diff = payload.quantity - old_qty;
    let mutation_id = Uuid::new_v4().to_string();
    let mutation_type = if diff >= 0 { "IN" } else { "OUT" };

    sqlx::query(
        r#"
        INSERT INTO stock_mutations (id, branch_id, product_id, mutation_type, quantity, balance_before, balance_after, reference_no, notes, created_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, 'MANUAL_ADJUST', 'Penyesuaian stok manual', ?);
        "#,
    )
    .bind(&mutation_id)
    .bind(&payload.branch_id)
    .bind(&payload.product_id)
    .bind(mutation_type)
    .bind(diff.abs())
    .bind(old_qty)
    .bind(payload.quantity)
    .bind(&now)
    .execute(&state.pool)
    .await?;

    let row = sqlx::query_as::<_, StockJoinRow>(
        r#"
        SELECT sb.id, sb.branch_id, sb.product_id, p.name as product_name, p.sku as product_sku,
               sb.quantity, sb.min_stock, sb.updated_at
        FROM stock_balances sb
        JOIN products p ON p.id = sb.product_id
        WHERE sb.id = ?
        "#,
    )
    .bind(&stock_id)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(StandardResponse::success(
        StockBalanceResponse {
            id: row.id,
            branch_id: row.branch_id,
            product_id: row.product_id,
            product_name: row.product_name,
            product_sku: row.product_sku,
            quantity: row.quantity,
            min_stock: row.min_stock,
            updated_at: row.updated_at,
        },
        Some("Stok berhasil diperbarui"),
    )))
}

pub async fn list_stock_mutations(
    State(state): State<AppState>,
    Query(query): Query<StockQuery>,
) -> Result<Json<StandardResponse<Vec<StockMutation>>>, AppError> {
    let mutations = if let Some(ref bid) = query.branch_id {
        sqlx::query_as::<_, StockMutation>(
            "SELECT * FROM stock_mutations WHERE branch_id = ? ORDER BY created_at DESC LIMIT 100",
        )
        .bind(bid)
        .fetch_all(&state.pool)
        .await?
    } else {
        sqlx::query_as::<_, StockMutation>(
            "SELECT * FROM stock_mutations ORDER BY created_at DESC LIMIT 100",
        )
        .fetch_all(&state.pool)
        .await?
    };

    Ok(Json(StandardResponse::success(mutations, None)))
}
