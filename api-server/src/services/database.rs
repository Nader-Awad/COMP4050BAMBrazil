use chrono::{DateTime, Datelike, FixedOffset, NaiveDate, Utc};
use serde_json;
use sqlx::types::time;
use sqlx::{Error as SqlxError, PgPool, Row};
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

    pub async fn create_user(
        &self,
        name: &str,
        email: &str,
        password_hash: &str,
        role: UserRole,
    ) -> Result<User, SqlxError> {
        let row = sqlx::query!(
            r#"
            INSERT INTO users (name, email, password_hash, role)
            VALUES ($1, $2, $3, $4)
            RETURNING id, name, email, role, created_at, updated_at
            "#,
            name,
            email,
            password_hash,
            match role {
                UserRole::Student => "Student",
                UserRole::Teacher => "Teacher",
                UserRole::Admin => "Admin",
            }
        )
        .fetch_one(&self.pool)
        .await?;

        let user_role = match row.role.as_str() {
            "Student" => UserRole::Student,
            "Teacher" => UserRole::Teacher,
            "Admin" => UserRole::Admin,
            _ => UserRole::Student, // default fallback
        };

        Ok(User {
            id: row.id,
            name: row.name,
            email: row.email,
            role: user_role,
            created_at: DateTime::from_timestamp(row.created_at.unix_timestamp(), 0)
                .unwrap()
                .fixed_offset(),
            updated_at: DateTime::from_timestamp(row.updated_at.unix_timestamp(), 0)
                .unwrap()
                .fixed_offset(),
        })
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
            created_at: DateTime::from_timestamp(row.created_at.unix_timestamp(), 0)
                .unwrap()
                .fixed_offset(),
            updated_at: DateTime::from_timestamp(row.updated_at.unix_timestamp(), 0)
                .unwrap()
                .fixed_offset(),
        }))
    }

    pub async fn get_user_by_id(&self, user_id: Uuid) -> Result<Option<User>, SqlxError> {
        let row = sqlx::query!(
            r#"
            SELECT id, name, email, role, created_at, updated_at 
            FROM users WHERE id = $1
            "#,
            user_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|row| {
            let user_role = match row.role.as_str() {
                "Student" => UserRole::Student,
                "Teacher" => UserRole::Teacher,
                "Admin" => UserRole::Admin,
                _ => UserRole::Student, // default fallback
            };

            User {
                id: row.id,
                name: row.name,
                email: row.email,
                role: user_role,
                created_at: DateTime::from_timestamp(row.created_at.unix_timestamp(), 0)
                    .unwrap()
                    .fixed_offset(),
                updated_at: DateTime::from_timestamp(row.updated_at.unix_timestamp(), 0)
                    .unwrap()
                    .fixed_offset(),
            }
        }))
    }

    pub async fn create_booking(&self, booking: &Booking) -> Result<Booking, SqlxError> {
        // Convert chrono NaiveDate to time Date
        let time_date =
            time::Date::from_ordinal_date(booking.date.year(), booking.date.ordinal() as u16)
                .unwrap();

        let row = sqlx::query!(
            r#"
            INSERT INTO bookings (
                microscope_id, date, slot_start, slot_end, title, 
                group_name, attendees, requester_id, requester_name, status, approved_by
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING id, microscope_id, date, slot_start, slot_end, title,
                     group_name, attendees, requester_id, requester_name, 
                     status, approved_by, created_at
            "#,
            booking.microscope_id,
            time_date,
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

        let status = match row.status.as_str() {
            "Pending" => BookingStatus::Pending,
            "Approved" => BookingStatus::Approved,
            "Rejected" => BookingStatus::Rejected,
            _ => BookingStatus::Pending, // default fallback
        };

        let naive_date = NaiveDate::from_ymd_opt(
            row.date.year(),
            row.date.month() as u32,
            row.date.day() as u32,
        )
        .unwrap();

        Ok(Booking {
            id: row.id,
            microscope_id: row.microscope_id,
            date: naive_date,
            slot_start: row.slot_start,
            slot_end: row.slot_end,
            title: row.title,
            group_name: row.group_name,
            attendees: row.attendees,
            requester_id: row.requester_id,
            requester_name: row.requester_name,
            status,
            approved_by: row.approved_by,
            created_at: DateTime::from_timestamp(row.created_at.unix_timestamp(), 0)
                .unwrap()
                .fixed_offset(),
        })
    }

    pub async fn get_bookings_by_date_and_microscope(
        &self,
        microscope_id: &str,
        date: NaiveDate,
    ) -> Result<Vec<Booking>, SqlxError> {
        // Convert chrono NaiveDate to time Date
        let time_date = time::Date::from_ordinal_date(date.year(), date.ordinal() as u16).unwrap();

        let rows = sqlx::query!(
            r#"
            SELECT id, microscope_id, date, slot_start, slot_end, title,
                   group_name, attendees, requester_id, requester_name,
                   status, approved_by, created_at
            FROM bookings 
            WHERE microscope_id = $1 AND date = $2
            ORDER BY slot_start
            "#,
            microscope_id,
            time_date
        )
        .fetch_all(&self.pool)
        .await?;

        let bookings = rows
            .into_iter()
            .map(|row| {
                let status = match row.status.as_str() {
                    "Pending" => BookingStatus::Pending,
                    "Approved" => BookingStatus::Approved,
                    "Rejected" => BookingStatus::Rejected,
                    _ => BookingStatus::Pending,
                };

                let naive_date = NaiveDate::from_ymd_opt(
                    row.date.year(),
                    row.date.month() as u32,
                    row.date.day() as u32,
                )
                .unwrap();

                Booking {
                    id: row.id,
                    microscope_id: row.microscope_id,
                    date: naive_date,
                    slot_start: row.slot_start,
                    slot_end: row.slot_end,
                    title: row.title,
                    group_name: row.group_name,
                    attendees: row.attendees,
                    requester_id: row.requester_id,
                    requester_name: row.requester_name,
                    status,
                    approved_by: row.approved_by,
                    created_at: DateTime::from_timestamp(row.created_at.unix_timestamp(), 0)
                        .unwrap()
                        .fixed_offset(),
                }
            })
            .collect();

        Ok(bookings)
    }

    pub async fn get_bookings_by_user(&self, user_id: Uuid) -> Result<Vec<Booking>, SqlxError> {
        let rows = sqlx::query!(
            r#"
            SELECT id, microscope_id, date, slot_start, slot_end, title,
                   group_name, attendees, requester_id, requester_name,
                   status, approved_by, created_at
            FROM bookings 
            WHERE requester_id = $1
            ORDER BY date DESC, slot_start DESC
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await?;

        let bookings = rows
            .into_iter()
            .map(|row| {
                let status = match row.status.as_str() {
                    "Pending" => BookingStatus::Pending,
                    "Approved" => BookingStatus::Approved,
                    "Rejected" => BookingStatus::Rejected,
                    _ => BookingStatus::Pending,
                };

                let naive_date = NaiveDate::from_ymd_opt(
                    row.date.year(),
                    row.date.month() as u32,
                    row.date.day() as u32,
                )
                .unwrap();

                Booking {
                    id: row.id,
                    microscope_id: row.microscope_id,
                    date: naive_date,
                    slot_start: row.slot_start,
                    slot_end: row.slot_end,
                    title: row.title,
                    group_name: row.group_name,
                    attendees: row.attendees,
                    requester_id: row.requester_id,
                    requester_name: row.requester_name,
                    status,
                    approved_by: row.approved_by,
                    created_at: DateTime::from_timestamp(row.created_at.unix_timestamp(), 0)
                        .unwrap()
                        .fixed_offset(),
                }
            })
            .collect();

        Ok(bookings)
    }

    pub async fn update_booking_status(
        &self,
        booking_id: Uuid,
        status: BookingStatus,
        approved_by: Option<Uuid>,
    ) -> Result<Booking, SqlxError> {
        let row = sqlx::query!(
            r#"
            UPDATE bookings 
            SET status = $2, approved_by = $3
            WHERE id = $1
            RETURNING id, microscope_id, date, slot_start, slot_end, title,
                     group_name, attendees, requester_id, requester_name,
                     status, approved_by, created_at
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

        let booking_status = match row.status.as_str() {
            "Pending" => BookingStatus::Pending,
            "Approved" => BookingStatus::Approved,
            "Rejected" => BookingStatus::Rejected,
            _ => BookingStatus::Pending,
        };

        let naive_date = NaiveDate::from_ymd_opt(
            row.date.year(),
            row.date.month() as u32,
            row.date.day() as u32,
        )
        .unwrap();

        Ok(Booking {
            id: row.id,
            microscope_id: row.microscope_id,
            date: naive_date,
            slot_start: row.slot_start,
            slot_end: row.slot_end,
            title: row.title,
            group_name: row.group_name,
            attendees: row.attendees,
            requester_id: row.requester_id,
            requester_name: row.requester_name,
            status: booking_status,
            approved_by: row.approved_by,
            created_at: DateTime::from_timestamp(row.created_at.unix_timestamp(), 0)
                .unwrap()
                .fixed_offset(),
        })
    }

    pub async fn create_session(&self, session: &Session) -> Result<Session, SqlxError> {
        let row = sqlx::query!(
            r#"
            INSERT INTO sessions (user_id, booking_id, microscope_id, status, notes)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, user_id, booking_id, microscope_id, 
                     status, started_at, ended_at, notes
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

        let session_status = match row.status.as_str() {
            "Active" => SessionStatus::Active,
            "Completed" => SessionStatus::Completed,
            "Aborted" => SessionStatus::Aborted,
            _ => SessionStatus::Active,
        };

        Ok(Session {
            id: row.id,
            user_id: row.user_id,
            booking_id: row.booking_id,
            microscope_id: row.microscope_id,
            status: session_status,
            started_at: DateTime::from_timestamp(row.started_at.unix_timestamp(), 0)
                .unwrap()
                .with_timezone(&Utc),
            ended_at: row.ended_at.map(|dt| {
                DateTime::from_timestamp(dt.unix_timestamp(), 0)
                    .unwrap()
                    .with_timezone(&Utc)
            }),
            notes: row.notes,
        })
    }

    pub async fn get_active_session_by_user(
        &self,
        user_id: Uuid,
    ) -> Result<Option<Session>, SqlxError> {
        let row = sqlx::query!(
            r#"
            SELECT id, user_id, booking_id, microscope_id,
                   status, started_at, ended_at, notes
            FROM sessions 
            WHERE user_id = $1 AND status = 'Active'
            ORDER BY started_at DESC
            LIMIT 1
            "#,
            user_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|row| {
            let session_status = match row.status.as_str() {
                "Active" => SessionStatus::Active,
                "Completed" => SessionStatus::Completed,
                "Aborted" => SessionStatus::Aborted,
                _ => SessionStatus::Active,
            };

            Session {
                id: row.id,
                user_id: row.user_id,
                booking_id: row.booking_id,
                microscope_id: row.microscope_id,
                status: session_status,
                started_at: DateTime::from_timestamp(row.started_at.unix_timestamp(), 0)
                    .unwrap()
                    .with_timezone(&Utc),
                ended_at: row.ended_at.map(|dt| {
                    DateTime::from_timestamp(dt.unix_timestamp(), 0)
                        .unwrap()
                        .with_timezone(&Utc)
                }),
                notes: row.notes,
            }
        }))
    }

    pub async fn end_session(
        &self,
        session_id: Uuid,
        notes: Option<String>,
    ) -> Result<Session, SqlxError> {
        let row = sqlx::query!(
            r#"
            UPDATE sessions 
            SET status = 'Completed', ended_at = NOW(), notes = COALESCE($2, notes)
            WHERE id = $1
            RETURNING id, user_id, booking_id, microscope_id,
                     status, started_at, ended_at, notes
            "#,
            session_id,
            notes
        )
        .fetch_one(&self.pool)
        .await?;

        let session_status = match row.status.as_str() {
            "Active" => SessionStatus::Active,
            "Completed" => SessionStatus::Completed,
            "Aborted" => SessionStatus::Aborted,
            _ => SessionStatus::Completed,
        };

        Ok(Session {
            id: row.id,
            user_id: row.user_id,
            booking_id: row.booking_id,
            microscope_id: row.microscope_id,
            status: session_status,
            started_at: DateTime::from_timestamp(row.started_at.unix_timestamp(), 0)
                .unwrap()
                .with_timezone(&Utc),
            ended_at: row.ended_at.map(|dt| {
                DateTime::from_timestamp(dt.unix_timestamp(), 0)
                    .unwrap()
                    .with_timezone(&Utc)
            }),
            notes: row.notes,
        })
    }

    pub async fn list_sessions(
        &self,
        microscope_id: Option<&str>,
        user_id: Option<Uuid>,
        status: Option<SessionStatus>,
        active_only: bool,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<Session>, SqlxError> {
        let mut query = r#"
            SELECT id, user_id, booking_id, microscope_id,
                   status, started_at, ended_at, notes
            FROM sessions 
            WHERE 1=1
        "#
        .to_string();

        let mut conditions = Vec::new();
        let mut param_count = 0;

        if let Some(mid) = microscope_id {
            param_count += 1;
            conditions.push(format!(" AND microscope_id = ${}", param_count));
        }

        if let Some(uid) = user_id {
            param_count += 1;
            conditions.push(format!(" AND user_id = ${}", param_count));
        }

        if let Some(s) = &status {
            param_count += 1;
            conditions.push(format!(" AND status = ${}", param_count));
        }

        if active_only {
            conditions.push(" AND status = 'Active'".to_string());
        }

        for condition in conditions {
            query.push_str(&condition);
        }

        query.push_str(" ORDER BY started_at DESC");
        param_count += 1;
        query.push_str(&format!(" LIMIT ${}", param_count));
        param_count += 1;
        query.push_str(&format!(" OFFSET ${}", param_count));

        let mut sql_query = sqlx::query(&query);

        if let Some(mid) = microscope_id {
            sql_query = sql_query.bind(mid);
        }
        if let Some(uid) = user_id {
            sql_query = sql_query.bind(uid);
        }
        if let Some(s) = &status {
            let status_str = match s {
                SessionStatus::Active => "Active",
                SessionStatus::Completed => "Completed",
                SessionStatus::Aborted => "Aborted",
            };
            sql_query = sql_query.bind(status_str);
        }

        sql_query = sql_query.bind(limit as i64).bind(offset as i64);

        let rows = sql_query.fetch_all(&self.pool).await?;

        let sessions = rows
            .into_iter()
            .map(|row| {
                let session_status = match row.get::<&str, _>("status") {
                    "Active" => SessionStatus::Active,
                    "Completed" => SessionStatus::Completed,
                    "Aborted" => SessionStatus::Aborted,
                    _ => SessionStatus::Active,
                };

                Session {
                    id: row.get("id"),
                    user_id: row.get("user_id"),
                    booking_id: row.get("booking_id"),
                    microscope_id: row.get("microscope_id"),
                    status: session_status,
                    started_at: DateTime::from_timestamp(
                        row.get::<time::OffsetDateTime, _>("started_at")
                            .unix_timestamp(),
                        0,
                    )
                    .unwrap()
                    .with_timezone(&Utc),
                    ended_at: row
                        .get::<Option<time::OffsetDateTime>, _>("ended_at")
                        .map(|dt| {
                            DateTime::from_timestamp(dt.unix_timestamp(), 0)
                                .unwrap()
                                .with_timezone(&Utc)
                        }),
                    notes: row.get("notes"),
                }
            })
            .collect();

        Ok(sessions)
    }

    pub async fn get_session_by_id(&self, session_id: Uuid) -> Result<Option<Session>, SqlxError> {
        let row = sqlx::query!(
            r#"
            SELECT id, user_id, booking_id, microscope_id,
                   status, started_at, ended_at, notes
            FROM sessions 
            WHERE id = $1
            "#,
            session_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|row| {
            let session_status = match row.status.as_str() {
                "Active" => SessionStatus::Active,
                "Completed" => SessionStatus::Completed,
                "Aborted" => SessionStatus::Aborted,
                _ => SessionStatus::Active,
            };

            Session {
                id: row.id,
                user_id: row.user_id,
                booking_id: row.booking_id,
                microscope_id: row.microscope_id,
                status: session_status,
                started_at: DateTime::from_timestamp(row.started_at.unix_timestamp(), 0)
                    .unwrap()
                    .with_timezone(&Utc),
                ended_at: row.ended_at.map(|dt| {
                    DateTime::from_timestamp(dt.unix_timestamp(), 0)
                        .unwrap()
                        .with_timezone(&Utc)
                }),
                notes: row.notes,
            }
        }))
    }

    pub async fn get_booking_by_id(&self, booking_id: Uuid) -> Result<Option<Booking>, SqlxError> {
        let row = sqlx::query!(
            r#"
            SELECT id, microscope_id, date, slot_start, slot_end, title,
                   group_name, attendees, requester_id, requester_name,
                   status, approved_by, created_at
            FROM bookings 
            WHERE id = $1
            "#,
            booking_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|row| {
            let status = match row.status.as_str() {
                "Pending" => BookingStatus::Pending,
                "Approved" => BookingStatus::Approved,
                "Rejected" => BookingStatus::Rejected,
                _ => BookingStatus::Pending,
            };

            let naive_date = NaiveDate::from_ymd_opt(
                row.date.year(),
                row.date.month() as u32,
                row.date.day() as u32,
            )
            .unwrap();

            Booking {
                id: row.id,
                microscope_id: row.microscope_id,
                date: naive_date,
                slot_start: row.slot_start,
                slot_end: row.slot_end,
                title: row.title,
                group_name: row.group_name,
                attendees: row.attendees,
                requester_id: row.requester_id,
                requester_name: row.requester_name,
                status,
                approved_by: row.approved_by,
                created_at: DateTime::from_timestamp(row.created_at.unix_timestamp(), 0)
                    .unwrap()
                    .fixed_offset(),
            }
        }))
    }

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
            time::OffsetDateTime::from_unix_timestamp(image.captured_at.timestamp()).unwrap()
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
            captured_at: DateTime::from_timestamp(created_image.captured_at.unix_timestamp(), 0)
                .unwrap()
                .with_timezone(&Utc),
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
                    captured_at: DateTime::from_timestamp(row.captured_at.unix_timestamp(), 0)
                        .unwrap()
                        .with_timezone(&Utc),
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
                captured_at: DateTime::from_timestamp(row.captured_at.unix_timestamp(), 0)
                    .unwrap()
                    .with_timezone(&Utc),
            }
        }))
    }

    pub async fn check_booking_conflicts(
        &self,
        microscope_id: &str,
        date: NaiveDate,
        slot_start: i32,
        slot_end: i32,
        exclude_booking_id: Option<Uuid>,
    ) -> Result<bool, SqlxError> {
        // Convert chrono NaiveDate to time Date
        let time_date = time::Date::from_ordinal_date(date.year(), date.ordinal() as u16).unwrap();

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
                time_date,
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
                time_date,
                slot_start,
                slot_end
            )
            .fetch_one(&self.pool)
            .await?;
            result.count.unwrap_or(0)
        };

        Ok(count > 0)
    }

    pub async fn delete_booking(&self, booking_id: Uuid) -> Result<u64, SqlxError> {
        let result = sqlx::query!(
            r#"
            DELETE
            FROM bookings
            WHERE id = $1
        "#,
            booking_id
        )
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected())
    }

    pub async fn get_booking_owner(&self, booking_id: Uuid) -> Result<Option<Uuid>, SqlxError> {
        let result = sqlx::query!(
            "SELECT requester_id FROM bookings WHERE id = $1",
            booking_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.map(|row| row.requester_id))
    }

    pub async fn delete_booking_by_owner(
        &self,
        booking_id: Uuid,
        user_id: Option<Uuid>,
    ) -> Result<u64, SqlxError> {
        let result = sqlx::query!(
            r#"
            DELETE
            FROM bookings
            WHERE id = $1
            AND requester_id = $2
        "#,
            booking_id,
            user_id
        )
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected())
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
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
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
