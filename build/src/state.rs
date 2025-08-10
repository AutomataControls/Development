// Application State Management

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use anyhow::Result;
use dashmap::DashMap;
use crate::auth::User;
use crate::boards::{Board, BoardState};
use crate::sensors::{VibrationSensor, SensorReading};

#[derive(Debug, Clone)]
pub struct AppState {
    pub users: Arc<RwLock<HashMap<String, User>>>,
    pub boards: Arc<DashMap<String, Board>>,
    pub board_states: Arc<DashMap<String, BoardState>>,
    pub sensors: Arc<DashMap<String, VibrationSensor>>,
    pub sensor_readings: Arc<DashMap<String, Vec<SensorReading>>>,
    pub weather_data: Arc<RwLock<Option<WeatherData>>>,
    pub config: Arc<RwLock<AppConfig>>,
    pub is_demo_mode: Arc<RwLock<bool>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub site_name: String,
    pub location: Location,
    pub weather_api_key: Option<String>,
    pub email_settings: EmailSettings,
    pub alarm_settings: AlarmSettings,
    pub database_path: String,
    pub backup_path: String,
    pub log_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub latitude: f64,
    pub longitude: f64,
    pub timezone: String,
    pub city: String,
    pub country: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailSettings {
    pub enabled: bool,
    pub smtp_server: String,
    pub smtp_port: u16,
    pub username: String,
    pub password: String,
    pub from_address: String,
    pub alert_recipients: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlarmSettings {
    pub email_alerts: bool,
    pub sms_alerts: bool,
    pub push_notifications: bool,
    pub alarm_delay_seconds: u32,
    pub repeat_interval_minutes: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherData {
    pub temperature: f32,
    pub humidity: f32,
    pub pressure: f32,
    pub wind_speed: f32,
    pub wind_direction: f32,
    pub description: String,
    pub icon: String,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl AppState {
    pub async fn new() -> Result<Self> {
        // Load configuration
        let config = Self::load_config().await?;
        
        // Initialize users
        let mut users = HashMap::new();
        users.insert("admin".to_string(), User {
            username: "admin".to_string(),
            password_hash: bcrypt::hash("Nexus", bcrypt::DEFAULT_COST)?,
            role: "admin".to_string(),
            created_at: chrono::Utc::now(),
            last_login: None,
        });

        Ok(Self {
            users: Arc::new(RwLock::new(users)),
            boards: Arc::new(DashMap::new()),
            board_states: Arc::new(DashMap::new()),
            sensors: Arc::new(DashMap::new()),
            sensor_readings: Arc::new(DashMap::new()),
            weather_data: Arc::new(RwLock::new(None)),
            config: Arc::new(RwLock::new(config)),
            is_demo_mode: Arc::new(RwLock::new(false)),
        })
    }

    async fn load_config() -> Result<AppConfig> {
        // Try to load from file, otherwise use defaults
        let config_path = "/etc/nexus/config.json";
        
        if std::path::Path::new(config_path).exists() {
            let config_str = tokio::fs::read_to_string(config_path).await?;
            Ok(serde_json::from_str(&config_str)?)
        } else {
            Ok(AppConfig {
                site_name: "Automata Nexus AI".to_string(),
                location: Location {
                    latitude: 40.7128,
                    longitude: -74.0060,
                    timezone: "America/New_York".to_string(),
                    city: "New York".to_string(),
                    country: "US".to_string(),
                },
                weather_api_key: None,
                email_settings: EmailSettings {
                    enabled: false,
                    smtp_server: "smtp.gmail.com".to_string(),
                    smtp_port: 587,
                    username: String::new(),
                    password: String::new(),
                    from_address: "nexus@automata.ai".to_string(),
                    alert_recipients: vec![],
                },
                alarm_settings: AlarmSettings {
                    email_alerts: true,
                    sms_alerts: false,
                    push_notifications: false,
                    alarm_delay_seconds: 30,
                    repeat_interval_minutes: 15,
                },
                database_path: "/var/lib/nexus/nexus.db".to_string(),
                backup_path: "/var/backups/nexus".to_string(),
                log_level: "info".to_string(),
            })
        }
    }

    pub async fn save_config(&self) -> Result<()> {
        let config = self.config.read().await;
        let config_str = serde_json::to_string_pretty(&*config)?;
        
        // Ensure directory exists
        tokio::fs::create_dir_all("/etc/nexus").await?;
        
        // Write config
        tokio::fs::write("/etc/nexus/config.json", config_str).await?;
        
        Ok(())
    }
}