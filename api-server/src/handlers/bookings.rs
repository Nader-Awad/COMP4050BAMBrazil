use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    Extension,
};
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;
use validator::Validate;

use crate::{
    middleware::auth::Claims,
    models::{ApiResponse, Booking, BookingStatus, UserRole},
    AppError, AppState,
};

// TODO question if this is the correct design for this model
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateBookingRequest {
    #[schema(example = "bio-1")]
    pub microscope_id: String,
    #[schema(example = "2024-01-15", format = "date")]
    // TODO see below
    pub date: String,
    #[schema(example = 540)]
    // TODO should this be combined with the date, e.g. directly use unix epoch time instead of
    // string typing the date
    pub slot_start: i32,
    #[schema(example = 600)]
    // TODO should this be combined with the date, e.g. directly use unix epoch time instead of
    // string typing the date
    pub slot_end: i32,
    #[validate(length(min = 1))]
    #[schema(example = "Cell Biology Lab")]
    pub title: String,
    #[schema(example = "Team Alpha")]
    pub group_name: Option<String>,
    #[schema(example = 4)]
    pub attendees: Option<i32>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateBookingRequest {
    #[schema(example = "Updated Lab Session")]
    pub title: Option<String>,
    #[schema(example = "Team Beta")]
    pub group_name: Option<String>,
    #[schema(example = 6)]
    pub attendees: Option<i32>,
    pub status: Option<BookingStatus>,
}

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct BookingQuery {
    #[schema(example = "bio-1")]
    pub microscope_id: Option<String>,
    // TODO should date be combined with the slot_start/end, e.g. directly use unix epoch time instead of
    // string typing the date
    #[schema(example = "2024-01-15", format = "date")]
    pub date: Option<String>,
    pub status: Option<BookingStatus>,
    pub user_id: Option<Uuid>,
    #[schema(example = 1)]
    pub page: Option<u64>,
    #[schema(example = 20)]
    pub limit: Option<u64>,
}

/// List bookings with filtering
#[utoipa::path(
    get,
    path = "/api/bookings",
    tag = "bookings",
    params(BookingQuery),
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "List of bookings", body = ApiResponse<Vec<Booking>>),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn list_bookings(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<BookingQuery>,
) -> Result<Json<ApiResponse<Vec<Booking>>>, AppError> {
    let bookings = if let Some(user_id) = query.user_id {
        // Get bookings for specific user (if admin/teacher or own bookings)
        match claims.role {
            UserRole::Admin | UserRole::Teacher => state.db.get_bookings_by_user(user_id).await?,
            UserRole::Student => {
                if user_id == claims.user_id {
                    state.db.get_bookings_by_user(user_id).await?
                } else {
                    return Err(AppError::Authorization(
                        "Cannot view other users' bookings".to_string(),
                    ));
                }
            }
        }
    } else if let (Some(microscope_id), Some(date_str)) = (&query.microscope_id, &query.date) {
        // Get bookings by microscope and date
        let date = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
            .map_err(|_| AppError::BadRequest("Invalid date format".to_string()))?;
        state
            .db
            .get_bookings_by_date_and_microscope(microscope_id, date)
            .await?
    } else {
        // For students, only show their own bookings unless they specify microscope+date
        match claims.role {
            UserRole::Student => state.db.get_bookings_by_user(claims.user_id).await?,
            UserRole::Teacher | UserRole::Admin => {
                // For now, return user's own bookings by default
                // TODO: Implement full booking listing for admins/teachers
                state.db.get_bookings_by_user(claims.user_id).await?
            }
        }
    };

    Ok(Json(ApiResponse::success(bookings)))
}

/// Create new booking
#[utoipa::path(
    post,
    path = "/api/bookings",
    tag = "bookings",
    request_body( content = CreateBookingRequest),
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Booking created successfully", body = ApiResponse<Booking>),
        (status = 400, description = "Invalid booking data or time conflict", body = ApiResponse<String>),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn create_booking(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(request): Json<CreateBookingRequest>,
) -> Result<Json<ApiResponse<Booking>>, AppError> {
    // Validate request
    if let Err(_) = request.validate() {
        return Ok(Json(ApiResponse::error("Invalid booking data".to_string())));
    }

    // Parse date string to NaiveDate
    let date = chrono::NaiveDate::parse_from_str(&request.date, "%Y-%m-%d")
        .map_err(|_| AppError::BadRequest("Invalid date format".to_string()))?;

    // Check for conflicts
    let has_conflicts = state
        .db
        .check_booking_conflicts(
            &request.microscope_id,
            date,
            request.slot_start,
            request.slot_end,
            None,
        )
        .await?;

    if has_conflicts {
        return Ok(Json(ApiResponse::error(
            "Time slot conflict with existing booking".to_string(),
        )));
    }

    // Get user information (fallback to claims if not present in DB)
    let user = state
        .db
        .get_user_by_id(claims.user_id)
        .await?
        .ok_or(AppError::NotFound("User not found".to_string()))?;

    let booking = Booking {
        id: Uuid::new_v4(),
        microscope_id: request.microscope_id,
        date,
        slot_start: request.slot_start,
        slot_end: request.slot_end,
        title: request.title,
        group_name: request.group_name,
        attendees: request.attendees,
        requester_id: user.id,
        requester_name: user.name,
        status: BookingStatus::Pending,
        approved_by: None,
        created_at: chrono::Utc::now().into(),
    };

    // Save to database
    let created_booking = state.db.create_booking(&booking).await?;

    Ok(Json(ApiResponse::success(created_booking)))
}

