-- Refrigerant diagnostics and P499 transducer tables

CREATE TABLE IF NOT EXISTS p499_transducers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    channel INTEGER UNIQUE NOT NULL,
    name TEXT NOT NULL,
    location TEXT NOT NULL,
    min_psi REAL NOT NULL,
    max_psi REAL NOT NULL,
    calibration_offset REAL DEFAULT 0,
    calibration_scale REAL DEFAULT 1,
    enabled BOOLEAN DEFAULT 1,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS transducer_readings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    channel INTEGER NOT NULL,
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    raw_voltage REAL,
    pressure_psi REAL,
    pressure_bar REAL,
    FOREIGN KEY (channel) REFERENCES p499_transducers(channel)
);

CREATE INDEX idx_transducer_readings_timestamp ON transducer_readings(timestamp);

CREATE TABLE IF NOT EXISTS system_configurations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT UNIQUE NOT NULL,
    refrigerant_type TEXT NOT NULL,
    system_type TEXT NOT NULL,
    manufacturer TEXT,
    model TEXT,
    serial TEXT,
    tonnage REAL,
    age_years INTEGER,
    equipment_type TEXT,
    design_subcooling REAL,
    design_superheat REAL,
    design_delta_t REAL,
    design_ambient REAL,
    active BOOLEAN DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS diagnostic_readings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    system_id INTEGER,
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    suction_pressure REAL,
    discharge_pressure REAL,
    suction_temperature REAL,
    discharge_temperature REAL,
    liquid_line_temperature REAL,
    ambient_temperature REAL,
    return_air_temperature REAL,
    supply_air_temperature REAL,
    indoor_wet_bulb REAL,
    indoor_dry_bulb REAL,
    compressor_amps REAL,
    FOREIGN KEY (system_id) REFERENCES system_configurations(id)
);

CREATE INDEX idx_diagnostic_readings_timestamp ON diagnostic_readings(timestamp);

CREATE TABLE IF NOT EXISTS diagnostic_results (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    reading_id INTEGER,
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    superheat REAL,
    subcooling REAL,
    approach_temperature REAL,
    delta_t REAL,
    pressure_ratio REAL,
    condensing_temp REAL,
    evaporating_temp REAL,
    discharge_superheat REAL,
    efficiency_score REAL,
    FOREIGN KEY (reading_id) REFERENCES diagnostic_readings(id)
);

CREATE TABLE IF NOT EXISTS diagnostic_faults (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    result_id INTEGER,
    fault_type TEXT NOT NULL,
    severity TEXT NOT NULL,
    description TEXT,
    confidence REAL,
    impact TEXT,
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (result_id) REFERENCES diagnostic_results(id)
);

CREATE TABLE IF NOT EXISTS refrigerant_database (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    r_number TEXT UNIQUE NOT NULL,
    name TEXT NOT NULL,
    molecular_weight REAL,
    boiling_point_f REAL,
    critical_temp_f REAL,
    critical_pressure_psi REAL,
    ozone_depletion REAL,
    gwp REAL,
    safety_class TEXT,
    pt_curve TEXT -- JSON array of PT points
);