use serde::{Deserialize, Serialize};
use std::{env, path::Path};

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub auth: AuthConfig,
    pub file_storage: FileStorageConfig,
    pub ia: IAConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub bind_address: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub token_expiry: u64,         // in seconds
    pub refresh_token_expiry: u64, // in seconds
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileStorageConfig {
    pub base_path: String,
    pub max_file_size: u64, // in bytes
    pub allowed_types: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IAConfig {
    pub base_url: String,
    pub timeout: u64, // in seconds
    pub auth_token: Option<String>,
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        if dotenvy::dotenv().is_err() {
            if Path::new(".env.example").exists() {
                dotenvy::from_filename(".env.example").ok();
            }
        }

        let server = ServerConfig {
            bind_address: env::var("BIND_ADDRESS").unwrap_or_else(|_| "0.0.0.0:3000".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()?,
        };

        let database = DatabaseConfig {
            url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            max_connections: env::var("DATABASE_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "10".to_string())
                .parse()?,
        };

        let auth = AuthConfig {
            jwt_secret: env::var("JWT_SECRET").expect("JWT_SECRET must be set"),
            token_expiry: env::var("TOKEN_EXPIRY")
                .unwrap_or_else(|_| "3600".to_string()) // 1 hour
                .parse()?,
            refresh_token_expiry: env::var("REFRESH_TOKEN_EXPIRY")
                .unwrap_or_else(|_| "604800".to_string()) // 1 week
                .parse()?,
        };

        let file_storage = FileStorageConfig {
            base_path: env::var("FILE_STORAGE_PATH").unwrap_or_else(|_| "./uploads".to_string()),
            max_file_size: env::var("MAX_FILE_SIZE")
                .unwrap_or_else(|_| "52428800".to_string()) // 50MB
                .parse()?,
            allowed_types: vec![
                "image/jpeg".to_string(),
                "image/png".to_string(),
                "image/tiff".to_string(),
                "image/bmp".to_string(),
            ],
        };

        let ia = IAConfig {
            base_url: env::var("IA_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:8080".to_string()),
            timeout: env::var("IA_TIMEOUT")
                .unwrap_or_else(|_| "30".to_string())
                .parse()?,
            auth_token: env::var("IA_AUTH_TOKEN").ok(),
        };

        Ok(Config {
            server,
            database,
            auth,
            file_storage,
            ia,
        })
    }
}
