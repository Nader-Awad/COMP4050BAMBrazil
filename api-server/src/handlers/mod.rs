use axum::response::Json;
use utoipa;

use crate::models::ApiResponse;

pub mod auth;
pub mod bookings;
pub mod images;
pub mod microscope;
pub mod sessions;

/// Health check endpoint
#[utoipa::path(
    get,
    path = "/health",
    tag = "health",
    responses(
        (status = 200, description = "Server is healthy", body = ApiResponse<String>)
    )
)]
pub async fn health_check() -> Json<ApiResponse<&'static str>> {
    Json(ApiResponse::success("API server is healthy"))
}