/// Get booking by ID
#[utoipa::path(
    get,
    path = "/api/bookings/{id}",
    tag = "bookings",
    params(
        ("id" = Uuid, Path, description = "Booking ID")
    ),
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Booking details", body = ApiResponse<Booking>),
        (status = 404, description = "Booking not found", body = ApiResponse<String>),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn get_booking(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(booking_id): Path<Uuid>,
) -> Result<Json<ApiResponse<Booking>>, AppError> {
    // Get all user's bookings and find the one with matching ID
    let bookings = state.db.get_bookings_by_user(claims.user_id).await?;

    if let Some(booking) = bookings.into_iter().find(|b| b.id == booking_id) {
        // Users can view their own bookings, admins/teachers can view any
        match claims.role {
            UserRole::Admin | UserRole::Teacher => Ok(Json(ApiResponse::success(booking))),
            UserRole::Student => {
                if booking.requester_id == claims.user_id {
                    Ok(Json(ApiResponse::success(booking)))
                } else {
                    Err(AppError::Authorization(
                        "Cannot view other users' bookings".to_string(),
                    ))
                }
            }
        }
    } else {
        Err(AppError::NotFound("Booking not found".to_string()))
    }
}

/// Update booking
#[utoipa::path(
    put,
    path = "/api/bookings/{id}",
    tag = "bookings",
    params(
        ("id" = Uuid, Path, description = "Booking ID")
    ),
    request_body = UpdateBookingRequest,
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Booking updated successfully", body = ApiResponse<Booking>),
        (status = 400, description = "Invalid booking data", body = ApiResponse<String>),
        (status = 403, description = "Insufficient permissions", body = ApiResponse<String>),
        (status = 404, description = "Booking not found", body = ApiResponse<String>),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn update_booking(
    State(_state): State<AppState>,
    Extension(_claims): Extension<Claims>,
    Path(_booking_id): Path<Uuid>,
    Json(_request): Json<UpdateBookingRequest>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    // TODO: Implement booking updates - this requires additional database methods
    // For now, return an error indicating the feature is not yet implemented
    Ok(Json(ApiResponse::error(
        "Booking updates not yet implemented".to_string(),
    )))
}

/// Delete booking
#[utoipa::path(
    delete,
    path = "/api/bookings/{id}",
    tag = "bookings",
    params(
        ("id" = Uuid, Path, description = "Booking ID")
    ),
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 204, description = "Booking deleted successfully"),
        (status = 403, description = "Insufficient permissions", body = ApiResponse<String>),
        (status = 404, description = "Booking not found", body = ApiResponse<String>),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn delete_booking(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(booking_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let deleted_rows = match claims.role {
        UserRole::Teacher | UserRole::Admin => state.db.delete_booking(booking_id).await?,
        _ => {
            // For non-admin users, check booking ownership first
            let booking_owner = state.db.get_booking_owner(booking_id).await?;

            match booking_owner {
                Some(owner_id) => {
                    if owner_id != claims.user_id {
                        return Err(AppError::Authorization(
                            "You can only delete your own bookings".to_string(),
                        ));
                    }
                    // User owns the booking, proceed with deletion
                    state
                        .db
                        .delete_booking_by_owner(booking_id, Some(claims.user_id))
                        .await?
                }
                None => {
                    return Err(AppError::NotFound("Booking not found".to_string()));
                }
            }
        }
    };

    tracing::info!(deleted_rows, "deleted booking with id {:?}", booking_id);
    Ok(StatusCode::NO_CONTENT)
}

/// Approve booking (teacher/admin only)
#[utoipa::path(
    post,
    path = "/api/bookings/{id}/approve",
    tag = "bookings",
    params(
        ("id" = Uuid, Path, description = "Booking ID")
    ),
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Booking approved successfully", body = ApiResponse<Booking>),
        (status = 403, description = "Insufficient permissions - only teachers and admins can approve", body = ApiResponse<String>),
        (status = 404, description = "Booking not found", body = ApiResponse<String>),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn approve_booking(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(booking_id): Path<Uuid>,
) -> Result<Json<ApiResponse<Booking>>, AppError> {
    // Check permissions
    match claims.role {
        UserRole::Teacher | UserRole::Admin => {}
        _ => {
            return Ok(Json(ApiResponse::error(
                "Insufficient permissions".to_string(),
            )))
        }
    }

    let booking = state
        .db
        .update_booking_status(booking_id, BookingStatus::Approved, Some(claims.user_id))
        .await?;

    Ok(Json(ApiResponse::success(booking)))
}

/// Reject booking (teacher/admin only)
#[utoipa::path(
    post,
    path = "/api/bookings/{id}/reject",
    tag = "bookings",
    params(
        ("id" = Uuid, Path, description = "Booking ID")
    ),
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Booking rejected successfully", body = ApiResponse<Booking>),
        (status = 403, description = "Insufficient permissions - only teachers and admins can reject", body = ApiResponse<String>),
        (status = 404, description = "Booking not found", body = ApiResponse<String>),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn reject_booking(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(booking_id): Path<Uuid>,
) -> Result<Json<ApiResponse<Booking>>, AppError> {
    // Check permissions
    match claims.role {
        UserRole::Teacher | UserRole::Admin => {}
        _ => {
            return Ok(Json(ApiResponse::error(
                "Insufficient permissions".to_string(),
            )))
        }
    }

    let booking = state
        .db
        .update_booking_status(booking_id, BookingStatus::Rejected, Some(claims.user_id))
        .await?;

    Ok(Json(ApiResponse::success(booking)))
}