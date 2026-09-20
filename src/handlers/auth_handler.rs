use axum::{
    extract::{Query, State},
    Json,
};
use chrono::Utc;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    errors::AppError,
    middleware::auth::{create_jwt, decode_jwt, AuthenticatedUser},
    models::user::{AuthResponseData, LoginRequest, StandardResponse, User, UserMeResponse},
    state::AppState,
};

#[derive(Deserialize)]
pub struct RefreshQuery {
    #[serde(rename = "refreshToken")]
    pub refresh_token: Option<String>,
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<StandardResponse<AuthResponseData>>, AppError> {
    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE username = ? AND is_active = 1",
    )
    .bind(&payload.username)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::Unauthorized("Username atau password salah".to_string()))?;

    let is_valid = bcrypt::verify(&payload.password, &user.password_hash)?;
    if !is_valid {
        return Err(AppError::Unauthorized("Username atau password salah".to_string()));
    }

    // Fetch user roles
    let roles: Vec<String> = sqlx::query_scalar(
        r#"
        SELECT r.name FROM roles r
        JOIN user_roles ur ON ur.role_id = r.id
        WHERE ur.user_id = ?
        "#,
    )
    .bind(&user.id)
    .fetch_all(&state.pool)
    .await?;

    // Fetch user permissions
    let perms: Vec<String> = sqlx::query_scalar(
        r#"
        SELECT DISTINCT p.slug FROM permissions p
        JOIN role_permissions rp ON rp.permission_id = p.id
        JOIN user_roles ur ON ur.role_id = rp.role_id
        WHERE ur.user_id = ?
        "#,
    )
    .bind(&user.id)
    .fetch_all(&state.pool)
    .await?;

    let access_token = create_jwt(
        &user.id,
        &user.username,
        roles.clone(),
        perms.clone(),
        &state.config.jwt_secret,
        state.config.jwt_expiration_hours,
    )?;

    let refresh_token = create_jwt(
        &user.id,
        &user.username,
        roles,
        perms,
        &state.config.jwt_secret,
        state.config.jwt_refresh_expiration_days * 24,
    )?;

    // Store refresh token in db
    let now = Utc::now().to_rfc3339();
    let token_id = Uuid::new_v4().to_string();
    let expires_at = (Utc::now() + chrono::Duration::days(state.config.jwt_refresh_expiration_days)).to_rfc3339();

    sqlx::query(
        "INSERT INTO tokens (id, user_id, token_type, token, expires_at, created_at) VALUES (?, ?, 'REFRESH', ?, ?, ?)"
    )
    .bind(&token_id)
    .bind(&user.id)
    .bind(&refresh_token)
    .bind(&expires_at)
    .bind(&now)
    .execute(&state.pool)
    .await?;

    Ok(Json(StandardResponse::success(
        AuthResponseData {
            access_token,
            refresh_token,
        },
        Some("Login berhasil"),
    )))
}

pub async fn me(
    State(state): State<AppState>,
    AuthenticatedUser(claims): AuthenticatedUser,
) -> Result<Json<StandardResponse<UserMeResponse>>, AppError> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
        .bind(&claims.sub)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Pengguna tidak ditemukan".to_string()))?;

    let roles: Vec<String> = sqlx::query_scalar(
        r#"
        SELECT r.name FROM roles r
        JOIN user_roles ur ON ur.role_id = r.id
        WHERE ur.user_id = ?
        "#,
    )
    .bind(&user.id)
    .fetch_all(&state.pool)
    .await?;

    let permissions: Vec<String> = sqlx::query_scalar(
        r#"
        SELECT DISTINCT p.slug FROM permissions p
        JOIN role_permissions rp ON rp.permission_id = p.id
        JOIN user_roles ur ON ur.role_id = rp.role_id
        WHERE ur.user_id = ?
        "#,
    )
    .bind(&user.id)
    .fetch_all(&state.pool)
    .await?;

    // Optional branch details
    let branch_name: Option<String> = if let Some(ref bid) = user.branch_id {
        sqlx::query_scalar("SELECT name FROM branches WHERE id = ?")
            .bind(bid)
            .fetch_optional(&state.pool)
            .await?
    } else {
        None
    };

    let warehouse_info: Option<(String, String)> = if let Some(ref bid) = user.branch_id {
        sqlx::query_as::<_, (String, String)>("SELECT id, name FROM warehouses WHERE branch_id = ? LIMIT 1")
            .bind(bid)
            .fetch_optional(&state.pool)
            .await?
    } else {
        None
    };

    let (warehouse_id, warehouse_name) = match warehouse_info {
        Some((wid, wname)) => (Some(wid), Some(wname)),
        None => (None, None),
    };

    Ok(Json(StandardResponse::success(
        UserMeResponse {
            id: user.id,
            username: user.username,
            fullname: user.fullname,
            avatar: user.avatar,
            roles,
            permissions,
            plan: "pro".to_string(),
            branch_id: user.branch_id,
            branch_name,
            warehouse_id,
            warehouse_name,
            partner_id: None,
            partner_name: None,
        },
        None,
    )))
}

pub async fn refresh(
    State(state): State<AppState>,
    Query(query): Query<RefreshQuery>,
) -> Result<Json<StandardResponse<AuthResponseData>>, AppError> {
    let token_str = query
        .refresh_token
        .ok_or_else(|| AppError::BadRequest("Parameter refreshToken wajib diisi".to_string()))?;

    let claims = decode_jwt(&token_str, &state.config.jwt_secret)
        .map_err(|e| AppError::Unauthorized(format!("Refresh token tidak valid: {}", e)))?;

    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ? AND is_active = 1")
        .bind(&claims.sub)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| AppError::Unauthorized("Pengguna tidak aktif atau tidak ditemukan".to_string()))?;

    let roles: Vec<String> = sqlx::query_scalar(
        r#"
        SELECT r.name FROM roles r
        JOIN user_roles ur ON ur.role_id = r.id
        WHERE ur.user_id = ?
        "#,
    )
    .bind(&user.id)
    .fetch_all(&state.pool)
    .await?;

    let perms: Vec<String> = sqlx::query_scalar(
        r#"
        SELECT DISTINCT p.slug FROM permissions p
        JOIN role_permissions rp ON rp.permission_id = p.id
        JOIN user_roles ur ON ur.role_id = rp.role_id
        WHERE ur.user_id = ?
        "#,
    )
    .bind(&user.id)
    .fetch_all(&state.pool)
    .await?;

    let new_access_token = create_jwt(
        &user.id,
        &user.username,
        roles,
        perms,
        &state.config.jwt_secret,
        state.config.jwt_expiration_hours,
    )?;

    Ok(Json(StandardResponse::success(
        AuthResponseData {
            access_token: new_access_token,
            refresh_token: token_str,
        },
        Some("Token berhasil diperbarui"),
    )))
}

pub async fn logout(
    State(state): State<AppState>,
    Query(query): Query<RefreshQuery>,
) -> Result<Json<StandardResponse<bool>>, AppError> {
    if let Some(token) = query.refresh_token {
        sqlx::query("DELETE FROM tokens WHERE token = ?")
            .bind(&token)
            .execute(&state.pool)
            .await?;
    }

    Ok(Json(StandardResponse::success(true, Some("Logout berhasil"))))
}
