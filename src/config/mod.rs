use std::env;

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub port: u16,
    pub host: String,
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_expiration_hours: i64,
    pub jwt_refresh_expiration_days: i64,
    pub upload_dir: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        let port = env::var("PORT")
            .unwrap_or_else(|_| "8090".to_string())
            .parse::<u16>()
            .unwrap_or(8090);

        let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());

        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "sqlite:lions_pos.db?mode=rwc".to_string());

        let jwt_secret = env::var("JWT_SECRET")
            .unwrap_or_else(|_| "lions_pos_super_secret_jwt_key_2026_change_in_production".to_string());

        let jwt_expiration_hours = env::var("JWT_EXPIRATION_HOURS")
            .unwrap_or_else(|_| "24".to_string())
            .parse::<i64>()
            .unwrap_or(24);

        let jwt_refresh_expiration_days = env::var("JWT_REFRESH_EXPIRATION_DAYS")
            .unwrap_or_else(|_| "30".to_string())
            .parse::<i64>()
            .unwrap_or(30);

        let upload_dir = env::var("UPLOAD_DIR").unwrap_or_else(|_| "./uploads".to_string());

        Self {
            port,
            host,
            database_url,
            jwt_secret,
            jwt_expiration_hours,
            jwt_refresh_expiration_days,
            upload_dir,
        }
    }
}
