use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// User model for authentication and authorization
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub role: UserRole,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, sqlx::Type, ToSchema)]
#[sqlx(type_name = "VARCHAR")]
pub enum UserRole {
    Student,
    Teacher,
    Admin,
}

/// Session model for tracking microscope usage
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub booking_id: Option<Uuid>,
    pub microscope_id: String,
    pub status: SessionStatus,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, sqlx::Type, ToSchema)]
#[sqlx(type_name = "VARCHAR")]
pub enum SessionStatus {
    Active,
    Completed,
    Aborted,
}

/// Image model for storing microscope captures
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Image {
    pub id: Uuid,
    pub session_id: Uuid,
    pub filename: String,
    pub file_path: String,
    pub content_type: String,
    pub file_size: i64,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub metadata: ImageMetadata,
    pub captured_at: DateTime<Utc>,
}

/// AI-generated metadata for images
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ImageMetadata {
    pub objects_detected: Vec<DetectedObject>,
    pub classification_tags: Vec<String>,
    pub confidence_scores: Vec<f32>,
    pub focus_quality: Option<f32>,
    pub magnification: Option<String>,
    pub lighting_conditions: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DetectedObject {
    pub class_name: String,
    pub confidence: f32,
    pub bounding_box: BoundingBox,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BoundingBox {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Booking model (from existing UI)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Booking {
    pub id: Uuid,
    pub microscope_id: String,
    pub date: NaiveDate,
    pub slot_start: i32,
    pub slot_end: i32,
    pub title: String,
    pub group_name: Option<String>,
    pub attendees: Option<i32>,
    pub requester_id: Uuid,
    pub requester_name: String,
    pub status: BookingStatus,
    pub approved_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, sqlx::Type, ToSchema)]
#[sqlx(type_name = "VARCHAR")]
pub enum BookingStatus {
    Pending,
    Approved,
    Rejected,
}

/// Microscope control commands
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MicroscopeCommand {
    pub command_type: CommandType,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum CommandType {
    Move,
    Focus,
    Capture,
    SetMagnification,
    SetLighting,
    StartTracking,
    StopTracking,
}

/// API Response types
#[derive(Debug, Serialize, ToSchema)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: None,
            error: None,
        }
    }

    pub fn error(error: String) -> Self {
        Self {
            success: false,
            data: None,
            message: None,
            error: Some(error),
        }
    }
}
