# BAM API Server

Rust backend API server for the Bioscope Booking and Management system.

## Architecture

This API server acts as the central backend for the BAM system, handling:

- **Authentication**: JWT-based user authentication and authorization
- **Booking Management**: CRUD operations for microscope bookings
- **Image Management**: Storage and retrieval of microscope images with AI metadata
- **Microscope Control**: Proxy commands to microscope hardware via IA system
- **Session Management**: Tracking active microscope usage sessions

## Key Components

### Models
- **User**: Authentication and role management (Student/Teacher/Admin)  
- **Booking**: Microscope reservation system from the UI
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
- Rust 1.70+
- PostgreSQL database
- IA system running on OrangePi

### Configuration
1. Copy `.env.example` to `.env` and configure:
   - Database connection
   - JWT secret
   - IA system URL
   - File storage path

### Running
```bash
cargo run
```

### Testing
```bash
cargo test
```

## Integration with IA System

The API server communicates with the IA code running on OrangePi through REST endpoints:

- **Image Capture**: Requests image capture and receives file + AI metadata
- **Microscope Control**: Proxies UI commands (move, focus, magnification)
- **Object Detection**: Receives AI analysis results for captured images
- **Auto Tracking**: Controls automated object tracking features

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