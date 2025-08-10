// REAL Hardware Interface - NO SIMULATION!
// This module provides ACTUAL hardware access for ALL UI components

use anyhow::{Result, anyhow};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::process::Command;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Clone)]
pub struct HardwareInterface {
    firmware_interface_path: String,
}

impl HardwareInterface {
    pub fn new() -> Self {
        Self {
            firmware_interface_path: "/usr/local/bin/nexus-firmware".to_string(),
        }
    }
    
    // REAL board scanning - calls actual firmware interface
    pub async fn scan_boards(&self) -> Result<Vec<BoardInfo>> {
        let output = Command::new("python3")
            .arg(&self.firmware_interface_path)
            .arg("scan")
            .output()?;
        
        if !output.status.success() {
            return Err(anyhow!("Failed to scan boards: {}", 
                String::from_utf8_lossy(&output.stderr)));
        }
        
        let boards: Vec<BoardInfo> = serde_json::from_slice(&output.stdout)?;
        Ok(boards)
    }
    
    // REAL I/O reading - gets actual values from hardware
    pub async fn read_input(&self, board_type: &str, stack: u8, channel: u8) -> Result<f32> {
        let output = Command::new("python3")
            .arg(&self.firmware_interface_path)
            .arg("read_input")
            .arg(board_type)
            .arg(stack.to_string())
            .arg(channel.to_string())
            .output()?;
        
        if !output.status.success() {
            return Err(anyhow!("Failed to read input: {}", 
                String::from_utf8_lossy(&output.stderr)));
        }
        
        let result: serde_json::Value = serde_json::from_slice(&output.stdout)?;
        Ok(result["value"].as_f64().unwrap_or(0.0) as f32)
    }
    
    // REAL I/O writing - sets actual hardware outputs
    pub async fn write_output(&self, board_type: &str, stack: u8, channel: u8, value: f32) -> Result<()> {
        let output = Command::new("python3")
            .arg(&self.firmware_interface_path)
            .arg("set_output")
            .arg(board_type)
            .arg(stack.to_string())
            .arg(channel.to_string())
            .arg(value.to_string())
            .output()?;
        
        if !output.status.success() {
            return Err(anyhow!("Failed to write output: {}", 
                String::from_utf8_lossy(&output.stderr)));
        }
        
        Ok(())
    }
    
    // REAL relay control
    pub async fn set_relay(&self, board_type: &str, stack: u8, channel: u8, state: bool) -> Result<()> {
        let output = Command::new("python3")
            .arg(&self.firmware_interface_path)
            .arg("set_relay")
            .arg(board_type)
            .arg(stack.to_string())
            .arg(channel.to_string())
            .arg(if state { "1" } else { "0" })
            .output()?;
        
        if !output.status.success() {
            return Err(anyhow!("Failed to set relay: {}", 
                String::from_utf8_lossy(&output.stderr)));
        }
        
        Ok(())
    }
    
    // REAL triac control for MegaBAS
    pub async fn set_triac(&self, stack: u8, channel: u8, value: u8) -> Result<()> {
        let output = Command::new("python3")
            .arg(&self.firmware_interface_path)
            .arg("set_output")
            .arg("megabas")
            .arg(stack.to_string())
            .arg("triac")
            .arg(channel.to_string())
            .arg(value.to_string())
            .output()?;
        
        if !output.status.success() {
            return Err(anyhow!("Failed to set triac: {}", 
                String::from_utf8_lossy(&output.stderr)));
        }
        
        Ok(())
    }
    
    // REAL temperature reading from sensors
    pub async fn read_temperature(&self, sensor_type: &str, channel: u8) -> Result<f32> {
        let output = Command::new("python3")
            .arg(&self.firmware_interface_path)
            .arg("read_temp")
            .arg(sensor_type)
            .arg(channel.to_string())
            .output()?;
        
        if !output.status.success() {
            return Err(anyhow!("Failed to read temperature: {}", 
                String::from_utf8_lossy(&output.stderr)));
        }
        
        let result: serde_json::Value = serde_json::from_slice(&output.stdout)?;
        Ok(result["temperature"].as_f64().unwrap_or(0.0) as f32)
    }
    
    // REAL pressure reading for refrigerant (P499 transducers)
    pub async fn read_pressure(&self, board_type: &str, stack: u8, channel: u8) -> Result<f32> {
        // P499 transducers output 0.5-4.5V for 0-500 PSI
        let voltage = self.read_input(board_type, stack, channel).await?;
        
        // Convert voltage to PSI (actual P499 scaling)
        let psi = ((voltage - 0.5) / 4.0) * 500.0;
        Ok(psi)
    }
    
