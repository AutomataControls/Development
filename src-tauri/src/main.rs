// AutomataNexus Universal Vibration Monitor
// Professional Industrial Monitoring Desktop Application
// Specifically configured for WIT-Motion WTVB01-485 sensors
// (c) 2025 AutomataNexus AI & AutomataControls

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod wtvb01_sensor;

use wtvb01_sensor::{WTVBSensorManager, WTVBSensorReading, WTVBSensorConfig};
use std::sync::Arc;
use tauri::State;
use std::collections::HashMap;

struct AppState {
    sensor_manager: Arc<WTVBSensorManager>,
}

// Scan for available USB ports
#[tauri::command]
fn scan_ports(state: State<AppState>) -> Vec<String> {
    println!("[COMMAND] scan_ports called from frontend");
    
    // Check permissions
    let test_port = "/dev/ttyUSB0";
    if std::path::Path::new(test_port).exists() {
        match std::fs::File::open(test_port) {
            Ok(_) => println!("[COMMAND] Serial port access OK"),
            Err(e) => {
                println!("[COMMAND] WARNING: Cannot access {}. Error: {}", test_port, e);
                println!("[COMMAND] You may need to run: sudo usermod -a -G dialout $USER");
                println!("[COMMAND] Or run with: sudo ./automatanexus-vibration-monitor");
            }
        }
    }
    
    let ports = state.sensor_manager.scan_ports();
    println!("[COMMAND] Found {} ports:", ports.len());
    for port in &ports {
        println!("[COMMAND]   - {}", port);
    }
    
    ports
}

// Get all sensor readings
#[tauri::command]
fn get_readings(state: State<AppState>) -> HashMap<String, WTVBSensorReading> {
    println!("[COMMAND] get_readings called");
    let readings = state.sensor_manager.get_all_readings();
    println!("[COMMAND] Returning {} readings", readings.len());
    readings
}

// Read a specific sensor
#[tauri::command]
fn read_sensor(port: String, state: State<AppState>) -> Result<WTVBSensorReading, String> {
    println!("[COMMAND] read_sensor called for port: {}", port);
    let result = state.sensor_manager.read_sensor(&port);
    
    match &result {
        Ok(reading) => {
            println!("[COMMAND] Successfully read WTVB01 sensor:");
            println!("  - Velocity: {:.2} mm/s", reading.velocity_mms);
            println!("  - Temperature: {:.1}°C / {:.1}°F", reading.temperature_c, reading.temperature_f);
            println!("  - Acceleration: X={:.3}g Y={:.3}g Z={:.3}g", 
                     reading.accel_x, reading.accel_y, reading.accel_z);
            println!("  - Frequency: X={:.1}Hz Y={:.1}Hz Z={:.1}Hz",
                     reading.frequency_x, reading.frequency_y, reading.frequency_z);
            println!("  - ISO Zone: {}", reading.iso_zone);
        },
        Err(e) => println!("[COMMAND] Failed to read sensor: {}", e),
    }
    
    result
}

// Configure a sensor
#[tauri::command]
fn configure_sensor(config: WTVBSensorConfig, state: State<AppState>) -> Result<String, String> {
    println!("[COMMAND] configure_sensor called for port: {}", config.port);
    println!("  - Name: {}", config.name);
    println!("  - Modbus ID: 0x{:02X}", config.modbus_id);
    println!("  - Baud Rate: {}", config.baud_rate);
    
    state.sensor_manager.configure_sensor(config)?;
    Ok("WTVB01 sensor configured successfully".to_string())
}

// Configure device parameters (Modbus ID, baud rate)
#[tauri::command]
fn configure_device(
    port: String, 
    modbus_id: u8, 
    new_id: Option<u8>, 
    new_baud: Option<u32>,
    state: State<AppState>
) -> Result<String, String> {
    println!("[COMMAND] configure_device called");
    println!("  - Port: {}", port);
    println!("  - Current ID: 0x{:02X}", modbus_id);
    
    if let Some(id) = new_id {
        println!("  - New ID: 0x{:02X}", id);
    }
    
    if let Some(baud) = new_baud {
        println!("  - New Baud: {}", baud);
    }
    
    state.sensor_manager.configure_device(&port, modbus_id, new_id, new_baud)
}

