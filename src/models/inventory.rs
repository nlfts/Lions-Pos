use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Branch {
    pub id: String,
    pub name: String,
    pub code: String,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateBranchRequest {
    pub name: String,
    pub code: String,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateBranchRequest {
    pub name: Option<String>,
    pub code: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Warehouse {
    pub id: String,
    pub branch_id: Option<String>,
    pub name: String,
    pub code: String,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateWarehouseRequest {
    pub branch_id: Option<String>,
    pub name: String,
    pub code: String,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct StockBalance {
    pub id: String,
    pub branch_id: String,
    pub product_id: String,
    pub quantity: i64,
    pub min_stock: i64,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockBalanceResponse {
    pub id: String,
    pub branch_id: String,
    pub product_id: String,
    pub product_name: Option<String>,
    pub product_sku: Option<String>,
    pub quantity: i64,
    pub min_stock: i64,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateStockRequest {
    pub branch_id: String,
    pub product_id: String,
    pub quantity: i64,
    pub min_stock: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct StockMutation {
    pub id: String,
    pub branch_id: String,
    pub product_id: String,
    pub mutation_type: String, // "SALE", "PURCHASE", "ADJUSTMENT", "TRANSFER_IN", "TRANSFER_OUT"
    pub quantity: i64,
    pub balance_before: i64,
    pub balance_after: i64,
    pub reference_no: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
}
