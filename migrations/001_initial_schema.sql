-- Initial database schema for Automata Nexus AI

-- Users table
CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    role TEXT NOT NULL DEFAULT 'operator',
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    last_login DATETIME,
    is_active BOOLEAN DEFAULT 1
);

-- Event logs table
CREATE TABLE IF NOT EXISTS event_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event_type TEXT NOT NULL,
    severity TEXT NOT NULL,
    module TEXT,
    message TEXT NOT NULL,
    details TEXT,
    user_id INTEGER,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id)
);

-- Board states table
CREATE TABLE IF NOT EXISTS board_states (
    board_id TEXT PRIMARY KEY,
    state_json TEXT NOT NULL,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Sensor readings table
CREATE TABLE IF NOT EXISTS sensor_readings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    sensor_id TEXT NOT NULL,
    vibration_x REAL,
    vibration_y REAL,
    vibration_z REAL,
    velocity_x REAL,
    velocity_y REAL,
    velocity_z REAL,
    temperature REAL,
    frequency REAL,
    magnitude REAL,
    status TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_sensor_readings_sensor_id ON sensor_readings(sensor_id);
CREATE INDEX IF NOT EXISTS idx_sensor_readings_created_at ON sensor_readings(created_at);
CREATE INDEX IF NOT EXISTS idx_event_logs_created_at ON event_logs(created_at);
CREATE INDEX IF NOT EXISTS idx_event_logs_severity ON event_logs(severity);

-- Alarms table
CREATE TABLE IF NOT EXISTS alarms (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    alarm_type TEXT NOT NULL,
    source TEXT NOT NULL,
    message TEXT NOT NULL,
    severity TEXT NOT NULL,
    is_acknowledged BOOLEAN DEFAULT 0,
    acknowledged_by INTEGER,
    acknowledged_at DATETIME,
    is_active BOOLEAN DEFAULT 1,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    resolved_at DATETIME,
    FOREIGN KEY (acknowledged_by) REFERENCES users(id)
);

-- Protocol devices table
CREATE TABLE IF NOT EXISTS protocol_devices (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    protocol TEXT NOT NULL,
    device_id TEXT NOT NULL,
    name TEXT NOT NULL,
    address TEXT,
    port INTEGER,
    configuration TEXT,
    is_online BOOLEAN DEFAULT 0,
    last_seen DATETIME,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(protocol, device_id)
);

-- Protocol points table
CREATE TABLE IF NOT EXISTS protocol_points (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    device_id INTEGER NOT NULL,
    point_id TEXT NOT NULL,
    name TEXT NOT NULL,
    point_type TEXT NOT NULL,
    value REAL,
    units TEXT,
    is_writable BOOLEAN DEFAULT 0,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (device_id) REFERENCES protocol_devices(id),
    UNIQUE(device_id, point_id)
);

-- Logic programs table
CREATE TABLE IF NOT EXISTS logic_programs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    description TEXT,
    program_code TEXT NOT NULL,
    is_enabled BOOLEAN DEFAULT 0,
    execution_interval INTEGER DEFAULT 1000,
    last_executed DATETIME,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- System configuration table
CREATE TABLE IF NOT EXISTS system_config (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Insert default admin user
INSERT OR IGNORE INTO users (username, password_hash, role) 
VALUES ('admin', '$2b$12$LQXMvC9kR2K6vt5z5vYPtOaZZ7Sy.N8XjDkFQ5bfKzXz3UqJZaAYC', 'admin');

-- Insert default configuration
INSERT OR IGNORE INTO system_config (key, value) VALUES 
    ('site_name', 'Automata Nexus AI'),
    ('version', '1.0.0'),
    ('timezone', 'America/New_York'),
    ('demo_mode', 'false');