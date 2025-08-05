use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogicFile {
    pub id: String,
    pub name: String,
    pub path: String,
    pub enabled: bool,
    pub equipment_type: String,
    pub equipment_id: String,
    pub location_id: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlLoop {
    pub logic_file: LogicFile,
    pub interval_seconds: u64,
    pub last_run: Option<Instant>,
    pub last_result: Option<ControlResult>,
    pub running: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlResult {
    pub timestamp: String,
    pub success: bool,
    pub outputs: HashMap<String, f64>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsInput {
    pub supply_temp: Option<f64>,
    pub return_temp: Option<f64>,
    pub outdoor_temp: Option<f64>,
    pub mixed_air_temp: Option<f64>,
    pub static_pressure: Option<f64>,
    pub custom: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsInput {
    pub equipment_id: String,
    pub location_id: String,
    pub temperature_setpoint: Option<f64>,
    pub custom: HashMap<String, serde_json::Value>,
}

pub struct LogicEngine {
    control_loops: Arc<Mutex<HashMap<String, ControlLoop>>>,
    logic_files: Arc<Mutex<Vec<LogicFile>>>,
    running: Arc<Mutex<bool>>,
}

impl LogicEngine {
    pub fn new() -> Self {
        LogicEngine {
            control_loops: Arc::new(Mutex::new(HashMap::new())),
            logic_files: Arc::new(Mutex::new(Vec::new())),
            running: Arc::new(Mutex::new(false)),
        }
    }
    
    pub fn start(&self) {
        let mut running = self.running.lock().unwrap();
        if *running {
            return; // Already running
        }
        *running = true;
        drop(running);
        
        let control_loops = self.control_loops.clone();
        let running_flag = self.running.clone();
        
        thread::spawn(move || {
            while *running_flag.lock().unwrap() {
                // Run control loops
                let loops_to_run: Vec<String> = {
                    let loops = control_loops.lock().unwrap();
                    loops.iter()
                        .filter(|(_, loop_)| {
                            loop_.logic_file.enabled && 
                            !loop_.running &&
                            loop_.last_run.map_or(true, |last| {
                                last.elapsed().as_secs() >= loop_.interval_seconds
                            })
                        })
                        .map(|(id, _)| id.clone())
                        .collect()
                };
                
                for loop_id in loops_to_run {
                    let control_loops_clone = control_loops.clone();
                    thread::spawn(move || {
                        run_control_loop(&loop_id, control_loops_clone);
                    });
                }
                
                thread::sleep(Duration::from_secs(1));
            }
        });
    }
    
    pub fn stop(&self) {
        *self.running.lock().unwrap() = false;
    }
    
    pub fn add_logic_file(&self, logic_file: LogicFile) -> Result<(), String> {
        // Validate file exists
        if !PathBuf::from(&logic_file.path).exists() {
            return Err("Logic file not found".to_string());
        }
        
        let mut files = self.logic_files.lock().unwrap();
        files.push(logic_file.clone());
        
        // Create control loop
        let control_loop = ControlLoop {
            logic_file,
            interval_seconds: 5, // Default 5 second interval
            last_run: None,
            last_result: None,
            running: false,
        };
        
        let mut loops = self.control_loops.lock().unwrap();
        loops.insert(control_loop.logic_file.id.clone(), control_loop);
        
        Ok(())
    }
    
    pub fn remove_logic_file(&self, id: &str) -> Result<(), String> {
        let mut files = self.logic_files.lock().unwrap();
        files.retain(|f| f.id != id);
        
        let mut loops = self.control_loops.lock().unwrap();
        loops.remove(id);
        
        Ok(())
    }
    
    pub fn set_logic_enabled(&self, id: &str, enabled: bool) -> Result<(), String> {
        let mut loops = self.control_loops.lock().unwrap();
        if let Some(loop_) = loops.get_mut(id) {
            loop_.logic_file.enabled = enabled;
            Ok(())
        } else {
            Err("Logic file not found".to_string())
        }
    }
    
    pub fn get_logic_files(&self) -> Vec<LogicFile> {
        self.logic_files.lock().unwrap().clone()
    }
    
    pub fn get_control_status(&self) -> Vec<ControlLoop> {
        self.control_loops.lock().unwrap().values().cloned().collect()
    }
}

fn run_control_loop(loop_id: &str, control_loops: Arc<Mutex<HashMap<String, ControlLoop>>>) {
    // Mark as running
    {
        let mut loops = control_loops.lock().unwrap();
        if let Some(loop_) = loops.get_mut(loop_id) {
            loop_.running = true;
            loop_.last_run = Some(Instant::now());
        } else {
            return;
        }
    }
    
    // Get loop data
    let (logic_file, equipment_id, location_id) = {
        let loops = control_loops.lock().unwrap();
        if let Some(loop_) = loops.get(loop_id) {
            (
                loop_.logic_file.clone(),
                loop_.logic_file.equipment_id.clone(),
                loop_.logic_file.location_id.clone(),
            )
        } else {
            return;
        }
    };
    
    // Get current metrics from hardware
    let metrics = match get_current_metrics(&equipment_id) {
        Ok(m) => m,
        Err(e) => {
            update_loop_result(loop_id, control_loops.clone(), Err(e));
            return;
        }
    };
    
    // Create settings
    let settings = SettingsInput {
        equipment_id: equipment_id.clone(),
        location_id: location_id.clone(),
        temperature_setpoint: None, // Could be loaded from config
        custom: HashMap::new(),
    };
    
    // Execute logic file
    let result = execute_logic_file(&logic_file.path, metrics, settings);
    
    // Apply control outputs if successful
    if let Ok(ref outputs) = result {
        apply_control_outputs(&equipment_id, outputs);
    }
    
    // Update loop result
    update_loop_result(loop_id, control_loops, result);
}

fn get_current_metrics(equipment_id: &str) -> Result<MetricsInput, String> {
    // Get current readings from MegaBAS
    let output = Command::new("python3")
        .arg("../scripts/megabas_interface.py")
        .arg("status")
        .arg("megabas")
        .arg("0")
        .output()
        .map_err(|e| format!("Failed to get metrics: {}", e))?;
    
    if !output.status.success() {
        return Err("Failed to read hardware status".to_string());
    }
    
    let data: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("Failed to parse status: {}", e))?;
    
    // Extract relevant metrics
    let metrics = MetricsInput {
        supply_temp: data["analog_inputs"]["ch1"]["voltage"]
            .as_f64()
            .map(|v| v * 10.0), // Convert 0-10V to 0-100°F
        return_temp: data["analog_inputs"]["ch2"]["voltage"]
            .as_f64()
            .map(|v| v * 10.0),
        outdoor_temp: data["analog_inputs"]["ch3"]["voltage"]
            .as_f64()
            .map(|v| v * 10.0),
        mixed_air_temp: data["analog_inputs"]["ch4"]["voltage"]
            .as_f64()
            .map(|v| v * 10.0),
        static_pressure: data["analog_inputs"]["ch5"]["voltage"].as_f64(),
        custom: HashMap::new(),
    };
    
    Ok(metrics)
}

fn execute_logic_file(
    path: &str,
    metrics: MetricsInput,
    settings: SettingsInput
) -> Result<HashMap<String, f64>, String> {
    // Create Node.js wrapper script
    let wrapper_script = format!(
        r#"
const {{ runLogic }} = require('{}');

const metrics = {};
const settings = {};
const currentTemp = metrics.supply_temp || 55;
const stateStorage = {{}};

runLogic(metrics, settings, currentTemp, stateStorage)
    .then(result => {{
        console.log(JSON.stringify(result));
    }})
    .catch(err => {{
        console.error(JSON.stringify({{error: err.message}}));
        process.exit(1);
    }});
"#,
        path,
        serde_json::to_string(&metrics).unwrap(),
        serde_json::to_string(&settings).unwrap()
    );
    
    // Execute with Node.js
    let output = Command::new("node")
        .arg("-e")
        .arg(&wrapper_script)
        .output()
        .map_err(|e| format!("Failed to execute logic: {}", e))?;
    
    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Logic execution failed: {}", error));
    }
    
    // Parse result
    let result: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("Failed to parse logic result: {}", e))?;
    
    // Convert to control outputs
    let mut outputs = HashMap::new();
    
    if let Some(heating) = result["heatingValvePosition"].as_f64() {
        outputs.insert("heating_valve".to_string(), heating);
    }
    if let Some(cooling) = result["coolingValvePosition"].as_f64() {
        outputs.insert("cooling_valve".to_string(), cooling);
    }
    if let Some(fan_speed) = result["fanVFDSpeed"].as_f64() {
        outputs.insert("fan_speed".to_string(), fan_speed);
    }
    if let Some(damper) = result["outdoorDamperPosition"].as_f64() {
        outputs.insert("outdoor_damper".to_string(), damper);
    }
    
    Ok(outputs)
}

fn apply_control_outputs(equipment_id: &str, outputs: &HashMap<String, f64>) {
    // Apply heating valve (analog output 1)
    if let Some(heating) = outputs.get("heating_valve") {
        let _ = Command::new("python3")
            .arg("../scripts/megabas_interface.py")
            .arg("set")
            .arg("megabas-analog")
            .arg("0")
            .arg("1")
            .arg(heating.to_string())
            .output();
    }
    
    // Apply cooling valve (analog output 2)
    if let Some(cooling) = outputs.get("cooling_valve") {
        let _ = Command::new("python3")
            .arg("../scripts/megabas_interface.py")
            .arg("set")
            .arg("megabas-analog")
            .arg("0")
            .arg("2")
            .arg(cooling.to_string())
            .output();
    }
    
    // Apply fan speed (analog output 3)
    if let Some(fan_speed) = outputs.get("fan_speed") {
        let _ = Command::new("python3")
            .arg("../scripts/megabas_interface.py")
            .arg("set")
            .arg("megabas-analog")
            .arg("0")
            .arg("3")
            .arg((fan_speed / 10.0).to_string()) // Convert 0-100% to 0-10V
            .output();
    }
    
    // Apply outdoor damper (triac 1 for binary control)
    if let Some(damper) = outputs.get("outdoor_damper") {
        let _ = Command::new("python3")
            .arg("../scripts/megabas_interface.py")
            .arg("set")
            .arg("megabas-triac")
            .arg("0")
            .arg("1")
            .arg(if *damper > 50.0 { "1" } else { "0" })
            .output();
    }
}

fn update_loop_result(
    loop_id: &str,
    control_loops: Arc<Mutex<HashMap<String, ControlLoop>>>,
    result: Result<HashMap<String, f64>, String>
) {
    let mut loops = control_loops.lock().unwrap();
    if let Some(loop_) = loops.get_mut(loop_id) {
        loop_.running = false;
        loop_.last_result = Some(ControlResult {
            timestamp: chrono::Utc::now().to_rfc3339(),
            success: result.is_ok(),
            outputs: result.unwrap_or_default(),
            error: result.err(),
        });
    }
}