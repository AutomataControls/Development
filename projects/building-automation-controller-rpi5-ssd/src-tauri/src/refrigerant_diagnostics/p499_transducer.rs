// P499 Series Electronic Pressure Transducer Interface
// Adapted to use existing MegaBAS HAT interface

use serde::{Deserialize, Serialize};
use std::error::Error;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P499Configuration {
    pub model: String,
    pub output_type: OutputType,
    pub pressure_range: PressureRange,
    pub channel: u8,
    pub board_id: String,  // Added to specify which board
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
    pub board_id: String,
    pub raw_value: f32,
    pub pressure_psi: f32,
    pub pressure_bar: f32,
    pub timestamp: u64,
}

pub struct P499Interface {
    transducers: Vec<P499Configuration>,
}

impl P499Interface {
    pub fn new() -> Self {
        P499Interface {
            transducers: Vec::new(),
        }
    }
    
    pub fn add_transducer(&mut self, config: P499Configuration) {
        self.transducers.push(config);
    }
    
    pub fn remove_transducer(&mut self, board_id: &str, channel: u8) {
        self.transducers.retain(|t| !(t.board_id == board_id && t.channel == channel));
    }
    
    pub fn get_transducers(&self) -> &Vec<P499Configuration> {
        &self.transducers
    }
    
    // Read voltage from MegaBAS HAT or 16univin board
    pub fn read_voltage_channel(&self, board_id: &str, channel: u8) -> Result<f32, Box<dyn Error>> {
        // Extract board type and stack level from board_id (e.g., "megabas_0")
        let parts: Vec<&str> = board_id.split('_').collect();
        if parts.len() != 2 {
            return Err("Invalid board ID format".into());
        }
        
        let board_type = parts[0];
        let stack_level = parts[1].parse::<u8>()
            .map_err(|_| "Invalid stack level")?;
        
        let output = match board_type {
            "megabas" => {
                // Use megabas Python library for 0-10V input
                Command::new("python3")
                    .arg("-c")
                    .arg(format!("import megabas; print(megabas.getUIn({}, {}))", stack_level, channel))
                    .output()?
            },
            "16univin" => {
                // Use 16univin library for universal inputs
                Command::new("python3")
                    .arg("-c")
                    .arg(format!("import lib16univin; card = lib16univin.SM16univin({}); print(card.get_u_in({}))", 
                        stack_level, channel))
                    .output()?
            },
            _ => return Err("Unsupported board type for pressure transducers".into()),
        };
        
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
    
    pub fn read_transducer(&self, board_id: &str, channel: u8) -> Result<TransducerReading, Box<dyn Error>> {
        let config = self.transducers
            .iter()
            .find(|t| t.board_id == board_id && t.channel == channel)
            .ok_or("Transducer not configured")?;
        
        let raw_value = match &config.output_type {
            OutputType::Voltage0to10V => self.read_voltage_channel(board_id, channel)?,
            OutputType::Voltage05to45V => {
                // For ratiometric output, scale appropriately
                let voltage = self.read_voltage_channel(board_id, channel)?;
                // Convert 0-10V reading to 0.5-4.5V scale
                voltage * 0.45 + 0.5
            },
            OutputType::Current4to20mA => {
                // For current loop, need to use different channel type
                // This would need special configuration on the universal input
                return Err("4-20mA not yet implemented - use 0-10V models".into());
            },
        };
        
        let pressure_psi = self.calculate_pressure(config, raw_value);
        let pressure_bar = pressure_psi * 0.0689476; // Convert PSI to bar
        
        Ok(TransducerReading {
            channel,
            board_id: board_id.to_string(),
            raw_value,
            pressure_psi,
            pressure_bar,
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
                self.read_transducer(&t.board_id, t.channel)
                    .map_err(|e| e.to_string())
            })
            .collect()
    }
    
    // Verify transducer operation
    pub fn verify_transducer(&self, board_id: &str, channel: u8, measured_pressure: f32) -> Result<bool, Box<dyn Error>> {
        let reading = self.read_transducer(board_id, channel)?;
        let config = self.transducers
            .iter()
            .find(|t| t.board_id == board_id && t.channel == channel)
            .ok_or("Transducer not found")?;
        
        // Calculate expected output based on measured pressure
        let pressure_range = config.pressure_range.max_psi - config.pressure_range.min_psi;
        let normalized = (measured_pressure - config.pressure_range.min_psi) / pressure_range;
        
        let expected_value = match &config.output_type {
            OutputType::Voltage0to10V => normalized * 10.0,
            OutputType::Voltage05to45V => 0.5 + (normalized * 4.0),
            OutputType::Current4to20mA => 4.0 + (normalized * 16.0),
        };
        
        // Check if reading is within 2% of expected
        let tolerance = expected_value * 0.02;
        let within_tolerance = (reading.raw_value - expected_value).abs() <= tolerance;
        
        Ok(within_tolerance)
    }
    
    // Get common P499 models
    pub fn get_common_models() -> Vec<P499Model> {
        vec![
            P499Model {
                model: "P499VAS-101C".to_string(),
                description: "0-100 PSI, 0-10V output".to_string(),
                pressure_range: PressureRange { min_psi: 0.0, max_psi: 100.0 },
                output_type: OutputType::Voltage0to10V,
            },
            P499Model {
                model: "P499VAS-501C".to_string(),
                description: "0-500 PSI, 0-10V output".to_string(),
                pressure_range: PressureRange { min_psi: 0.0, max_psi: 500.0 },
                output_type: OutputType::Voltage0to10V,
            },
            P499Model {
                model: "P499VAS-751C".to_string(),
                description: "0-750 PSI, 0-10V output".to_string(),
                pressure_range: PressureRange { min_psi: 0.0, max_psi: 750.0 },
                output_type: OutputType::Voltage0to10V,
            },
            P499Model {
                model: "P499RAS-101C".to_string(),
                description: "0-100 PSI, 0.5-4.5V ratiometric".to_string(),
                pressure_range: PressureRange { min_psi: 0.0, max_psi: 100.0 },
                output_type: OutputType::Voltage05to45V,
            },
            P499Model {
                model: "P499RAS-501C".to_string(),
                description: "0-500 PSI, 0.5-4.5V ratiometric".to_string(),
                pressure_range: PressureRange { min_psi: 0.0, max_psi: 500.0 },
                output_type: OutputType::Voltage05to45V,
            },
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P499Model {
    pub model: String,
    pub description: String,
    pub pressure_range: PressureRange,
    pub output_type: OutputType,
}