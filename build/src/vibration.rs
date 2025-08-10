// WIT-Motion WTVB01-485 Vibration Sensor Implementation
// Full Modbus RTU protocol support with ISO 10816-3 compliance

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use tokio_modbus::prelude::*;
use tokio_serial::SerialStream;
use std::time::Duration;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

// WTVB01-485 Modbus register addresses
const REG_X_AXIS_G: u16 = 0x3400;      // X-axis acceleration (g)
const REG_Y_AXIS_G: u16 = 0x3401;      // Y-axis acceleration (g)
const REG_Z_AXIS_G: u16 = 0x3402;      // Z-axis acceleration (g)
const REG_X_VELOCITY: u16 = 0x3403;    // X-axis velocity (mm/s)
const REG_Y_VELOCITY: u16 = 0x3404;    // Y-axis velocity (mm/s)
const REG_Z_VELOCITY: u16 = 0x3405;    // Z-axis velocity (mm/s)
const REG_TEMPERATURE: u16 = 0x3406;   // Temperature (°C)
const REG_FREQ_X: u16 = 0x3407;        // X-axis dominant frequency
const REG_FREQ_Y: u16 = 0x3408;        // Y-axis dominant frequency
const REG_FREQ_Z: u16 = 0x3409;        // Z-axis dominant frequency

