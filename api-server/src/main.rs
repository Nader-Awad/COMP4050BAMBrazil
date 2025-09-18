use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing_subscriber;

use bam::{
    create_router,
    services::{DatabaseService, FileStorageService, IAClient},
    AppState, Config,
};

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load configuration
    let config = Arc::new(Config::from_env().expect("Failed to load configuration"));

    // Initialize database connection pool
    let db_pool = PgPoolOptions::new()
        .max_connections(config.database.max_connections)
        .connect(&config.database.url)
        .await
        .expect("Failed to connect to database");

    // Run database migrations
    sqlx::migrate!()
        .run(&db_pool)
        .await
        .expect("Failed to run database migrations");

    // Initialize services
    let database_service = Arc::new(DatabaseService::new(db_pool));
    let file_storage_service = Arc::new(
        FileStorageService::new(config.file_storage.clone())
            .expect("Failed to initialize file storage service"),
    );
    let ia_client = Arc::new(IAClient::new(&config.ia));

    // Initialize application state
    let state = AppState {
        config: Arc::clone(&config),
        db: database_service,
        file_store: file_storage_service,
        ia_client,
    };

    // Build the application router
    let app = create_router(state);

    // Start the server
    let listener = TcpListener::bind(&config.server.bind_address)
        .await
        .expect("Failed to bind to address");

    tracing::info!("Server starting on {}", config.server.bind_address);
    tracing::info!("Database connected to: {}", config.database.url);
    tracing::info!("File storage path: {}", config.file_storage.base_path);
    tracing::info!("IA system URL: {}", config.ia.base_url);

    axum::serve(listener, app)
        .await
        .expect("Server failed to start");
}
