-- Vibration sensor configuration and data tables

CREATE TABLE IF NOT EXISTS vibration_sensors (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    sensor_id INTEGER UNIQUE NOT NULL,
    port TEXT NOT NULL,
    name TEXT NOT NULL,
    location TEXT,
    equipment TEXT,
    baud_rate INTEGER DEFAULT 9600,
    enabled BOOLEAN DEFAULT 1,
    calibration_data TEXT, -- JSON
    alarm_thresholds TEXT, -- JSON
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS vibration_readings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    sensor_id INTEGER NOT NULL,
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    accel_x REAL,
    accel_y REAL,
    accel_z REAL,
    velocity_x REAL,
    velocity_y REAL,
    velocity_z REAL,
    freq_x REAL,
    freq_y REAL,
    freq_z REAL,
    temperature REAL,
    rms_velocity REAL,
    severity TEXT,
    peak_velocity REAL,
    crest_factor REAL,
    FOREIGN KEY (sensor_id) REFERENCES vibration_sensors(sensor_id)
);

CREATE INDEX idx_vibration_readings_timestamp ON vibration_readings(timestamp);
CREATE INDEX idx_vibration_readings_sensor ON vibration_readings(sensor_id);

CREATE TABLE IF NOT EXISTS vibration_alarms (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    sensor_id INTEGER NOT NULL,
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    alarm_type TEXT NOT NULL,
    severity TEXT NOT NULL,
    value REAL,
    threshold REAL,
    message TEXT,
    acknowledged BOOLEAN DEFAULT 0,
    acknowledged_by TEXT,
    acknowledged_at TIMESTAMP,
    FOREIGN KEY (sensor_id) REFERENCES vibration_sensors(sensor_id)
);

CREATE TABLE IF NOT EXISTS vibration_trends (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    sensor_id INTEGER NOT NULL,
    period_hours INTEGER,
    calculated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    data_points INTEGER,
    mean_velocity REAL,
    max_velocity REAL,
    min_velocity REAL,
    trend TEXT,
    slope REAL,
    FOREIGN KEY (sensor_id) REFERENCES vibration_sensors(sensor_id)
);