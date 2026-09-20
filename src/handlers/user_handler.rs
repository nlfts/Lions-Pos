use axum::{
    extract::State,
    Json,
};
use chrono::Utc;
use serde_json::json;
use uuid::Uuid;

use crate::{
    errors::AppError,
    models::user::{CreateUserRequest, Permission, Role, StandardResponse, User},
    state::AppState,
};

pub async fn list_users(
    State(state): State<AppState>,
) -> Result<Json<StandardResponse<Vec<serde_json::Value>>>, AppError> {
    let users = sqlx::query_as::<_, User>("SELECT * FROM users ORDER BY created_at DESC")
        .fetch_all(&state.pool)
        .await?;

    let mut result = Vec::new();
    for u in users {
        let roles: Vec<Role> = sqlx::query_as::<_, Role>(
            r#"
            SELECT r.* FROM roles r
            JOIN user_roles ur ON ur.role_id = r.id
            WHERE ur.user_id = ?
            "#,
        )
        .bind(&u.id)
        .fetch_all(&state.pool)
        .await?;

        result.push(json!({
            "id": u.id,
            "username": u.username,
            "fullname": u.fullname,
            "phone": u.phone,
            "avatar": u.avatar,
            "branchId": u.branch_id,
            "isActive": u.is_active,
            "roles": roles,
            "createdAt": u.created_at,
            "updatedAt": u.updated_at,
        }));
    }

    Ok(Json(StandardResponse::success(result, None)))
}

pub async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<Json<StandardResponse<serde_json::Value>>, AppError> {
    let exists: Option<String> = sqlx::query_scalar("SELECT id FROM users WHERE username = ?")
        .bind(&payload.username)
        .fetch_optional(&state.pool)
        .await?;

    if exists.is_some() {
        return Err(AppError::BadRequest("Username sudah digunakan".to_string()));
    }

    let password_hash = bcrypt::hash(&payload.password, bcrypt::DEFAULT_COST)?;
    let user_id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    sqlx::query(
        r#"
        INSERT INTO users (id, username, password_hash, fullname, phone, avatar, branch_id, is_active, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, 1, ?, ?);
        "#,
    )
    .bind(&user_id)
    .bind(&payload.username)
    .bind(&password_hash)
    .bind(&payload.fullname)
    .bind(&payload.phone)
    .bind(&payload.avatar)
    .bind(&payload.branch_id)
    .bind(&now)
    .bind(&now)
    .execute(&state.pool)
    .await?;

    if let Some(role_ids) = payload.role_ids {
        for rid in role_ids {
            sqlx::query("INSERT OR IGNORE INTO user_roles (user_id, role_id) VALUES (?, ?)")
                .bind(&user_id)
                .bind(&rid)
                .execute(&state.pool)
                .await?;
        }
    }

    Ok(Json(StandardResponse::success(
        json!({
            "id": user_id,
            "username": payload.username,
            "fullname": payload.fullname,
        }),
        Some("Pengguna berhasil dibuat"),
    )))
}

pub async fn list_roles(
    State(state): State<AppState>,
) -> Result<Json<StandardResponse<Vec<serde_json::Value>>>, AppError> {
    let roles = sqlx::query_as::<_, Role>("SELECT * FROM roles ORDER BY name ASC")
        .fetch_all(&state.pool)
        .await?;

    let mut result = Vec::new();
    for r in roles {
        let perms: Vec<Permission> = sqlx::query_as::<_, Permission>(
            r#"
            SELECT p.* FROM permissions p
            JOIN role_permissions rp ON rp.permission_id = p.id
            WHERE rp.role_id = ?
            "#,
        )
        .bind(&r.id)
        .fetch_all(&state.pool)
        .await?;

        result.push(json!({
            "id": r.id,
            "name": r.name,
            "slug": r.slug,
            "description": r.description,
            "permissions": perms,
        }));
    }

    Ok(Json(StandardResponse::success(result, None)))
}

pub async fn list_permissions(
    State(state): State<AppState>,
) -> Result<Json<StandardResponse<Vec<Permission>>>, AppError> {
    let perms = sqlx::query_as::<_, Permission>("SELECT * FROM permissions ORDER BY module ASC, action ASC")
        .fetch_all(&state.pool)
        .await?;

    Ok(Json(StandardResponse::success(perms, None)))
}

pub async fn get_role_permissions_map(
    State(state): State<AppState>,
) -> Result<Json<StandardResponse<serde_json::Value>>, AppError> {
    let perms = sqlx::query_as::<_, Permission>("SELECT * FROM permissions ORDER BY module ASC, action ASC")
        .fetch_all(&state.pool)
        .await?;

    let mut map = std::collections::HashMap::<String, Vec<Permission>>::new();
    for p in perms {
        map.entry(p.module.clone()).or_default().push(p);
    }

    Ok(Json(StandardResponse::success(json!(map), None)))
}
