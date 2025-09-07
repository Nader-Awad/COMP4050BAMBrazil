use chrono::{DateTime, NaiveDate, Utc};
use serde_json;
use sqlx::{Error as SqlxError, PgPool};
use uuid::Uuid;

use crate::models::{
    Booking, BookingStatus, Image, ImageMetadata, Session, SessionStatus, User, UserRole,
};

/// Database service for handling all database operations
#[derive(Clone)]
pub struct DatabaseService {
    pool: PgPool,
}

impl DatabaseService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // User operations
    pub async fn create_user(
        &self,
        name: &str,
        email: &str,
        password_hash: &str,
        role: UserRole,
    ) -> Result<User, SqlxError> {
        let user = sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (name, email, password_hash, role)
            VALUES ($1, $2, $3, $4)
            RETURNING id, name, email, role as "role: UserRole", created_at, updated_at
            "#,
            name,
            email,
            password_hash,
            role as UserRole
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn get_user_by_email(
        &self,
        email: &str,
    ) -> Result<Option<UserWithPassword>, SqlxError> {
        let user = sqlx::query!(
            "SELECT id, name, email, password_hash, role, created_at, updated_at FROM users WHERE email = $1",
            email
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(user.map(|row| UserWithPassword {
            id: row.id,
            name: row.name,
            email: row.email,
            password_hash: row.password_hash,
            role: match row.role.as_str() {
                "Student" => UserRole::Student,
                "Teacher" => UserRole::Teacher,
                "Admin" => UserRole::Admin,
                _ => UserRole::Student, // default fallback
            },
            created_at: row.created_at,
            updated_at: row.updated_at,
        }))
    }

    pub async fn get_user_by_id(&self, user_id: Uuid) -> Result<Option<User>, SqlxError> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT id, name, email, role as "role: UserRole", created_at, updated_at 
            FROM users WHERE id = $1
            "#,
            user_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    // Booking operations
    pub async fn create_booking(&self, booking: &Booking) -> Result<Booking, SqlxError> {
        let created_booking = sqlx::query_as!(
            Booking,
            r#"
            INSERT INTO bookings (
                microscope_id, date, slot_start, slot_end, title, 
                group_name, attendees, requester_id, requester_name, status, approved_by
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING id, microscope_id, date, slot_start, slot_end, title,
                     group_name, attendees, requester_id, requester_name, 
                     status as "status: BookingStatus", approved_by, created_at
            "#,
            booking.microscope_id,
            booking.date,
            booking.slot_start,
            booking.slot_end,
            booking.title,
            booking.group_name,
            booking.attendees,
            booking.requester_id,
            booking.requester_name,
            match booking.status {
                BookingStatus::Pending => "Pending",
                BookingStatus::Approved => "Approved",
                BookingStatus::Rejected => "Rejected",
            },
            booking.approved_by
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(created_booking)
    }

    pub async fn get_bookings_by_date_and_microscope(
        &self,
        microscope_id: &str,
        date: NaiveDate,
    ) -> Result<Vec<Booking>, SqlxError> {
        let bookings = sqlx::query_as!(
            Booking,
            r#"
            SELECT id, microscope_id, date, slot_start, slot_end, title,
                   group_name, attendees, requester_id, requester_name,
                   status as "status: BookingStatus", approved_by, created_at
            FROM bookings 
            WHERE microscope_id = $1 AND date = $2
            ORDER BY slot_start
            "#,
            microscope_id,
            date
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(bookings)
    }

    pub async fn get_bookings_by_user(&self, user_id: Uuid) -> Result<Vec<Booking>, SqlxError> {
        let bookings = sqlx::query_as!(
            Booking,
            r#"
            SELECT id, microscope_id, date, slot_start, slot_end, title,
                   group_name, attendees, requester_id, requester_name,
                   status as "status: BookingStatus", approved_by, created_at
            FROM bookings 
            WHERE requester_id = $1
            ORDER BY date DESC, slot_start DESC
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(bookings)
    }

    pub async fn update_booking_status(
        &self,
        booking_id: Uuid,
        status: BookingStatus,
        approved_by: Option<Uuid>,
    ) -> Result<Booking, SqlxError> {
        let booking = sqlx::query_as!(
            Booking,
            r#"
            UPDATE bookings 
            SET status = $2, approved_by = $3
            WHERE id = $1
            RETURNING id, microscope_id, date, slot_start, slot_end, title,
                     group_name, attendees, requester_id, requester_name,
                     status as "status: BookingStatus", approved_by, created_at
            "#,
            booking_id,
            match status {
                BookingStatus::Pending => "Pending",
                BookingStatus::Approved => "Approved",
                BookingStatus::Rejected => "Rejected",
            },
            approved_by
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(booking)
    }

    // Session operations
    pub async fn create_session(&self, session: &Session) -> Result<Session, SqlxError> {
        let created_session = sqlx::query_as!(
            Session,
            r#"
            INSERT INTO sessions (user_id, booking_id, microscope_id, status, notes)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, user_id, booking_id, microscope_id, 
                     status as "status: SessionStatus", started_at, ended_at, notes
            "#,
            session.user_id,
            session.booking_id,
            session.microscope_id,
            match session.status {
                SessionStatus::Active => "Active",
                SessionStatus::Completed => "Completed",
                SessionStatus::Aborted => "Aborted",
            },
            session.notes
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(created_session)
    }

    pub async fn get_active_session_by_user(
        &self,
        user_id: Uuid,
    ) -> Result<Option<Session>, SqlxError> {
        let session = sqlx::query_as!(
            Session,
            r#"
            SELECT id, user_id, booking_id, microscope_id,
                   status as "status: SessionStatus", started_at, ended_at, notes
            FROM sessions 
            WHERE user_id = $1 AND status = 'Active'
            ORDER BY started_at DESC
            LIMIT 1
            "#,
            user_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(session)
    }

    pub async fn end_session(
        &self,
        session_id: Uuid,
        notes: Option<String>,
    ) -> Result<Session, SqlxError> {
        let session = sqlx::query_as!(
            Session,
            r#"
            UPDATE sessions 
            SET status = 'Completed', ended_at = NOW(), notes = COALESCE($2, notes)
            WHERE id = $1
            RETURNING id, user_id, booking_id, microscope_id,
                     status as "status: SessionStatus", started_at, ended_at, notes
            "#,
            session_id,
            notes
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(session)
    }

    // Image operations
    pub async fn create_image(&self, image: &Image) -> Result<Image, SqlxError> {
        let metadata_json = serde_json::to_value(&image.metadata).unwrap();

        let created_image = sqlx::query!(
            r#"
            INSERT INTO images (
                session_id, filename, file_path, content_type, file_size,
                width, height, metadata, captured_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id, session_id, filename, file_path, content_type, file_size,
                     width, height, metadata, captured_at
            "#,
            image.session_id,
            image.filename,
            image.file_path,
            image.content_type,
            image.file_size,
            image.width,
            image.height,
            metadata_json,
            image.captured_at
        )
        .fetch_one(&self.pool)
        .await?;

        let metadata: ImageMetadata = serde_json::from_value(created_image.metadata).unwrap();

        Ok(Image {
            id: created_image.id,
            session_id: created_image.session_id,
            filename: created_image.filename,
            file_path: created_image.file_path,
            content_type: created_image.content_type,
            file_size: created_image.file_size,
            width: created_image.width,
            height: created_image.height,
            metadata,
            captured_at: created_image.captured_at,
        })
    }

    pub async fn get_images_by_session(&self, session_id: Uuid) -> Result<Vec<Image>, SqlxError> {
        let rows = sqlx::query!(
            r#"
            SELECT id, session_id, filename, file_path, content_type, file_size,
                   width, height, metadata, captured_at
            FROM images 
            WHERE session_id = $1
            ORDER BY captured_at DESC
            "#,
            session_id
        )
        .fetch_all(&self.pool)
        .await?;

        let images = rows
            .into_iter()
            .map(|row| {
                let metadata: ImageMetadata =
                    serde_json::from_value(row.metadata).unwrap_or_default();
                Image {
                    id: row.id,
                    session_id: row.session_id,
                    filename: row.filename,
                    file_path: row.file_path,
                    content_type: row.content_type,
                    file_size: row.file_size,
                    width: row.width,
                    height: row.height,
                    metadata,
                    captured_at: row.captured_at,
                }
            })
            .collect();

        Ok(images)
    }

    pub async fn get_latest_image_by_session(
        &self,
        session_id: Uuid,
    ) -> Result<Option<Image>, SqlxError> {
        let row = sqlx::query!(
            r#"
            SELECT id, session_id, filename, file_path, content_type, file_size,
                   width, height, metadata, captured_at
            FROM images 
            WHERE session_id = $1
            ORDER BY captured_at DESC
            LIMIT 1
            "#,
            session_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|row| {
            let metadata: ImageMetadata = serde_json::from_value(row.metadata).unwrap_or_default();
            Image {
                id: row.id,
                session_id: row.session_id,
                filename: row.filename,
                file_path: row.file_path,
                content_type: row.content_type,
                file_size: row.file_size,
                width: row.width,
                height: row.height,
                metadata,
                captured_at: row.captured_at,
            }
        }))
    }

    // Utility method to check booking conflicts
    pub async fn check_booking_conflicts(
        &self,
        microscope_id: &str,
        date: NaiveDate,
        slot_start: i32,
        slot_end: i32,
        exclude_booking_id: Option<Uuid>,
    ) -> Result<bool, SqlxError> {
        let count = if let Some(exclude_id) = exclude_booking_id {
            let result = sqlx::query!(
                r#"
                SELECT COUNT(*) as count
                FROM bookings 
                WHERE microscope_id = $1 
                  AND date = $2 
                  AND status IN ('Pending', 'Approved')
                  AND id != $5
                  AND NOT (slot_end <= $3 OR slot_start >= $4)
                "#,
                microscope_id,
                date,
                slot_start,
                slot_end,
                exclude_id
            )
            .fetch_one(&self.pool)
            .await?;
            result.count.unwrap_or(0)
        } else {
            let result = sqlx::query!(
                r#"
                SELECT COUNT(*) as count
                FROM bookings 
                WHERE microscope_id = $1 
                  AND date = $2 
                  AND status IN ('Pending', 'Approved')
                  AND NOT (slot_end <= $3 OR slot_start >= $4)
                "#,
                microscope_id,
                date,
                slot_start,
                slot_end
            )
            .fetch_one(&self.pool)
            .await?;
            result.count.unwrap_or(0)
        };

        Ok(count > 0)
    }
}

/// User with password hash for authentication
#[derive(Debug, Clone)]
pub struct UserWithPassword {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub password_hash: String,
    pub role: UserRole,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Default for ImageMetadata {
    fn default() -> Self {
        Self {
            objects_detected: vec![],
            classification_tags: vec![],
            confidence_scores: vec![],
            focus_quality: None,
            magnification: None,
            lighting_conditions: None,
        }
    }
}
