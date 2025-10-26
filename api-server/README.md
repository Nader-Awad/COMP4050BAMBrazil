# BAM API Server

Rust backend API server for the Bioscope Booking and Management (BAM) system.

## Architecture

This API server acts as the central backend for the BAM system, handling:

- **Authentication**: JWT-based user authentication and authorization with role-based access control
- **Booking Management**: CRUD operations for microscope bookings with approval workflows
- **Image Management**: Storage and retrieval of microscope images with AI metadata
- **Microscope Control**: Proxy commands to microscope hardware via IA system integration
- **Session Management**: Tracking active microscope usage sessions
- **OpenAPI Documentation**: Auto-generated API documentation with Swagger UI

## Technology Stack

### Core Framework
- **Axum 0.8**: Modern async web framework
- **Tokio**: Async runtime with full feature set
- **Tower**: Service middleware and utilities
- **Hyper**: HTTP implementation

### Database & ORM
- **PostgreSQL**: Primary database
- **SQLx 0.8**: Async database driver with compile-time query checking
- **Sea-ORM 1.1**: Modern ORM with migration support

### Authentication & Security
- **JWT**: JSON Web Token authentication
- **bcrypt**: Password hashing
- **UUID v4**: Secure identifier generation

### External Integration
- **Reqwest**: HTTP client for IA system communication
- **Multipart**: File upload handling

### Documentation
- **utoipa**: OpenAPI 3.0 specification generation
- **utoipa-swagger-ui**: Interactive API documentation
- **utoipa-axum**: Axum integration for OpenAPI

### Development & Observability
- **tracing**: Structured logging
- **anyhow/thiserror**: Error handling
- **validator**: Request validation

## Key Components

### Models
- **User**: Authentication and role management (Student/Teacher/Admin)  
- **Booking**: Microscope reservation system with approval workflow
- **Session**: Active microscope usage tracking
- **Image**: Microscope captures with AI-generated metadata
- **MicroscopeCommand**: Control commands sent to hardware

### API Endpoints

#### Authentication
- `POST /api/auth/login` - User login with JWT token generation
- `POST /api/auth/logout` - User logout 
- `POST /api/auth/refresh` - Refresh JWT token

#### Bookings (from existing UI)
- `GET /api/bookings` - List bookings with filtering
- `POST /api/bookings` - Create new booking request
- `PUT /api/bookings/{id}` - Update booking
- `POST /api/bookings/{id}/approve` - Approve booking (teacher/admin)
- `POST /api/bookings/{id}/reject` - Reject booking (teacher/admin)

#### Sessions
- `GET /api/sessions` - List active sessions
- `POST /api/sessions` - Start new microscope session
- `POST /api/sessions/{id}/end` - End session

#### Images
- `GET /api/images/{id}` - Get image metadata
- `GET /api/images/{id}/file` - Serve image file
- `GET /api/sessions/{session_id}/images` - List images for session
- `GET /api/sessions/{session_id}/images/latest` - Get latest image
- `GET /api/users/{user_id}/images` - List user's images

#### Microscope Control (Proxy to IA System)
- `POST /api/microscope/{id}/command` - Send control command
- `GET /api/microscope/{id}/status` - Get microscope status
- `POST /api/microscope/{id}/capture` - Capture image
- `POST /api/microscope/{id}/focus` - Auto focus
- `POST /api/microscope/{id}/tracking/start` - Start object tracking
- `POST /api/microscope/{id}/tracking/stop` - Stop tracking

## Development Setup

### Prerequisites
- **Rust 1.70+**: Latest stable Rust toolchain will work
- **PostgreSQL**: Database server (version 12+)
- **IA System**: OrangePi hardware controller running IA integration code (not needed for testing)

### Quick Start

1. **Environment Configuration**
   ```bash
   cp .env.example .env
   # Edit .env with your configuration
   ```

2. **Database Setup**
   ```bash
   # Create database
   createdb bam_db
   
   # Run migrations
   cargo run --bin migrate
   ```

3. **Development Server**
   ```bash
   cargo run
   ```
   Server starts on `http://localhost:3000` by default.

4. **API Documentation**
   Visit `http://localhost:3000/swagger-ui/` for interactive API documentation.

### Available Commands

- `cargo run` - Start development server
- `cargo test` - Run test suite
- `cargo build --release` - Build optimized production binary
- `cargo check` - Fast compile-time checks
- `sqlx migrate run` - Apply database migrations

### Configuration

Edit `.env` file with your settings:

```env
# Server
BIND_ADDRESS=0.0.0.0:3000
PORT=3000

# Database
DATABASE_URL=postgresql://user:pass@localhost:5432/bam_db
DATABASE_MAX_CONNECTIONS=10

# Authentication
JWT_SECRET=your-super-secret-jwt-key-here
TOKEN_EXPIRY=3600

# File Storage
FILE_STORAGE_PATH=./uploads
MAX_FILE_SIZE=52428800

# IA System Integration
IA_BASE_URL=http://192.168.1.100:8080
IA_TIMEOUT=30
IA_MOCK_MODE=false

# Logging
RUST_LOG=info
```

## Integration with IA System

The API server communicates with the IA code running on OrangePi through REST endpoints:

- **Image Capture**: Requests image capture and receives file + AI metadata
- **Microscope Control**: Proxies UI commands (move, focus, magnification)
- **Object Detection**: Receives AI analysis results for captured images
- **Auto Tracking**: Controls automated object tracking features

### IA Mock Mode

Set `IA_MOCK_MODE=true` (or run `make dev` from the repo root) to exercise the entire capture pipeline without IA hardware. Mock mode short-circuits the IA client so `capture_image`, `download_image`, session sync, and metadata uploads return deterministic fake responses suitable for local development and automated testing.

## File Storage

Images are stored in the configured file storage directory with metadata saved to PostgreSQL. The system supports:

- Multiple image formats (JPEG, PNG, TIFF, BMP)
- File size limits
- Organized directory structure by session/date
- Secure file serving with authentication

## Authentication & Authorization

Role-based access control:
- **Students**: Can create bookings, view their sessions/images
- **Teachers**: Can approve/reject bookings, view all sessions
- **Admins**: Full system access including user management

JWT tokens include user role and session information for fine-grained authorization.
