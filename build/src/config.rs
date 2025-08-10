// Configuration Management Module

use serde::{Deserialize, Serialize};
use anyhow::Result;

pub const DEFAULT_CONFIG_PATH: &str = "/etc/nexus/config.json";
pub const DEFAULT_DATABASE_PATH: &str = "/var/lib/nexus/nexus.db";
pub const DEFAULT_LOG_PATH: &str = "/var/log/nexus";
pub const DEFAULT_BACKUP_PATH: &str = "/var/backups/nexus";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardConfig {
    pub enabled: bool,
    pub name: String,
    pub board_type: String,
    pub address: u8,
    pub port: Option<String>,
    pub polling_interval_ms: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorConfig {
    pub enabled: bool,
    pub name: String,
    pub port: String,
    pub modbus_address: u8,
    pub polling_interval_ms: u32,
    pub calibration: CalibrationConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationConfig {
    pub zero_point_x: f32,
    pub zero_point_y: f32,
    pub zero_point_z: f32,
    pub sensitivity: f32,
    pub filter_frequency: f32,
    pub noise_threshold: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolConfig {
    pub bacnet: BacnetConfig,
    pub modbus: ModbusConfig,
    pub knx: KnxConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacnetConfig {
    pub enabled: bool,
    pub device_id: u32,
    pub device_name: String,
    pub port: u16,
    pub max_masters: u8,
    pub max_info_frames: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModbusConfig {
    pub tcp_enabled: bool,
    pub tcp_port: u16,
    pub rtu_enabled: bool,
    pub rtu_ports: Vec<String>,
    pub default_timeout_ms: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnxConfig {
    pub enabled: bool,
    pub interface_type: String,
    pub gateway_ip: Option<String>,
    pub gateway_port: Option<u16>,
}

pub async fn load_board_configs() -> Result<Vec<BoardConfig>> {
    let config_path = format!("{}/boards.json", DEFAULT_CONFIG_PATH.rsplit_once('/').unwrap().0);
    
    if std::path::Path::new(&config_path).exists() {
        let config_str = tokio::fs::read_to_string(&config_path).await?;
        Ok(serde_json::from_str(&config_str)?)
    } else {
        // Return default configs
        Ok(vec![
            BoardConfig {
                enabled: true,
                name: "Megabas I/O Board".to_string(),
                board_type: "megabas".to_string(),
                address: 0x48,
                port: None,
                polling_interval_ms: 1000,
            },
            BoardConfig {
                enabled: true,
                name: "Building Automation".to_string(),
                board_type: "building_automation".to_string(),
                address: 0x50,
                port: None,
                polling_interval_ms: 1000,
            },
        ])
    }
}

pub async fn save_board_configs(configs: &[BoardConfig]) -> Result<()> {
    let config_dir = DEFAULT_CONFIG_PATH.rsplit_once('/').unwrap().0;
    tokio::fs::create_dir_all(config_dir).await?;
    
    let config_path = format!("{}/boards.json", config_dir);
    let json = serde_json::to_string_pretty(configs)?;
    tokio::fs::write(&config_path, json).await?;
    
    Ok(())
}