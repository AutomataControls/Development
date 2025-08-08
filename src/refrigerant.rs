// HVAC Refrigerant Diagnostics Engine
// Implements ASHRAE 207-2021 FDD and NIST guidelines
// Supports P499 pressure transducers and refrigerant database

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

// P499 Pressure Transducer Support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P499Config {
    pub channel: u8,            // MegaBAS 0-10V input channel (0-7)
    pub name: String,
    pub location: PressureLocation,
    pub min_psi: f32,          // Minimum pressure range
    pub max_psi: f32,          // Maximum pressure range  
    pub calibration_offset: f32,
    pub calibration_scale: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PressureLocation {
    Suction,
    Discharge,
    Liquid,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransducerReading {
    pub channel: u8,
    pub raw_voltage: f32,
    pub pressure_psi: f32,
    pub pressure_bar: f32,
    pub timestamp: DateTime<Utc>,
    pub location: PressureLocation,
}

// Refrigerant Properties Database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefrigerantProperties {
    pub name: String,
    pub r_number: String,
    pub molecular_weight: f32,
    pub boiling_point_f: f32,
    pub critical_temp_f: f32,
    pub critical_pressure_psi: f32,
    pub ozone_depletion: f32,
    pub gwp: f32,  // Global Warming Potential
    pub safety_class: String,
    pub pt_curve: Vec<PTPoint>,  // Pressure-Temperature curve
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PTPoint {
    pub temperature_f: f32,
    pub pressure_psi: f32,
}

// System Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemType {
    TXV,           // Thermostatic Expansion Valve
    FixedOrifice,  // Fixed orifice/capillary tube
    EEV,           // Electronic Expansion Valve
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    pub refrigerant_type: String,
    pub system_type: SystemType,
    pub equipment_info: EquipmentInfo,
    pub design_conditions: DesignConditions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquipmentInfo {
    pub manufacturer: String,
    pub model: String,
    pub serial: String,
    pub tonnage: f32,
    pub age_years: u32,
    pub equipment_type: String,  // RTU, Split, Chiller, etc.
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignConditions {
    pub design_subcooling: f32,  // °F
    pub design_superheat: f32,   // °F
    pub design_delta_t: f32,     // °F
    pub design_ambient: f32,     // °F
}

// Diagnostic Reading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticReading {
    pub timestamp: DateTime<Utc>,
    pub suction_pressure: f32,         // psi
    pub discharge_pressure: f32,       // psi
    pub suction_temperature: f32,      // °F
    pub discharge_temperature: f32,    // °F
    pub liquid_line_temperature: f32,  // °F
    pub ambient_temperature: f32,      // °F
    pub return_air_temperature: Option<f32>,
    pub supply_air_temperature: Option<f32>,
    pub indoor_wet_bulb: Option<f32>,
    pub indoor_dry_bulb: Option<f32>,
    pub compressor_amps: Option<f32>,
}

// Diagnostic Results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticResult {
    pub timestamp: DateTime<Utc>,
    pub superheat: f32,
    pub subcooling: f32,
    pub approach_temperature: f32,
    pub delta_t: Option<f32>,
    pub pressure_ratio: f32,
    pub condensing_temp: f32,
    pub evaporating_temp: f32,
    pub discharge_superheat: f32,
    pub efficiency_score: f32,
    pub faults: Vec<Fault>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Severity {
    Critical,
    Major,
    Minor,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fault {
    pub fault_type: String,
    pub severity: Severity,
    pub description: String,
    pub confidence: f32,  // 0.0 to 1.0
    pub impact: String,
}

pub struct RefrigerantDiagnostics {
    refrigerants: HashMap<String, RefrigerantProperties>,
    transducers: HashMap<u8, P499Config>,
    system_config: Option<SystemConfig>,
    history: Vec<DiagnosticResult>,
    max_history: usize,
}

impl RefrigerantDiagnostics {
    pub fn new() -> Self {
        let mut diagnostics = Self {
            refrigerants: HashMap::new(),
            transducers: HashMap::new(),
            system_config: None,
            history: Vec::new(),
            max_history: 1000,
        };
        
        // Load common refrigerants
        diagnostics.load_refrigerant_database();
        diagnostics
    }
    
    fn load_refrigerant_database(&mut self) {
        // R-410A
        self.refrigerants.insert("R410A".to_string(), RefrigerantProperties {
            name: "R-410A".to_string(),
            r_number: "R410A".to_string(),
            molecular_weight: 72.6,
            boiling_point_f: -55.3,
            critical_temp_f: 163.0,
            critical_pressure_psi: 711.0,
            ozone_depletion: 0.0,
            gwp: 2088.0,
            safety_class: "A1".to_string(),
            pt_curve: vec![
                PTPoint { temperature_f: -40.0, pressure_psi: 14.9 },
                PTPoint { temperature_f: -20.0, pressure_psi: 26.5 },
                PTPoint { temperature_f: 0.0, pressure_psi: 44.1 },
                PTPoint { temperature_f: 20.0, pressure_psi: 69.9 },
                PTPoint { temperature_f: 40.0, pressure_psi: 106.5 },
                PTPoint { temperature_f: 60.0, pressure_psi: 156.6 },
                PTPoint { temperature_f: 80.0, pressure_psi: 223.4 },
                PTPoint { temperature_f: 100.0, pressure_psi: 310.1 },
                PTPoint { temperature_f: 120.0, pressure_psi: 420.5 },
            ],
        });
        
        // R-22
        self.refrigerants.insert("R22".to_string(), RefrigerantProperties {
            name: "R-22".to_string(),
            r_number: "R22".to_string(),
            molecular_weight: 86.5,
            boiling_point_f: -41.5,
            critical_temp_f: 205.0,
            critical_pressure_psi: 723.7,
            ozone_depletion: 0.055,
            gwp: 1810.0,
            safety_class: "A1".to_string(),
            pt_curve: vec![
                PTPoint { temperature_f: -40.0, pressure_psi: 4.0 },
                PTPoint { temperature_f: -20.0, pressure_psi: 11.0 },
                PTPoint { temperature_f: 0.0, pressure_psi: 23.8 },
                PTPoint { temperature_f: 20.0, pressure_psi: 43.0 },
                PTPoint { temperature_f: 40.0, pressure_psi: 69.0 },
                PTPoint { temperature_f: 60.0, pressure_psi: 105.0 },
                PTPoint { temperature_f: 80.0, pressure_psi: 151.0 },
                PTPoint { temperature_f: 100.0, pressure_psi: 211.0 },
                PTPoint { temperature_f: 120.0, pressure_psi: 286.0 },
            ],
        });
        
        // R-134a
        self.refrigerants.insert("R134A".to_string(), RefrigerantProperties {
            name: "R-134a".to_string(),
            r_number: "R134A".to_string(),
            molecular_weight: 102.0,
            boiling_point_f: -15.0,
            critical_temp_f: 214.0,
            critical_pressure_psi: 588.7,
            ozone_depletion: 0.0,
            gwp: 1430.0,
            safety_class: "A1".to_string(),
            pt_curve: vec![
                PTPoint { temperature_f: -20.0, pressure_psi: 2.4 },
                PTPoint { temperature_f: 0.0, pressure_psi: 9.2 },
                PTPoint { temperature_f: 20.0, pressure_psi: 21.2 },
                PTPoint { temperature_f: 40.0, pressure_psi: 40.0 },
                PTPoint { temperature_f: 60.0, pressure_psi: 67.2 },
                PTPoint { temperature_f: 80.0, pressure_psi: 105.4 },
                PTPoint { temperature_f: 100.0, pressure_psi: 157.0 },
                PTPoint { temperature_f: 120.0, pressure_psi: 225.0 },
            ],
        });
        
        // R-404A
        self.refrigerants.insert("R404A".to_string(), RefrigerantProperties {
            name: "R-404A".to_string(),
            r_number: "R404A".to_string(),
            molecular_weight: 97.6,
            boiling_point_f: -51.0,
            critical_temp_f: 162.0,
            critical_pressure_psi: 541.0,
            ozone_depletion: 0.0,
            gwp: 3922.0,
            safety_class: "A1".to_string(),
            pt_curve: vec![
                PTPoint { temperature_f: -40.0, pressure_psi: 12.8 },
                PTPoint { temperature_f: -20.0, pressure_psi: 25.2 },
                PTPoint { temperature_f: 0.0, pressure_psi: 44.5 },
                PTPoint { temperature_f: 20.0, pressure_psi: 72.7 },
                PTPoint { temperature_f: 40.0, pressure_psi: 112.3 },
                PTPoint { temperature_f: 60.0, pressure_psi: 166.0 },
                PTPoint { temperature_f: 80.0, pressure_psi: 237.0 },
                PTPoint { temperature_f: 100.0, pressure_psi: 329.0 },
            ],
        });
    }
    
    pub fn configure_transducer(&mut self, config: P499Config) {
        self.transducers.insert(config.channel, config);
    }
    
    pub fn set_system_config(&mut self, config: SystemConfig) {
        self.system_config = Some(config);
    }
    
    pub async fn read_p499(&self, channel: u8) -> Result<TransducerReading> {
        let config = self.transducers.get(&channel)
            .ok_or_else(|| anyhow!("Transducer on channel {} not configured", channel))?;
        
        // Read voltage from MegaBAS (0-10V)
        // This would interface with the actual MegaBAS board
        let raw_voltage = self.read_megabas_voltage(channel).await?;
        
        // Convert voltage to pressure
        // Linear scaling: 0V = min_psi, 10V = max_psi
        let pressure_range = config.max_psi - config.min_psi;
        let pressure_psi = config.min_psi + (raw_voltage / 10.0) * pressure_range;
        
        // Apply calibration
        let calibrated_pressure = (pressure_psi + config.calibration_offset) * config.calibration_scale;
        
        Ok(TransducerReading {
            channel,
            raw_voltage,
            pressure_psi: calibrated_pressure,
            pressure_bar: calibrated_pressure * 0.0689476,  // Convert to bar
            timestamp: Utc::now(),
            location: config.location.clone(),
        })
    }
    
    async fn read_megabas_voltage(&self, channel: u8) -> Result<f32> {
        // Interface with MegaBAS board via I2C
        // This would use the boards.rs module
        // For now, return simulated value
        Ok(5.0 + (channel as f32 * 0.5))  // Simulated
    }
    
    pub fn pressure_to_temperature(&self, pressure: f32, refrigerant: &str) -> Result<f32> {
        let refrigerant = self.refrigerants.get(refrigerant)
            .ok_or_else(|| anyhow!("Unknown refrigerant: {}", refrigerant))?;
        
        // Interpolate PT curve
        for i in 0..refrigerant.pt_curve.len() - 1 {
            let p1 = refrigerant.pt_curve[i].pressure_psi;
            let p2 = refrigerant.pt_curve[i + 1].pressure_psi;
            
            if pressure >= p1 && pressure <= p2 {
                let t1 = refrigerant.pt_curve[i].temperature_f;
                let t2 = refrigerant.pt_curve[i + 1].temperature_f;
                let ratio = (pressure - p1) / (p2 - p1);
                return Ok(t1 + ratio * (t2 - t1));
            }
        }
        
        Err(anyhow!("Pressure {} psi out of range for {}", pressure, refrigerant.name))
    }
    
    pub async fn perform_diagnostics(&mut self, reading: DiagnosticReading) -> Result<DiagnosticResult> {
        let config = self.system_config.as_ref()
            .ok_or_else(|| anyhow!("System not configured"))?;
        
        // Calculate saturated temperatures
        let evaporating_temp = self.pressure_to_temperature(
            reading.suction_pressure,
            &config.refrigerant_type
        )?;
        
        let condensing_temp = self.pressure_to_temperature(
            reading.discharge_pressure,
            &config.refrigerant_type
        )?;
        
        // Calculate superheat (actual temp - saturated temp)
        let superheat = reading.suction_temperature - evaporating_temp;
        
        // Calculate subcooling (saturated temp - actual temp)
        let subcooling = condensing_temp - reading.liquid_line_temperature;
        
        // Calculate approach temperature
        let approach_temperature = reading.liquid_line_temperature - reading.ambient_temperature;
        
        // Calculate discharge superheat
        let discharge_superheat = reading.discharge_temperature - condensing_temp;
        
        // Calculate pressure ratio
        let pressure_ratio = reading.discharge_pressure / reading.suction_pressure.max(1.0);
        
        // Calculate delta T if air temps available
        let delta_t = match (reading.return_air_temperature, reading.supply_air_temperature) {
            (Some(ret), Some(sup)) => Some(ret - sup),
            _ => None,
        };
        
        // Fault detection
        let mut faults = Vec::new();
        let mut recommendations = Vec::new();
        
        // Check superheat
        match config.system_type {
            SystemType::TXV => {
                if superheat < 5.0 {
                    faults.push(Fault {
                        fault_type: "Low Superheat".to_string(),
                        severity: Severity::Major,
                        description: format!("Superheat {:.1}°F is below minimum", superheat),
                        confidence: 0.9,
                        impact: "Risk of liquid flood back to compressor".to_string(),
                    });
                    recommendations.push("Check TXV operation and adjustment".to_string());
                } else if superheat > 20.0 {
                    faults.push(Fault {
                        fault_type: "High Superheat".to_string(),
                        severity: Severity::Minor,
                        description: format!("Superheat {:.1}°F is above normal", superheat),
                        confidence: 0.8,
                        impact: "Reduced cooling capacity".to_string(),
                    });
                    recommendations.push("Check for low refrigerant charge or restricted TXV".to_string());
                }
            },
            SystemType::FixedOrifice => {
                if superheat < 8.0 || superheat > 25.0 {
                    faults.push(Fault {
                        fault_type: "Abnormal Superheat".to_string(),
                        severity: Severity::Minor,
                        description: format!("Superheat {:.1}°F outside normal range", superheat),
                        confidence: 0.7,
                        impact: "System efficiency reduced".to_string(),
                    });
                }
            },
            _ => {}
        }
        
        // Check subcooling
        if subcooling < 5.0 {
            faults.push(Fault {
                fault_type: "Low Subcooling".to_string(),
                severity: Severity::Minor,
                description: format!("Subcooling {:.1}°F is below normal", subcooling),
                confidence: 0.8,
                impact: "Possible undercharge condition".to_string(),
            });
            recommendations.push("Check refrigerant charge level".to_string());
        } else if subcooling > 20.0 {
            faults.push(Fault {
                fault_type: "High Subcooling".to_string(),
                severity: Severity::Minor,
                description: format!("Subcooling {:.1}°F is above normal", subcooling),
                confidence: 0.8,
                impact: "Possible overcharge or restriction".to_string(),
            });
            recommendations.push("Check for overcharge or liquid line restriction".to_string());
        }
        
        // Check pressure ratio
        if pressure_ratio > 10.0 {
            faults.push(Fault {
                fault_type: "High Pressure Ratio".to_string(),
                severity: Severity::Major,
                description: format!("Pressure ratio {:.1} exceeds limits", pressure_ratio),
                confidence: 0.9,
                impact: "Compressor stress and reduced efficiency".to_string(),
            });
            recommendations.push("Check for dirty condenser or high ambient conditions".to_string());
        }
        
        // Check discharge temperature
        if reading.discharge_temperature > 225.0 {
            faults.push(Fault {
                fault_type: "High Discharge Temperature".to_string(),
                severity: Severity::Critical,
                description: format!("Discharge temp {:.1}°F exceeds safe limit", reading.discharge_temperature),
                confidence: 1.0,
                impact: "Compressor damage risk".to_string(),
            });
            recommendations.push("IMMEDIATE ACTION: Check system operation".to_string());
        }
        
        // Calculate efficiency score (0-100)
        let mut efficiency_score = 100.0;
        
        // Deduct for superheat deviation
        let target_superheat = match config.system_type {
            SystemType::TXV => 12.0,
            SystemType::FixedOrifice => 15.0,
            SystemType::EEV => 10.0,
        };
        efficiency_score -= (superheat - target_superheat).abs() * 2.0;
        
        // Deduct for subcooling deviation
        efficiency_score -= (subcooling - config.design_conditions.design_subcooling).abs() * 2.0;
        
        // Deduct for high pressure ratio
        if pressure_ratio > 4.0 {
            efficiency_score -= (pressure_ratio - 4.0) * 5.0;
        }
        
        efficiency_score = efficiency_score.max(0.0).min(100.0);
        
        let result = DiagnosticResult {
            timestamp: reading.timestamp,
            superheat,
            subcooling,
            approach_temperature,
            delta_t,
            pressure_ratio,
            condensing_temp,
            evaporating_temp,
            discharge_superheat,
            efficiency_score,
            faults,
            recommendations,
        };
        
        // Store in history
        self.history.push(result.clone());
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }
        
        Ok(result)
    }
    
    pub fn get_refrigerant_list(&self) -> Vec<String> {
        self.refrigerants.keys().cloned().collect()
    }
    
    pub fn get_refrigerant_properties(&self, name: &str) -> Option<&RefrigerantProperties> {
        self.refrigerants.get(name)
    }
}