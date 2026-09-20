use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Voucher {
    pub id: String,
    pub code: String,
    pub description: Option<String>,
    pub discount_type: String, // "PERCENTAGE" or "FIXED"
    pub discount_value: f64,
    pub min_spend: Option<f64>,
    pub max_discount: Option<f64>,
    pub quota: Option<i64>,
    pub used_count: i64,
    pub is_active: bool,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateVoucherRequest {
    pub code: String,
    pub description: Option<String>,
    pub discount_type: String,
    pub discount_value: f64,
    pub min_spend: Option<f64>,
    pub max_discount: Option<f64>,
    pub quota: Option<i64>,
    pub is_active: Option<bool>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateVoucherRequest {
    pub code: Option<String>,
    pub description: Option<String>,
    pub discount_type: Option<String>,
    pub discount_value: Option<f64>,
    pub min_spend: Option<f64>,
    pub max_discount: Option<f64>,
    pub quota: Option<i64>,
    pub is_active: Option<bool>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}
