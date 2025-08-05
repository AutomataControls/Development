// WIT-Motion WTVB01-485 Vibration Sensor Integration
// Simplified version for building automation integration

use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use rand::Rng;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VibrationReading {
    pub sensor_id: String,
    pub port: String,
    pub temperature_f: f32,
    pub velocity_mms: f32,  // RMS vibration velocity in mm/s
    pub iso_zone: String,   // ISO 10816-3 classification (A/B/C/D)
    pub alert_level: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VibrationSensorConfig {
    pub enabled: bool,
    pub port: String,
    pub sensor_id: String,
    pub equipment_name: String,
    pub modbus_id: u8,
    pub baud_rate: u32,
    pub alert_threshold_mms: f32,  // Alert when velocity exceeds this
}

pub struct VibrationManager {
    readings: Mutex<HashMap<String, VibrationReading>>,
    configs: Mutex<HashMap<String, VibrationSensorConfig>>,
}

impl VibrationManager {
    pub fn new() -> Self {
        VibrationManager {
            readings: Mutex::new(HashMap::new()),
            configs: Mutex::new(HashMap::new()),
        }
    }

    // Calculate CRC16 for Modbus RTU
    fn calculate_crc16(data: &[u8]) -> u16 {
        let mut crc: u16 = 0xFFFF;
        
        for byte in data {
            crc ^= *byte as u16;
            for _ in 0..8 {
                if (crc & 0x0001) != 0 {
                    crc >>= 1;
                    crc ^= 0xA001;
                } else {
                    crc >>= 1;
                }
            }
        }
        
        crc
    }

    // Build Modbus read command
    fn build_read_command(modbus_id: u8, start_reg: u16, num_regs: u16) -> Vec<u8> {
        let mut cmd = vec![
            modbus_id,
            0x03, // Read holding registers
            (start_reg >> 8) as u8,
            (start_reg & 0xFF) as u8,
            (num_regs >> 8) as u8,
            (num_regs & 0xFF) as u8,
        ];
        
        let crc = Self::calculate_crc16(&cmd);
        cmd.push((crc & 0xFF) as u8);
        cmd.push((crc >> 8) as u8);
        
        cmd
    }

    // Get ISO 10816-3 classification based on velocity
    fn get_iso_classification(velocity_mms: f32) -> (String, String) {
        let zone = match velocity_mms {
            v if v <= 2.8 => "A",
            v if v <= 7.1 => "B", 
            v if v <= 11.0 => "C",
            _ => "D",
        };
        
        let alert_level = match zone {
            "A" => "Good",
            "B" => "Acceptable",
            "C" => "Unsatisfactory",
            "D" => "Unacceptable",
            _ => "Unknown",
        }.to_string();
        
        (zone.to_string(), alert_level)
    }

    // Scan for available USB ports
    pub fn scan_ports(&self) -> Vec<String> {
        let mut ports = Vec::new();
        
        // Try using serialport library
        if let Ok(available) = serialport::available_ports() {
            for p in available {
                ports.push(p.port_name);
            }
        }
        
        // Manual check for common Linux USB serial ports
        if ports.is_empty() {
            for i in 0..4 {
                let port = format!("/dev/ttyUSB{}", i);
                if std::path::Path::new(&port).exists() {
                    ports.push(port);
                }
            }
        }
        
        ports
    }

    // Configure a vibration sensor
    pub fn configure_sensor(&self, config: VibrationSensorConfig) -> Result<(), String> {
        let mut configs = self.configs.lock().unwrap();
        configs.insert(config.sensor_id.clone(), config);
        Ok(())
    }

    // Get all sensor configurations
    pub fn get_configs(&self) -> Vec<VibrationSensorConfig> {
        self.configs.lock().unwrap().values().cloned().collect()
    }

    // Read vibration sensor (real or simulated)
    pub fn read_sensor(&self, sensor_id: &str) -> Result<VibrationReading, String> {
        let configs = self.configs.lock().unwrap();
        let config = configs.get(sensor_id)
            .ok_or_else(|| "Sensor not configured".to_string())?;

        if !config.enabled {
            return Err("Sensor is disabled".to_string());
        }

        // For now, generate simulated data
        // In production, this would use serialport to read actual Modbus data
        let reading = self.generate_simulated_reading(config);
        
        // Store the reading
        self.readings.lock().unwrap()
            .insert(sensor_id.to_string(), reading.clone());
        
        Ok(reading)
    }

    // Generate simulated vibration reading
    fn generate_simulated_reading(&self, config: &VibrationSensorConfig) -> VibrationReading {
        let mut rng = rand::thread_rng();
        
        // Simulate realistic vibration values
        let base_velocity = 2.0 + rng.gen::<f32>() * 3.0;
        let variation = rng.gen::<f32>() * 2.0 - 1.0; // ±1 mm/s variation
        let velocity_mms = (base_velocity + variation).max(0.1);
        
        // Temperature simulation (60-90°F typical motor temperature)
        let temperature_f = 60.0 + rng.gen::<f32>() * 30.0;
        
        let (iso_zone, alert_level) = Self::get_iso_classification(velocity_mms);
        
        VibrationReading {
            sensor_id: config.sensor_id.clone(),
            port: config.port.clone(),
            temperature_f,
            velocity_mms,
            iso_zone,
            alert_level,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    // Get all current readings
    pub fn get_all_readings(&self) -> Vec<VibrationReading> {
        self.readings.lock().unwrap().values().cloned().collect()
    }

    // Check if any sensor is in alert state
    pub fn check_alerts(&self) -> Vec<(String, String, f32)> {
        let mut alerts = Vec::new();
        let configs = self.configs.lock().unwrap();
        let readings = self.readings.lock().unwrap();
        
        for (sensor_id, reading) in readings.iter() {
            if let Some(config) = configs.get(sensor_id) {
                if reading.velocity_mms > config.alert_threshold_mms {
                    alerts.push((
                        config.equipment_name.clone(),
                        reading.alert_level.clone(),
                        reading.velocity_mms,
                    ));
                }
            }
        }
        
        alerts
    }

    // Read sensor with actual Modbus RTU (for future implementation)
    #[allow(dead_code)]
    pub fn read_sensor_modbus(&self, config: &VibrationSensorConfig) -> Result<VibrationReading, String> {
        use std::io::{Read, Write};
        
        // Open serial port
        let mut port = serialport::new(&config.port, config.baud_rate)
            .timeout(Duration::from_millis(100))
            .data_bits(serialport::DataBits::Eight)
            .stop_bits(serialport::StopBits::One)
            .parity(serialport::Parity::None)
            .flow_control(serialport::FlowControl::None)
            .open()
            .map_err(|e| format!("Failed to open port: {}", e))?;

        // WTVB01 register addresses
        const REG_TEMP: u16 = 0x40;        // Temperature
        const REG_VIB_VEL_X: u16 = 0x3A;  // X-axis velocity
        const REG_VIB_VEL_Y: u16 = 0x3B;  // Y-axis velocity
        const REG_VIB_VEL_Z: u16 = 0x3C;  // Z-axis velocity

        // Read temperature (1 register)
        let temp_cmd = Self::build_read_command(config.modbus_id, REG_TEMP, 1);
        port.write_all(&temp_cmd)
            .map_err(|e| format!("Failed to send command: {}", e))?;

        let mut temp_response = vec![0u8; 7];
        port.read_exact(&mut temp_response)
            .map_err(|e| format!("Failed to read temperature: {}", e))?;

        let temp_raw = ((temp_response[3] as i16) << 8) | (temp_response[4] as i16);
        let temperature_c = temp_raw as f32 / 100.0;
        let temperature_f = temperature_c * 9.0 / 5.0 + 32.0;

        // Read vibration velocity (3 registers)
        let vel_cmd = Self::build_read_command(config.modbus_id, REG_VIB_VEL_X, 3);
        port.write_all(&vel_cmd)
            .map_err(|e| format!("Failed to send velocity command: {}", e))?;

        let mut vel_response = vec![0u8; 11];
        port.read_exact(&mut vel_response)
            .map_err(|e| format!("Failed to read velocity: {}", e))?;

        // Parse velocity values
        let vel_x = (((vel_response[3] as i16) << 8) | (vel_response[4] as i16)) as f32 / 1000.0;
        let vel_y = (((vel_response[5] as i16) << 8) | (vel_response[6] as i16)) as f32 / 1000.0;
        let vel_z = (((vel_response[7] as i16) << 8) | (vel_response[8] as i16)) as f32 / 1000.0;

        // Calculate RMS velocity
        let velocity_mms = (vel_x * vel_x + vel_y * vel_y + vel_z * vel_z).sqrt();
        
        let (iso_zone, alert_level) = Self::get_iso_classification(velocity_mms);

        Ok(VibrationReading {
            sensor_id: config.sensor_id.clone(),
            port: config.port.clone(),
            temperature_f,
            velocity_mms,
            iso_zone,
            alert_level,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        })
    }
}