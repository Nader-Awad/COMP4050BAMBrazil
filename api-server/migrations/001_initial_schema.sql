-- BAM Database Schema
-- Initial migration for Bioscope Booking and Management system

-- Create extension for UUID generation
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Users table
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    role VARCHAR(50) NOT NULL CHECK (role IN ('Student', 'Teacher', 'Admin')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Microscopes table
CREATE TABLE IF NOT EXISTS microscopes (
    id VARCHAR(50) PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    location VARCHAR(255),
    status VARCHAR(50) NOT NULL DEFAULT 'Available' CHECK (status IN ('Available', 'InUse', 'Maintenance', 'Offline')),
    specs JSONB, -- Store microscope specifications
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Bookings table
CREATE TABLE IF NOT EXISTS bookings (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    microscope_id VARCHAR(50) NOT NULL REFERENCES microscopes(id),
    date DATE NOT NULL,
    slot_start INTEGER NOT NULL, -- Minutes from midnight
    slot_end INTEGER NOT NULL, -- Minutes from midnight
    title VARCHAR(255) NOT NULL,
    group_name VARCHAR(255),
    attendees INTEGER,
    requester_id UUID NOT NULL REFERENCES users(id),
    requester_name VARCHAR(255) NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'Pending' CHECK (status IN ('Pending', 'Approved', 'Rejected')),
    approved_by UUID REFERENCES users(id),
    approved_at TIMESTAMPTZ,
    rejection_reason TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT booking_time_valid CHECK (slot_end > slot_start),
    CONSTRAINT booking_slots_school_hours CHECK (slot_start >= 480 AND slot_end <= 1020) -- 8AM to 5PM
);

-- Sessions table (active microscope usage)
CREATE TABLE IF NOT EXISTS sessions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id),
    booking_id UUID REFERENCES bookings(id),
    microscope_id VARCHAR(50) NOT NULL REFERENCES microscopes(id),
    status VARCHAR(50) NOT NULL DEFAULT 'Active' CHECK (status IN ('Active', 'Completed', 'Aborted')),
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ended_at TIMESTAMPTZ,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT session_end_after_start CHECK (ended_at IS NULL OR ended_at > started_at)
);

-- Images table
CREATE TABLE IF NOT EXISTS images (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    session_id UUID NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    filename VARCHAR(255) NOT NULL,
    file_path VARCHAR(500) NOT NULL,
    content_type VARCHAR(100) NOT NULL,
    file_size BIGINT NOT NULL,
    width INTEGER,
    height INTEGER,
    metadata JSONB NOT NULL DEFAULT '{}', -- AI-generated metadata
    captured_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT positive_file_size CHECK (file_size > 0),
    CONSTRAINT positive_dimensions CHECK (
        (width IS NULL AND height IS NULL) OR 
        (width > 0 AND height > 0)
    )
);

-- Create indexes for better query performance
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_users_role ON users(role);

CREATE INDEX IF NOT EXISTS idx_bookings_microscope_date ON bookings(microscope_id, date);
CREATE INDEX IF NOT EXISTS idx_bookings_requester ON bookings(requester_id);
CREATE INDEX IF NOT EXISTS idx_bookings_status ON bookings(status);
CREATE INDEX IF NOT EXISTS idx_bookings_date_slots ON bookings(date, slot_start, slot_end);

CREATE INDEX IF NOT EXISTS idx_sessions_user ON sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_sessions_microscope ON sessions(microscope_id);
CREATE INDEX IF NOT EXISTS idx_sessions_booking ON sessions(booking_id);
CREATE INDEX IF NOT EXISTS idx_sessions_status ON sessions(status);
CREATE INDEX IF NOT EXISTS idx_sessions_active ON sessions(status) WHERE status = 'Active';

CREATE INDEX IF NOT EXISTS idx_images_session ON images(session_id);
CREATE INDEX IF NOT EXISTS idx_images_captured_at ON images(captured_at);
CREATE INDEX IF NOT EXISTS idx_images_metadata_gin ON images USING gin(metadata); -- For JSON queries

-- Create updated_at trigger function
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Add updated_at triggers to all tables (drop and recreate to ensure idempotency)
DROP TRIGGER IF EXISTS update_users_updated_at ON users;
CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON users 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_microscopes_updated_at ON microscopes;
CREATE TRIGGER update_microscopes_updated_at BEFORE UPDATE ON microscopes 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_bookings_updated_at ON bookings;
CREATE TRIGGER update_bookings_updated_at BEFORE UPDATE ON bookings 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_sessions_updated_at ON sessions;
CREATE TRIGGER update_sessions_updated_at BEFORE UPDATE ON sessions 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Insert default microscopes (only if they don't exist)
INSERT INTO microscopes (id, name, location, specs) 
SELECT 'bio-1', 'Bioscope A', 'Lab Room 101', '{"max_magnification": "1000x", "type": "compound"}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM microscopes WHERE id = 'bio-1');

INSERT INTO microscopes (id, name, location, specs) 
SELECT 'bio-2', 'Bioscope B', 'Lab Room 102', '{"max_magnification": "1000x", "type": "compound"}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM microscopes WHERE id = 'bio-2');

INSERT INTO microscopes (id, name, location, specs) 
SELECT 'bio-3', 'Bioscope C', 'Lab Room 103', '{"max_magnification": "400x", "type": "stereo"}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM microscopes WHERE id = 'bio-3');

-- Insert default users (only if they don't exist) - password: admin123
INSERT INTO users (name, email, password_hash, role) 
SELECT 'Admin User', 'admin@bam.edu', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewdBPj4MRGdbtK/K', 'Admin'
WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'admin@bam.edu');

INSERT INTO users (name, email, password_hash, role) 
SELECT 'Teacher User', 'teacher@bam.edu', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewdBPj4MRGdbtK/K', 'Teacher'
WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'teacher@bam.edu');

INSERT INTO users (name, email, password_hash, role) 
SELECT 'Student User', 'student@bam.edu', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewdBPj4MRGdbtK/K', 'Student'
WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'student@bam.edu');
