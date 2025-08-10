// System API Module

use serde::{Deserialize, Serialize};
use anyhow::Result;
use sysinfo::{System, SystemExt, CpuExt, DiskExt, NetworkExt};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::state::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemInfo {
    pub hostname: String,
    pub os: String,
    pub kernel_version: String,
    pub uptime: u64,
    pub cpu_usage: f32,
    pub memory_used: u64,
    pub memory_total: u64,
    pub disk_used: u64,
    pub disk_total: u64,
    pub temperature: Option<f32>,
    pub network_interfaces: Vec<NetworkInterface>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkInterface {
    pub name: String,
    pub ip_address: String,
    pub mac_address: String,
    pub is_up: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogEntry {
    pub id: u64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub level: String,
    pub module: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiagnosticResult {
    pub component: String,
    pub status: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

#[tauri::command]
pub async fn get_system_info() -> Result<SystemInfo, String> {
    let mut sys = System::new_all();
    sys.refresh_all();
    
    // Get CPU usage
    let cpu_usage = sys.global_cpu_info().cpu_usage();
    
    // Get memory info
    let memory_used = sys.used_memory();
    let memory_total = sys.total_memory();
    
    // Get disk info
    let mut disk_used = 0u64;
    let mut disk_total = 0u64;
    for disk in sys.disks() {
        if disk.mount_point().to_str() == Some("/") {
            disk_used = disk.total_space() - disk.available_space();
            disk_total = disk.total_space();
            break;
        }
    }
    
    // Get network interfaces
    let mut network_interfaces = Vec::new();
    for (interface_name, data) in sys.networks() {
        // Skip loopback
        if interface_name == "lo" {
            continue;
        }
        
        network_interfaces.push(NetworkInterface {
            name: interface_name.clone(),
            ip_address: "Unknown".to_string(), // Would need additional crate for IP
            mac_address: format!("{:02x}", data.mac_address()),
            is_up: data.received() > 0 || data.transmitted() > 0,
        });
    }
    
    // Try to read RPi temperature
    let temperature = read_rpi_temperature().await;
    
    Ok(SystemInfo {
        hostname: sys.host_name().unwrap_or_else(|| "unknown".to_string()),
        os: sys.name().unwrap_or_else(|| "unknown".to_string()),
        kernel_version: sys.kernel_version().unwrap_or_else(|| "unknown".to_string()),
        uptime: sys.uptime(),
        cpu_usage,
        memory_used,
        memory_total,
        disk_used,
        disk_total,
        temperature,
        network_interfaces,
    })
}

async fn read_rpi_temperature() -> Option<f32> {
    // Read Raspberry Pi CPU temperature
    match tokio::fs::read_to_string("/sys/class/thermal/thermal_zone0/temp").await {
        Ok(temp_str) => {
            temp_str.trim().parse::<f32>().ok().map(|t| t / 1000.0)
        }
        Err(_) => None,
    }
}

#[tauri::command]
pub async fn get_logs(
    level: Option<String>,
    module: Option<String>,
    limit: Option<usize>,
) -> Result<Vec<LogEntry>, String> {
    // In real implementation, would read from log database
    // For now, return mock logs
    let logs = vec![
        LogEntry {
            id: 1,
            timestamp: chrono::Utc::now(),
            level: "INFO".to_string(),
            module: "auth".to_string(),
            message: "User admin logged in".to_string(),
        },
        LogEntry {
            id: 2,
            timestamp: chrono::Utc::now() - chrono::Duration::minutes(5),
            level: "INFO".to_string(),
            module: "boards".to_string(),
            message: "Megabas board detected at address 0x48".to_string(),
        },
        LogEntry {
            id: 3,
            timestamp: chrono::Utc::now() - chrono::Duration::minutes(10),
            level: "WARNING".to_string(),
            module: "sensors".to_string(),
            message: "Vibration sensor /dev/ttyUSB0 connection lost".to_string(),
        },
    ];
    
    Ok(logs)
}

#[tauri::command]
pub async fn run_diagnostics() -> Result<Vec<DiagnosticResult>, String> {
    let mut results = Vec::new();
    
    // Check I2C bus
    results.push(DiagnosticResult {
        component: "I2C Bus".to_string(),
        status: if std::path::Path::new("/dev/i2c-1").exists() {
            "OK".to_string()
        } else {
            "ERROR".to_string()
        },
        message: "I2C bus availability".to_string(),
        details: None,
    });
    
    // Check database
    results.push(DiagnosticResult {
        component: "Database".to_string(),
        status: if std::path::Path::new("/var/lib/nexus/nexus.db").exists() {
            "OK".to_string()
        } else {
            "WARNING".to_string()
        },
        message: "Database file exists".to_string(),
        details: None,
    });
    
    // Check network
    results.push(DiagnosticResult {
        component: "Network".to_string(),
        status: "OK".to_string(),
        message: "Network connectivity".to_string(),
        details: None,
    });
    
    // Check disk space
    let sys = System::new_all();
    let mut low_disk = false;
    for disk in sys.disks() {
        if disk.mount_point().to_str() == Some("/") {
            let free_percent = (disk.available_space() as f64 / disk.total_space() as f64) * 100.0;
            if free_percent < 10.0 {
                low_disk = true;
            }
        }
    }
    
    results.push(DiagnosticResult {
        component: "Disk Space".to_string(),
        status: if low_disk { "WARNING" } else { "OK" }.to_string(),
        message: if low_disk { "Low disk space" } else { "Adequate disk space" }.to_string(),
        details: None,
    });
    
    Ok(results)
}

#[tauri::command]
pub async fn backup_config(
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
) -> Result<String, String> {
    let app_state = state.lock().await;
    let config = app_state.config.read().await;
    
    // Create backup directory
    let backup_dir = &config.backup_path;
    tokio::fs::create_dir_all(backup_dir).await
        .map_err(|e| e.to_string())?;
    
    // Generate backup filename with timestamp
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let backup_file = format!("{}/nexus_backup_{}.json", backup_dir, timestamp);
    
    // Serialize state to JSON
    let backup_data = serde_json::json!({
        "config": *config,
        "timestamp": chrono::Utc::now(),
        "version": "1.0.0",
    });
    
    let json = serde_json::to_string_pretty(&backup_data)
        .map_err(|e| e.to_string())?;
    
    // Write backup file
    tokio::fs::write(&backup_file, json).await
        .map_err(|e| e.to_string())?;
    
    Ok(backup_file)
}

#[tauri::command]
pub async fn restore_config(
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
    backup_file: String,
) -> Result<(), String> {
    // Read backup file
    let json = tokio::fs::read_to_string(&backup_file).await
        .map_err(|e| format!("Failed to read backup file: {}", e))?;
    
    // Parse backup data
    let backup_data: serde_json::Value = serde_json::from_str(&json)
        .map_err(|e| format!("Invalid backup file: {}", e))?;
    
    // Extract config
    let config = backup_data.get("config")
        .ok_or_else(|| "Backup file missing config data".to_string())?;
    
    let restored_config: crate::state::AppConfig = serde_json::from_value(config.clone())
        .map_err(|e| format!("Failed to restore config: {}", e))?;
    
    // Update state
    let app_state = state.lock().await;
    *app_state.config.write().await = restored_config;
    
    // Save to disk
    app_state.save_config().await
        .map_err(|e| e.to_string())?;
    
    Ok(())
}