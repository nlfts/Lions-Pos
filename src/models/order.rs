use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct CashierShift {
    pub id: String,
    pub user_id: String,
    pub user_name: Option<String>,
    pub branch_id: String,
    pub branch_name: Option<String>,
    pub start_time: String,
    pub end_time: Option<String>,
    pub starting_cash: f64,
    pub actual_cash: Option<f64>,
    pub expected_cash: Option<f64>,
    pub difference: Option<f64>,
    pub notes: Option<String>,
    pub status: String, // "OPEN", "CLOSED"
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenShiftRequest {
    pub branch_id: String,
    pub starting_cash: f64,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CloseShiftRequest {
    pub shift_id: String,
    pub actual_cash: f64,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct OrderRow {
    pub id: String,
    pub order_number: String,
    pub branch_id: String,
    pub user_id: Option<String>,
    pub cashier_name: Option<String>,
    pub shift_id: Option<String>,
    pub voucher_id: Option<String>,
    pub buyer_name: Option<String>,
    pub subtotal: f64,
    pub discount_amount: f64,
    pub tax_amount: f64,
    pub total: f64,
    pub status: String,
    pub notes: Option<String>,
    pub manual_discount_type: Option<String>,
    pub manual_discount_value: Option<f64>,
    pub manual_discount_note: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct OrderItemRow {
    pub id: String,
    pub order_id: String,
    pub product_id: String,
    pub product_name: String,
    pub product_sku: Option<String>,
    pub unit_price: f64,
    pub quantity: i64,
    pub discount_type: Option<String>,
    pub discount_value: Option<f64>,
    pub discount_amount: f64,
    pub subtotal: f64,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct PaymentRow {
    pub id: String,
    pub order_id: String,
    pub method: String,
    pub amount: f64,
    pub cash_tendered: Option<f64>,
    pub change_due: Option<f64>,
    pub bank_name: Option<String>,
    pub reference_no: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderDetailResponse {
    pub id: String,
    pub order_number: String,
    pub branch_id: String,
    pub branch_name: Option<String>,
    pub user_id: Option<String>,
    pub cashier_name: Option<String>,
    pub shift_id: Option<String>,
    pub voucher_id: Option<String>,
    pub buyer_name: Option<String>,
    pub subtotal: f64,
    pub discount_amount: f64,
    pub tax_amount: f64,
    pub total: f64,
    pub status: String,
    pub notes: Option<String>,
    pub manual_discount_type: Option<String>,
    pub manual_discount_value: Option<f64>,
    pub manual_discount_note: Option<String>,
    pub created_at: String,
    pub items: Vec<OrderItemDetail>,
    pub payments: Vec<PaymentDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderItemDetail {
    pub id: String,
    pub product_id: String,
    pub product_name: String,
    pub product_sku: Option<String>,
    pub unit_price: f64,
    pub qty: i64,
    pub discount_type: Option<String>,
    pub discount_value: Option<f64>,
    pub discount_amount: f64,
    pub subtotal: f64,
    pub item_note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentDetail {
    pub id: String,
    pub method: String,
    pub amount: f64,
    pub cash_tendered: Option<f64>,
    pub change_due: Option<f64>,
    pub bank_name: Option<String>,
    pub reference_no: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateOrderRequest {
    pub branch_id: String,
    pub order_number: Option<String>,
    pub voucher_id: Option<String>,
    pub shift_id: Option<String>,
    pub buyer_name: Option<String>,
    pub notes: Option<String>,
    pub manual_discount_type: Option<String>,
    pub manual_discount_value: Option<f64>,
    pub manual_discount_note: Option<String>,
    pub items: Vec<CreateOrderItemRequest>,
    pub payment: Option<CreatePaymentRequest>,
    pub payments: Option<Vec<CreatePaymentRequest>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateOrderItemRequest {
    pub product_id: String,
    pub qty: i64,
    pub unit_price: f64,
    pub item_discount_type: Option<String>,
    pub item_discount_value: Option<f64>,
    pub item_discount_amount: Option<f64>,
    pub item_note: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePaymentRequest {
    pub method: String,
    pub amount: f64,
    pub cash_tendered: Option<f64>,
    pub change_due: Option<f64>,
    pub bank_name: Option<String>,
    pub reference_no: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardSummaryResponse {
    pub today_sales: f64,
    pub today_transactions: i64,
    pub month_sales: f64,
    pub total_products: i64,
    pub low_stock_count: i64,
}
