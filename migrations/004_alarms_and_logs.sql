-- Alarm monitoring and system logs

CREATE TABLE IF NOT EXISTS alarm_definitions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT UNIQUE NOT NULL,
    category TEXT NOT NULL,
    severity TEXT NOT NULL,
    enabled BOOLEAN DEFAULT 1,
    condition_type TEXT NOT NULL, -- 'threshold', 'rate_of_change', 'boolean'
    condition_config TEXT, -- JSON configuration
    delay_seconds INTEGER DEFAULT 0,
    auto_reset BOOLEAN DEFAULT 0,
    notification_emails TEXT, -- Comma-separated
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS active_alarms (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    alarm_definition_id INTEGER,
    triggered_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    value REAL,
    message TEXT,
    acknowledged BOOLEAN DEFAULT 0,
    acknowledged_by TEXT,
    acknowledged_at TIMESTAMP,
    cleared BOOLEAN DEFAULT 0,
    cleared_at TIMESTAMP,
    FOREIGN KEY (alarm_definition_id) REFERENCES alarm_definitions(id)
);

CREATE INDEX idx_active_alarms_triggered ON active_alarms(triggered_at);
CREATE INDEX idx_active_alarms_cleared ON active_alarms(cleared);

CREATE TABLE IF NOT EXISTS alarm_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    alarm_definition_id INTEGER,
    triggered_at TIMESTAMP,
    cleared_at TIMESTAMP,
    duration_seconds INTEGER,
    max_value REAL,
    acknowledged BOOLEAN,
    acknowledged_by TEXT,
    notes TEXT,
    FOREIGN KEY (alarm_definition_id) REFERENCES alarm_definitions(id)
);

CREATE TABLE IF NOT EXISTS audit_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    user_id INTEGER,
    username TEXT,
    action TEXT NOT NULL,
    resource TEXT,
    resource_id INTEGER,
    old_value TEXT,
    new_value TEXT,
    ip_address TEXT,
    user_agent TEXT,
    success BOOLEAN DEFAULT 1,
    error_message TEXT
);

CREATE INDEX idx_audit_logs_timestamp ON audit_logs(timestamp);
CREATE INDEX idx_audit_logs_user ON audit_logs(user_id);

CREATE TABLE IF NOT EXISTS system_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    event_type TEXT NOT NULL,
    severity TEXT,
    source TEXT,
    message TEXT,
    details TEXT -- JSON
);

CREATE INDEX idx_system_events_timestamp ON system_events(timestamp);

CREATE TABLE IF NOT EXISTS email_queue (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    to_address TEXT NOT NULL,
    cc_address TEXT,
    subject TEXT NOT NULL,
    body TEXT NOT NULL,
    body_html TEXT,
    priority INTEGER DEFAULT 5,
    attempts INTEGER DEFAULT 0,
    sent BOOLEAN DEFAULT 0,
    sent_at TIMESTAMP,
    error TEXT
);

CREATE TABLE IF NOT EXISTS maintenance_schedules (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    equipment TEXT NOT NULL,
    task TEXT NOT NULL,
    frequency_days INTEGER,
    last_performed TIMESTAMP,
    next_due TIMESTAMP,
    assigned_to TEXT,
    priority TEXT,
    notes TEXT,
    enabled BOOLEAN DEFAULT 1,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);