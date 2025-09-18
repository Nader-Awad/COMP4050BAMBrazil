use axum::{
    extract::{Path, Query, State},
    response::Json,
    Extension,
};
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;
use validator::Validate;

use crate::{
    middleware::auth::Claims,
    models::{ApiResponse, Session, SessionStatus, UserRole},
    AppError, AppState,
};

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateSessionRequest {
    #[schema()]
    pub booking_id: Option<Uuid>,
    #[schema(example = "bio-1")]
    pub microscope_id: String,
    #[schema(example = "Starting cell division observation")]
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct SessionQuery {
    #[schema(example = "bio-1")]
    pub microscope_id: Option<String>,
    pub user_id: Option<Uuid>,
    pub status: Option<SessionStatus>,
    #[schema(example = true)]
    pub active_only: Option<bool>,
    #[schema(example = 1)]
    pub page: Option<u64>,
    #[schema(example = 20)]
    pub limit: Option<u64>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct EndSessionRequest {
    #[schema(example = "Completed cell division observation. Found 15 dividing cells.")]
    pub notes: Option<String>,
}

/// List sessions with filtering
#[utoipa::path(
    get,
    path = "/api/sessions",
    tag = "sessions",
    params(SessionQuery),
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "List of sessions (filtered by user role)", body = ApiResponse<Vec<Session>>),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn list_sessions(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<SessionQuery>,
) -> Result<Json<ApiResponse<Vec<Session>>>, AppError> {
    let limit = query.limit.unwrap_or(20).min(100);
    let page = query.page.unwrap_or(1).max(1);
    let offset = (page - 1) * limit;

    // Role-based filtering
    let (microscope_id, user_id) = match claims.role {
        UserRole::Student => {
            // Students can only see their own sessions
            (query.microscope_id.as_deref(), Some(claims.user_id))
        }
        UserRole::Teacher | UserRole::Admin => {
            // Teachers and admins can see all sessions with optional filtering
            (query.microscope_id.as_deref(), query.user_id)
        }
    };

    let active_only = query.active_only.unwrap_or(false);

    let sessions = state
        .db
        .list_sessions(
            microscope_id,
            user_id,
            query.status,
            active_only,
            limit,
            offset,
        )
        .await?;

    Ok(Json(ApiResponse::success(sessions)))
}

/// Create new session (start microscope usage)
#[utoipa::path(
    post,
    path = "/api/sessions",
    tag = "sessions",
    request_body = CreateSessionRequest,
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Session started successfully", body = ApiResponse<Session>),
        (status = 400, description = "Invalid session data or microscope unavailable", body = ApiResponse<String>),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn create_session(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(request): Json<CreateSessionRequest>,
) -> Result<Json<ApiResponse<Session>>, AppError> {
    // Validate request
    if let Err(_) = request.validate() {
        return Ok(Json(ApiResponse::error("Invalid session data".to_string())));
    }

    // Check if user already has an active session
    if let Some(_active_session) = state.db.get_active_session_by_user(claims.user_id).await? {
        return Ok(Json(ApiResponse::error(
            "User already has an active session".to_string(),
        )));
    }

    // Check if user has an approved booking for this time (if booking_id provided)
    if let Some(booking_id) = request.booking_id {
        let booking = state
            .db
            .get_booking_by_id(booking_id)
            .await?
            .ok_or(AppError::NotFound("Booking not found".to_string()))?;

        // Validate booking belongs to user (unless admin/teacher)
        match claims.role {
            UserRole::Student => {
                if booking.requester_id != claims.user_id {
                    return Ok(Json(ApiResponse::error(
                        "Cannot start session - booking does not belong to user".to_string(),
                    )));
                }
            }
            UserRole::Teacher | UserRole::Admin => {
                // Teachers and admins can start sessions for any approved booking
            }
        }

        // Validate booking is approved
        if booking.status != crate::models::BookingStatus::Approved {
            return Ok(Json(ApiResponse::error(
                "Cannot start session - booking is not approved".to_string(),
            )));
        }

        // Validate microscope matches
        if booking.microscope_id != request.microscope_id {
            return Ok(Json(ApiResponse::error(
                "Cannot start session - microscope ID does not match booking".to_string(),
            )));
        }

        tracing::info!("Starting session for approved booking: {}", booking_id);
    }

    let session = Session {
        id: Uuid::new_v4(),
        user_id: claims.user_id,
        booking_id: request.booking_id,
        microscope_id: request.microscope_id.clone(),
        status: SessionStatus::Active,
        started_at: chrono::Utc::now(),
        ended_at: None,
        notes: request.notes,
    };

    // Save to database
    let created_session = state.db.create_session(&session).await?;

    tracing::info!(
        "Started new session: {} for user: {} on microscope: {}",
        created_session.id,
        claims.user_id,
        request.microscope_id
    );

    Ok(Json(ApiResponse::success(created_session)))
}

/// Get session by ID
#[utoipa::path(
    get,
    path = "/api/sessions/{id}",
    tag = "sessions",
    params(
        ("id" = Uuid, Path, description = "Session ID")
    ),
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Session details", body = ApiResponse<Session>),
        (status = 403, description = "Access denied - can only access own sessions unless admin/teacher", body = ApiResponse<String>),
        (status = 404, description = "Session not found", body = ApiResponse<String>),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn get_session(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(session_id): Path<Uuid>,
) -> Result<Json<ApiResponse<Session>>, AppError> {
    let session = state
        .db
        .get_session_by_id(session_id)
        .await?
        .ok_or(AppError::NotFound("Session not found".to_string()))?;

    // Permission checking based on user role
    match claims.role {
        UserRole::Student => {
            // Students can only access their own sessions
            if session.user_id != claims.user_id {
                return Err(AppError::Authorization("Access denied - can only view own sessions".to_string()));
            }
        }
        UserRole::Teacher | UserRole::Admin => {
            // Teachers and admins can access all sessions
        }
    }

    Ok(Json(ApiResponse::success(session)))
}

/// End session (stop microscope usage)
#[utoipa::path(
    post,
    path = "/api/sessions/{id}/end",
    tag = "sessions",
    params(
        ("id" = Uuid, Path, description = "Session ID")
    ),
    request_body = EndSessionRequest,
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Session ended successfully", body = ApiResponse<Session>),
        (status = 400, description = "Session is not active", body = ApiResponse<String>),
        (status = 403, description = "Access denied - can only end own sessions unless admin/teacher", body = ApiResponse<String>),
        (status = 404, description = "Session not found", body = ApiResponse<String>),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn end_session(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(session_id): Path<Uuid>,
    Json(request): Json<EndSessionRequest>,
) -> Result<Json<ApiResponse<Session>>, AppError> {
    // Get the active session for this user
    let active_session = state
        .db
        .get_active_session_by_user(claims.user_id)
        .await?
        .ok_or(AppError::NotFound("No active session found".to_string()))?;

    // Check if the session ID matches the active session
    if active_session.id != session_id {
        return Ok(Json(ApiResponse::error(
            "Cannot end session - session ID does not match active session".to_string(),
        )));
    }

    // Check permissions - only session owner or admin can end session
    match claims.role {
        UserRole::Student => {
            if active_session.user_id != claims.user_id {
                return Err(AppError::Authorization("Access denied".to_string()));
            }
        }
        UserRole::Teacher | UserRole::Admin => {
            // Teachers and admins can end any session
        }
    }

    // Check if session is already ended
    if active_session.status != SessionStatus::Active {
        return Ok(Json(ApiResponse::error(
            "Session is not active".to_string(),
        )));
    }

    // End the session in database
    let ended_session = state.db.end_session(session_id, request.notes).await?;
    // TODO: Update microscope status in IA system

    tracing::info!(
        "Ended session: {} for user: {}",
        ended_session.id,
        claims.user_id
    );

    Ok(Json(ApiResponse::success(ended_session)))
}

/// Get current user's active session
#[utoipa::path(
    get,
    path = "/api/sessions/current",
    tag = "sessions",
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Current active session or null if none", body = ApiResponse<Option<Session>>),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn get_current_session(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<ApiResponse<Option<Session>>>, AppError> {
    let active_session = state.db.get_active_session_by_user(claims.user_id).await?;
    Ok(Json(ApiResponse::success(active_session)))
}
