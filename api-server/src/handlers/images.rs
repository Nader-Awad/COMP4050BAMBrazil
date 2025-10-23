use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{Json, Response},
    Extension,
};
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::{
    middleware::auth::Claims,
    models::{ApiResponse, Image, UserRole},
    AppState,
};

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct ImageQuery {
    pub session_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    #[schema(example = "cell,biology")]
    pub tags: Option<String>,
    #[schema(example = "2024-01-01", format = "date")]
    pub date_from: Option<String>,
    #[schema(example = "2024-01-31", format = "date")]
    pub date_to: Option<String>,
    #[schema(example = 1)]
    pub page: Option<u64>,
    #[schema(example = 20)]
    pub limit: Option<u64>,
}

/// Get image metadata by ID
#[utoipa::path(
    get,
    path = "/api/images/{id}",
    tag = "images",
    params(
        ("id" = Uuid, Path, description = "Image ID")
    ),
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Image metadata", body = ApiResponse<Image>),
        (status = 403, description = "Access denied", body = ApiResponse<String>),
        (status = 404, description = "Image not found", body = ApiResponse<String>),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn get_image(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(image_id): Path<Uuid>,
) -> Result<Json<ApiResponse<Image>>, StatusCode> {
    let image = state
        .db
        .get_image_by_id(image_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Check permissions based on role and ownership
    if !can_access_image(&state, &claims, &image).await {
        return Ok(Json(ApiResponse::error("Access denied".to_string())));
    }

    Ok(Json(ApiResponse::success(image)))
}

/// Serve image file content
#[utoipa::path(
    get,
    path = "/api/images/{id}/file",
    tag = "images",
    params(
        ("id" = Uuid, Path, description = "Image ID")
    ),
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Image file content", content_type = "image/jpeg"),
        (status = 403, description = "Access denied"),
        (status = 404, description = "Image not found")
    )
)]
pub async fn serve_image_file(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(image_id): Path<Uuid>,
) -> Result<Response, StatusCode> {
    let image = state
        .db
        .get_image_by_id(image_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Check permissions
    if !can_access_image(&state, &claims, &image).await {
        return Err(StatusCode::FORBIDDEN);
    }

    // Read actual file from storage
    use axum::response::IntoResponse;

    let file_contents = state
        .file_store
        .read_file(&image.file_path)
        .await
        .map_err(|e| {
            tracing::error!("Failed to read file {}: {}", image.file_path, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let mut headers = HeaderMap::new();
    headers.insert("content-type", image.content_type.parse().unwrap());
    headers.insert(
        "content-length",
        file_contents.len().to_string().parse().unwrap(),
    );
    headers.insert(
        "content-disposition",
        format!("inline; filename=\"{}\"", image.filename)
            .parse()
            .unwrap(),
    );

    Ok((headers, file_contents).into_response())
}

/// Get latest image for a session
#[utoipa::path(
    get,
    path = "/api/sessions/{session_id}/images/latest",
    tag = "images",
    params(
        ("session_id" = Uuid, Path, description = "Session ID")
    ),
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Latest image for session", body = ApiResponse<Image>),
        (status = 403, description = "Access denied to session", body = ApiResponse<String>),
        (status = 404, description = "No images found for session", body = ApiResponse<String>),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn get_latest_image_for_session(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(session_id): Path<Uuid>,
) -> Result<Json<ApiResponse<Image>>, StatusCode> {
    // Check if user has access to this session
    let session = state
        .db
        .get_session_by_id(session_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Permission checking based on user role
    match claims.role {
        UserRole::Student => {
            if session.user_id != claims.user_id {
                return Ok(Json(ApiResponse::error(
                    "Access denied - can only view own sessions".to_string(),
                )));
            }
        }
        UserRole::Teacher | UserRole::Admin => {
            // Teachers and admins can access all sessions
        }
    }

    // Get latest image for the session
    let image = state
        .db
        .get_latest_image_by_session(session_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or_else(|| StatusCode::NOT_FOUND)?;

    Ok(Json(ApiResponse::success(image)))
}

/// Get all images for a session
#[utoipa::path(
    get,
    path = "/api/sessions/{session_id}/images",
    tag = "images",
    params(
        ("session_id" = Uuid, Path, description = "Session ID"),
        ImageQuery
    ),
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "List of images for session", body = ApiResponse<Vec<Image>>),
        (status = 403, description = "Access denied to session", body = ApiResponse<String>),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn get_all_images_for_session(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(session_id): Path<Uuid>,
    Query(_query): Query<ImageQuery>,
) -> Result<Json<ApiResponse<Vec<Image>>>, StatusCode> {
    // Check if user has access to this session
    let session = state
        .db
        .get_session_by_id(session_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Permission checking based on user role
    match claims.role {
        UserRole::Student => {
            if session.user_id != claims.user_id {
                return Ok(Json(ApiResponse::error(
                    "Access denied - can only view own sessions".to_string(),
                )));
            }
        }
        UserRole::Teacher | UserRole::Admin => {
            // Teachers and admins can access all sessions
        }
    }

    // Get all images for the session
    let images = state
        .db
        .get_images_by_session(session_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse::success(images)))
}

/// Get all images for a user
#[utoipa::path(
    get,
    path = "/api/users/{user_id}/images",
    tag = "images",
    params(
        ("user_id" = Uuid, Path, description = "User ID"),
        ImageQuery
    ),
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "List of images for user", body = ApiResponse<Vec<Image>>),
        (status = 403, description = "Access denied - can only access own images unless admin/teacher", body = ApiResponse<String>),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn get_all_images_for_user(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(user_id): Path<Uuid>,
    Query(query): Query<ImageQuery>,
) -> Result<Json<ApiResponse<Vec<Image>>>, StatusCode> {
    // Check permissions - users can only access their own images unless they're admin/teacher
    match claims.role {
        UserRole::Student => {
            if claims.user_id != user_id {
                return Ok(Json(ApiResponse::error("Access denied".to_string())));
            }
        }
        UserRole::Teacher | UserRole::Admin => {
            // Teachers and admins can access any user's images
        }
    }

    // Extract pagination and filtering parameters
    let limit = query.limit.unwrap_or(20).min(100);
    let page = query.page.unwrap_or(1).max(1);
    let offset = (page - 1) * limit;

    // Get images with filtering
    let images = state
        .db
        .get_images_by_user(
            user_id,
            limit,
            offset,
            query.tags,
            query.date_from,
            query.date_to,
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse::success(images)))
}

/// Search images by metadata tags
#[utoipa::path(
    get,
    path = "/api/images/search",
    tag = "images",
    params(ImageQuery),
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Search results", body = ApiResponse<Vec<Image>>),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn search_images(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<ImageQuery>,
) -> Result<Json<ApiResponse<Vec<Image>>>, StatusCode> {
    // Extract pagination parameters
    let limit = query.limit.unwrap_or(20).min(100);
    let page = query.page.unwrap_or(1).max(1);
    let offset = (page - 1) * limit;

    // Role-based filtering - students can only search their own images
    let user_id = match claims.role {
        UserRole::Student => Some(claims.user_id),
        UserRole::Teacher | UserRole::Admin => query.user_id,
    };

    // Search images with filters
    let images = state
        .db
        .search_images(
            user_id,
            query.session_id,
            query.tags,
            query.date_from,
            query.date_to,
            limit,
            offset,
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse::success(images)))
}

/// Check if user can access an image based on role and ownership
async fn can_access_image(state: &AppState, claims: &Claims, image: &Image) -> bool {
    match claims.role {
        UserRole::Admin | UserRole::Teacher => true, // Admin and teachers can access all images
        UserRole::Student => {
            // Students can only access images from their own sessions
            // Check if image belongs to user's session via database lookup
            if let Ok(Some(session)) = state.db.get_session_by_id(image.session_id).await {
                session.user_id == claims.user_id
            } else {
                false
            }
        }
    }
}
