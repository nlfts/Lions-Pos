use axum::{
    extract::{Path, Query, State},
    Json,
};
use chrono::Utc;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    errors::AppError,
    middleware::auth::OptionalUser,
    models::{
        catalog::ProductRaw,
        order::{
            CreateOrderRequest, DashboardSummaryResponse, OrderDetailResponse, OrderItemDetail,
            OrderItemRow, OrderRow, PaymentDetail, PaymentRow,
        },
        user::StandardResponse,
    },
    state::AppState,
};

#[derive(Deserialize)]
pub struct OrderFilterQuery {
    #[serde(rename = "branchId")]
    pub branch_id: Option<String>,
    pub status: Option<String>,
}

pub async fn create_order(
    State(state): State<AppState>,
    OptionalUser(user): OptionalUser,
    Json(payload): Json<CreateOrderRequest>,
) -> Result<Json<StandardResponse<OrderDetailResponse>>, AppError> {
    if payload.items.is_empty() {
        return Err(AppError::BadRequest("Keranjang belanja kosong".to_string()));
    }

    let user_id = user.as_ref().map(|u| u.sub.clone());
    let now = Utc::now().to_rfc3339();
    let order_id = Uuid::new_v4().to_string();
    let order_number = payload
        .order_number
        .unwrap_or_else(|| format!("ORD-{}", Utc::now().timestamp_millis()));

    // Verify stock availability & compute subtotal
    let mut calculated_subtotal = 0.0;
    let mut total_item_discount = 0.0;

    for item in &payload.items {
        let product = sqlx::query_as::<_, ProductRaw>(
            "SELECT p.*, '' as category_name FROM products p WHERE p.id = ?",
        )
        .bind(&item.product_id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| AppError::BadRequest(format!("Produk id {} tidak ditemukan", item.product_id)))?;

        if product.track_stock {
            let stock_qty: Option<i64> = sqlx::query_scalar(
                "SELECT quantity FROM stock_balances WHERE branch_id = ? AND product_id = ?",
            )
            .bind(&payload.branch_id)
            .bind(&item.product_id)
            .fetch_optional(&state.pool)
            .await?;

            let available = stock_qty.unwrap_or(0);
            if available < item.qty {
                return Err(AppError::BadRequest(format!(
                    "Stok produk '{}' tidak mencukupi (tersedia: {}, diminta: {})",
                    product.name, available, item.qty
                )));
            }
        }

        let item_subtotal = item.unit_price * (item.qty as f64);
        calculated_subtotal += item_subtotal;
        if let Some(disc_amt) = item.item_discount_amount {
            total_item_discount += disc_amt;
        }
    }

    // Voucher Discount calculation
    let mut voucher_discount = 0.0;
    if let Some(ref vid) = payload.voucher_id {
        let voucher_info: Option<(String, f64, Option<f64>, Option<f64>)> = sqlx::query_as(
            "SELECT discount_type, discount_value, min_spend, max_discount FROM vouchers WHERE id = ? AND is_active = 1",
        )
        .bind(vid)
        .fetch_optional(&state.pool)
        .await?;

        if let Some((dtype, dval, min_spend, max_disc)) = voucher_info {
            let min_s = min_spend.unwrap_or(0.0);
            if calculated_subtotal >= min_s {
                let disc = if dtype == "PERCENTAGE" {
                    (calculated_subtotal * (dval / 100.0)).min(max_disc.unwrap_or(f64::MAX))
                } else {
                    dval.min(max_disc.unwrap_or(f64::MAX))
                };
                voucher_discount = disc;
            }
        }
    }

    // Manual Discount calculation
    let mut manual_disc = 0.0;
    if let (Some(mtype), Some(mval)) = (&payload.manual_discount_type, payload.manual_discount_value) {
        if mtype == "PERCENTAGE" || mtype == "percent" {
            manual_disc = (calculated_subtotal - total_item_discount) * (mval / 100.0);
        } else {
            manual_disc = mval;
        }
    }

    let total_discount = total_item_discount + voucher_discount + manual_disc;
    let total_amount = (calculated_subtotal - total_discount).max(0.0);

    // Atomic DB Transaction
    let mut tx = state.pool.begin().await?;

    // 1. Insert Order
    sqlx::query(
        r#"
        INSERT INTO orders (
            id, order_number, branch_id, user_id, shift_id, voucher_id, buyer_name,
            subtotal, discount_amount, tax_amount, total, status, notes,
            manual_discount_type, manual_discount_value, manual_discount_note,
            created_at, updated_at
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 0.0, ?, 'PAID', ?, ?, ?, ?, ?, ?);
        "#,
    )
    .bind(&order_id)
    .bind(&order_number)
    .bind(&payload.branch_id)
    .bind(&user_id)
    .bind(&payload.shift_id)
    .bind(&payload.voucher_id)
    .bind(&payload.buyer_name)
    .bind(calculated_subtotal)
    .bind(total_discount)
    .bind(total_amount)
    .bind(&payload.notes)
    .bind(&payload.manual_discount_type)
    .bind(payload.manual_discount_value)
    .bind(&payload.manual_discount_note)
    .bind(&now)
    .bind(&now)
    .execute(&mut *tx)
    .await?;

    // 2. Insert Order Items & Deduct Stock
    for item in &payload.items {
        let item_id = Uuid::new_v4().to_string();
        let prod_name: String = sqlx::query_scalar("SELECT name FROM products WHERE id = ?")
            .bind(&item.product_id)
            .fetch_one(&mut *tx)
            .await?;

        let prod_sku: Option<String> = sqlx::query_scalar("SELECT sku FROM products WHERE id = ?")
            .bind(&item.product_id)
            .fetch_optional(&mut *tx)
            .await?;

        let disc_amt = item.item_discount_amount.unwrap_or(0.0);
        let item_sub = (item.unit_price * (item.qty as f64)) - disc_amt;

        sqlx::query(
            r#"
            INSERT INTO order_items (
                id, order_id, product_id, product_name, product_sku,
                unit_price, quantity, discount_type, discount_value, discount_amount,
                subtotal, note
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?);
            "#,
        )
        .bind(&item_id)
        .bind(&order_id)
        .bind(&item.product_id)
        .bind(&prod_name)
        .bind(&prod_sku)
        .bind(item.unit_price)
        .bind(item.qty)
        .bind(&item.item_discount_type)
        .bind(item.item_discount_value)
        .bind(disc_amt)
        .bind(item_sub)
        .bind(&item.item_note)
        .execute(&mut *tx)
        .await?;

        // Check if track_stock is enabled for product
        let track: bool = sqlx::query_scalar("SELECT track_stock FROM products WHERE id = ?")
            .bind(&item.product_id)
            .fetch_one(&mut *tx)
            .await?;

        if track {
            let current_stock: i64 = sqlx::query_scalar(
                "SELECT quantity FROM stock_balances WHERE branch_id = ? AND product_id = ?",
            )
            .bind(&payload.branch_id)
            .bind(&item.product_id)
            .fetch_optional(&mut *tx)
            .await?
            .unwrap_or(0);

            let new_stock = current_stock - item.qty;

            sqlx::query(
                "UPDATE stock_balances SET quantity = ?, updated_at = ? WHERE branch_id = ? AND product_id = ?",
            )
            .bind(new_stock)
            .bind(&now)
            .bind(&payload.branch_id)
            .bind(&item.product_id)
            .execute(&mut *tx)
            .await?;

            // Stock Mutation Record
            let mutation_id = Uuid::new_v4().to_string();
            sqlx::query(
                r#"
                INSERT INTO stock_mutations (id, branch_id, product_id, mutation_type, quantity, balance_before, balance_after, reference_no, notes, created_at)
                VALUES (?, ?, ?, 'SALE', ?, ?, ?, ?, 'Penjualan Kasir POS', ?);
                "#,
            )
            .bind(&mutation_id)
            .bind(&payload.branch_id)
            .bind(&item.product_id)
            .bind(item.qty)
            .bind(current_stock)
            .bind(new_stock)
            .bind(&order_number)
            .bind(&now)
            .execute(&mut *tx)
            .await?;
        }
    }

    // 3. Insert Payments
    if let Some(payments) = payload.payments {
        for p in payments {
            let pid = Uuid::new_v4().to_string();
            sqlx::query(
                r#"
                INSERT INTO payments (id, order_id, method, amount, cash_tendered, change_due, bank_name, reference_no, created_at)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?);
                "#,
            )
            .bind(&pid)
            .bind(&order_id)
            .bind(&p.method)
            .bind(p.amount)
            .bind(p.cash_tendered)
            .bind(p.change_due)
            .bind(&p.bank_name)
            .bind(&p.reference_no)
            .bind(&now)
            .execute(&mut *tx)
            .await?;
        }
    } else if let Some(p) = payload.payment {
        let pid = Uuid::new_v4().to_string();
        sqlx::query(
            r#"
            INSERT INTO payments (id, order_id, method, amount, cash_tendered, change_due, bank_name, reference_no, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?);
            "#,
        )
        .bind(&pid)
        .bind(&order_id)
        .bind(&p.method)
        .bind(p.amount)
        .bind(p.cash_tendered)
        .bind(p.change_due)
        .bind(&p.bank_name)
        .bind(&p.reference_no)
        .bind(&now)
        .execute(&mut *tx)
        .await?;
    }

    // 4. Update Voucher usage
    if let Some(ref vid) = payload.voucher_id {
        sqlx::query("UPDATE vouchers SET used_count = used_count + 1 WHERE id = ?")
            .bind(vid)
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await?;

    // Return full order detail
    get_order_detail_by_id(&state, &order_id).await
}

