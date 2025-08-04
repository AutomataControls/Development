// P499 Series Electronic Pressure Transducer Interface
// Supports 0-10V and 4-20mA models via Sequent Microsystems HAT

use serde::{Deserialize, Serialize};
use std::error::Error;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P499Configuration {
    pub model: String,
    pub output_type: OutputType,
    pub pressure_range: PressureRange,
    pub channel: u8,
    pub calibration_offset: f32,
    pub calibration_scale: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputType {
    Voltage0to10V,
    Voltage05to45V,
    Current4to20mA,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PressureRange {
    pub min_psi: f32,
    pub max_psi: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransducerReading {
    pub channel: u8,
    pub raw_value: f32,
    pub pressure_psi: f32,
    pub timestamp: u64,
}

pub struct P499Interface {
    hat_stack_level: u8,  // Sequent Microsystems HAT stack level (0-7)
    transducers: Vec<P499Configuration>,
}

impl P499Interface {
    pub fn new(hat_stack_level: u8) -> Self {
        P499Interface {
            hat_stack_level,
            transducers: Vec::new(),
        }
    }
    
    pub fn add_transducer(&mut self, config: P499Configuration) {
        self.transducers.push(config);
    }
    
    // Read voltage from Sequent Microsystems 0-10V input HAT
    fn read_voltage_channel(&self, channel: u8) -> Result<f32, Box<dyn Error>> {
        // Using Python script interface for Sequent Microsystems HAT
        // The HAT provides 8 channels of 0-10V input with 12-bit resolution
        
        let output = Command::new("python3")
            .arg("-c")
            .arg(format!(
                "import sm_4_20ma; print(sm_4_20ma.get_0_10v({}, {}))",
                self.hat_stack_level,
                channel
            ))
            .output()?;
        
        if output.status.success() {
            let voltage_str = String::from_utf8_lossy(&output.stdout);
            let voltage = voltage_str.trim().parse::<f32>()?;
            Ok(voltage)
        } else {
            Err(format!("Failed to read channel {}: {}", 
                channel, 
                String::from_utf8_lossy(&output.stderr)
            ).into())
        }
    }
    
    // Read current from Sequent Microsystems 4-20mA input HAT
    fn read_current_channel(&self, channel: u8) -> Result<f32, Box<dyn Error>> {
        let output = Command::new("python3")
            .arg("-c")
            .arg(format!(
                "import sm_4_20ma; print(sm_4_20ma.get_4_20ma({}, {}))",
                self.hat_stack_level,
                channel
            ))
            .output()?;
        
        if output.status.success() {
            let current_str = String::from_utf8_lossy(&output.stdout);
            let current = current_str.trim().parse::<f32>()?;
            Ok(current)
        } else {
            Err(format!("Failed to read channel {}: {}", 
                channel, 
                String::from_utf8_lossy(&output.stderr)
            ).into())
        }
    }
    
    pub fn read_transducer(&self, channel: u8) -> Result<TransducerReading, Box<dyn Error>> {
        let config = self.transducers
            .iter()
            .find(|t| t.channel == channel)
            .ok_or("Transducer not found")?;
        
        let raw_value = match &config.output_type {
            OutputType::Voltage0to10V => self.read_voltage_channel(channel)?,
            OutputType::Voltage05to45V => {
                // For ratiometric output, we need to scale differently
                let voltage = self.read_voltage_channel(channel)?;
                // Convert 0-10V reading to 0.5-4.5V scale
                voltage * 0.45 + 0.5
            },
            OutputType::Current4to20mA => self.read_current_channel(channel)?,
        };
        
        let pressure = self.calculate_pressure(config, raw_value);
        
        Ok(TransducerReading {
            channel,
            raw_value,
            pressure_psi: pressure,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        })
    }
    
    fn calculate_pressure(&self, config: &P499Configuration, raw_value: f32) -> f32 {
        let normalized_value = match &config.output_type {
            OutputType::Voltage0to10V => raw_value / 10.0,
            OutputType::Voltage05to45V => (raw_value - 0.5) / 4.0,
            OutputType::Current4to20mA => (raw_value - 4.0) / 16.0,
        };
        
        let pressure_range = config.pressure_range.max_psi - config.pressure_range.min_psi;
        let pressure = config.pressure_range.min_psi + (normalized_value * pressure_range);
        
        // Apply calibration
        (pressure + config.calibration_offset) * config.calibration_scale
    }
    
    pub fn read_all_transducers(&self) -> Vec<Result<TransducerReading, String>> {
        self.transducers
            .iter()
            .map(|t| {
                self.read_transducer(t.channel)
                    .map_err(|e| e.to_string())
            })
            .collect()
    }
    
    // Verify transducer operation as per manual procedures
    pub fn verify_transducer(&self, channel: u8, measured_pressure: f32) -> Result<bool, Box<dyn Error>> {
        let reading = self.read_transducer(channel)?;
        let config = self.transducers
            .iter()
            .find(|t| t.channel == channel)
            .ok_or("Transducer not found")?;
        
        // Calculate expected output based on measured pressure
        let pressure_range = config.pressure_range.max_psi - config.pressure_range.min_psi;
        let normalized = (measured_pressure - config.pressure_range.min_psi) / pressure_range;
        
        let expected_value = match &config.output_type {
            OutputType::Voltage0to10V => normalized * 10.0,
            OutputType::Voltage05to45V => 0.5 + (normalized * 4.0),
            OutputType::Current4to20mA => 4.0 + (normalized * 16.0),
        };
        
        // Check if within 1% accuracy as per P499 specification
        let tolerance = expected_value * 0.01;
        let difference = (reading.raw_value - expected_value).abs();
        
        Ok(difference <= tolerance)
    }
}

// Common P499 models with their specifications
pub fn create_p499_rcp_101() -> P499Configuration {
    P499Configuration {
        model: "P499RCP-101C".to_string(),
        output_type: OutputType::Voltage05to45V,
        pressure_range: PressureRange {
            min_psi: 0.0,
            max_psi: 100.0,
        },
        channel: 0,
        calibration_offset: 0.0,
        calibration_scale: 1.0,
    }
}

pub fn create_p499_vcp_105() -> P499Configuration {
    P499Configuration {
        model: "P499VCP-105C".to_string(),
        output_type: OutputType::Voltage0to10V,
        pressure_range: PressureRange {
            min_psi: 0.0,
            max_psi: 500.0,
        },
        channel: 0,
        calibration_offset: 0.0,
        calibration_scale: 1.0,
    }
}

pub fn create_p499_acp_107() -> P499Configuration {
    P499Configuration {
        model: "P499ACP-107C".to_string(),
        output_type: OutputType::Current4to20mA,
        pressure_range: PressureRange {
            min_psi: 0.0,
            max_psi: 750.0,
        },
        channel: 0,
        calibration_offset: 0.0,
        calibration_scale: 1.0,
    }
}

pub fn create_p499_rcps_100() -> P499Configuration {
    P499Configuration {
        model: "P499RCPS100C".to_string(),
        output_type: OutputType::Voltage05to45V,
        pressure_range: PressureRange {
            min_psi: -10.0,  // 20 in. Hg vacuum
            max_psi: 100.0,
        },
        channel: 0,
        calibration_offset: 0.0,
        calibration_scale: 1.0,
    }
}