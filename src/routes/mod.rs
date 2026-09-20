use axum::{
    http::{header, Method},
    routing::{get, post, put},
    Router,
};
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
    trace::TraceLayer,
};

use crate::{
    handlers::{
        auth_handler, branch_handler, category_handler, order_handler, product_handler,
        shift_handler, stock_handler, upload_handler, user_handler, voucher_handler,
    },
    state::AppState,
};

pub fn create_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE, header::ACCEPT]);

    let api_routes = Router::new()
        // Health
        .route("/health", get(health_check))
        // Auth
        .route("/v1/auth/login", post(auth_handler::login))
        .route("/v1/auth/me", get(auth_handler::me))
        .route("/v1/auth/refresh", post(auth_handler::refresh))
        .route("/v1/auth/logout", post(auth_handler::logout))
        // Users & RBAC
        .route("/v1/users", get(user_handler::list_users).post(user_handler::create_user))
        .route("/v1/roles", get(user_handler::list_roles))
        .route("/v1/roles/permissions", get(user_handler::get_role_permissions_map))
        .route("/v1/permissions", get(user_handler::list_permissions))
        // Categories
        .route("/v1/categories", get(category_handler::list_categories).post(category_handler::create_category))
        .route("/v1/categories/admin", get(category_handler::list_categories_admin))
        .route(
            "/v1/categories/{id}",
            get(category_handler::get_category)
                .put(category_handler::update_category)
                .delete(category_handler::delete_category),
        )
        // Products
        .route("/v1/products", get(product_handler::list_products).post(product_handler::create_product))
        .route("/v1/products/admin", get(product_handler::list_products_admin))
        .route(
            "/v1/products/{id}",
            get(product_handler::get_product)
                .put(product_handler::update_product)
                .patch(product_handler::patch_product)
                .delete(product_handler::delete_product),
        )
        // Product Photos & Uploads
        .route("/v1/product-photos", post(product_handler::create_photo))
        .route(
            "/v1/product-photos/{id}",
            put(product_handler::update_photo).delete(product_handler::delete_photo),
        )
        .route(
            "/v1/product-photos/product/{productId}",
            get(product_handler::list_photos_by_product),
        )
        .route("/v1/upload/product", post(upload_handler::upload_product_image))
        // Branches & Warehouses
        .route("/v1/branches", get(branch_handler::list_branches).post(branch_handler::create_branch))
        .route(
            "/v1/branches/{id}",
            get(branch_handler::get_branch)
                .put(branch_handler::update_branch)
                .delete(branch_handler::delete_branch),
        )
        .route("/v1/warehouses", get(branch_handler::list_warehouses).post(branch_handler::create_warehouse))
        // Stock & Inventory
        .route(
            "/v1/stock-balances",
            get(stock_handler::list_stock_balances).post(stock_handler::update_stock_balance),
        )
        .route("/v1/stock-mutations", get(stock_handler::list_stock_mutations))
        // Vouchers
        .route("/v1/vouchers", get(voucher_handler::list_vouchers).post(voucher_handler::create_voucher))
        .route(
            "/v1/vouchers/{id}",
            put(voucher_handler::update_voucher).delete(voucher_handler::delete_voucher),
        )
        .route("/v1/vouchers/code/{code}", get(voucher_handler::get_voucher_by_code))
        // Cashier Shifts
        .route("/v1/shifts", get(shift_handler::list_shifts))
        .route("/v1/shifts/active", get(shift_handler::get_active_shift))
        .route("/v1/shifts/open", post(shift_handler::open_shift))
        .route("/v1/shifts/close", post(shift_handler::close_shift))
        // Orders (POS Checkout) & Dashboard
        .route("/v1/orders", get(order_handler::list_orders).post(order_handler::create_order))
        .route("/v1/orders/{id}", get(order_handler::get_order))
        .route("/v1/dashboard/summary", get(order_handler::get_dashboard_summary));

    Router::new()
        .nest("/api", api_routes)
        .nest_service("/uploads", ServeDir::new(&state.config.upload_dir))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn health_check() -> &'static str {
    "Lion POS API is healthy and running!"
}
