use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    models::{ApiResponse, CommandType, MicroscopeCommand},
    services::ia_client::IAClient,
    AppState,
};

/// Send command to microscope via IA system
#[utoipa::path(
    post,
    path = "/api/microscope/{microscope_id}/command",
    tag = "microscope",
    params(
        ("microscope_id" = String, Path, description = "Microscope identifier")
    ),
    request_body = MicroscopeCommand,
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Command sent successfully", body = ApiResponse<CommandResponse>),
        (status = 500, description = "Failed to communicate with microscope"),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn send_command(
    State(state): State<AppState>,
    Path(microscope_id): Path<String>,
    Json(command): Json<MicroscopeCommand>,
) -> Result<Json<ApiResponse<CommandResponse>>, StatusCode> {
    let ia_client = IAClient::new(&state.config.ia);

    match ia_client.send_command(&microscope_id, &command).await {
        Ok(response) => Ok(Json(ApiResponse::success(response))),
        Err(e) => {
            tracing::error!(
                "Failed to send command to microscope {}: {}",
                microscope_id,
                e
            );
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get microscope status
#[utoipa::path(
    get,
    path = "/api/microscope/{microscope_id}/status",
    tag = "microscope",
    params(
        ("microscope_id" = String, Path, description = "Microscope identifier")
    ),
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Microscope status", body = ApiResponse<MicroscopeStatus>),
        (status = 500, description = "Failed to get microscope status"),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn get_status(
    State(state): State<AppState>,
    Path(microscope_id): Path<String>,
) -> Result<Json<ApiResponse<MicroscopeStatus>>, StatusCode> {
    let ia_client = IAClient::new(&state.config.ia);

    match ia_client.get_status(&microscope_id).await {
        Ok(status) => Ok(Json(ApiResponse::success(status))),
        Err(e) => {
            tracing::error!(
                "Failed to get status for microscope {}: {}",
                microscope_id,
                e
            );
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Capture image from microscope
#[utoipa::path(
    post,
    path = "/api/microscope/{microscope_id}/capture",
    tag = "microscope",
    params(
        ("microscope_id" = String, Path, description = "Microscope identifier")
    ),
    request_body = CaptureRequest,
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Image captured successfully", body = ApiResponse<CaptureResponse>),
        (status = 500, description = "Failed to capture image"),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn capture_image(
    State(state): State<AppState>,
    Path(microscope_id): Path<String>,
    Json(request): Json<CaptureRequest>,
) -> Result<Json<ApiResponse<CaptureResponse>>, StatusCode> {
    let ia_client = IAClient::new(&state.config.ia);

    match ia_client.capture_image(&microscope_id, &request).await {
        Ok(response) => {
            // TODO: Save image metadata to database
            // TODO: Store image file in file storage
            Ok(Json(ApiResponse::success(response)))
        }
        Err(e) => {
            tracing::error!(
                "Failed to capture image from microscope {}: {}",
                microscope_id,
                e
            );
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Auto focus microscope
#[utoipa::path(
    post,
    path = "/api/microscope/{microscope_id}/focus",
    tag = "microscope",
    params(
        ("microscope_id" = String, Path, description = "Microscope identifier")
    ),
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Auto focus completed", body = ApiResponse<FocusResponse>),
        (status = 500, description = "Failed to auto focus"),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn auto_focus(
    State(state): State<AppState>,
    Path(microscope_id): Path<String>,
) -> Result<Json<ApiResponse<FocusResponse>>, StatusCode> {
    let ia_client = IAClient::new(&state.config.ia);

    let command = MicroscopeCommand {
        command_type: CommandType::Focus,
        parameters: serde_json::json!({}),
    };

    match ia_client.send_command(&microscope_id, &command).await {
        Ok(response) => Ok(Json(ApiResponse::success(FocusResponse {
            success: response.success,
            focus_score: response.data.get("focus_score").and_then(|v| v.as_f64()),
            message: response.message,
        }))),
        Err(e) => {
            tracing::error!("Failed to auto focus microscope {}: {}", microscope_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Start object tracking
#[utoipa::path(
    post,
    path = "/api/microscope/{microscope_id}/tracking/start",
    tag = "microscope",
    params(
        ("microscope_id" = String, Path, description = "Microscope identifier")
    ),
    request_body = TrackingRequest,
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Object tracking started", body = ApiResponse<TrackingResponse>),
        (status = 500, description = "Failed to start tracking"),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn start_tracking(
    State(state): State<AppState>,
    Path(microscope_id): Path<String>,
    Json(request): Json<TrackingRequest>,
) -> Result<Json<ApiResponse<TrackingResponse>>, StatusCode> {
    let ia_client = IAClient::new(&state.config.ia);

    let command = MicroscopeCommand {
        command_type: CommandType::StartTracking,
        parameters: serde_json::to_value(&request).unwrap(),
    };

    match ia_client.send_command(&microscope_id, &command).await {
        Ok(response) => Ok(Json(ApiResponse::success(TrackingResponse {
            tracking_id: response
                .data
                .get("tracking_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            status: "started".to_string(),
        }))),
        Err(e) => {
            tracing::error!(
                "Failed to start tracking on microscope {}: {}",
                microscope_id,
                e
            );
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Stop object tracking
#[utoipa::path(
    post,
    path = "/api/microscope/{microscope_id}/tracking/stop",
    tag = "microscope",
    params(
        ("microscope_id" = String, Path, description = "Microscope identifier")
    ),
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Object tracking stopped", body = ApiResponse<TrackingResponse>),
        (status = 500, description = "Failed to stop tracking"),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn stop_tracking(
    State(state): State<AppState>,
    Path(microscope_id): Path<String>,
) -> Result<Json<ApiResponse<TrackingResponse>>, StatusCode> {
    let ia_client = IAClient::new(&state.config.ia);

    let command = MicroscopeCommand {
        command_type: CommandType::StopTracking,
        parameters: serde_json::json!({}),
    };

    match ia_client.send_command(&microscope_id, &command).await {
        Ok(_response) => Ok(Json(ApiResponse::success(TrackingResponse {
            tracking_id: None,
            status: "stopped".to_string(),
        }))),
        Err(e) => {
            tracing::error!(
                "Failed to stop tracking on microscope {}: {}",
                microscope_id,
                e
            );
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CommandResponse {
    #[schema(example = true)]
    pub success: bool,
    #[schema(example = "Command executed successfully")]
    pub message: Option<String>,
    pub data: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct MicroscopeStatus {
    #[schema(example = "bio-1")]
    pub microscope_id: String,
    #[schema(example = true)]
    pub is_connected: bool,
    pub current_session: Option<Uuid>,
    pub position: Position,
    pub focus: FocusInfo,
    #[schema(example = "400x")]
    pub magnification: String,
    pub lighting: LightingInfo,
    #[schema(example = false)]
    pub tracking_active: bool,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Position {
    #[schema(example = 100.5)]
    pub x: f64,
    #[schema(example = 200.3)]
    pub y: f64,
    #[schema(example = 50.1)]
    pub z: f64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct FocusInfo {
    #[schema(example = true)]
    pub is_focused: bool,
    #[schema(example = 0.92)]
    pub focus_score: Option<f64>,
    #[schema(example = false)]
    pub auto_focus_active: bool,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LightingInfo {
    #[schema(example = 75, maximum = 100)]
    pub intensity: u8,
    #[schema(example = 5500)]
    pub color_temperature: Option<u16>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CaptureRequest {
    pub session_id: Uuid,
    #[schema(example = true)]
    pub auto_focus: Option<bool>,
    #[schema(example = "high")]
    pub quality: Option<String>,
    #[schema(example = "jpeg")]
    pub format: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CaptureResponse {
    pub image_id: Uuid,
    #[schema(example = "microscope_20240115_143022.jpg")]
    pub filename: String,
    pub metadata: crate::models::ImageMetadata,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct FocusResponse {
    #[schema(example = true)]
    pub success: bool,
    #[schema(example = 0.95)]
    pub focus_score: Option<f64>,
    #[schema(example = "Auto focus completed successfully")]
    pub message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TrackingRequest {
    #[schema(example = "cell")]
    pub target_object: Option<String>,
    #[schema(example = 0.8)]
    pub detection_threshold: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TrackingResponse {
    #[schema(example = "track_123456")]
    pub tracking_id: Option<String>,
    #[schema(example = "started")]
    pub status: String,
}
