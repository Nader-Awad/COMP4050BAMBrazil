pub mod config;
pub mod error;
pub mod handlers;
pub mod middleware;
pub mod models;
pub mod services;

pub use config::Config;
pub use error::{AppError, AppResult};

use axum::{
    routing::{delete, get, post, put},
    Router,
};
use std::sync::Arc;
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
};
use tracing::Level;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;

/// OpenAPI specification for BAM API Server
#[derive(OpenApi)]
#[openapi(
    info(
        title = "BAM API Server",
        version = "0.1.0",
        description = "Bioscope Booking and Management API for microscope control, booking management, and image capture",
        contact(
            name = "Calleum Pecqueux",
            email = "hello@calleum.au"
        )
    ),
    paths(
        handlers::health_check,
        handlers::auth::login,
        handlers::auth::logout,
        handlers::auth::refresh_token,
        handlers::bookings::list_bookings,
        handlers::bookings::create_booking,
        handlers::bookings::get_booking,
        handlers::bookings::update_booking,
        handlers::bookings::delete_booking,
        handlers::bookings::approve_booking,
        handlers::bookings::reject_booking,
        handlers::sessions::list_sessions,
        handlers::sessions::create_session,
        handlers::sessions::get_current_session,
        handlers::sessions::get_session,
        handlers::sessions::end_session,
        handlers::images::get_image,
        handlers::images::serve_image_file,
        handlers::images::search_images,
        handlers::images::get_all_images_for_session,
        handlers::images::get_latest_image_for_session,
        handlers::images::get_all_images_for_user,
        handlers::microscope::send_command,
        handlers::microscope::get_status,
        handlers::microscope::capture_image,
        handlers::microscope::auto_focus,
        handlers::microscope::start_tracking,
        handlers::microscope::stop_tracking,
        // All new endpoints must be added here with #[utoipa::path] annotations
    ),
    components(
        schemas(
            models::User,
            models::UserRole,
            models::Session,
            models::SessionStatus,
            models::Image,
            models::ImageMetadata,
            models::DetectedObject,
            models::BoundingBox,
            models::Booking,
            models::BookingStatus,
            models::MicroscopeCommand,
            models::CommandType,
            models::ApiResponse<String>,
            handlers::bookings::CreateBookingRequest,
            handlers::bookings::UpdateBookingRequest,
            handlers::sessions::EndSessionRequest,
            handlers::sessions::CreateSessionRequest,
        )
    ),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "authentication", description = "Authentication and authorization"),
        (name = "bookings", description = "Booking management"),
        (name = "sessions", description = "Session tracking"),
        (name = "images", description = "Image management and serving"),
        (name = "microscope", description = "Microscope control and commands")
    )
)]
struct ApiDoc;

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub db: Arc<services::database::DatabaseService>,
    pub file_store: Arc<services::file_storage::FileStorageService>,
    pub ia_client: Arc<services::ia_client::IAClient>,
}

/// Create the main application router with all routes and middleware
pub fn create_router(state: AppState) -> Router {
    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        // OpenAPI documentation
        // Health check
        .route("/health", get(handlers::health_check))
        // Authentication routes
        .route("/api/auth/login", post(handlers::auth::login))
        .route("/api/auth/logout", post(handlers::auth::logout))
        .route("/api/auth/refresh", post(handlers::auth::refresh_token))
        // Booking routes (from existing UI)
        .route("/api/bookings", get(handlers::bookings::list_bookings))
        .route("/api/bookings", post(handlers::bookings::create_booking))
        .route("/api/bookings/{id}", get(handlers::bookings::get_booking))
        .route(
            "/api/bookings/{id}",
            put(handlers::bookings::update_booking),
        )
        .route(
            "/api/bookings/{id}",
            delete(handlers::bookings::delete_booking),
        )
        .route(
            "/api/bookings/{id}/approve",
            post(handlers::bookings::approve_booking),
        )
        .route(
            "/api/bookings/{id}/reject",
            post(handlers::bookings::reject_booking),
        )
        // Session management
        .route("/api/sessions", get(handlers::sessions::list_sessions))
        .route("/api/sessions", post(handlers::sessions::create_session))
        .route(
            "/api/sessions/current",
            get(handlers::sessions::get_current_session),
        )
        .route("/api/sessions/{id}", get(handlers::sessions::get_session))
        .route(
            "/api/sessions/{id}/end",
            post(handlers::sessions::end_session),
        )
        // Image routes
        .route("/api/images/{id}", get(handlers::images::get_image))
        .route(
            "/api/images/{id}/file",
            get(handlers::images::serve_image_file),
        )
        .route("/api/images/search", get(handlers::images::search_images))
        .route(
            "/api/sessions/{session_id}/images",
            get(handlers::images::get_all_images_for_session),
        )
        .route(
            "/api/sessions/{session_id}/images/latest",
            get(handlers::images::get_latest_image_for_session),
        )
        .route(
            "/api/users/{user_id}/images",
            get(handlers::images::get_all_images_for_user),
        )
        // Microscope control routes (proxy to IA)
        .route(
            "/api/microscope/{microscope_id}/command",
            post(handlers::microscope::send_command),
        )
        .route(
            "/api/microscope/{microscope_id}/status",
            get(handlers::microscope::get_status),
        )
        .route(
            "/api/microscope/{microscope_id}/capture",
            post(handlers::microscope::capture_image),
        )
        .route(
            "/api/microscope/{microscope_id}/focus",
            post(handlers::microscope::auto_focus),
        )
        .route(
            "/api/microscope/{microscope_id}/tracking/start",
            post(handlers::microscope::start_tracking),
        )
        .route(
            "/api/microscope/{microscope_id}/tracking/stop",
            post(handlers::microscope::stop_tracking),
        )
        // File serving for static content
        .nest_service("/files", ServeDir::new("uploads"))
        // Add middleware
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            middleware::auth::auth_middleware,
        ))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        )
        // Add state
        .with_state(state)
        .split_for_parts();

    router.merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", api.clone()))
}
