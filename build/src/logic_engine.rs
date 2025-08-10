// Logic Engine for Critical HVAC Control
// Executes control logic with safety interlocks and failsafes

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use chrono::{DateTime, Utc};
use std::path::PathBuf;

// CRITICAL: Safety limits for HVAC equipment
const MAX_DISCHARGE_PRESSURE: f32 = 450.0; // PSI
const MIN_SUCTION_PRESSURE: f32 = 20.0;    // PSI
const MAX_DISCHARGE_TEMP: f32 = 225.0;     // °F
const MIN_SUPERHEAT: f32 = 5.0;            // °F
const MAX_RUNTIME_HOURS: u32 = 24;         // Max continuous runtime
const COMPRESSOR_DELAY_SECONDS: u64 = 180; // Anti-short-cycle delay

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogicFile {
    pub id: String,
    pub name: String,
    pub file_path: PathBuf,
    pub equipment_type: String,
    pub description: String,
    pub is_active: bool,
    pub execution_interval: u32, // seconds
    pub last_execution: Option<DateTime<Utc>>,
    pub next_execution: DateTime<Utc>,
    pub execution_count: u64,
    pub last_error: Option<String>,
    pub safety_critical: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquipmentState {
    pub equipment_id: String,
    pub equipment_type: EquipmentType,
    pub is_running: bool,
    pub runtime_seconds: u64,
    pub last_start: Option<DateTime<Utc>>,
    pub last_stop: Option<DateTime<Utc>>,
    pub fault_count: u32,
    pub lockout: bool,
    pub lockout_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EquipmentType {
    Compressor,
    Fan,
    Pump,
    Valve,
    Damper,
    Heater,
    Humidifier,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyInterlock {
    pub name: String,
    pub condition: String,
    pub active: bool,
    pub prevents_start: bool,
    pub forces_stop: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlPoint {
    pub name: String,
    pub value: f32,
    pub setpoint: f32,
    pub deadband: f32,
    pub control_mode: ControlMode,
    pub output: f32,
    pub min_output: f32,
    pub max_output: f32,
    pub rate_limit: f32, // Max change per second
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ControlMode {
    Off,
    Manual,
    Auto,
    Override,
}

pub struct LogicEngine {
    logic_files: Arc<RwLock<HashMap<String, LogicFile>>>,
    equipment_states: Arc<RwLock<HashMap<String, EquipmentState>>>,
    safety_interlocks: Arc<RwLock<Vec<SafetyInterlock>>>,
    control_points: Arc<RwLock<HashMap<String, ControlPoint>>>,
    execution_enabled: Arc<RwLock<bool>>,
    maintenance_mode: Arc<RwLock<bool>>,
    emergency_stop: Arc<RwLock<bool>>,
}

impl LogicEngine {
    pub fn new() -> Self {
        let mut engine = Self {
            logic_files: Arc::new(RwLock::new(HashMap::new())),
            equipment_states: Arc::new(RwLock::new(HashMap::new())),
            safety_interlocks: Arc::new(RwLock::new(Vec::new())),
            control_points: Arc::new(RwLock::new(HashMap::new())),
            execution_enabled: Arc::new(RwLock::new(true)),
            maintenance_mode: Arc::new(RwLock::new(false)),
            emergency_stop: Arc::new(RwLock::new(false)),
        };
        
        // Initialize safety interlocks
        engine.init_safety_interlocks();
        
        // Load default logic files
        engine.load_default_logic();
        
        engine
    }
    
    fn init_safety_interlocks(&self) {
        let interlocks = vec![
            SafetyInterlock {
                name: "High Discharge Pressure".to_string(),
                condition: format!("discharge_pressure > {}", MAX_DISCHARGE_PRESSURE),
                active: false,
                prevents_start: true,
                forces_stop: true,
                message: "Discharge pressure exceeds safe limit".to_string(),
            },
            SafetyInterlock {
                name: "Low Suction Pressure".to_string(),
                condition: format!("suction_pressure < {}", MIN_SUCTION_PRESSURE),
                active: false,
                prevents_start: true,
                forces_stop: true,
                message: "Suction pressure below safe limit".to_string(),
            },
            SafetyInterlock {
                name: "High Discharge Temperature".to_string(),
                condition: format!("discharge_temp > {}", MAX_DISCHARGE_TEMP),
                active: false,
                prevents_start: true,
                forces_stop: true,
                message: "Discharge temperature exceeds safe limit".to_string(),
            },
            SafetyInterlock {
                name: "Compressor Short Cycle".to_string(),
                condition: "time_since_stop < 180".to_string(),
                active: false,
                prevents_start: true,
                forces_stop: false,
                message: "Anti-short-cycle timer active".to_string(),
            },
            SafetyInterlock {
                name: "Emergency Stop".to_string(),
                condition: "emergency_stop == true".to_string(),
                active: false,
                prevents_start: true,
                forces_stop: true,
                message: "Emergency stop activated".to_string(),
            },
            SafetyInterlock {
                name: "Maintenance Mode".to_string(),
                condition: "maintenance_mode == true".to_string(),
                active: false,
                prevents_start: true,
                forces_stop: false,
                message: "System in maintenance mode".to_string(),
            },
        ];
        
        let mut interlocks_guard = self.safety_interlocks.blocking_write();
        *interlocks_guard = interlocks;
    }
    
    fn load_default_logic(&self) {
        // Cooling tower control logic
        let cooling_tower = LogicFile {
            id: "cooling-tower-001".to_string(),
            name: "Cooling Tower Control".to_string(),
            file_path: PathBuf::from("/opt/nexus/logic/cooling-tower.js"),
            equipment_type: "Cooling Tower".to_string(),
            description: "Condenser water temperature control with VFD optimization".to_string(),
            is_active: true,
            execution_interval: 30,
            last_execution: None,
            next_execution: Utc::now() + chrono::Duration::seconds(30),
            execution_count: 0,
            last_error: None,
            safety_critical: true,
        };
        
        // AHU control logic
        let ahu_control = LogicFile {
            id: "ahu-001".to_string(),
            name: "AHU Supply Air Control".to_string(),
            file_path: PathBuf::from("/opt/nexus/logic/ahu-control.js"),
            equipment_type: "Air Handler".to_string(),
            description: "Supply air temperature and static pressure control".to_string(),
            is_active: true,
            execution_interval: 60,
            last_execution: None,
            next_execution: Utc::now() + chrono::Duration::seconds(60),
            execution_count: 0,
            last_error: None,
            safety_critical: true,
        };
        
        let mut files = self.logic_files.blocking_write();
        files.insert(cooling_tower.id.clone(), cooling_tower);
        files.insert(ahu_control.id.clone(), ahu_control);
    }
    
    // Check all safety interlocks before allowing equipment operation
    pub async fn check_safety_interlocks(&self, equipment_id: &str) -> Result<Vec<SafetyInterlock>> {
        let interlocks = self.safety_interlocks.read().await;
        let active_interlocks: Vec<SafetyInterlock> = interlocks
            .iter()
            .filter(|i| i.active)
            .cloned()
            .collect();
        
        if !active_interlocks.is_empty() {
            println!("⚠️ SAFETY INTERLOCKS ACTIVE:");
            for interlock in &active_interlocks {
                println!("  - {}: {}", interlock.name, interlock.message);
            }
        }
        
        Ok(active_interlocks)
    }
    
    // Start equipment with safety checks
    pub async fn start_equipment(&self, equipment_id: &str) -> Result<()> {
        // Check emergency stop
        if *self.emergency_stop.read().await {
            return Err(anyhow!("EMERGENCY STOP is active"));
        }
        
        // Check maintenance mode
        if *self.maintenance_mode.read().await {
            return Err(anyhow!("System is in maintenance mode"));
        }
        
        // Check safety interlocks
        let interlocks = self.check_safety_interlocks(equipment_id).await?;
        for interlock in interlocks {
            if interlock.prevents_start {
                return Err(anyhow!("Safety interlock prevents start: {}", interlock.name));
            }
        }
        
        // Check equipment state
        let mut states = self.equipment_states.write().await;
        let state = states.entry(equipment_id.to_string()).or_insert(EquipmentState {
            equipment_id: equipment_id.to_string(),
            equipment_type: EquipmentType::Compressor,
            is_running: false,
            runtime_seconds: 0,
            last_start: None,
            last_stop: None,
            fault_count: 0,
            lockout: false,
            lockout_reason: None,
        });
        
        // Check if locked out
        if state.lockout {
            return Err(anyhow!("Equipment is locked out: {}", 
                state.lockout_reason.as_ref().unwrap_or(&"Unknown".to_string())));
        }
        
        // Check anti-short-cycle timer for compressors
        if matches!(state.equipment_type, EquipmentType::Compressor) {
            if let Some(last_stop) = state.last_stop {
                let time_since_stop = Utc::now().signed_duration_since(last_stop).num_seconds();
                if time_since_stop < COMPRESSOR_DELAY_SECONDS as i64 {
                    return Err(anyhow!("Anti-short-cycle timer active: {} seconds remaining", 
                        COMPRESSOR_DELAY_SECONDS - time_since_stop as u64));
                }
            }
        }
        
        // Start equipment
        state.is_running = true;
        state.last_start = Some(Utc::now());
        
        println!("✅ Started equipment: {}", equipment_id);
        
        Ok(())
    }
    
    // Stop equipment safely
    pub async fn stop_equipment(&self, equipment_id: &str) -> Result<()> {
        let mut states = self.equipment_states.write().await;
        if let Some(state) = states.get_mut(equipment_id) {
            if state.is_running {
                state.is_running = false;
                state.last_stop = Some(Utc::now());
                
                // Calculate runtime
                if let Some(last_start) = state.last_start {
                    let runtime = Utc::now().signed_duration_since(last_start).num_seconds() as u64;
                    state.runtime_seconds += runtime;
                }
                
                println!("⏹️ Stopped equipment: {}", equipment_id);
            }
        }
        
        Ok(())
    }
    
    // Emergency stop - immediately stop all equipment
    pub async fn emergency_stop_all(&self) -> Result<()> {
        println!("🚨 EMERGENCY STOP ACTIVATED!");
        
        *self.emergency_stop.write().await = true;
        *self.execution_enabled.write().await = false;
        
        // Stop all equipment
        let states = self.equipment_states.read().await;
        let equipment_ids: Vec<String> = states.keys().cloned().collect();
        drop(states);
        
        for equipment_id in equipment_ids {
            self.stop_equipment(&equipment_id).await?;
        }
        
        Ok(())
    }
    
    // Reset emergency stop
    pub async fn reset_emergency_stop(&self) -> Result<()> {
        *self.emergency_stop.write().await = false;
        println!("✅ Emergency stop reset");
        Ok(())
    }
    
    // Enable/disable maintenance mode
    pub async fn set_maintenance_mode(&self, enabled: bool) -> Result<()> {
        *self.maintenance_mode.write().await = enabled;
        
        if enabled {
            println!("🔧 MAINTENANCE MODE ENABLED");
            *self.execution_enabled.write().await = false;
        } else {
            println!("✅ Maintenance mode disabled");
            *self.execution_enabled.write().await = true;
        }
        
        Ok(())
    }
    
    // Execute control logic
    pub async fn execute_logic(&self, logic_id: &str) -> Result<String> {
        // Check if execution is enabled
        if !*self.execution_enabled.read().await {
            return Err(anyhow!("Logic execution is disabled"));
        }
        
        let mut files = self.logic_files.write().await;
        let logic = files.get_mut(logic_id)
            .ok_or_else(|| anyhow!("Logic file not found"))?;
        
        if !logic.is_active {
            return Err(anyhow!("Logic file is not active"));
        }
        
        // Simulate logic execution (in production, would use embedded JS engine)
        logic.last_execution = Some(Utc::now());
        logic.execution_count += 1;
        logic.next_execution = Utc::now() + chrono::Duration::seconds(logic.execution_interval as i64);
        
        // Example control logic for cooling tower
        if logic_id == "cooling-tower-001" {
            self.execute_cooling_tower_logic().await?;
        }
        
        Ok(format!("Logic {} executed successfully", logic_id))
    }
    
    // Cooling tower control logic
    async fn execute_cooling_tower_logic(&self) -> Result<()> {
        // Get control points
        let mut points = self.control_points.write().await;
        
        // Get or create condenser water temp control point
        let cwt = points.entry("condenser_water_temp".to_string()).or_insert(ControlPoint {
            name: "Condenser Water Temp".to_string(),
            value: 85.0,
            setpoint: 85.0,
            deadband: 2.0,
            control_mode: ControlMode::Auto,
            output: 50.0,
            min_output: 20.0,
            max_output: 100.0,
            rate_limit: 5.0, // 5% per second max change
        });
        
        // PID control logic
        let error = cwt.setpoint - cwt.value;
        
        if cwt.control_mode == ControlMode::Auto {
            if error > cwt.deadband {
                // Too warm - increase fan speed
                cwt.output = (cwt.output + cwt.rate_limit).min(cwt.max_output);
            } else if error < -cwt.deadband {
                // Too cold - decrease fan speed
                cwt.output = (cwt.output - cwt.rate_limit).max(cwt.min_output);
            }
            
            println!("Cooling Tower: Temp={:.1}°F, SP={:.1}°F, Output={:.1}%", 
                cwt.value, cwt.setpoint, cwt.output);
        }
        
        Ok(())
    }
    
    // Get all logic files
    pub async fn list_logic_files(&self) -> Vec<LogicFile> {
        self.logic_files.read().await.values().cloned().collect()
    }
    
    // Add new logic file
    pub async fn add_logic_file(&self, logic: LogicFile) -> Result<()> {
        let mut files = self.logic_files.write().await;
        files.insert(logic.id.clone(), logic);
        Ok(())
    }
    
    // Remove logic file
    pub async fn remove_logic_file(&self, logic_id: &str) -> Result<()> {
        let mut files = self.logic_files.write().await;
        files.remove(logic_id)
            .ok_or_else(|| anyhow!("Logic file not found"))?;
        Ok(())
    }
    
    // Get equipment states
    pub async fn get_equipment_states(&self) -> HashMap<String, EquipmentState> {
        self.equipment_states.read().await.clone()
    }
    
    // Get control points
    pub async fn get_control_points(&self) -> HashMap<String, ControlPoint> {
        self.control_points.read().await.clone()
    }
    
    // Set control point value (for manual override)
    pub async fn set_control_point(&self, name: &str, value: f32) -> Result<()> {
        let mut points = self.control_points.write().await;
        if let Some(point) = points.get_mut(name) {
            point.value = value;
            Ok(())
        } else {
            Err(anyhow!("Control point not found"))
        }
    }
    
    // Set control mode
    pub async fn set_control_mode(&self, name: &str, mode: ControlMode) -> Result<()> {
        let mut points = self.control_points.write().await;
        if let Some(point) = points.get_mut(name) {
            point.control_mode = mode;
            Ok(())
        } else {
            Err(anyhow!("Control point not found"))
        }
    }
}