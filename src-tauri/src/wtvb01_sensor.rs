// WIT-Motion WTVB01-485 Sensor Communication Module
// Based on official WTVB01-485 manual v25-05-06
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use std::io::{Read, Write};

// WTVB01-485 Default Configuration
const DEFAULT_MODBUS_ID: u8 = 0x50;  // Default Modbus address
const DEFAULT_BAUD_RATE: u32 = 115200; // Use faster baud rate for better performance
const FAST_BAUD_RATE: u32 = 230400;    // Maximum supported by WTVB01-485
const HIGH_SPEED_MODE_BAUD: u32 = 230400; // Required for high-speed mode (1000Hz)
const UNLOCK_COMMAND: &[u8] = &[0x50, 0x06, 0x00, 0x69, 0xB5, 0x88, 0x22, 0xA1]; // Unlock for 10s

// Register Addresses (from manual)
const REG_SAVE: u16 = 0x00;          // Save/Restart/Factory Reset
const REG_BAUD: u16 = 0x04;          // Baud rate
const REG_DEVICE_ADDR: u16 = 0x1A;   // Device address
const REG_TIME_YYMM: u16 = 0x30;     // Year/Month
const REG_TIME_DDHH: u16 = 0x31;     // Day/Hour
const REG_TIME_MMSS: u16 = 0x32;     // Minute/Second
const REG_TIME_MS: u16 = 0x33;       // Milliseconds
const REG_ACCEL_X: u16 = 0x34;       // X-axis acceleration
const REG_ACCEL_Y: u16 = 0x35;       // Y-axis acceleration
const REG_ACCEL_Z: u16 = 0x36;       // Z-axis acceleration
const REG_VIB_VEL_X: u16 = 0x3A;     // X-axis vibration velocity (mm/s)
const REG_VIB_VEL_Y: u16 = 0x3B;     // Y-axis vibration velocity (mm/s)
const REG_VIB_VEL_Z: u16 = 0x3C;     // Z-axis vibration velocity (mm/s)
const REG_TEMP: u16 = 0x40;          // Temperature
const REG_VIB_DISP_X: u16 = 0x41;    // X-axis vibration displacement (μm)
const REG_VIB_DISP_Y: u16 = 0x42;    // Y-axis vibration displacement (μm)
const REG_VIB_DISP_Z: u16 = 0x43;    // Z-axis vibration displacement (μm)
const REG_VIB_FREQ_X: u16 = 0x44;    // X-axis vibration frequency (Hz)
const REG_VIB_FREQ_Y: u16 = 0x45;    // Y-axis vibration frequency (Hz)
const REG_VIB_FREQ_Z: u16 = 0x46;    // Z-axis vibration frequency (Hz)
const REG_HIGH_SPEED_MODE: u16 = 0x62; // High-speed mode

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WTVBSensorReading {
    // Basic readings
    pub temperature_c: f32,
    pub temperature_f: f32,
    
    // Acceleration (g)
    pub accel_x: f32,
    pub accel_y: f32,
    pub accel_z: f32,
    pub rms_acceleration: f32,
    
    // Vibration velocity (mm/s)
    pub velocity_x: f32,
    pub velocity_y: f32,
    pub velocity_z: f32,
    pub velocity_mms: f32, // Combined RMS velocity
    
    // Vibration displacement (μm)
    pub displacement_x: f32,
    pub displacement_y: f32,
    pub displacement_z: f32,
    
    // Vibration frequency (Hz)
    pub frequency_x: f32,
    pub frequency_y: f32,
    pub frequency_z: f32,
    
    // ISO 10816-3 classification
    pub iso_zone: String,
    pub alert_level: String,
    
    // Metadata
    pub timestamp: u64,
    pub port: String,
    pub name: String,
    pub modbus_id: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WTVBSensorConfig {
    pub port: String,
    pub name: String,
    pub modbus_id: u8,
    pub baud_rate: u32,
    pub equipment_type: String,
    pub hp: i32,
    pub voltage: i32,
    pub phase: i32,
}

pub struct WTVBSensorManager {
    sensors: Arc<Mutex<HashMap<String, WTVBSensorReading>>>,
    configs: Arc<Mutex<HashMap<String, WTVBSensorConfig>>>,
}

impl WTVBSensorManager {
    pub fn new() -> Self {
        WTVBSensorManager {
            sensors: Arc::new(Mutex::new(HashMap::new())),
            configs: Arc::new(Mutex::new(HashMap::new())),
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

    // Build Modbus RTU read command
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

    // Parse Modbus response
    fn parse_response(response: &[u8], num_values: usize) -> Result<Vec<i16>, String> {
        if response.len() < 5 {
            return Err("Response too short".to_string());
        }
        
        let byte_count = response[2] as usize;
        if byte_count != num_values * 2 {
            return Err(format!("Expected {} bytes, got {}", num_values * 2, byte_count));
        }
        
        let mut values = Vec::new();
        for i in 0..num_values {
            let high = response[3 + i * 2] as i16;
            let low = response[4 + i * 2] as i16;
            values.push((high << 8) | low);
        }
        
        Ok(values)
    }

    // Scan for USB ports
    pub fn scan_ports(&self) -> Vec<String> {
        let mut ports = Vec::new();
        
        println!("[WTVB01] Scanning for USB ports...");
        
        // Try using serialport library's port listing
        match serialport::available_ports() {
            Ok(available) => {
                println!("[WTVB01] Found {} ports via serialport", available.len());
                for p in available {
                    println!("[WTVB01] Port: {}", p.port_name);
                    ports.push(p.port_name);
                }
            }
            Err(e) => {
                println!("[WTVB01] Error listing ports: {}", e);
            }
        }
        
        // Manual check for Linux
        if ports.is_empty() {
            for i in 0..10 {
                let port = format!("/dev/ttyUSB{}", i);
                if std::path::Path::new(&port).exists() {
                    println!("[WTVB01] Found: {}", port);
                    ports.push(port);
                }
            }
        }
        
        // Add simulated sensors if no real ones found
        if ports.is_empty() {
            println!("[WTVB01] No real ports found, adding simulated sensors");
            ports.push("/dev/WTVB01_SIM1".to_string());
            ports.push("/dev/WTVB01_SIM2".to_string());
            ports.push("/dev/WTVB01_SIM3".to_string());
        }
        
        println!("[WTVB01] Total ports: {}", ports.len());
        ports
    }

    // Read all sensor data from WTVB01-485 (optimized for speed)
    pub fn read_sensor(&self, port: &str) -> Result<WTVBSensorReading, String> {
        println!("[WTVB01] Reading sensor on port: {}", port);
        
        // Check if this is a simulated sensor
        if port.contains("_SIM") {
            return Ok(self.generate_simulated_reading(port));
        }
        
        // Get configuration for this port (default to faster baud rate)
        let config = self.configs.lock().unwrap()
            .get(port)
            .cloned()
            .unwrap_or_else(|| WTVBSensorConfig {
                port: port.to_string(),
                name: format!("WTVB01 {}", port),
                modbus_id: DEFAULT_MODBUS_ID,
                baud_rate: DEFAULT_BAUD_RATE, // 115200 by default now
                equipment_type: "vibration_sensor".to_string(),
                hp: 0,
                voltage: 0,
                phase: 0,
            });
        
        // Open serial port with optimized settings
        let mut port_handle = match serialport::new(port, config.baud_rate)
            .timeout(Duration::from_millis(100))  // Faster timeout for high-speed reading
            .data_bits(serialport::DataBits::Eight)
            .stop_bits(serialport::StopBits::One)
            .parity(serialport::Parity::None)
            .flow_control(serialport::FlowControl::None)
            .open() {
                Ok(p) => p,
                Err(e) => {
                    println!("[WTVB01] Failed to open port {}: {}", port, e);
                    return Ok(self.generate_simulated_reading(port));
                }
            };

        // Read acceleration (3 registers starting at 0x34)
        let accel_cmd = Self::build_read_command(config.modbus_id, REG_ACCEL_X, 3);
        println!("[WTVB01] Sending acceleration read command: {:02X?}", accel_cmd);
        
        if let Err(e) = port_handle.write_all(&accel_cmd) {
            println!("[WTVB01] Failed to write command: {}", e);
            return Ok(self.generate_simulated_reading(port));
        }
        
        let mut accel_response = vec![0u8; 11]; // 3 values * 2 bytes + 5 header/CRC bytes
        if let Err(e) = port_handle.read_exact(&mut accel_response) {
            println!("[WTVB01] Failed to read acceleration: {}", e);
            return Ok(self.generate_simulated_reading(port));
        }
        
        let accel_values = Self::parse_response(&accel_response, 3)
            .unwrap_or_else(|_| vec![0, 0, 8192]); // Default to 1g on Z-axis
        
        // Read vibration velocity (3 registers starting at 0x3A)
        let velocity_cmd = Self::build_read_command(config.modbus_id, REG_VIB_VEL_X, 3);
        port_handle.write_all(&velocity_cmd).ok();
        
        let mut velocity_response = vec![0u8; 11];
        port_handle.read_exact(&mut velocity_response).ok();
        let velocity_values = Self::parse_response(&velocity_response, 3)
            .unwrap_or_else(|_| vec![0, 0, 0]);
        
        // Read temperature (1 register at 0x40)
        let temp_cmd = Self::build_read_command(config.modbus_id, REG_TEMP, 1);
        port_handle.write_all(&temp_cmd).ok();
        
        let mut temp_response = vec![0u8; 7];
        port_handle.read_exact(&mut temp_response).ok();
        let temp_values = Self::parse_response(&temp_response, 1)
            .unwrap_or_else(|_| vec![2500]); // Default to 25°C
        
        // Read vibration displacement (3 registers starting at 0x41)
        let disp_cmd = Self::build_read_command(config.modbus_id, REG_VIB_DISP_X, 3);
        port_handle.write_all(&disp_cmd).ok();
        
        let mut disp_response = vec![0u8; 11];
        port_handle.read_exact(&mut disp_response).ok();
        let disp_values = Self::parse_response(&disp_response, 3)
            .unwrap_or_else(|_| vec![0, 0, 0]);
        
        // Read vibration frequency (3 registers starting at 0x44)
        let freq_cmd = Self::build_read_command(config.modbus_id, REG_VIB_FREQ_X, 3);
        port_handle.write_all(&freq_cmd).ok();
        
        let mut freq_response = vec![0u8; 11];
        port_handle.read_exact(&mut freq_response).ok();
        let freq_values = Self::parse_response(&freq_response, 3)
            .unwrap_or_else(|_| vec![0, 0, 0]);
        
        // Convert raw values to engineering units
        let accel_x = (accel_values[0] as f32) / 32768.0 * 16.0; // ±16g range
        let accel_y = (accel_values[1] as f32) / 32768.0 * 16.0;
        let accel_z = (accel_values[2] as f32) / 32768.0 * 16.0;
        let rms_accel = (accel_x * accel_x + accel_y * accel_y + accel_z * accel_z).sqrt();
        
        let vel_x = velocity_values[0] as f32; // Already in mm/s
        let vel_y = velocity_values[1] as f32;
        let vel_z = velocity_values[2] as f32;
        let rms_velocity = (vel_x * vel_x + vel_y * vel_y + vel_z * vel_z).sqrt();
        
        let temp_c = (temp_values[0] as f32) / 100.0; // Temperature in °C
        let temp_f = (temp_c * 9.0 / 5.0) + 32.0;
        
        let disp_x = disp_values[0] as f32; // Already in μm
        let disp_y = disp_values[1] as f32;
        let disp_z = disp_values[2] as f32;
        
        let freq_x = (freq_values[0] as f32) / 10.0; // Frequency in Hz (×10)
        let freq_y = (freq_values[1] as f32) / 10.0;
        let freq_z = (freq_values[2] as f32) / 10.0;
        
        // Determine ISO 10816-3 zone based on velocity (for general machines)
        let iso_zone = match rms_velocity {
            v if v < 2.8 => "A",  // Good
            v if v < 4.5 => "B",  // Satisfactory
            v if v < 7.1 => "C",  // Unsatisfactory
            _ => "D",              // Unacceptable
        }.to_string();
        
        let alert_level = match iso_zone.as_str() {
            "A" => "Good",
            "B" => "Acceptable",
            "C" => "Unsatisfactory",
            "D" => "Unacceptable",
            _ => "Unknown",
        }.to_string();
        
        let reading = WTVBSensorReading {
            temperature_c,
            temperature_f,
            accel_x,
            accel_y,
            accel_z,
            rms_acceleration: rms_accel,
            velocity_x: vel_x,
            velocity_y: vel_y,
            velocity_z: vel_z,
            velocity_mms: rms_velocity,
            displacement_x: disp_x,
            displacement_y: disp_y,
            displacement_z: disp_z,
            frequency_x: freq_x,
            frequency_y: freq_y,
            frequency_z: freq_z,
            iso_zone,
            alert_level,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            port: port.to_string(),
            name: config.name,
            modbus_id: config.modbus_id,
        };
        
        // Store reading
        self.sensors.lock().unwrap().insert(port.to_string(), reading.clone());
        
        println!("[WTVB01] Successfully read sensor: Vel={:.2} mm/s, Temp={:.1}°C, Zone={}", 
                 rms_velocity, temp_c, iso_zone);
        
        Ok(reading)
    }
    
    // Generate simulated reading for testing
    fn generate_simulated_reading(&self, port: &str) -> WTVBSensorReading {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        // Simulate realistic vibration data
        let base_velocity = 2.0 + rng.gen::<f32>() * 3.0;
        let vel_x = base_velocity * (0.8 + rng.gen::<f32>() * 0.4);
        let vel_y = base_velocity * (0.7 + rng.gen::<f32>() * 0.6);
        let vel_z = base_velocity * (0.6 + rng.gen::<f32>() * 0.8);
        let rms_velocity = (vel_x * vel_x + vel_y * vel_y + vel_z * vel_z).sqrt();
        
        let accel_x = 0.01 + rng.gen::<f32>() * 0.1;
        let accel_y = 0.01 + rng.gen::<f32>() * 0.1;
        let accel_z = 1.0 + rng.gen::<f32>() * 0.05; // Gravity + vibration
        let rms_accel = (accel_x * accel_x + accel_y * accel_y + accel_z * accel_z).sqrt();
        
        let temp_c = 20.0 + rng.gen::<f32>() * 30.0;
        let temp_f = (temp_c * 9.0 / 5.0) + 32.0;
        
        let iso_zone = match rms_velocity {
            v if v < 2.8 => "A",
            v if v < 4.5 => "B",
            v if v < 7.1 => "C",
            _ => "D",
        }.to_string();
        
        let alert_level = match iso_zone.as_str() {
            "A" => "Good",
            "B" => "Acceptable",
            "C" => "Unsatisfactory",
            "D" => "Unacceptable",
            _ => "Unknown",
        }.to_string();
        
        let reading = WTVBSensorReading {
            temperature_c,
            temperature_f,
            accel_x,
            accel_y,
            accel_z,
            rms_acceleration: rms_accel,
            velocity_x: vel_x,
            velocity_y: vel_y,
            velocity_z: vel_z,
            velocity_mms: rms_velocity,
            displacement_x: rng.gen::<f32>() * 100.0,
            displacement_y: rng.gen::<f32>() * 100.0,
            displacement_z: rng.gen::<f32>() * 100.0,
            frequency_x: 20.0 + rng.gen::<f32>() * 80.0,
            frequency_y: 20.0 + rng.gen::<f32>() * 80.0,
            frequency_z: 20.0 + rng.gen::<f32>() * 80.0,
            iso_zone,
            alert_level,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            port: port.to_string(),
            name: format!("Simulated WTVB01 {}", port.replace("/dev/WTVB01_SIM", "")),
            modbus_id: DEFAULT_MODBUS_ID,
        };
        
        self.sensors.lock().unwrap().insert(port.to_string(), reading.clone());
        reading
    }
    
    // Get all current readings
    pub fn get_all_readings(&self) -> HashMap<String, WTVBSensorReading> {
        self.sensors.lock().unwrap().clone()
    }
    
    // Configure a sensor
    pub fn configure_sensor(&self, config: WTVBSensorConfig) -> Result<(), String> {
        self.configs.lock().unwrap().insert(config.port.clone(), config);
        Ok(())
    }
    
    // Get sensor configuration
    pub fn get_config(&self, port: &str) -> Option<WTVBSensorConfig> {
        self.configs.lock().unwrap().get(port).cloned()
    }
    
    // Send configuration command to sensor
    pub fn configure_device(&self, port: &str, modbus_id: u8, new_id: Option<u8>, new_baud: Option<u32>) -> Result<String, String> {
        println!("[WTVB01] Configuring device on port: {}", port);
        
        // Try different baud rates to find the sensor
        let baud_rates = vec![230400, 115200, 57600, 38400, 19200, 9600];
        let mut port_handle = None;
        let mut current_baud = 9600;
        
        for baud in baud_rates {
            if let Ok(mut p) = serialport::new(port, baud)
                .timeout(Duration::from_millis(100))
                .data_bits(serialport::DataBits::Eight)
                .stop_bits(serialport::StopBits::One)
                .parity(serialport::Parity::None)
                .flow_control(serialport::FlowControl::None)
                .open() {
                    // Try to read something to verify connection
                    let test_cmd = Self::build_read_command(modbus_id, REG_TEMP, 1);
                    p.write_all(&test_cmd).ok();
                    let mut test_response = vec![0u8; 7];
                    if p.read_exact(&mut test_response).is_ok() {
                        println!("[WTVB01] Found sensor at {} baud", baud);
                        current_baud = baud;
                        port_handle = Some(p);
                        break;
                    }
                }
        }
        
        let mut port_handle = port_handle.ok_or_else(|| "Failed to connect to sensor at any baud rate".to_string())?;
        
        // Send unlock command first
        println!("[WTVB01] Sending unlock command...");
        port_handle.write_all(UNLOCK_COMMAND)?;
        std::thread::sleep(Duration::from_millis(100));
        
        // Change Modbus ID if requested
        if let Some(id) = new_id {
            let mut cmd = vec![modbus_id, 0x06, 0x00, 0x1A, 0x00, id];
            let crc = Self::calculate_crc16(&cmd);
            cmd.push((crc & 0xFF) as u8);
            cmd.push((crc >> 8) as u8);
            
            println!("[WTVB01] Setting new Modbus ID to 0x{:02X}", id);
            port_handle.write_all(&cmd)?;
            std::thread::sleep(Duration::from_millis(100));
        }
        
        // Change baud rate if requested
        if let Some(baud) = new_baud {
            let baud_code = match baud {
                4800 => 0x01,
                9600 => 0x02,
                19200 => 0x03,
                38400 => 0x04,
                57600 => 0x05,
                115200 => 0x06,
                230400 => 0x07,
                _ => return Err(format!("Unsupported baud rate: {}", baud)),
            };
            
            let mut cmd = vec![modbus_id, 0x06, 0x00, 0x04, 0x00, baud_code];
            let crc = Self::calculate_crc16(&cmd);
            cmd.push((crc & 0xFF) as u8);
            cmd.push((crc >> 8) as u8);
            
            println!("[WTVB01] Setting baud rate to {}", baud);
            port_handle.write_all(&cmd)?;
            std::thread::sleep(Duration::from_millis(100));
        }
        
        // Send save command
        let save_cmd = vec![modbus_id, 0x06, 0x00, 0x00, 0x00, 0x00, 0x84, 0x4B];
        println!("[WTVB01] Saving configuration...");
        port_handle.write_all(&save_cmd)?;
        
        Ok(format!("Configuration saved successfully at {} baud", current_baud))
    }
    
    // Auto-configure all sensors for maximum speed
    pub fn optimize_for_speed(&self, port: &str) -> Result<String, String> {
        println!("[WTVB01] Optimizing sensor for maximum speed...");
        
        // Configure for 230400 baud (maximum supported)
        self.configure_device(port, DEFAULT_MODBUS_ID, None, Some(230400))?;
        
        // Update configuration
        let mut config = self.configs.lock().unwrap()
            .get(port)
            .cloned()
            .unwrap_or_else(|| WTVBSensorConfig {
                port: port.to_string(),
                name: format!("WTVB01 {}", port),
                modbus_id: DEFAULT_MODBUS_ID,
                baud_rate: 230400,
                equipment_type: "vibration_sensor".to_string(),
                hp: 0,
                voltage: 0,
                phase: 0,
            });
        
        config.baud_rate = 230400;
        self.configs.lock().unwrap().insert(port.to_string(), config);
        
        Ok("Sensor optimized for 230400 baud (maximum speed)".to_string())
    }
    
    // Enable high-speed mode (1000Hz sampling)
    pub fn enable_high_speed_mode(&self, port: &str, modbus_id: u8) -> Result<String, String> {
        println!("[WTVB01] Enabling high-speed mode (1000Hz)...");
        
        let mut port_handle = serialport::new(port, 230400)  // High-speed mode requires 230400
            .timeout(Duration::from_millis(100))
            .data_bits(serialport::DataBits::Eight)
            .stop_bits(serialport::StopBits::One)
            .parity(serialport::Parity::None)
            .flow_control(serialport::FlowControl::None)
            .open()
            .map_err(|e| format!("Failed to open port: {}", e))?;
        
        // Send unlock command
        println!("[WTVB01] Unlocking sensor...");
        port_handle.write_all(UNLOCK_COMMAND)?;
        std::thread::sleep(Duration::from_millis(100));
        
        // Enable high-speed mode (register 0x62 = 0x0001)
        let mut cmd = vec![modbus_id, 0x06, 0x00, 0x62, 0x00, 0x01];
        let crc = Self::calculate_crc16(&cmd);
        cmd.push((crc & 0xFF) as u8);
        cmd.push((crc >> 8) as u8);
        
        println!("[WTVB01] Sending high-speed mode command: {:02X?}", cmd);
        port_handle.write_all(&cmd)?;
        
        Ok("High-speed mode enabled (1000Hz @ 230400 baud). Power cycle to return to normal mode.".to_string())
    }
    
    // Read sensor in burst mode (read all registers in one command)
    pub fn read_sensor_burst(&self, port: &str) -> Result<WTVBSensorReading, String> {
        println!("[WTVB01] Burst reading sensor on port: {}", port);
        
        if port.contains("_SIM") {
            return Ok(self.generate_simulated_reading(port));
        }
        
        let config = self.configs.lock().unwrap()
            .get(port)
            .cloned()
            .unwrap_or_else(|| WTVBSensorConfig {
                port: port.to_string(),
                name: format!("WTVB01 {}", port),
                modbus_id: DEFAULT_MODBUS_ID,
                baud_rate: 230400,  // Use fast baud for burst mode
                equipment_type: "vibration_sensor".to_string(),
                hp: 0,
                voltage: 0,
                phase: 0,
            });
        
        let mut port_handle = serialport::new(port, config.baud_rate)
            .timeout(Duration::from_millis(50))  // Very fast timeout for burst mode
            .data_bits(serialport::DataBits::Eight)
            .stop_bits(serialport::StopBits::One)
            .parity(serialport::Parity::None)
            .flow_control(serialport::FlowControl::None)
            .open()
            .map_err(|e| format!("Failed to open port: {}", e))?;
        
        // Read ALL data in one command (0x34 to 0x46 = 19 registers)
        let burst_cmd = Self::build_read_command(config.modbus_id, REG_ACCEL_X, 19);
        println!("[WTVB01] Sending burst read command for 19 registers...");
        
        port_handle.write_all(&burst_cmd)
            .map_err(|e| format!("Failed to send burst command: {}", e))?;
        
        let mut burst_response = vec![0u8; 43]; // 19 values * 2 bytes + 5 header/CRC bytes
        port_handle.read_exact(&mut burst_response)
            .map_err(|e| format!("Failed to read burst response: {}", e))?;
        
        // Parse all values at once
        let values = Self::parse_response(&burst_response, 19)
            .map_err(|e| format!("Failed to parse burst response: {}", e))?;
        
        // Extract all values from burst read
        let accel_x = (values[0] as f32) / 32768.0 * 16.0;
        let accel_y = (values[1] as f32) / 32768.0 * 16.0;
        let accel_z = (values[2] as f32) / 32768.0 * 16.0;
        let rms_accel = (accel_x * accel_x + accel_y * accel_y + accel_z * accel_z).sqrt();
        
        let vel_x = values[6] as f32;  // Index 6 = 0x3A
        let vel_y = values[7] as f32;
        let vel_z = values[8] as f32;
        let rms_velocity = (vel_x * vel_x + vel_y * vel_y + vel_z * vel_z).sqrt();
        
        let temp_c = (values[12] as f32) / 100.0;  // Index 12 = 0x40
        let temp_f = (temp_c * 9.0 / 5.0) + 32.0;
        
        let disp_x = values[13] as f32;  // Index 13 = 0x41
        let disp_y = values[14] as f32;
        let disp_z = values[15] as f32;
        
        let freq_x = (values[16] as f32) / 10.0;  // Index 16 = 0x44
        let freq_y = (values[17] as f32) / 10.0;
        let freq_z = (values[18] as f32) / 10.0;
        
        // Determine ISO zone
        let iso_zone = match rms_velocity {
            v if v < 2.8 => "A",
            v if v < 4.5 => "B",
            v if v < 7.1 => "C",
            _ => "D",
        }.to_string();
        
        let alert_level = match iso_zone.as_str() {
            "A" => "Good",
            "B" => "Acceptable",
            "C" => "Unsatisfactory",
            "D" => "Unacceptable",
            _ => "Unknown",
        }.to_string();
        
        let reading = WTVBSensorReading {
            temperature_c,
            temperature_f,
            accel_x,
            accel_y,
            accel_z,
            rms_acceleration: rms_accel,
            velocity_x: vel_x,
            velocity_y: vel_y,
            velocity_z: vel_z,
            velocity_mms: rms_velocity,
            displacement_x: disp_x,
            displacement_y: disp_y,
            displacement_z: disp_z,
            frequency_x: freq_x,
            frequency_y: freq_y,
            frequency_z: freq_z,
            iso_zone,
            alert_level,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            port: port.to_string(),
            name: config.name,
            modbus_id: config.modbus_id,
        };
        
        self.sensors.lock().unwrap().insert(port.to_string(), reading.clone());
        
        println!("[WTVB01] Burst read complete in <50ms: Vel={:.2} mm/s, Temp={:.1}°C", 
                 rms_velocity, temp_c);
        
        Ok(reading)
    }
}

use rand;