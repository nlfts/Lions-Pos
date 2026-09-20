use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use lions_pos::{config::AppConfig, database::init_db, routes::create_router, state::AppState};
use serde_json::{json, Value};
use tower::ServiceExt;

async fn setup_app() -> axum::Router {
    let mut config = AppConfig::from_env();
    // Use in-memory SQLite with a unique shared name for tests
    let test_db_url = format!("sqlite:file:test_{}?mode=memory&cache=shared", uuid::Uuid::new_v4());
    config.database_url = test_db_url;

    let pool = init_db(&config.database_url).await.unwrap();
    let state = AppState::new(pool, config);
    create_router(state)
}

#[tokio::test]
async fn test_health_check() {
    let app = setup_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let text = String::from_utf8(body.to_vec()).unwrap();
    assert!(text.contains("Lion POS API is healthy"));
}

#[tokio::test]
async fn test_auth_login_and_me() {
    let app = setup_app().await;

    // 1. Login with seeded admin
    let login_payload = json!({
        "username": "admin",
        "password": "password123"
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/login")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&login_payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body_json: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body_json["success"], true);

    let access_token = body_json["data"]["accessToken"].as_str().unwrap();
    assert!(!access_token.is_empty());

    // 2. Fetch /api/v1/auth/me with Bearer token
    let me_res = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/auth/me")
                .header("Authorization", format!("Bearer {}", access_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(me_res.status(), StatusCode::OK);
    let me_body_bytes = me_res.into_body().collect().await.unwrap().to_bytes();
    let me_json: Value = serde_json::from_slice(&me_body_bytes).unwrap();
    assert_eq!(me_json["data"]["username"], "admin");
    assert_eq!(me_json["data"]["roles"][0], "ADMIN");
}

#[tokio::test]
async fn test_categories_and_products_crud() {
    let app = setup_app().await;

    // 1. List seeded categories
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/categories")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    assert!(json["data"].as_array().unwrap().len() >= 4);

    // 2. Create a new product
    let new_prod = json!({
        "name": "Es Teh Manis Segar",
        "sku": "TEH-001",
        "base_price": 6000.0,
        "cost_price": 2000.0,
        "track_stock": true,
        "is_active": true
    });

    let prod_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/products")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&new_prod).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(prod_res.status(), StatusCode::OK);
    let prod_bytes = prod_res.into_body().collect().await.unwrap().to_bytes();
    let prod_json: Value = serde_json::from_slice(&prod_bytes).unwrap();
    assert_eq!(prod_json["data"]["name"], "Es Teh Manis Segar");
    assert_eq!(prod_json["data"]["sku"], "TEH-001");
}

#[tokio::test]
async fn test_pos_order_checkout_flow() {
    let app = setup_app().await;

    // 1. Check stock before order for prod-1 in branch-main
    let stock_res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/stock-balances?branchId=branch-main")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(stock_res.status(), StatusCode::OK);
    let stock_bytes = stock_res.into_body().collect().await.unwrap().to_bytes();
    let stock_json: Value = serde_json::from_slice(&stock_bytes).unwrap();
    let initial_stock = stock_json["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["product_id"] == "prod-1")
        .unwrap()["quantity"]
        .as_i64()
        .unwrap();

    assert_eq!(initial_stock, 50);

    // 2. Perform POS Checkout Order
    let order_payload = json!({
        "branchId": "branch-main",
        "orderNumber": "ORD-TEST-001",
        "buyerName": "Pelanggan Setia",
        "items": [
            {
                "productId": "prod-1",
                "qty": 3,
                "unitPrice": 18000.0,
                "itemDiscountAmount": 0.0,
                "itemNote": "Sedikit gula"
            }
        ],
        "payment": {
            "method": "CASH",
            "amount": 54000.0,
            "cashTendered": 60000.0,
            "changeDue": 6000.0
        }
    });

    let order_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/orders")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&order_payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(order_res.status(), StatusCode::OK);
    let order_bytes = order_res.into_body().collect().await.unwrap().to_bytes();
    let order_json: Value = serde_json::from_slice(&order_bytes).unwrap();
    assert_eq!(order_json["data"]["orderNumber"], "ORD-TEST-001");
    assert_eq!(order_json["data"]["total"], 54000.0);
    assert_eq!(order_json["data"]["payments"][0]["method"], "CASH");

    // 3. Verify stock after order was deducted from 50 to 47
    let stock_after_res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/stock-balances?branchId=branch-main")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let stock_after_bytes = stock_after_res.into_body().collect().await.unwrap().to_bytes();
    let stock_after_json: Value = serde_json::from_slice(&stock_after_bytes).unwrap();
    let new_stock = stock_after_json["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["product_id"] == "prod-1")
        .unwrap()["quantity"]
        .as_i64()
        .unwrap();

    assert_eq!(new_stock, 47);

    // 4. Verify Dashboard Summary
    let dash_res = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/dashboard/summary")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(dash_res.status(), StatusCode::OK);
    let dash_bytes = dash_res.into_body().collect().await.unwrap().to_bytes();
    let dash_json: Value = serde_json::from_slice(&dash_bytes).unwrap();
    assert_eq!(dash_json["data"]["todayTransactions"], 1);
    assert_eq!(dash_json["data"]["todaySales"], 54000.0);
}
