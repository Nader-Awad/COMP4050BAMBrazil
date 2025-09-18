use axum::{extract::State, http::StatusCode, response::Json};
use bcrypt::{hash, verify, DEFAULT_COST};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::{
    middleware::auth::generate_jwt_token,
    models::{ApiResponse, User, UserRole},
    AppState,
};

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct LoginRequest {
    #[validate(email)]
    #[schema(example = "user@example.com")]
    pub email: String,
    #[validate(length(min = 6))]
    #[schema(example = "password123", min_length = 6)]
    pub password: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LoginResponse {
    #[schema(example = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...")]
    pub token: String,
    #[schema(example = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...")]
    pub refresh_token: String,
    pub user: UserInfo,
    #[schema(example = 3600)]
    pub expires_in: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UserInfo {
    pub id: Uuid,
    #[schema(example = "John Doe")]
    pub name: String,
    #[schema(example = "john.doe@example.com")]
    pub email: String,
    pub role: UserRole,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RefreshTokenRequest {
    #[schema(example = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...")]
    pub refresh_token: String,
}

/// User login endpoint
#[utoipa::path(
    post,
    path = "/api/auth/login",
    tag = "authentication",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = ApiResponse<LoginResponse>),
        (status = 400, description = "Invalid credentials or validation error", body = ApiResponse<String>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn login(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<ApiResponse<LoginResponse>>, StatusCode> {
    // Validate request
    if let Err(_) = request.validate() {
        return Ok(Json(ApiResponse::error(
            "Invalid email or password format".to_string(),
        )));
    }

    // TODO: Replace with actual database lookup
    let user = match authenticate_user(&request.email, &request.password).await {
        Ok(user) => user,
        Err(AuthError::InvalidCredentials) => {
            return Ok(Json(ApiResponse::error("Invalid credentials".to_string())));
        }
        Err(AuthError::UserNotFound) => {
            return Ok(Json(ApiResponse::error("User not found".to_string())));
        }
        Err(_) => {
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Generate JWT tokens
    let token = generate_jwt_token(
        user.id,
        user.role.clone(),
        None, // No session ID for login
        &state.config.auth.jwt_secret,
        state.config.auth.token_expiry,
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let refresh_token = generate_jwt_token(
        user.id,
        user.role.clone(),
        None,
        &state.config.auth.jwt_secret,
        state.config.auth.refresh_token_expiry,
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let response = LoginResponse {
        token,
        refresh_token,
        user: UserInfo {
            id: user.id,
            name: user.name,
            email: user.email,
            role: user.role,
        },
        expires_in: state.config.auth.token_expiry,
    };

    Ok(Json(ApiResponse::success(response)))
}

/// User logout endpoint
#[utoipa::path(
    post,
    path = "/api/auth/logout",
    tag = "auth",
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Logout successful", body = ApiResponse<String>)
    )
)]
pub async fn logout(State(_state): State<AppState>) -> Json<ApiResponse<&'static str>> {
    // TODO: Implement token blacklisting in Redis/database
    // For now, just return success - client should discard tokens
    Json(ApiResponse::success("Logged out successfully"))
}

/// Refresh JWT token
#[utoipa::path(
    post,
    path = "/api/auth/refresh",
    tag = "authentication",
    request_body = RefreshTokenRequest,
    responses(
        (status = 200, description = "Token refreshed successfully", body = ApiResponse<LoginResponse>),
        (status = 400, description = "Invalid refresh token", body = ApiResponse<String>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn refresh_token(
    State(state): State<AppState>,
    Json(request): Json<RefreshTokenRequest>,
) -> Result<Json<ApiResponse<LoginResponse>>, StatusCode> {
    // TODO: Validate refresh token and generate new access token
    // This is a simplified implementation

    use crate::middleware::auth::Claims;
    use jsonwebtoken::{decode, DecodingKey, Validation};

    let decoding_key = DecodingKey::from_secret(state.config.auth.jwt_secret.as_ref());
    let validation = Validation::default();

    let token_data = match decode::<Claims>(&request.refresh_token, &decoding_key, &validation) {
        Ok(data) => data,
        Err(_) => {
            return Ok(Json(ApiResponse::error(
                "Invalid refresh token".to_string(),
            )))
        }
    };

    let claims = token_data.claims;

    // Generate new access token
    let new_token = generate_jwt_token(
        claims.user_id,
        claims.role.clone(),
        claims.session_id,
        &state.config.auth.jwt_secret,
        state.config.auth.token_expiry,
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // TODO: Get user info from database
    let user_info = UserInfo {
        id: claims.user_id,
        name: "User".to_string(),              // TODO: Get from DB
        email: "user@example.com".to_string(), // TODO: Get from DB
        role: claims.role,
    };

    let response = LoginResponse {
        token: new_token,
        refresh_token: request.refresh_token, // Keep same refresh token
        user: user_info,
        expires_in: state.config.auth.token_expiry,
    };

    Ok(Json(ApiResponse::success(response)))
}

#[derive(Debug)]
pub enum AuthError {
    InvalidCredentials,
    UserNotFound,
    DatabaseError,
    HashError,
}

/// Authenticate user with email and password
async fn authenticate_user(email: &str, password: &str) -> Result<User, AuthError> {
    // TODO: Replace with actual database query
    // This is a mock implementation for testing

    let mock_users = vec![
        ("admin@bam.edu", "admin123", UserRole::Admin),
        ("teacher@bam.edu", "teacher123", UserRole::Teacher),
        ("student@bam.edu", "student123", UserRole::Student),
    ];

    for (mock_email, mock_password, role) in mock_users {
        if email == mock_email {
            if password == mock_password {
                return Ok(User {
                    id: Uuid::new_v4(),
                    name: format!(
                        "{} User",
                        match role {
                            UserRole::Admin => "Admin",
                            UserRole::Teacher => "Teacher",
                            UserRole::Student => "Student",
                        }
                    ),
                    email: email.to_string(),
                    role,
                    created_at: chrono::Utc::now().into(),
                    updated_at: chrono::Utc::now().into(),
                });
            } else {
                return Err(AuthError::InvalidCredentials);
            }
        }
    }

    Err(AuthError::UserNotFound)
}

/// Hash password for storage
pub fn hash_password(password: &str) -> Result<String, AuthError> {
    hash(password, DEFAULT_COST).map_err(|_| AuthError::HashError)
}

/// Verify password against hash
pub fn verify_password(password: &str, hash: &str) -> Result<bool, AuthError> {
    verify(password, hash).map_err(|_| AuthError::HashError)
}