// Start monitoring (reads all configured sensors)
#[tauri::command]
fn start_monitoring(state: State<AppState>) -> Result<String, String> {
    println!("[COMMAND] start_monitoring called");
    let ports = state.sensor_manager.scan_ports();
    println!("[COMMAND] Starting monitoring on {} ports", ports.len());
    
    let mut success_count = 0;
    let mut error_count = 0;
    
    for port in &ports {
        println!("[COMMAND] Reading from port: {}", port);
        match state.sensor_manager.read_sensor(port) {
            Ok(_) => success_count += 1,
            Err(e) => {
                println!("[COMMAND] Error reading {}: {}", port, e);
                error_count += 1;
            }
        }
    }
    
    Ok(format!("Monitoring started: {} sensors OK, {} errors", success_count, error_count))
}

// Stop monitoring
#[tauri::command]
fn stop_monitoring(_state: State<AppState>) -> Result<String, String> {
    println!("[COMMAND] stop_monitoring called");
    // In a real implementation, this would stop background threads
    Ok("Monitoring stopped".to_string())
}

// Burst read sensor (all 19 registers in one command)
#[tauri::command]
fn read_sensor_burst(port: String, state: State<AppState>) -> Result<WTVBSensorReading, String> {
    println!("[COMMAND] read_sensor_burst called for port: {}", port);
    let result = state.sensor_manager.read_sensor_burst(&port);
    
    match &result {
        Ok(reading) => {
            println!("[COMMAND] BURST READ successful:");
            println!("  - Read all 19 registers in single command");
            println!("  - Velocity: {:.2} mm/s", reading.velocity_mms);
            println!("  - Temperature: {:.1}°C", reading.temperature_c);
        },
        Err(e) => println!("[COMMAND] Burst read failed: {}", e),
    }
    
    result
}

// Optimize sensor for maximum speed (230400 baud)
#[tauri::command]
fn optimize_for_speed(port: String, state: State<AppState>) -> Result<String, String> {
    println!("[COMMAND] optimize_for_speed called for port: {}", port);
    state.sensor_manager.optimize_for_speed(&port)
}

// Enable high-speed mode (1000Hz sampling)
#[tauri::command]
fn enable_high_speed_mode(
    port: String,
    modbus_id: u8,
    state: State<AppState>
) -> Result<String, String> {
    println!("[COMMAND] enable_high_speed_mode called");
    println!("  - Port: {}", port);
    println!("  - Modbus ID: 0x{:02X}", modbus_id);
    state.sensor_manager.enable_high_speed_mode(&port, modbus_id)
}

// Get device info
#[tauri::command]
fn get_device_info() -> Result<HashMap<String, String>, String> {
    let mut info = HashMap::new();
    info.insert("sensor_model".to_string(), "WIT-Motion WTVB01-485".to_string());
    info.insert("protocol".to_string(), "Modbus RTU".to_string());
    info.insert("default_baud".to_string(), "115200".to_string());
    info.insert("max_baud".to_string(), "230400".to_string());
    info.insert("default_modbus_id".to_string(), "0x50".to_string());
    info.insert("manual_version".to_string(), "v25-05-06".to_string());
    info.insert("software_version".to_string(), "1.0.0".to_string());
    info.insert("features".to_string(), "Burst Reading, High-Speed Mode (1000Hz)".to_string());
    Ok(info)
}

fn main() {
    println!("=====================================");
    println!("AutomataNexus Vibration Monitor v1.0");
    println!("WIT-Motion WTVB01-485 Edition");
    println!("=====================================");
    
    let app_state = AppState {
        sensor_manager: Arc::new(WTVBSensorManager::new()),
    };

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            scan_ports,
            get_readings,
            read_sensor,
            read_sensor_burst,
            configure_sensor,
            configure_device,
            optimize_for_speed,
            enable_high_speed_mode,
            start_monitoring,
            stop_monitoring,
            get_device_info
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}