pub async fn list_orders(
    State(state): State<AppState>,
    Query(query): Query<OrderFilterQuery>,
) -> Result<Json<StandardResponse<Vec<OrderDetailResponse>>>, AppError> {
    let order_rows = if let Some(ref bid) = query.branch_id {
        sqlx::query_as::<_, OrderRow>(
            r#"
            SELECT o.*, u.fullname as cashier_name
            FROM orders o
            LEFT JOIN users u ON u.id = o.user_id
            WHERE o.branch_id = ?
            ORDER BY o.created_at DESC
            LIMIT 200
            "#,
        )
        .bind(bid)
        .fetch_all(&state.pool)
        .await?
    } else {
        sqlx::query_as::<_, OrderRow>(
            r#"
            SELECT o.*, u.fullname as cashier_name
            FROM orders o
            LEFT JOIN users u ON u.id = o.user_id
            ORDER BY o.created_at DESC
            LIMIT 200
            "#,
        )
        .fetch_all(&state.pool)
        .await?
    };

    let mut result = Vec::new();
    for row in order_rows {
        let items = sqlx::query_as::<_, OrderItemRow>(
            "SELECT * FROM order_items WHERE order_id = ?",
        )
        .bind(&row.id)
        .fetch_all(&state.pool)
        .await?
        .into_iter()
        .map(|i| OrderItemDetail {
            id: i.id,
            product_id: i.product_id,
            product_name: i.product_name,
            product_sku: i.product_sku,
            unit_price: i.unit_price,
            qty: i.quantity,
            discount_type: i.discount_type,
            discount_value: i.discount_value,
            discount_amount: i.discount_amount,
            subtotal: i.subtotal,
            item_note: i.note,
        })
        .collect();

        let payments = sqlx::query_as::<_, PaymentRow>(
            "SELECT * FROM payments WHERE order_id = ?",
        )
        .bind(&row.id)
        .fetch_all(&state.pool)
        .await?
        .into_iter()
        .map(|p| PaymentDetail {
            id: p.id,
            method: p.method,
            amount: p.amount,
            cash_tendered: p.cash_tendered,
            change_due: p.change_due,
            bank_name: p.bank_name,
            reference_no: p.reference_no,
        })
        .collect();

        let branch_name: Option<String> = sqlx::query_scalar("SELECT name FROM branches WHERE id = ?")
            .bind(&row.branch_id)
            .fetch_optional(&state.pool)
            .await?;

        result.push(OrderDetailResponse {
            id: row.id,
            order_number: row.order_number,
            branch_id: row.branch_id,
            branch_name,
            user_id: row.user_id,
            cashier_name: row.cashier_name,
            shift_id: row.shift_id,
            voucher_id: row.voucher_id,
            buyer_name: row.buyer_name,
            subtotal: row.subtotal,
            discount_amount: row.discount_amount,
            tax_amount: row.tax_amount,
            total: row.total,
            status: row.status,
            notes: row.notes,
            manual_discount_type: row.manual_discount_type,
            manual_discount_value: row.manual_discount_value,
            manual_discount_note: row.manual_discount_note,
            created_at: row.created_at,
            items,
            payments,
        });
    }

    Ok(Json(StandardResponse::success(result, None)))
}

