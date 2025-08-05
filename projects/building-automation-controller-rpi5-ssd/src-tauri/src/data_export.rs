use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BMSConfig {
    pub enabled: bool,
    pub location_name: String,
    pub system_name: String,
    pub location_id: String,
    pub equipment_id: String,
    pub equipment_type: String,
    pub zone: String,
    pub url: String,
    pub mappings: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingConfig {
    pub enabled: bool,
    pub location_name: String,
    pub system_name: String,
    pub location_id: String,
    pub equipment_id: String,
    pub equipment_type: String,
    pub zone: String,
    pub url: String,
    pub port: u16,
    pub mappings: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportConfigs {
    pub bms: Option<BMSConfig>,
    pub processing: Option<ProcessingConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingEngineCommand {
    pub timestamp: String,
    pub equipment_id: String,
    pub location_id: String,
    pub command_type: String,
    pub command_data: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct CommandFetcher {
    last_fetch: Option<Instant>,
    cached_commands: Vec<ProcessingEngineCommand>,
    cache_duration: Duration,
}

fn get_config_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("building-automation-controller");
    path.push("export-config.json");
    path
}

#[tauri::command]
pub async fn save_bms_config(config: BMSConfig) -> Result<(), String> {
    let path = get_config_path();
    
    // Create directory if it doesn't exist
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create config directory: {}", e))?;
    }
    
    // Load existing configs
    let mut configs = get_export_configs().await.unwrap_or_default();
    configs.bms = Some(config);
    
    // Save to file
    let json = serde_json::to_string_pretty(&configs)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    
    fs::write(&path, json)
        .map_err(|e| format!("Failed to write config: {}", e))?;
    
    Ok(())
}

#[tauri::command]
pub async fn save_processing_config(config: ProcessingConfig) -> Result<(), String> {
    let path = get_config_path();
    
    // Create directory if it doesn't exist
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create config directory: {}", e))?;
    }
    
    // Load existing configs
    let mut configs = get_export_configs().await.unwrap_or_default();
    configs.processing = Some(config);
    
    // Save to file
    let json = serde_json::to_string_pretty(&configs)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    
    fs::write(&path, json)
        .map_err(|e| format!("Failed to write config: {}", e))?;
    
    Ok(())
}

#[tauri::command]
pub async fn get_export_configs() -> Result<ExportConfigs, String> {
    let path = get_config_path();
    
    if !path.exists() {
        return Ok(ExportConfigs {
            bms: None,
            processing: None,
        });
    }
    
    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read config: {}", e))?;
    
    let configs: ExportConfigs = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse config: {}", e))?;
    
    Ok(configs)
}

#[tauri::command]
pub async fn send_metrics(
    metric_type: String,
    data: HashMap<String, serde_json::Value>
) -> Result<(), String> {
    let configs = get_export_configs().await?;
    
    match metric_type.as_str() {
        "bms" => {
            if let Some(config) = configs.bms {
                if config.enabled {
                    send_bms_metrics(config, data).await?;
                }
            }
        }
        "processing" => {
            if let Some(config) = configs.processing {
                if config.enabled {
                    send_processing_metrics(config, data).await?;
                }
            }
        }
        _ => return Err("Unknown metric type".to_string()),
    }
    
    Ok(())
}

async fn send_bms_metrics(
    config: BMSConfig,
    data: HashMap<String, serde_json::Value>
) -> Result<(), String> {
    // Build line protocol
    let mut line = format!(
        "metrics,location={},system={},equipment_type={},location_id={},equipmentId={},zone={} ",
        config.location_name,
        config.system_name,
        config.equipment_type,
        config.location_id,
        config.equipment_id,
        config.zone
    );
    
    // Add mapped values
    let mut values = Vec::new();
    for (key, mapping) in &config.mappings {
        if let Some(value) = data.get(mapping) {
            values.push(format!("{}={}", key, value));
        }
    }
    
    line.push_str(&values.join(","));
    
    // Send to InfluxDB
    let client = reqwest::Client::new();
    let response = client
        .post(&config.url)
        .header("Content-Type", "text/plain")
        .body(line)
        .send()
        .await
        .map_err(|e| format!("Failed to send BMS metrics: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("BMS metrics failed: {}", response.status()));
    }
    
    Ok(())
}

async fn send_processing_metrics(
    config: ProcessingConfig,
    data: HashMap<String, serde_json::Value>
) -> Result<(), String> {
    // Build JSON payload
    let mut payload_data = serde_json::json!({
        "location_id": config.location_id.parse::<i32>().unwrap_or(0),
        "system": config.system_name,
        "location": config.location_name,
        "command_type": "metrics",
        "source": "BuildingAutomation",
        "zone": config.zone,
    });
    
    // Add mapped values
    if let Some(obj) = payload_data.as_object_mut() {
        for (key, mapping) in &config.mappings {
            if let Some(value) = data.get(mapping) {
                obj.insert(key.clone(), value.clone());
            }
        }
    }
    
    let payload = serde_json::json!({
        "equipmentId": config.equipment_id,
        "equipmentType": config.equipment_type,
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "data": payload_data
    });
    
    // Send to processing endpoint
    let url = format!("{}:{}/validate", config.url, config.port);
    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&vec![payload])
        .timeout(std::time::Duration::from_secs(8))
        .send()
        .await
        .map_err(|e| format!("Failed to send processing metrics: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("Processing metrics failed: {}", response.status()));
    }
    
    Ok(())
}

impl CommandFetcher {
    pub fn new() -> Self {
        CommandFetcher {
            last_fetch: None,
            cached_commands: Vec::new(),
            cache_duration: Duration::from_secs(30), // Cache for 30 seconds
        }
    }
    
    pub async fn fetch_commands(
        &mut self,
        equipment_id: &str,
        location_id: &str,
        bms_config: &BMSConfig,
    ) -> Result<Vec<ProcessingEngineCommand>, String> {
        // Check cache
        if let Some(last_fetch) = self.last_fetch {
            if last_fetch.elapsed() < self.cache_duration && !self.cached_commands.is_empty() {
                return Ok(self.cached_commands.clone());
            }
        }
        
        // Build SQL query
        let query = format!(
            r#"
            SELECT *
            FROM "ProcessingEngineCommands"
            WHERE equipment_id = '{}'
              AND location_id = '{}'
              AND time >= now() - INTERVAL '5 minutes'
            ORDER BY time DESC
            LIMIT 35
            "#,
            equipment_id, location_id
        );
        
        // Send request to InfluxDB
        let client = reqwest::Client::new();
        let response = client
            .post("http://143.198.162.31:8205/api/v3/query_sql")
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .json(&serde_json::json!({
                "q": query,
                "db": "AggregatedProcessingEngineCommands"
            }))
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| format!("Failed to fetch commands: {}", e))?;
        
        if !response.status().is_success() {
            return Err(format!("Server returned error: {}", response.status()));
        }
        
        let result: serde_json::Value = response.json().await
            .map_err(|e| format!("Failed to parse response: {}", e))?;
        
        // Parse commands from response
        let commands = parse_influx_commands(result)?;
        
        // Update cache
        self.cached_commands = commands.clone();
        self.last_fetch = Some(Instant::now());
        
        Ok(commands)
    }
}

fn parse_influx_commands(result: serde_json::Value) -> Result<Vec<ProcessingEngineCommand>, String> {
    let mut commands = Vec::new();
    
    if let Some(rows) = result.get("rows").and_then(|r| r.as_array()) {
        for row in rows {
            if let Some(obj) = row.as_object() {
                let command = ProcessingEngineCommand {
                    timestamp: obj.get("time")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    equipment_id: obj.get("equipment_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    location_id: obj.get("location_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    command_type: obj.get("command_type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    command_data: obj.get("command_data")
                        .unwrap_or(&serde_json::Value::Null)
                        .clone(),
                };
                commands.push(command);
            }
        }
    }
    
    Ok(commands)
}

#[tauri::command]
pub async fn fetch_processing_commands(
    equipment_id: String,
    location_id: String,
) -> Result<Vec<ProcessingEngineCommand>, String> {
    // Get BMS config to check if connected
    let configs = get_export_configs().await?;
    
    if let Some(bms_config) = configs.bms {
        if bms_config.enabled {
            // Try to fetch from server
            let mut fetcher = CommandFetcher::new();
            match fetcher.fetch_commands(&equipment_id, &location_id, &bms_config).await {
                Ok(commands) => return Ok(commands),
                Err(e) => {
                    // Log error but continue to fallback
                    eprintln!("Failed to fetch commands from server: {}", e);
                }
            }
        }
    }
    
    // Fallback to local logic file
    // This will be handled by the logic engine
    Err("BMS not connected, use local logic file".to_string())
}

#[tauri::command]
pub async fn test_connection(endpoint_type: String) -> Result<String, String> {
    let configs = get_export_configs().await?;
    
    match endpoint_type.as_str() {
        "bms" => {
            if let Some(config) = configs.bms {
                // Test with empty metrics
                let test_line = format!(
                    "test,location={},system={} value=1",
                    config.location_name,
                    config.system_name
                );
                
                let client = reqwest::Client::new();
                let response = client
                    .post(&config.url)
                    .header("Content-Type", "text/plain")
                    .body(test_line)
                    .send()
                    .await
                    .map_err(|e| format!("Connection failed: {}", e))?;
                
                if response.status().is_success() {
                    Ok("BMS connection successful".to_string())
                } else {
                    Err(format!("BMS returned status: {}", response.status()))
                }
            } else {
                Err("BMS not configured".to_string())
            }
        }
        "processing" => {
            if let Some(config) = configs.processing {
                let url = format!("{}:{}/validate", config.url, config.port);
                
                let test_payload = vec![serde_json::json!({
                    "equipmentId": "test",
                    "equipmentType": "test",
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                    "data": {
                        "test": true
                    }
                })];
                
                let client = reqwest::Client::new();
                let response = client
                    .post(&url)
                    .header("Content-Type", "application/json")
                    .json(&test_payload)
                    .timeout(std::time::Duration::from_secs(5))
                    .send()
                    .await
                    .map_err(|e| format!("Connection failed: {}", e))?;
                
                if response.status().is_success() {
                    Ok("Processing connection successful".to_string())
                } else {
                    Err(format!("Processing returned status: {}", response.status()))
                }
            } else {
                Err("Processing not configured".to_string())
            }
        }
        _ => Err("Unknown endpoint type".to_string()),
    }
}