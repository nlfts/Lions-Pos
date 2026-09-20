use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Category {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub icon: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateCategoryRequest {
    pub name: String,
    pub slug: Option<String>,
    pub icon: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCategoryRequest {
    pub name: Option<String>,
    pub slug: Option<String>,
    pub icon: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ProductRaw {
    pub id: String,
    pub name: String,
    pub sku: Option<String>,
    pub barcode: Option<String>,
    pub base_price: f64,
    pub cost_price: Option<f64>,
    pub category_id: Option<String>,
    pub category_name: Option<String>,
    pub track_stock: bool,
    pub is_active: bool,
    pub image_url: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategorySummary {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductResponse {
    pub id: String,
    pub name: String,
    pub sku: Option<String>,
    pub barcode: Option<String>,
    pub base_price: f64,
    pub cost_price: Option<f64>,
    pub category_id: Option<CategorySummary>,
    pub track_stock: bool,
    pub is_active: bool,
    pub image_url: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateProductRequest {
    pub name: String,
    pub sku: Option<String>,
    pub barcode: Option<String>,
    pub base_price: f64,
    pub cost_price: Option<f64>,
    pub category_id: Option<String>,
    pub track_stock: Option<bool>,
    pub is_active: Option<bool>,
    pub image_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProductRequest {
    pub name: Option<String>,
    pub sku: Option<String>,
    pub barcode: Option<String>,
    pub base_price: Option<f64>,
    pub cost_price: Option<f64>,
    pub category_id: Option<String>,
    pub track_stock: Option<bool>,
    pub is_active: Option<bool>,
    pub image_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ProductPhoto {
    pub id: String,
    pub product_id: String,
    pub url: String,
    pub is_primary: bool,
    pub sort_order: i32,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateProductPhotoRequest {
    pub product_id: String,
    pub url: String,
    pub is_primary: Option<bool>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProductPhotoRequest {
    pub product_id: Option<String>,
    pub url: Option<String>,
    pub is_primary: Option<bool>,
    pub sort_order: Option<i32>,
}