pub async fn get_order(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<StandardResponse<OrderDetailResponse>>, AppError> {
    get_order_detail_by_id(&state, &id).await
}

async fn get_order_detail_by_id(
    state: &AppState,
    order_id: &str,
) -> Result<Json<StandardResponse<OrderDetailResponse>>, AppError> {
    let row = sqlx::query_as::<_, OrderRow>(
        r#"
        SELECT o.*, u.fullname as cashier_name
        FROM orders o
        LEFT JOIN users u ON u.id = o.user_id
        WHERE o.id = ? OR o.order_number = ?
        "#,
    )
    .bind(order_id)
    .bind(order_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Order tidak ditemukan".to_string()))?;

    let items = sqlx::query_as::<_, OrderItemRow>(
        "SELECT * FROM order_items WHERE order_id = ?",
    )
    .bind(&row.id)
    .fetch_all(&state.pool)
    .await?
    .into_iter()
    .map(|i| OrderItemDetail {
        id: i.id,
        product_id: i.product_id,
        product_name: i.product_name,
        product_sku: i.product_sku,
        unit_price: i.unit_price,
        qty: i.quantity,
        discount_type: i.discount_type,
        discount_value: i.discount_value,
        discount_amount: i.discount_amount,
        subtotal: i.subtotal,
        item_note: i.note,
    })
    .collect();

    let payments = sqlx::query_as::<_, PaymentRow>(
        "SELECT * FROM payments WHERE order_id = ?",
    )
    .bind(&row.id)
    .fetch_all(&state.pool)
    .await?
    .into_iter()
    .map(|p| PaymentDetail {
        id: p.id,
        method: p.method,
        amount: p.amount,
        cash_tendered: p.cash_tendered,
        change_due: p.change_due,
        bank_name: p.bank_name,
        reference_no: p.reference_no,
    })
    .collect();

    let branch_name: Option<String> = sqlx::query_scalar("SELECT name FROM branches WHERE id = ?")
        .bind(&row.branch_id)
        .fetch_optional(&state.pool)
        .await?;

    Ok(Json(StandardResponse::success(
        OrderDetailResponse {
            id: row.id,
            order_number: row.order_number,
            branch_id: row.branch_id,
            branch_name,
            user_id: row.user_id,
            cashier_name: row.cashier_name,
            shift_id: row.shift_id,
            voucher_id: row.voucher_id,
            buyer_name: row.buyer_name,
            subtotal: row.subtotal,
            discount_amount: row.discount_amount,
            tax_amount: row.tax_amount,
            total: row.total,
            status: row.status,
            notes: row.notes,
            manual_discount_type: row.manual_discount_type,
            manual_discount_value: row.manual_discount_value,
            manual_discount_note: row.manual_discount_note,
            created_at: row.created_at,
            items,
            payments,
        },
        Some("Transaksi berhasil diproses"),
    )))
}

pub async fn get_dashboard_summary(
    State(state): State<AppState>,
) -> Result<Json<StandardResponse<DashboardSummaryResponse>>, AppError> {
    let today_date = Utc::now().format("%Y-%m-%d").to_string();
    let month_date = Utc::now().format("%Y-%m").to_string();

    let today_sales: f64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(total), 0.0) FROM orders WHERE status = 'PAID' AND created_at LIKE ? || '%'",
    )
    .bind(&today_date)
    .fetch_one(&state.pool)
    .await?;

    let today_transactions: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM orders WHERE status = 'PAID' AND created_at LIKE ? || '%'",
    )
    .bind(&today_date)
    .fetch_one(&state.pool)
    .await?;

    let month_sales: f64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(total), 0.0) FROM orders WHERE status = 'PAID' AND created_at LIKE ? || '%'",
    )
    .bind(&month_date)
    .fetch_one(&state.pool)
    .await?;

    let total_products: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM products WHERE is_active = 1")
        .fetch_one(&state.pool)
        .await?;

    let low_stock_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM stock_balances WHERE quantity <= min_stock",
    )
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(StandardResponse::success(
        DashboardSummaryResponse {
            today_sales,
            today_transactions,
            month_sales,
            total_products,
            low_stock_count,
        },
        None,
    )))
}