// ISO 10816-3 Severity Zones for rotating machinery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VibrationSeverity {
    Good,           // Zone A: < 2.3 mm/s
    Satisfactory,   // Zone B: 2.3-4.5 mm/s
    Unsatisfactory, // Zone C: 4.5-7.1 mm/s
    Unacceptable,   // Zone D: > 7.1 mm/s
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VibrationData {
    pub timestamp: DateTime<Utc>,
    pub sensor_id: u8,
    pub port: String,
    pub acceleration: AxisData<f32>,  // g
    pub velocity: AxisData<f32>,      // mm/s
    pub frequency: AxisData<f32>,     // Hz
    pub temperature: f32,              // °C
    pub rms_velocity: f32,            // mm/s RMS
    pub severity: VibrationSeverity,
    pub peak_velocity: f32,
    pub crest_factor: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxisData<T> {
    pub x: T,
    pub y: T,
    pub z: T,
    pub magnitude: T,
}

impl<T> AxisData<T> 
where T: Copy + std::ops::Mul<Output = T> + std::ops::Add<Output = T> + Into<f64>
{
    pub fn new(x: T, y: T, z: T) -> Self {
        let x_f: f64 = x.into();
        let y_f: f64 = y.into();
        let z_f: f64 = z.into();
        let mag = (x_f * x_f + y_f * y_f + z_f * z_f).sqrt();
        
        Self {
            x,
            y,
            z,
            magnitude: unsafe { std::mem::transmute_copy(&(mag as f32)) },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorConfig {
    pub sensor_id: u8,
    pub port: String,
    pub baud_rate: u32,
    pub name: String,
    pub location: String,
    pub equipment: String,
    pub calibration: CalibrationData,
    pub alarm_thresholds: AlarmThresholds,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationData {
    pub zero_offset: AxisData<f32>,
    pub sensitivity_scale: AxisData<f32>,
    pub temperature_offset: f32,
}

impl Default for CalibrationData {
    fn default() -> Self {
        Self {
            zero_offset: AxisData { x: 0.0, y: 0.0, z: 0.0, magnitude: 0.0 },
            sensitivity_scale: AxisData { x: 1.0, y: 1.0, z: 1.0, magnitude: 1.0 },
            temperature_offset: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlarmThresholds {
    pub velocity_warning: f32,  // mm/s
    pub velocity_alarm: f32,    // mm/s
    pub acceleration_warning: f32, // g
    pub acceleration_alarm: f32,   // g
    pub temperature_warning: f32,  // °C
    pub temperature_alarm: f32,    // °C
}

impl Default for AlarmThresholds {
    fn default() -> Self {
        Self {
            velocity_warning: 4.5,      // ISO 10816-3 Zone B/C boundary
            velocity_alarm: 7.1,        // ISO 10816-3 Zone C/D boundary
            acceleration_warning: 2.0,   // 2g typical warning
            acceleration_alarm: 5.0,     // 5g typical alarm
            temperature_warning: 70.0,   // 70°C warning
            temperature_alarm: 85.0,     // 85°C alarm
        }
    }
}

pub struct VibrationSensorManager {
    sensors: HashMap<u8, SensorConfig>,
    connections: HashMap<u8, Box<dyn AsyncRead + AsyncWrite + Send>>,
    data_history: HashMap<u8, Vec<VibrationData>>,
    max_history: usize,
}

impl VibrationSensorManager {
    pub fn new() -> Self {
        Self {
            sensors: HashMap::new(),
            connections: HashMap::new(),
            data_history: HashMap::new(),
            max_history: 1000,
        }
    }
    
    pub async fn scan_ports(&self) -> Result<Vec<String>> {
        let ports = tokio_serial::available_ports()?;
        let usb_ports: Vec<String> = ports
            .iter()
            .filter(|p| p.port_name.contains("USB") || p.port_name.contains("ACM"))
            .map(|p| p.port_name.clone())
            .collect();
        
        Ok(usb_ports)
    }
    
    pub async fn add_sensor(&mut self, config: SensorConfig) -> Result<()> {
        let port = config.port.clone();
        let sensor_id = config.sensor_id;
        
        // Configure serial port
        let builder = tokio_serial::new(&port, config.baud_rate)
            .timeout(Duration::from_millis(100))
            .data_bits(tokio_serial::DataBits::Eight)
            .parity(tokio_serial::Parity::None)
            .stop_bits(tokio_serial::StopBits::One);
        
        let port = SerialStream::open(&builder)?;
        
        // Create Modbus context
        let slave = Slave(sensor_id);
        let mut ctx = rtu::connect_slave(port, slave).await?;
        
        // Test connection by reading temperature
        let data = ctx.read_holding_registers(REG_TEMPERATURE, 1).await?;
        if !data.is_empty() {
            println!("✅ Sensor {} connected on {}", sensor_id, config.port);
            self.sensors.insert(sensor_id, config);
            self.connections.insert(sensor_id, Box::new(ctx));
            self.data_history.insert(sensor_id, Vec::new());
            Ok(())
        } else {
            Err(anyhow!("Failed to communicate with sensor"))
        }
    }
    
    pub async fn read_sensor(&mut self, sensor_id: u8) -> Result<VibrationData> {
        let config = self.sensors.get(&sensor_id)
            .ok_or_else(|| anyhow!("Sensor {} not configured", sensor_id))?
            .clone();
        
        let ctx = self.connections.get_mut(&sensor_id)
            .ok_or_else(|| anyhow!("No connection for sensor {}", sensor_id))?;
        
        // Read all registers in one batch (10 registers)
        let data = ctx.read_holding_registers(REG_X_AXIS_G, 10).await?;
        
        // Parse register data
        let accel_x = Self::parse_register(&data[0]) / 1000.0; // Convert to g
        let accel_y = Self::parse_register(&data[1]) / 1000.0;
        let accel_z = Self::parse_register(&data[2]) / 1000.0;
        
        let vel_x = Self::parse_register(&data[3]) / 100.0; // Convert to mm/s
        let vel_y = Self::parse_register(&data[4]) / 100.0;
        let vel_z = Self::parse_register(&data[5]) / 100.0;
        
        let temperature = Self::parse_register(&data[6]) / 100.0; // Convert to °C
        
        let freq_x = Self::parse_register(&data[7]) / 10.0; // Convert to Hz
        let freq_y = Self::parse_register(&data[8]) / 10.0;
        let freq_z = Self::parse_register(&data[9]) / 10.0;
        
        // Apply calibration
        let acceleration = AxisData::new(
            (accel_x - config.calibration.zero_offset.x) * config.calibration.sensitivity_scale.x,
            (accel_y - config.calibration.zero_offset.y) * config.calibration.sensitivity_scale.y,
            (accel_z - config.calibration.zero_offset.z) * config.calibration.sensitivity_scale.z,
        );
        
        let velocity = AxisData::new(vel_x, vel_y, vel_z);
        let frequency = AxisData::new(freq_x, freq_y, freq_z);
        
        // Calculate RMS velocity
        let rms_velocity = velocity.magnitude;
        
        // Determine severity based on ISO 10816-3
        let severity = match rms_velocity {
            v if v < 2.3 => VibrationSeverity::Good,
            v if v < 4.5 => VibrationSeverity::Satisfactory,
            v if v < 7.1 => VibrationSeverity::Unsatisfactory,
            _ => VibrationSeverity::Unacceptable,
        };
        
        // Calculate crest factor (simplified)
        let peak_velocity = velocity.magnitude * 1.414; // Assuming sinusoidal
        let crest_factor = peak_velocity / rms_velocity;
        
        let data = VibrationData {
            timestamp: Utc::now(),
            sensor_id,
            port: config.port.clone(),
            acceleration,
            velocity,
            frequency,
            temperature: temperature + config.calibration.temperature_offset,
            rms_velocity,
            severity,
            peak_velocity,
            crest_factor,
        };
        
        // Store in history
        if let Some(history) = self.data_history.get_mut(&sensor_id) {
            history.push(data.clone());
            if history.len() > self.max_history {
                history.remove(0);
            }
        }
        
        Ok(data)
    }
    
    pub async fn monitor_all(&mut self) -> Result<Vec<VibrationData>> {
        let mut results = Vec::new();
        let sensor_ids: Vec<u8> = self.sensors.keys().copied().collect();
        
        for sensor_id in sensor_ids {
            match self.read_sensor(sensor_id).await {
                Ok(data) => results.push(data),
                Err(e) => eprintln!("Error reading sensor {}: {}", sensor_id, e),
            }
        }
        
        Ok(results)
    }
    
    pub fn check_alarms(&self, data: &VibrationData) -> Vec<String> {
        let mut alarms = Vec::new();
        
        if let Some(config) = self.sensors.get(&data.sensor_id) {
            let thresholds = &config.alarm_thresholds;
            
            // Check velocity alarms
            if data.rms_velocity > thresholds.velocity_alarm {
                alarms.push(format!("🚨 ALARM: Velocity {:.2} mm/s exceeds alarm threshold", data.rms_velocity));
            } else if data.rms_velocity > thresholds.velocity_warning {
                alarms.push(format!("⚠️ WARNING: Velocity {:.2} mm/s exceeds warning threshold", data.rms_velocity));
            }
            
            // Check acceleration alarms
            if data.acceleration.magnitude > thresholds.acceleration_alarm {
                alarms.push(format!("🚨 ALARM: Acceleration {:.2}g exceeds alarm threshold", data.acceleration.magnitude));
            } else if data.acceleration.magnitude > thresholds.acceleration_warning {
                alarms.push(format!("⚠️ WARNING: Acceleration {:.2}g exceeds warning threshold", data.acceleration.magnitude));
            }
            
            // Check temperature alarms
            if data.temperature > thresholds.temperature_alarm {
                alarms.push(format!("🚨 ALARM: Temperature {:.1}°C exceeds alarm threshold", data.temperature));
            } else if data.temperature > thresholds.temperature_warning {
                alarms.push(format!("⚠️ WARNING: Temperature {:.1}°C exceeds warning threshold", data.temperature));
            }
        }
        
        alarms
    }
    
    pub fn get_trend_analysis(&self, sensor_id: u8, hours: usize) -> Result<TrendAnalysis> {
        let history = self.data_history.get(&sensor_id)
            .ok_or_else(|| anyhow!("No history for sensor {}", sensor_id))?;
        
        if history.is_empty() {
            return Err(anyhow!("No data available for analysis"));
        }
        
        let cutoff = Utc::now() - chrono::Duration::hours(hours as i64);
        let recent_data: Vec<&VibrationData> = history
            .iter()
            .filter(|d| d.timestamp > cutoff)
            .collect();
        
        if recent_data.is_empty() {
            return Err(anyhow!("No recent data available"));
        }
        
        // Calculate statistics
        let velocities: Vec<f32> = recent_data.iter().map(|d| d.rms_velocity).collect();
        let mean = velocities.iter().sum::<f32>() / velocities.len() as f32;
        let max = velocities.iter().fold(0.0f32, |a, &b| a.max(b));
        let min = velocities.iter().fold(f32::MAX, |a, &b| a.min(b));
        
        // Calculate trend (simplified linear regression)
        let n = velocities.len() as f32;
        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        let mut sum_xy = 0.0;
        let mut sum_x2 = 0.0;
        
        for (i, &v) in velocities.iter().enumerate() {
            let x = i as f32;
            sum_x += x;
            sum_y += v;
            sum_xy += x * v;
            sum_x2 += x * x;
        }
        
        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x * sum_x);
        let trend = if slope > 0.1 {
            "Increasing"
        } else if slope < -0.1 {
            "Decreasing"
        } else {
            "Stable"
        };
        
        Ok(TrendAnalysis {
            period_hours: hours,
            data_points: recent_data.len(),
            mean_velocity: mean,
            max_velocity: max,
            min_velocity: min,
            trend: trend.to_string(),
            slope,
        })
    }
    
    fn parse_register(data: &u16) -> f32 {
        *data as i16 as f32
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysis {
    pub period_hours: usize,
    pub data_points: usize,
    pub mean_velocity: f32,
    pub max_velocity: f32,
    pub min_velocity: f32,
    pub trend: String,
    pub slope: f32,
}