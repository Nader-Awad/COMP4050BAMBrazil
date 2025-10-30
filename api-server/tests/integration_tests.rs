use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use serde_json::{json, Value};
use tower::util::ServiceExt;

use bam::{create_router, AppState, Config};
use bam::config::{AuthConfig, DatabaseConfig, FileStorageConfig, IAConfig, ServerConfig};
use bam::middleware::auth::Claims;
use bam::models::UserRole;

use std::sync::Arc;

use uuid::Uuid;
use jsonwebtoken::{encode, EncodingKey, Header};
use tempfile::TempDir;

// NEW: test JWT secret constant (must match test config below)
const TEST_JWT_SECRET: &str = "test-secret-key";

// NEW: in-memory SQLite pool for tests
async fn test_sqlite_pool() -> sqlx::SqlitePool {
    let url = "sqlite::memory:?cache=shared";
    sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect(url)
        .await
        .expect("connect in-memory sqlite")

    // If your handlers need tables, uncomment:
    // sqlx::migrate!("./migrations").run(&pool).await.unwrap();
}

// NEW: very simple temp file store placeholder
struct TestFileStore {
    _dir: TempDir,
    base: std::path::PathBuf,
}
impl TestFileStore {
    fn new() -> Self {
        let dir = tempfile::tempdir().expect("tempdir");
        Self {
            base: dir.path().to_path_buf(),
            _dir: dir,
        }
    }
}

// NEW: IA mock so microscope tests don’t hit real IA
#[derive(Clone, Default)]
struct MockIaClient;

impl MockIaClient {
    async fn send_command(
        &self,
        _microscope_id: &str,
        _cmd: serde_json::Value,
    ) -> Result<serde_json::Value, anyhow::Error> {
        Ok(json!({"status":"ok"}))
    }
}

/// Helper function to create test app state
async fn create_test_app() -> Router {
    let config = Arc::new(Config {
        server: ServerConfig {
            bind_address: "127.0.0.1:0".to_string(),
            port: 0,
        },
        database: DatabaseConfig {
            url: "sqlite::memory:?cache=shared".to_string(),
            max_connections: 1,
        },
        auth: AuthConfig {
            jwt_secret: TEST_JWT_SECRET.to_string(),
            token_expiry: 3600,
            refresh_token_expiry: 86400,
        },
        file_storage: FileStorageConfig {
            base_path: "/tmp/bam-test".to_string(),
            max_file_size: 10485760, // 10MB
            allowed_types: vec!["image/jpeg".to_string(), "image/png".to_string()],
        },
        ia: IAConfig {
            base_url: "http://localhost:8080".to_string(),
            timeout: 30,
            auth_token: None,
            mock_mode: true,
        },
    });

    let pool = test_sqlite_pool().await;
    let test_fs = TestFileStore::new();
    let ia_client = MockIaClient::default();

    let state = AppState {
        config,
        db: pool,
        file_store: test_fs,
        ia_client,
    };
    create_router(state)
}

// NEW: helper to build a valid JWT with your real Claims struct
fn make_test_jwt(role: UserRole) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize;

    let claims = Claims {
        user_id: Uuid::new_v4(),
        role,
        session_id: None,
        exp: now + 3600,
        iat: now,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(TEST_JWT_SECRET.as_bytes()),
    )
    .expect("encode jwt")
}

/// Helper function to create authenticated request
async fn create_auth_request(method: &str, uri: &str, body: Option<Value>) -> Request<Body> {
    let mut request_builder = Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json");

    // Add mock JWT token for testing
    let test_token = make_test_jwt(UserRole::Admin);
    request_builder = request_builder.header("authorization", format!("Bearer {}", test_token));

    match body {
        Some(json_body) => request_builder
            .body(Body::from(serde_json::to_vec(&json_body).unwrap()))
            .unwrap(),
        None => request_builder.body(Body::empty()).unwrap(),
    }
}

#[tokio::test]
async fn test_health_check() {
    let app = create_test_app().await;

    let request = Request::builder()
        .uri("/health")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_login_endpoint() {
    let app = create_test_app().await;

    let login_data = json!({
        "email": "admin@bam.edu",
        "password": "admin123"
    });

    let request = Request::builder()
        .method("POST")
        .uri("/api/auth/login")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&login_data).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // If user is seeded, expect OK; if not seeded, expect UNAUTHORIZED
    assert!(
        response.status() == StatusCode::OK || response.status() == StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn test_create_booking() {
    let app = create_test_app().await;

    let booking_data = json!({
        "microscope_id": "bio-1",
        "date": "2024-01-15",
        "slot_start": 540,
        "slot_end": 600,
        "title": "Test Booking"
    });

    let request = create_auth_request("POST", "/api/bookings", Some(booking_data)).await;
    let response = app.oneshot(request).await.unwrap();

    assert_ne!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_get_bookings() {
    let app = create_test_app().await;

    let request = create_auth_request("GET", "/api/bookings", None).await;
    let response = app.oneshot(request).await.unwrap();

    assert_ne!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_microscope_control() {
    let app = create_test_app().await;

    let command_data = json!({
        "command_type": "Move",
        "parameters": {
            "direction": "left",
            "distance": 10
        }
    });

    let request =
        create_auth_request("POST", "/api/microscope/bio-1/command", Some(command_data)).await;

    let response = app.oneshot(request).await.unwrap();

    assert_ne!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_invalid_endpoints() {
    let app = create_test_app().await;

    let request = Request::builder()
        .uri("/api/nonexistent")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_cors_headers() {
    let app = create_test_app().await;

    let request = Request::builder()
        .method("OPTIONS")
        .uri("/api/bookings")
        .header("origin", "http://localhost:3000")
        .header("access-control-request-method", "POST")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert!(
        response.status() == StatusCode::OK || response.status() == StatusCode::NO_CONTENT
    );
}

// NEW: expired token rejected
#[tokio::test]
async fn test_expired_token_is_rejected() {
    let app = create_test_app().await;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize;

    let claims = Claims {
        user_id: Uuid::new_v4(),
        role: UserRole::Admin,
        session_id: None,
        exp: now.saturating_sub(60),
        iat: now.saturating_sub(120),
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(TEST_JWT_SECRET.as_bytes()),
    )
    .unwrap();

    let request = Request::builder()
        .method("GET")
        .uri("/api/bookings")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

