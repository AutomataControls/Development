use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;
use chrono::{DateTime, Utc};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LogicFile {
    pub id: String,
    pub name: String,
    pub file_path: String,
    pub equipment_type: String,
    pub location_id: String,
    pub equipment_id: String,
    pub description: String,
    pub last_modified: DateTime<Utc>,
    pub is_active: bool,
    pub execution_interval: u32, // seconds
    pub last_execution: Option<DateTime<Utc>>,
    pub execution_count: u64,
    pub last_error: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LogicExecution {
    pub logic_id: String,
    pub timestamp: DateTime<Utc>,
    pub inputs: LogicInputs,
    pub outputs: LogicOutputs,
    pub execution_time_ms: u64,
    pub success: bool,
    pub error_message: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LogicInputs {
    pub metrics: HashMap<String, f64>,
    pub settings: HashMap<String, serde_json::Value>,
    pub current_temp: f64,
    pub state_storage: HashMap<String, serde_json::Value>,
    pub board_io: HashMap<String, serde_json::Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LogicOutputs {
    pub heating_valve_position: Option<f64>,
    pub cooling_valve_position: Option<f64>,
    pub fan_enabled: Option<bool>,
    pub fan_vfd_speed: Option<f64>,
    pub outdoor_damper_position: Option<f64>,
    pub supply_air_temp_setpoint: Option<f64>,
    pub temperature_setpoint: Option<f64>,
    pub unit_enable: Option<bool>,
    pub is_occupied: Option<bool>,
    pub analog_outputs: HashMap<u8, f64>,
    pub relay_states: HashMap<u8, bool>,
    pub triac_states: HashMap<u8, bool>,
    pub custom_outputs: HashMap<String, serde_json::Value>,
}

pub struct LogicEngine {
    pub logic_files: HashMap<String, LogicFile>,
    pub execution_history: Vec<LogicExecution>,
    pub state_storage: HashMap<String, HashMap<String, serde_json::Value>>,
}

impl LogicEngine {
    pub fn new() -> Self {
        Self {
            logic_files: HashMap::new(),
            execution_history: Vec::new(),
            state_storage: HashMap::new(),
        }
    }

    pub fn load_logic_file(&mut self, file_path: &str) -> Result<String, String> {
        let path = Path::new(file_path);
        if !path.exists() {
            return Err("Logic file does not exist".to_string());
        }

        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read logic file: {}", e))?;

        // Parse logic file metadata from comments
        let metadata = self.parse_logic_metadata(&content)?;
        
        let logic_id = uuid::Uuid::new_v4().to_string();
        let logic_file = LogicFile {
            id: logic_id.clone(),
            name: metadata.get("name").unwrap_or(&path.file_name().unwrap().to_string_lossy().to_string()).clone(),
            file_path: file_path.to_string(),
            equipment_type: metadata.get("equipment_type").unwrap_or(&"unknown".to_string()).clone(),
            location_id: metadata.get("location_id").unwrap_or(&"0".to_string()).clone(),
            equipment_id: metadata.get("equipment_id").unwrap_or(&"unknown".to_string()).clone(),
            description: metadata.get("description").unwrap_or(&"No description".to_string()).clone(),
            last_modified: Utc::now(),
            is_active: false,
            execution_interval: metadata.get("execution_interval")
                .and_then(|s| s.parse().ok())
                .unwrap_or(30),
            last_execution: None,
            execution_count: 0,
            last_error: None,
        };

        self.logic_files.insert(logic_id.clone(), logic_file);
        Ok(logic_id)
    }

    fn parse_logic_metadata(&self, content: &str) -> Result<HashMap<String, String>, String> {
        let mut metadata = HashMap::new();
        
        // Extract metadata from comments
        for line in content.lines().take(50) { // Check first 50 lines
            let line = line.trim();
            if line.starts_with("//") {
                let comment = line.trim_start_matches("//").trim();
                
                // Look for key-value patterns
                if comment.contains("Equipment ID:") {
                    if let Some(id) = comment.split("Equipment ID:").nth(1) {
                        metadata.insert("equipment_id".to_string(), id.trim().to_string());
                    }
                } else if comment.contains("Location ID:") {
                    if let Some(id) = comment.split("Location ID:").nth(1) {
                        metadata.insert("location_id".to_string(), id.trim().to_string());
                    }
                } else if comment.contains("Primary Equipment:") {
                    if let Some(eq_type) = comment.split("Primary Equipment:").nth(1) {
                        metadata.insert("equipment_type".to_string(), eq_type.trim().to_string());
                    }
                } else if comment.contains("OVERVIEW:") {
                    // Next few lines might contain description
                    metadata.insert("description".to_string(), comment.to_string());
                }
            }
        }

        // Try to extract function name for equipment type
        if let Some(func_line) = content.lines().find(|line| line.contains("function") && line.contains("Control")) {
            if let Some(func_name) = func_line.split("function").nth(1) {
                if let Some(name) = func_name.split("(").next() {
                    let clean_name = name.trim().replace("Control", "").replace("control", "");
                    metadata.insert("equipment_type".to_string(), clean_name);
                }
            }
        }

        Ok(metadata)
    }

    pub async fn execute_logic(
        &mut self,
        logic_id: &str,
        inputs: LogicInputs,
    ) -> Result<LogicOutputs, String> {
        // Check if logic file exists and is active
        let (is_active, location_id, equipment_id, logic_content) = {
            let logic_file = self.logic_files.get(logic_id)
                .ok_or("Logic file not found")?;
            
            if !logic_file.is_active {
                return Err("Logic file is not active".to_string());
            }
            
            // Read the file content from disk
            let content = fs::read_to_string(&logic_file.file_path)
                .map_err(|e| format!("Failed to read logic file: {}", e))?;
            
            (logic_file.is_active, 
             logic_file.location_id.clone(), 
             logic_file.equipment_id.clone(),
             content)
        };

        let start_time = std::time::Instant::now();
        
        // Get or create state storage for this logic
        let state_key = format!("{}_{}", location_id, equipment_id);
        if !self.state_storage.contains_key(&state_key) {
            self.state_storage.insert(state_key.clone(), HashMap::new());
        }

        // Create a temporary logic file struct for execution
        let temp_logic_file = LogicFile {
            id: logic_id.to_string(),
            name: String::new(),
            file_path: self.logic_files.get(logic_id)
                .ok_or("Logic file not found")?
                .file_path.clone(),
            equipment_type: String::new(),
            location_id: location_id.clone(),
            equipment_id: equipment_id.clone(),
            description: String::new(),
            last_modified: Utc::now(),
            is_active,
            execution_interval: 30,
            last_execution: None,
            execution_count: 0,
            last_error: None,
        };
        
        // Execute the JavaScript logic using Node.js
        let result = self.execute_javascript_logic(&temp_logic_file, &inputs).await;
        
        let execution_time = start_time.elapsed().as_millis() as u64;
        
        // Update logic file metrics
        if let Some(logic_file) = self.logic_files.get_mut(logic_id) {
            logic_file.execution_count += 1;
            logic_file.last_execution = Some(Utc::now());
            
            match &result {
                Ok(_) => {
                    logic_file.last_error = None;
                }
                Err(e) => {
                    logic_file.last_error = Some(e.clone());
                }
            }
        }

        match result {
            Ok(outputs) => {
                
                let execution = LogicExecution {
                    logic_id: logic_id.to_string(),
                    timestamp: Utc::now(),
                    inputs,
                    outputs: outputs.clone(),
                    execution_time_ms: execution_time,
                    success: true,
                    error_message: None,
                };

                self.execution_history.push(execution);
                
                // Keep only last 1000 executions
                if self.execution_history.len() > 1000 {
                    self.execution_history.remove(0);
                }

                Ok(outputs)
            }
            Err(error) => {
                // Update the logic file's last_error in the hashmap
                if let Some(logic_file) = self.logic_files.get_mut(logic_id) {
                    logic_file.last_error = Some(error.clone());
                }
                
                let execution = LogicExecution {
                    logic_id: logic_id.to_string(),
                    timestamp: Utc::now(),
                    inputs,
                    outputs: LogicOutputs::default(),
                    execution_time_ms: execution_time,
                    success: false,
                    error_message: Some(error.clone()),
                };

                self.execution_history.push(execution);
                Err(error)
            }
        }
    }

    async fn execute_javascript_logic(
        &mut self,
        logic_file: &LogicFile,
        inputs: &LogicInputs,
    ) -> Result<LogicOutputs, String> {
        // Create a temporary wrapper script that loads the logic file and executes it
        let wrapper_script = format!(r#"
const fs = require('fs');
const path = require('path');

// Mock the location logger
const location_logger_1 = {{
    logLocationEquipment: (locationId, equipmentId, type, message, stack) => {{
        console.log(`[LOG] ${{locationId}}/${{equipmentId}} (${{type}}): ${{message}}`);
        if (stack) console.error(stack);
    }}
}};

// Load the logic file
const logicPath = '{}';
const logicContent = fs.readFileSync(logicPath, 'utf8');

// Create a module-like environment
const module = {{ exports: {{}} }};
const exports = module.exports;

// Execute the logic file
eval(logicContent);

// Prepare inputs
const metricsInput = {};
const settingsInput = {};
const currentTempArgument = {};
const stateStorageInput = {};

// Execute the main function
async function runLogic() {{
    try {{
        let result;
        
        // Try different function names
        if (typeof airHandlerControl === 'function') {{
            result = await airHandlerControl(metricsInput, settingsInput, currentTempArgument, stateStorageInput);
        }} else if (typeof module.exports.airHandlerControl === 'function') {{
            result = await module.exports.airHandlerControl(metricsInput, settingsInput, currentTempArgument, stateStorageInput);
        }} else if (typeof exports.airHandlerControl === 'function') {{
            result = await exports.airHandlerControl(metricsInput, settingsInput, currentTempArgument, stateStorageInput);
        }} else if (typeof processEquipment === 'function') {{
            result = await processEquipment(metricsInput, settingsInput, currentTempArgument, stateStorageInput);
        }} else if (typeof runLogic === 'function') {{
            result = await runLogic(metricsInput, settingsInput, currentTempArgument, stateStorageInput);
        }} else {{
            throw new Error('No recognized control function found in logic file');
        }}
        
        console.log('LOGIC_OUTPUT:' + JSON.stringify(result));
    }} catch (error) {{
        console.error('LOGIC_ERROR:' + error.message);
        console.error(error.stack);
    }}
}}

runLogic();
"#, 
            logic_file.file_path,
            serde_json::to_string(&inputs.metrics).unwrap_or_default(),
            serde_json::to_string(&inputs.settings).unwrap_or_default(),
            inputs.current_temp,
            serde_json::to_string(&inputs.state_storage).unwrap_or_default()
        );

        // Write wrapper script to temporary file
        let temp_script_path = format!("/tmp/logic_wrapper_{}.js", logic_file.id);
        fs::write(&temp_script_path, wrapper_script)
            .map_err(|e| format!("Failed to write wrapper script: {}", e))?;

        // Execute with Node.js
        let output = Command::new("node")
            .arg(&temp_script_path)
            .output()
            .map_err(|e| format!("Failed to execute Node.js: {}", e))?;

        // Clean up temporary file
        let _ = fs::remove_file(&temp_script_path);

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        // Look for logic output
        if let Some(output_line) = stdout.lines().find(|line| line.starts_with("LOGIC_OUTPUT:")) {
            let json_str = output_line.trim_start_matches("LOGIC_OUTPUT:");
            let result: serde_json::Value = serde_json::from_str(json_str)
                .map_err(|e| format!("Failed to parse logic output: {}", e))?;

            // Convert to LogicOutputs
            let outputs = LogicOutputs {
                heating_valve_position: result.get("heatingValvePosition").and_then(|v| v.as_f64()),
                cooling_valve_position: result.get("coolingValvePosition").and_then(|v| v.as_f64()),
                fan_enabled: result.get("fanEnabled").and_then(|v| v.as_bool()),
                fan_vfd_speed: result.get("fanVFDSpeed").and_then(|v| v.as_f64()),
                outdoor_damper_position: result.get("outdoorDamperPosition").and_then(|v| v.as_f64()),
                supply_air_temp_setpoint: result.get("supplyAirTempSetpoint").and_then(|v| v.as_f64()),
                temperature_setpoint: result.get("temperatureSetpoint").and_then(|v| v.as_f64()),
                unit_enable: result.get("unitEnable").and_then(|v| v.as_bool()),
                is_occupied: result.get("isOccupied").and_then(|v| v.as_bool()),
                analog_outputs: HashMap::new(), // Can be extended
                relay_states: HashMap::new(),   // Can be extended
                triac_states: HashMap::new(),   // Can be extended
                custom_outputs: HashMap::new(), // Can be extended
            };

            return Ok(outputs);
        }

        // Look for errors
        if let Some(error_line) = stderr.lines().find(|line| line.starts_with("LOGIC_ERROR:")) {
            let error_msg = error_line.trim_start_matches("LOGIC_ERROR:");
            return Err(format!("Logic execution error: {}", error_msg));
        }

        if !output.status.success() {
            return Err(format!("Logic execution failed: {}", stderr));
        }

        Err("No output received from logic execution".to_string())
    }

    pub fn get_active_logic_files(&self) -> Vec<&LogicFile> {
        self.logic_files.values().filter(|lf| lf.is_active).collect()
    }

    pub fn get_execution_history(&self, logic_id: Option<&str>) -> Vec<&LogicExecution> {
        match logic_id {
            Some(id) => self.execution_history.iter().filter(|ex| ex.logic_id == id).collect(),
            None => self.execution_history.iter().collect(),
        }
    }
}

impl Default for LogicOutputs {
    fn default() -> Self {
        Self {
            heating_valve_position: None,
            cooling_valve_position: None,
            fan_enabled: None,
            fan_vfd_speed: None,
            outdoor_damper_position: None,
            supply_air_temp_setpoint: None,
            temperature_setpoint: None,
            unit_enable: None,
            is_occupied: None,
            analog_outputs: HashMap::new(),
            relay_states: HashMap::new(),
            triac_states: HashMap::new(),
            custom_outputs: HashMap::new(),
        }
    }
}
