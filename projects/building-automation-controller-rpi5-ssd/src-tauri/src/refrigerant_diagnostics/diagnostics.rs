// HVAC/Refrigeration Diagnostic Engine
// Implements ASHRAE 207-2021 and NIST FDD guidelines

use serde::{Deserialize, Serialize};
use crate::refrigerants::{RefrigerantDatabase};
use crate::p499_transducer::TransducerReading;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfiguration {
    pub refrigerant_type: String,
    pub system_type: SystemType,
    pub equipment_info: EquipmentInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemType {
    TXV,            // Thermostatic Expansion Valve
    FixedOrifice,   // Piston/Cap tube
    EEV,            // Electronic Expansion Valve
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquipmentInfo {
    pub manufacturer: String,
    pub model: String,
    pub tonnage: f32,
    pub age_years: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticReading {
    pub timestamp: u64,
    pub suction_pressure: f32,
    pub discharge_pressure: f32,
    pub suction_temperature: f32,
    pub discharge_temperature: f32,
    pub liquid_line_temperature: f32,
    pub ambient_temperature: f32,
    pub indoor_wet_bulb: Option<f32>,
    pub indoor_dry_bulb: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticResult {
    pub superheat: f32,
    pub subcooling: f32,
    pub approach_temperature: f32,
    pub delta_t: Option<f32>,
    pub pressure_ratio: f32,
    pub faults: Vec<Fault>,
    pub efficiency_score: f32,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fault {
    pub fault_type: FaultType,
    pub severity: Severity,
    pub confidence: f32,
    pub description: String,
    pub impact: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FaultType {
    LowRefrigerantCharge,
    Overcharge,
    DirtyCondenser,
    DirtyEvaporator,
    TXVMalfunction,
    CompressorIssue,
    LiquidLineRestriction,
    NonCondensables,
    AirflowIssue,
    SensorFault,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Severity {
    Critical,   // Immediate action required
    Major,      // Significant efficiency loss
    Minor,      // Moderate impact
    Info,       // Informational only
}

pub struct DiagnosticEngine {
    refrigerant_db: RefrigerantDatabase,
}

impl DiagnosticEngine {
    pub fn new() -> Self {
        DiagnosticEngine {
            refrigerant_db: RefrigerantDatabase::new(),
        }
    }
    
    pub fn analyze_system(
        &self,
        config: &SystemConfiguration,
        reading: &DiagnosticReading,
    ) -> Result<DiagnosticResult, String> {
        // Get refrigerant properties
        let refrigerant = self.refrigerant_db
            .get_refrigerant(&config.refrigerant_type)
            .ok_or("Unknown refrigerant type")?;
        
        // Calculate saturation temperatures
        let suction_sat_temp = self.refrigerant_db
            .calculate_saturation_temperature(&config.refrigerant_type, reading.suction_pressure)
            .ok_or("Cannot calculate suction saturation temperature")?;
        
        let discharge_sat_temp = self.refrigerant_db
            .calculate_saturation_temperature(&config.refrigerant_type, reading.discharge_pressure)
            .ok_or("Cannot calculate discharge saturation temperature")?;
        
        // Calculate key diagnostic parameters
        let superheat = reading.suction_temperature - suction_sat_temp;
        let subcooling = discharge_sat_temp - reading.liquid_line_temperature;
        let approach_temperature = discharge_sat_temp - reading.ambient_temperature;
        let pressure_ratio = reading.discharge_pressure / reading.suction_pressure;
        
        // Calculate delta T if indoor temperatures available
        let delta_t = if let (Some(indoor_db), Some(supply_temp)) = 
            (reading.indoor_dry_bulb, Some(reading.suction_temperature - 20.0)) {
            Some(indoor_db - supply_temp)
        } else {
            None
        };
        
        // Run fault detection algorithms
        let mut faults = Vec::new();
        
        // Check for low refrigerant charge
        if superheat > 15.0 && subcooling < 5.0 {
            faults.push(Fault {
                fault_type: FaultType::LowRefrigerantCharge,
                severity: Severity::Major,
                confidence: self.calculate_confidence(superheat - 15.0, 5.0 - subcooling),
                description: "Low refrigerant charge detected".to_string(),
                impact: "Reduced cooling capacity, increased energy consumption".to_string(),
            });
        }
        
        // Check for overcharge
        if superheat < 5.0 && subcooling > 18.0 {
            faults.push(Fault {
                fault_type: FaultType::Overcharge,
                severity: Severity::Major,
                confidence: self.calculate_confidence(5.0 - superheat, subcooling - 18.0),
                description: "System overcharged with refrigerant".to_string(),
                impact: "Risk of liquid slugging, reduced efficiency".to_string(),
            });
        }
        
        // Check for dirty condenser
        if approach_temperature > 25.0 && subcooling > 15.0 {
            faults.push(Fault {
                fault_type: FaultType::DirtyCondenser,
                severity: Severity::Minor,
                confidence: self.calculate_confidence(approach_temperature - 25.0, 0.0),
                description: "Condenser coil may be dirty or airflow restricted".to_string(),
                impact: "High head pressure, reduced efficiency".to_string(),
            });
        }
        
        // Check for TXV issues
        match config.system_type {
            SystemType::TXV => {
                if superheat < 6.0 || superheat > 15.0 {
                    faults.push(Fault {
                        fault_type: FaultType::TXVMalfunction,
                        severity: Severity::Major,
                        confidence: 0.7,
                        description: format!("TXV may be malfunctioning (superheat: {:.1}°F)", superheat),
                        impact: "Poor temperature control, potential compressor damage".to_string(),
                    });
                }
            },
            _ => {}
        }
        
        // Check pressure ratio
        if pressure_ratio > 4.0 || pressure_ratio < 2.0 {
            faults.push(Fault {
                fault_type: FaultType::CompressorIssue,
                severity: Severity::Critical,
                confidence: 0.8,
                description: format!("Abnormal pressure ratio: {:.1}", pressure_ratio),
                impact: "Possible compressor failure or system malfunction".to_string(),
            });
        }
        
        // Calculate efficiency score (0-100)
        let efficiency_score = self.calculate_efficiency_score(
            superheat,
            subcooling,
            approach_temperature,
            pressure_ratio,
            &config.system_type,
        );
        
        // Generate recommendations
        let recommendations = self.generate_recommendations(&faults, efficiency_score);
        
        Ok(DiagnosticResult {
            superheat,
            subcooling,
            approach_temperature,
            delta_t,
            pressure_ratio,
            faults,
            efficiency_score,
            recommendations,
        })
    }
    
    fn calculate_confidence(&self, deviation1: f32, deviation2: f32) -> f32 {
        // Simple confidence calculation based on deviation from normal
        let total_deviation = deviation1.abs() + deviation2.abs();
        (1.0 - (1.0 / (1.0 + total_deviation * 0.1))).min(0.95)
    }
    
    fn calculate_efficiency_score(
        &self,
        superheat: f32,
        subcooling: f32,
        approach_temp: f32,
        pressure_ratio: f32,
        system_type: &SystemType,
    ) -> f32 {
        let mut score = 100.0;
        
        // Superheat scoring
        let target_superheat = match system_type {
            SystemType::TXV => 10.0,
            SystemType::FixedOrifice => 15.0,
            SystemType::EEV => 8.0,
        };
        score -= (superheat - target_superheat).abs() * 2.0;
        
        // Subcooling scoring
        let target_subcooling = 10.0;
        score -= (subcooling - target_subcooling).abs() * 2.0;
        
        // Approach temperature scoring
        if approach_temp > 20.0 {
            score -= (approach_temp - 20.0) * 1.5;
        }
        
        // Pressure ratio scoring
        if pressure_ratio < 2.5 || pressure_ratio > 3.5 {
            score -= 10.0;
        }
        
        score.max(0.0).min(100.0)
    }
    
    fn generate_recommendations(&self, faults: &[Fault], efficiency_score: f32) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        for fault in faults {
            match fault.fault_type {
                FaultType::LowRefrigerantCharge => {
                    recommendations.push("1. Check for refrigerant leaks using electronic leak detector".to_string());
                    recommendations.push("2. Repair any leaks found and evacuate system".to_string());
                    recommendations.push("3. Recharge to manufacturer specifications".to_string());
                },
                FaultType::Overcharge => {
                    recommendations.push("1. Recover excess refrigerant to proper charge level".to_string());
                    recommendations.push("2. Verify subcooling matches manufacturer target".to_string());
                },
                FaultType::DirtyCondenser => {
                    recommendations.push("1. Clean condenser coil with appropriate cleaner".to_string());
                    recommendations.push("2. Check and clear any airflow obstructions".to_string());
                    recommendations.push("3. Verify condenser fan operation".to_string());
                },
                FaultType::TXVMalfunction => {
                    recommendations.push("1. Check TXV bulb mounting and insulation".to_string());
                    recommendations.push("2. Verify TXV superheat setting".to_string());
                    recommendations.push("3. Consider TXV replacement if hunting persists".to_string());
                },
                _ => {}
            }
        }
        
        if efficiency_score < 70.0 {
            recommendations.push("System efficiency is poor - comprehensive service recommended".to_string());
        }
        
        recommendations
    }
    
    // Target superheat calculation for fixed orifice systems
    pub fn calculate_target_superheat_fixed_orifice(
        &self,
        indoor_wet_bulb: f32,
        outdoor_dry_bulb: f32,
    ) -> f32 {
        // Based on manufacturer charging charts
        // This is a simplified calculation
        let wb_factor = (75.0 - indoor_wet_bulb) * 0.5;
        let db_factor = (outdoor_dry_bulb - 85.0) * 0.3;
        
        (12.0 + wb_factor - db_factor).max(5.0).min(25.0)
    }
    
    // Continuous monitoring analysis
    pub fn analyze_trend(
        &self,
        readings: &[DiagnosticResult],
        _time_window_hours: f32,
    ) -> TrendAnalysis {
        if readings.is_empty() {
            return TrendAnalysis::default();
        }
        
        // Calculate trends
        let avg_superheat = readings.iter().map(|r| r.superheat).sum::<f32>() / readings.len() as f32;
        let avg_subcooling = readings.iter().map(|r| r.subcooling).sum::<f32>() / readings.len() as f32;
        let avg_efficiency = readings.iter().map(|r| r.efficiency_score).sum::<f32>() / readings.len() as f32;
        
        // Detect trending issues
        let superheat_trend = self.calculate_trend(
            &readings.iter().map(|r| r.superheat).collect::<Vec<_>>()
        );
        
        let efficiency_trend = self.calculate_trend(
            &readings.iter().map(|r| r.efficiency_score).collect::<Vec<_>>()
        );
        
        TrendAnalysis {
            average_superheat: avg_superheat,
            average_subcooling: avg_subcooling,
            average_efficiency: avg_efficiency,
            superheat_trend,
            efficiency_trend,
            fault_frequency: self.calculate_fault_frequency(readings),
        }
    }
    
    fn calculate_trend(&self, values: &[f32]) -> Trend {
        if values.len() < 2 {
            return Trend::Stable;
        }
        
        let first_half_avg = values[..values.len()/2].iter().sum::<f32>() / (values.len()/2) as f32;
        let second_half_avg = values[values.len()/2..].iter().sum::<f32>() / (values.len() - values.len()/2) as f32;
        
        let change = second_half_avg - first_half_avg;
        
        if change > 2.0 {
            Trend::Increasing
        } else if change < -2.0 {
            Trend::Decreasing
        } else {
            Trend::Stable
        }
    }
    
    fn calculate_fault_frequency(&self, readings: &[DiagnosticResult]) -> f32 {
        let total_faults: usize = readings.iter().map(|r| r.faults.len()).sum();
        total_faults as f32 / readings.len() as f32
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TrendAnalysis {
    pub average_superheat: f32,
    pub average_subcooling: f32,
    pub average_efficiency: f32,
    pub superheat_trend: Trend,
    pub efficiency_trend: Trend,
    pub fault_frequency: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Trend {
    Increasing,
    Stable,
    Decreasing,
}

impl Default for Trend {
    fn default() -> Self {
        Trend::Stable
    }
}