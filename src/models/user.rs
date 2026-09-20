use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: String,
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub fullname: String,
    pub phone: Option<String>,
    pub avatar: Option<String>,
    pub branch_id: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Role {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Permission {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub module: String,
    pub action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user id
    pub username: String,
    pub roles: Vec<String>,
    pub perms: Vec<String>,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthResponseData {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct StandardResponse<T> {
    pub success: bool,
    pub message: Option<String>,
    pub data: T,
}

impl<T> StandardResponse<T> {
    pub fn success(data: T, message: Option<&str>) -> Self {
        Self {
            success: true,
            message: message.map(|m| m.to_string()),
            data,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserMeResponse {
    pub id: String,
    pub username: String,
    pub fullname: String,
    pub avatar: Option<String>,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
    pub plan: String,
    pub branch_id: Option<String>,
    pub branch_name: Option<String>,
    pub warehouse_id: Option<String>,
    pub warehouse_name: Option<String>,
    pub partner_id: Option<String>,
    pub partner_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
    pub fullname: String,
    pub phone: Option<String>,
    pub avatar: Option<String>,
    pub branch_id: Option<String>,
    pub role_ids: Option<Vec<String>>,
}
