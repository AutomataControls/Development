// Processing Rules Engine - Evaluates conditions and executes actions
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingRule {
    pub id: String,
    pub name: String,
    pub priority: u32,
    pub enabled: bool,
    pub condition: String,  // e.g., "discharge_pressure > 450"
    pub action: String,     // e.g., "set_output('compressor_1', false)"
    pub delay_ms: u32,
    pub retrigger_delay_ms: u32,
    pub last_triggered: Option<chrono::DateTime<chrono::Utc>>,
}

pub struct ProcessingEngine {
    rules: Arc<RwLock<Vec<ProcessingRule>>>,
    values: Arc<RwLock<HashMap<String, f64>>>,
    outputs: Arc<RwLock<HashMap<String, bool>>>,
}

impl ProcessingEngine {
    pub fn new() -> Self {
        let mut engine = Self {
            rules: Arc::new(RwLock::new(Vec::new())),
            values: Arc::new(RwLock::new(HashMap::new())),
            outputs: Arc::new(RwLock::new(HashMap::new())),
        };
        
        // Load default rules
        let rules = engine.rules.clone();
        tokio::spawn(async move {
            let mut rules = rules.write().await;
            
            // Critical safety rules
            rules.push(ProcessingRule {
                id: "high_pressure_cutout".to_string(),
                name: "High Pressure Cutout".to_string(),
                priority: 1,
                enabled: true,
                condition: "discharge_pressure > 450".to_string(),
                action: "set_output('compressor_1', false)".to_string(),
                delay_ms: 0,
                retrigger_delay_ms: 60000,
                last_triggered: None,
            });
            
            rules.push(ProcessingRule {
                id: "low_pressure_cutout".to_string(),
                name: "Low Pressure Cutout".to_string(),
                priority: 2,
                enabled: true,
                condition: "suction_pressure < 20".to_string(),
                action: "set_output('compressor_1', false)".to_string(),
                delay_ms: 5000,
                retrigger_delay_ms: 60000,
                last_triggered: None,
            });
            
            rules.push(ProcessingRule {
                id: "high_temp_cutout".to_string(),
                name: "High Temperature Cutout".to_string(),
                priority: 3,
                enabled: true,
                condition: "discharge_temp > 225".to_string(),
                action: "set_output('compressor_1', false)".to_string(),
                delay_ms: 0,
                retrigger_delay_ms: 60000,
                last_triggered: None,
            });
            
            rules.push(ProcessingRule {
                id: "vibration_alarm".to_string(),
                name: "High Vibration Alarm".to_string(),
                priority: 10,
                enabled: true,
                condition: "vibration_velocity > 7.1".to_string(),
                action: "send_alarm('High Vibration Detected')".to_string(),
                delay_ms: 30000,
                retrigger_delay_ms: 300000,
                last_triggered: None,
            });
            
            // Control rules
            rules.push(ProcessingRule {
                id: "cooling_tower_fan".to_string(),
                name: "Cooling Tower Fan Control".to_string(),
                priority: 20,
                enabled: true,
                condition: "condenser_temp > 95".to_string(),
                action: "set_output('cooling_tower_fan', true)".to_string(),
                delay_ms: 10000,
                retrigger_delay_ms: 0,
                last_triggered: None,
            });
            
            rules.push(ProcessingRule {
                id: "economizer_control".to_string(),
                name: "Economizer Control".to_string(),
                priority: 25,
                enabled: true,
                condition: "outdoor_temp < 55 && outdoor_temp < return_temp - 5".to_string(),
                action: "set_output('economizer_damper', true)".to_string(),
                delay_ms: 30000,
                retrigger_delay_ms: 0,
                last_triggered: None,
            });
        });
        
        engine
    }
    
    // Update sensor value
    pub async fn update_value(&self, name: &str, value: f64) {
        self.values.write().await.insert(name.to_string(), value);
    }
    
    // Evaluate all rules
    pub async fn evaluate_rules(&self) -> Vec<String> {
        let mut triggered = Vec::new();
        let mut rules = self.rules.write().await;
        let values = self.values.read().await;
        let mut outputs = self.outputs.write().await;
        
        // Sort by priority
        rules.sort_by_key(|r| r.priority);
        
        for rule in rules.iter_mut() {
            if !rule.enabled {
                continue;
            }
            
            // Check retrigger delay
            if let Some(last) = rule.last_triggered {
                let elapsed = chrono::Utc::now().signed_duration_since(last).num_milliseconds() as u32;
                if elapsed < rule.retrigger_delay_ms {
                    continue;
                }
            }
            
            // Evaluate condition
            if self.evaluate_condition(&rule.condition, &values) {
                // Execute action
                self.execute_action(&rule.action, &mut outputs);
                
                rule.last_triggered = Some(chrono::Utc::now());
                triggered.push(format!("{}: {}", rule.name, rule.action));
            }
        }
        
        triggered
    }
    
    // Simple expression evaluator
    fn evaluate_condition(&self, condition: &str, values: &HashMap<String, f64>) -> bool {
        // Parse simple conditions like "discharge_pressure > 450"
        let parts: Vec<&str> = condition.split_whitespace().collect();
        
        if parts.len() == 3 {
            let var_name = parts[0];
            let operator = parts[1];
            let threshold = parts[2].parse::<f64>().unwrap_or(0.0);
            
            if let Some(&value) = values.get(var_name) {
                match operator {
                    ">" => return value > threshold,
                    "<" => return value < threshold,
                    ">=" => return value >= threshold,
                    "<=" => return value <= threshold,
                    "==" => return (value - threshold).abs() < 0.001,
                    "!=" => return (value - threshold).abs() >= 0.001,
                    _ => {}
                }
            }
        }
        
        // Handle compound conditions with AND
        if condition.contains(" && ") {
            let parts: Vec<&str> = condition.split(" && ").collect();
            return parts.iter().all(|p| self.evaluate_condition(p, values));
        }
        
        // Handle compound conditions with OR
        if condition.contains(" || ") {
            let parts: Vec<&str> = condition.split(" || ").collect();
            return parts.iter().any(|p| self.evaluate_condition(p, values));
        }
        
        false
    }
    
    // Execute action
    fn execute_action(&self, action: &str, outputs: &mut HashMap<String, bool>) {
        // Parse actions like "set_output('compressor_1', false)"
        if action.starts_with("set_output(") {
            let content = action.trim_start_matches("set_output(").trim_end_matches(")");
            let parts: Vec<&str> = content.split(',').collect();
            
            if parts.len() == 2 {
                let output_name = parts[0].trim().trim_matches('\'').trim_matches('"');
                let value = parts[1].trim() == "true";
                outputs.insert(output_name.to_string(), value);
            }
        }
        // Other action types would be handled here
    }
    
    // Get all rules
    pub async fn get_rules(&self) -> Vec<ProcessingRule> {
        self.rules.read().await.clone()
    }
    
    // Update rule
    pub async fn update_rule(&self, rule: ProcessingRule) {
        let mut rules = self.rules.write().await;
        if let Some(existing) = rules.iter_mut().find(|r| r.id == rule.id) {
            *existing = rule;
        } else {
            rules.push(rule);
        }
    }
    
    // Delete rule
    pub async fn delete_rule(&self, id: &str) {
        self.rules.write().await.retain(|r| r.id != id);
    }
}