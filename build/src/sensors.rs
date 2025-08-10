// Vibration Sensor Module - WTVB01-485 Support

use serde::{Deserialize, Serialize};
use anyhow::{Result, anyhow};
use std::sync::Arc;
use tokio::sync::Mutex;
use serialport::{SerialPort, SerialPortInfo};
use std::time::Duration;
use std::io::{Read, Write};
use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VibrationSensor {
    pub port: String,
    pub name: String,
    pub location: String,
    pub modbus_address: u8,
    pub is_connected: bool,
    pub calibration: SensorCalibration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorCalibration {
    pub zero_point: Vector3,
    pub sensitivity: f32,
    pub filter_frequency: f32,
    pub noise_threshold: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorReading {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub vibration: Vector3,
    pub velocity: Vector3,
    pub temperature: f32,
    pub frequency: f32,
    pub magnitude: f32,
    pub status: VibrationStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VibrationStatus {
    Good,
    Warning,
    Alert,
    Unknown,
}

impl Default for SensorCalibration {
    fn default() -> Self {
        Self {
            zero_point: Vector3 { x: 0.0, y: 0.0, z: 0.0 },
            sensitivity: 1.0,
            filter_frequency: 10.0,
            noise_threshold: 0.05,
        }
    }
}

// WTVB01 Modbus registers
const REG_VIBRATION_X: u16 = 0x0000;
const REG_VIBRATION_Y: u16 = 0x0002;
const REG_VIBRATION_Z: u16 = 0x0004;
const REG_VELOCITY_X: u16 = 0x0006;
const REG_VELOCITY_Y: u16 = 0x0008;
const REG_VELOCITY_Z: u16 = 0x000A;
const REG_TEMPERATURE: u16 = 0x000C;
const REG_FREQUENCY: u16 = 0x000E;

#[tauri::command]
pub async fn scan_vibration_sensors() -> Result<Vec<String>, String> {
    let ports = serialport::available_ports()
        .map_err(|e| format!("Failed to scan ports: {}", e))?;
    
    let usb_ports: Vec<String> = ports
        .into_iter()
        .filter_map(|p| {
            if p.port_name.contains("ttyUSB") || p.port_name.contains("ttyACM") {
                Some(p.port_name)
            } else {
                None
            }
        })
        .collect();
    
    Ok(usb_ports)
}

#[tauri::command]
pub async fn add_vibration_sensor(
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
    port: String,
    name: String,
    location: String,
    modbus_address: u8,
) -> Result<(), String> {
    let app_state = state.lock().await;
    
    let sensor = VibrationSensor {
        port: port.clone(),
        name,
        location,
        modbus_address,
        is_connected: false,
        calibration: SensorCalibration::default(),
    };
    
    app_state.sensors.insert(port, sensor);
    
    Ok(())
}

#[tauri::command]
pub async fn remove_vibration_sensor(
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
    port: String,
) -> Result<(), String> {
    let app_state = state.lock().await;
    app_state.sensors.remove(&port);
    Ok(())
}

#[tauri::command]
pub async fn calibrate_sensor(
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
    port: String,
    calibration: SensorCalibration,
) -> Result<(), String> {
    let app_state = state.lock().await;
    
    app_state.sensors.entry(port)
        .and_modify(|sensor| {
            sensor.calibration = calibration;
        });
    
    Ok(())
}

#[tauri::command]
pub async fn get_sensor_readings(
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
    port: String,
) -> Result<Vec<SensorReading>, String> {
    let app_state = state.lock().await;
    
    if let Some(readings) = app_state.sensor_readings.get(&port) {
        return Ok(readings.clone());
    }
    
    Ok(Vec::new())
}

#[tauri::command]
pub async fn start_vibration_monitoring(
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
) -> Result<(), String> {
    // Start monitoring is handled by the background task
    Ok(())
}

#[tauri::command]
pub async fn stop_vibration_monitoring(
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
) -> Result<(), String> {
    // Stop monitoring would set a flag checked by background task
    Ok(())
}

async fn read_sensor(sensor: &VibrationSensor) -> Result<SensorReading> {
    let mut port = serialport::new(&sensor.port, 9600)
        .timeout(Duration::from_millis(1000))
        .open()
        .map_err(|e| anyhow!("Failed to open port: {}", e))?;
    
    // Build Modbus RTU read request
    let request = build_modbus_read_request(sensor.modbus_address, REG_VIBRATION_X, 16);
    
    // Send request
    port.write_all(&request)
        .map_err(|e| anyhow!("Failed to write to port: {}", e))?;
    
    // Read response
    let mut response = vec![0u8; 37]; // Expected response size
    port.read_exact(&mut response)
        .map_err(|e| anyhow!("Failed to read from port: {}", e))?;
    
    // Parse response
    parse_sensor_response(&response, &sensor.calibration)
}

fn build_modbus_read_request(address: u8, start_reg: u16, count: u16) -> Vec<u8> {
    let mut request = vec![
        address,
        0x03,  // Function code: Read Holding Registers
        (start_reg >> 8) as u8,
        (start_reg & 0xFF) as u8,
        (count >> 8) as u8,
        (count & 0xFF) as u8,
    ];
    
    // Calculate CRC16
    let crc = calculate_crc16(&request);
    request.push((crc & 0xFF) as u8);
    request.push((crc >> 8) as u8);
    
    request
}

fn calculate_crc16(data: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;
    
    for byte in data {
        crc ^= *byte as u16;
        for _ in 0..8 {
            if crc & 0x0001 != 0 {
                crc = (crc >> 1) ^ 0xA001;
            } else {
                crc >>= 1;
            }
        }
    }
    
    crc
}

fn parse_sensor_response(data: &[u8], calibration: &SensorCalibration) -> Result<SensorReading> {
    if data.len() < 37 {
        return Err(anyhow!("Invalid response length"));
    }
    
    // Skip address and function code, get to data
    let data_start = 3;
    
    // Parse values (assuming they're sent as floats in Modbus registers)
    let vibration_x = parse_float(&data[data_start..data_start+4]);
    let vibration_y = parse_float(&data[data_start+4..data_start+8]);
    let vibration_z = parse_float(&data[data_start+8..data_start+12]);
    let velocity_x = parse_float(&data[data_start+12..data_start+16]);
    let velocity_y = parse_float(&data[data_start+16..data_start+20]);
    let velocity_z = parse_float(&data[data_start+20..data_start+24]);
    let temperature = parse_float(&data[data_start+24..data_start+28]);
    let frequency = parse_float(&data[data_start+28..data_start+32]);
    
    // Apply calibration
    let calibrated_vibration = Vector3 {
        x: (vibration_x - calibration.zero_point.x) * calibration.sensitivity,
        y: (vibration_y - calibration.zero_point.y) * calibration.sensitivity,
        z: (vibration_z - calibration.zero_point.z) * calibration.sensitivity,
    };
    
    let calibrated_velocity = Vector3 {
        x: velocity_x * calibration.sensitivity,
        y: velocity_y * calibration.sensitivity,
        z: velocity_z * calibration.sensitivity,
    };
    
    // Apply noise threshold
    let filtered_vibration = Vector3 {
        x: if calibrated_vibration.x.abs() < calibration.noise_threshold { 0.0 } else { calibrated_vibration.x },
        y: if calibrated_vibration.y.abs() < calibration.noise_threshold { 0.0 } else { calibrated_vibration.y },
        z: if calibrated_vibration.z.abs() < calibration.noise_threshold { 0.0 } else { calibrated_vibration.z },
    };
    
    // Calculate magnitude
    let magnitude = (filtered_vibration.x.powi(2) + 
                    filtered_vibration.y.powi(2) + 
                    filtered_vibration.z.powi(2)).sqrt();
    
    // Determine status based on ISO 10816-3
    let status = match magnitude {
        m if m < 1.4 => VibrationStatus::Good,
        m if m < 4.5 => VibrationStatus::Warning,
        _ => VibrationStatus::Alert,
    };
    
    Ok(SensorReading {
        timestamp: chrono::Utc::now(),
        vibration: filtered_vibration,
        velocity: calibrated_velocity,
        temperature,
        frequency,
        magnitude,
        status,
    })
}

fn parse_float(bytes: &[u8]) -> f32 {
    if bytes.len() >= 4 {
        let array: [u8; 4] = [bytes[0], bytes[1], bytes[2], bytes[3]];
        f32::from_be_bytes(array)
    } else {
        0.0
    }
}

pub async fn start_monitoring(state: Arc<Mutex<AppState>>) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(1));
        
        loop {
            interval.tick().await;
            
            let app_state = state.lock().await;
            let sensors: Vec<VibrationSensor> = app_state.sensors.iter()
                .map(|entry| entry.value().clone())
                .collect();
            
            for sensor in sensors {
                // Try to read sensor
                match read_sensor(&sensor).await {
                    Ok(reading) => {
                        // Store reading
                        app_state.sensor_readings.entry(sensor.port.clone())
                            .and_modify(|readings| {
                                readings.push(reading.clone());
                                // Keep only last 100 readings
                                if readings.len() > 100 {
                                    readings.remove(0);
                                }
                            })
                            .or_insert_with(|| vec![reading]);
                        
                        // Update connection status
                        app_state.sensors.entry(sensor.port.clone())
                            .and_modify(|s| s.is_connected = true);
                    }
                    Err(e) => {
                        eprintln!("Failed to read sensor {}: {}", sensor.port, e);
                        // Update connection status
                        app_state.sensors.entry(sensor.port)
                            .and_modify(|s| s.is_connected = false);
                    }
                }
            }
        }
    });
}