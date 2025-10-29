use std::str::FromStr;

use axum::{
    extract::{Request, State},
    http::{HeaderMap, Method, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{models::UserRole, AppState};

/// JWT Claims structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    #[serde(skip)]
    pub user_id: Uuid,
    pub role: UserRole,
    pub session_id: Option<Uuid>,
    pub exp: usize, // Expiration time
    pub iat: usize, // Issued at
}

/// Authentication middleware
pub async fn auth_middleware(
    State(state): State<AppState>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let path = request.uri().path();

    if request.method() == Method::OPTIONS {
        return Ok(next.run(request).await);
    }

    // Skip authentication for health check and auth endpoints
    if path == "/health" || path.starts_with("/api/auth") || path.starts_with("/swagger") {
        return Ok(next.run(request).await);
    }

    // Extract JWT token from Authorization header
    let token = extract_token_from_headers(&headers).ok_or(StatusCode::UNAUTHORIZED)?;

    // Validate and decode JWT token
    let claims = validate_jwt_token(&token, &state.config.auth.jwt_secret)
        .map_err(|_| {
            tracing::warn!("Unauthorized: Invalid or expired token");
            StatusCode::UNAUTHORIZED
        })?;

    // Add user information to request extensions
    request.extensions_mut().insert(claims);

    Ok(next.run(request).await)
}

/// Extract JWT token from Authorization header
fn extract_token_from_headers(headers: &HeaderMap) -> Option<String> {
    let auth_header = headers.get("authorization")?;
    let auth_str = auth_header.to_str().ok()?;

    if auth_str.starts_with("Bearer ") {
        Some(auth_str[7..].to_string())
    } else {
        None
    }
}

/// Validate JWT token and extract claims
fn validate_jwt_token(token: &str, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let decoding_key = DecodingKey::from_secret(secret.as_ref());
    let validation = Validation::default();

    let mut token_data = decode::<Claims>(token, &decoding_key, &validation)?;

    token_data.claims.user_id = Uuid::parse_str(&token_data.claims.sub).map_err(|_| jsonwebtoken::errors::ErrorKind::InvalidToken)?.into();
    Ok(token_data.claims)
}


/// Generate JWT token for user
pub fn generate_jwt_token(
    user_id: Uuid,
    role: UserRole,
    session_id: Option<Uuid>,
    secret: &str,
    expiry: u64,
) -> Result<String, jsonwebtoken::errors::Error> {
    use jsonwebtoken::{encode, EncodingKey, Header};

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();

    let claims = Claims {
        sub: user_id.to_string(),
        role,
        session_id,
        exp: (now + expiry) as usize,
        iat: now as usize,
    };

    let encoding_key = EncodingKey::from_secret(secret.as_ref());
    encode(&Header::default(), &claims, &encoding_key)
}

/// Middleware to require admin role
pub async fn require_admin(request: Request, next: Next) -> Result<Response, StatusCode> {
    let claims = request
        .extensions()
        .get::<Claims>()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if claims.role != UserRole::Admin {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(request).await)
}

/// Middleware to require teacher or admin role
pub async fn require_teacher_or_admin(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let claims = request
        .extensions()
        .get::<Claims>()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    match claims.role {
        UserRole::Teacher | UserRole::Admin => Ok(next.run(request).await),
        _ => Err(StatusCode::FORBIDDEN),
    }
}