    // REAL vibration sensor reading via Modbus
    pub async fn read_vibration_sensor(&self, port: &str, sensor_id: u8) -> Result<VibrationData> {
        let output = Command::new("python3")
            .arg("/opt/nexus/read_vibration.py")
            .arg(port)
            .arg(sensor_id.to_string())
            .output()?;
        
        if !output.status.success() {
            return Err(anyhow!("Failed to read vibration: {}", 
                String::from_utf8_lossy(&output.stderr)));
        }
        
        let data: VibrationData = serde_json::from_slice(&output.stdout)?;
        Ok(data)
    }
    
    // REAL system status from actual system
    pub async fn get_system_status(&self) -> Result<SystemStatus> {
        use crate::system_commands::SystemCommands;
        
        let status = SystemCommands::get_status().await?;
        Ok(SystemStatus {
            cpu_temp: status.cpu_temp,
            cpu_usage: self.get_cpu_usage().await?,
            memory_used: status.memory_used,
            memory_total: status.memory_total,
            disk_used: status.disk_used,
            disk_total: status.disk_total,
            uptime_hours: self.parse_uptime(&status.uptime),
        })
    }
    
    // REAL CPU usage from /proc/stat
    async fn get_cpu_usage(&self) -> Result<f32> {
        let output = Command::new("bash")
            .arg("-c")
            .arg("top -bn1 | grep 'Cpu(s)' | awk '{print $2}' | cut -d'%' -f1")
            .output()?;
        
        let usage_str = String::from_utf8_lossy(&output.stdout);
        Ok(usage_str.trim().parse::<f32>().unwrap_or(0.0))
    }
    
    fn parse_uptime(&self, uptime_str: &str) -> f32 {
        // Parse uptime string to hours
        if let Some(days_pos) = uptime_str.find(" day") {
            let days_str = uptime_str[..days_pos].split_whitespace().last().unwrap_or("0");
            let days = days_str.parse::<f32>().unwrap_or(0.0);
            days * 24.0
        } else {
            0.0
        }
    }
    
    // REAL BMS/Modbus communication
    pub async fn read_modbus_register(&self, device: &str, register: u16) -> Result<u16> {
        let output = Command::new("python3")
            .arg("/opt/nexus/modbus_reader.py")
            .arg(device)
            .arg(register.to_string())
            .output()?;
        
        if !output.status.success() {
            return Err(anyhow!("Modbus read failed: {}", 
                String::from_utf8_lossy(&output.stderr)));
        }
        
        let value_str = String::from_utf8_lossy(&output.stdout);
        Ok(value_str.trim().parse::<u16>().unwrap_or(0))
    }
    
    // REAL database queries
    pub async fn query_database(&self, query: &str) -> Result<Vec<serde_json::Value>> {
        use crate::database_service::DatabaseService;
        use crate::database;
        
        let pool = database::get_pool()?;
        let db_service = DatabaseService::new(pool.clone());
        
        let results = db_service.execute_sql(query).await?;
        Ok(results.into_iter().map(|r| serde_json::to_value(r).unwrap()).collect())
    }
    
    // REAL alarm checking
    pub async fn get_active_alarms(&self) -> Result<Vec<Alarm>> {
        let pool = crate::database::get_pool()?;
        
        let alarms = sqlx::query_as!(
            Alarm,
            "SELECT id, alarm_type, source, message, severity, created_at 
             FROM alarms WHERE is_active = 1 ORDER BY created_at DESC"
        )
        .fetch_all(pool)
        .await?;
        
        Ok(alarms)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardInfo {
    pub board_type: String,
    pub stack: u8,
    pub name: String,
    pub version: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VibrationData {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub temperature: f32,
    pub magnitude: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStatus {
    pub cpu_temp: f32,
    pub cpu_usage: f32,
    pub memory_used: u64,
    pub memory_total: u64,
    pub disk_used: u64,
    pub disk_total: u64,
    pub uptime_hours: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alarm {
    pub id: i64,
    pub alarm_type: String,
    pub source: String,
    pub message: String,
    pub severity: String,
    pub created_at: DateTime<Utc>,
}

// Global hardware interface instance
lazy_static::lazy_static! {
    pub static ref HARDWARE: Arc<Mutex<HardwareInterface>> = Arc::new(Mutex::new(HardwareInterface::new()));
}

// Helper function to get hardware interface
pub async fn get_hardware() -> Arc<Mutex<HardwareInterface>> {
    HARDWARE.clone()
}