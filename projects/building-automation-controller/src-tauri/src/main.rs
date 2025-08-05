#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod logic_engine;
mod metrics_db;

use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::collections::HashMap;
use std::process::Command;
use chrono::{DateTime, Utc};
use logic_engine::{LogicEngine, LogicFile, LogicInputs, LogicOutputs, LogicExecution};
use metrics_db::{MetricsDatabase, MetricQuery, TrendData};

struct AppState {
    boards: Mutex<HashMap<String, BoardInfo>>,
    io_states: Mutex<HashMap<String, IoState>>,
    bms_configs: Mutex<HashMap<String, BmsConfig>>,
    processing_configs: Mutex<HashMap<String, ProcessingConfig>>,
    logic_engine: Mutex<LogicEngine>,
    bms_connection_status: Mutex<BmsConnectionStatus>,
    maintenance_mode: Mutex<MaintenanceMode>,
    metrics_db: Mutex<Option<MetricsDatabase>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct BoardInfo {
    board_type: String,
    stack_level: u8,
    firmware_version: String,
    status: String,
    capabilities: BoardCapabilities,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct BoardCapabilities {
    analog_inputs: u8,
    analog_outputs: u8,
    digital_inputs: u8,
    digital_outputs: u8,
    relays: u8,
    triacs: u8,
    has_rtc: bool,
    has_watchdog: bool,
    has_1wire: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct IoState {
    board_id: String,
    analog_inputs: Vec<f64>,
    analog_outputs: Vec<f64>,
    digital_inputs: Vec<bool>,
    digital_outputs: Vec<bool>,
    relay_states: Vec<bool>,
    triac_states: Vec<bool>,
    timestamp: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct BmsConfig {
    enabled: bool,
    location_name: String,
    system_name: String,
    location_id: String,
    equipment_id: String,
    equipment_type: String,
    zone: String,
    influx_url: String,
    update_interval: u32,
    field_mappings: HashMap<String, String>,
    // Add BMS command server config
    bms_server_url: String,
    command_query_interval: u32,
    fallback_to_local: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct ProcessingConfig {
    enabled: bool,
    location_name: String,
    system_name: String,
    location_id: String,
    equipment_id: String,
    equipment_type: String,
    zone: String,
    validation_url: String,
    location_port: u16,
    timeout: u32,
    retry_count: u8,
    field_mappings: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct FirmwareRepo {
    name: String,
    display_name: String,
    repo_url: String,
    local_path: String,
    update_command: String,
    is_cloned: bool,
    last_updated: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct BmsConnectionStatus {
    connected: bool,
    last_successful_query: Option<DateTime<Utc>>,
    last_error: Option<String>,
    command_source: String, // "bms" or "local"
    retry_count: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct BmsCommand {
    equipment_id: String,
    location_id: String,
    command_type: String,
    command_data: serde_json::Value,
    timestamp: DateTime<Utc>,
    priority: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct InfluxDbResponse {
    series: Option<Vec<InfluxDbSeries>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct InfluxDbSeries {
    name: String,
    columns: Vec<String>,
    values: Vec<Vec<serde_json::Value>>,
}

impl Default for BmsConnectionStatus {
    fn default() -> Self {
        Self {
            connected: false,
            last_successful_query: None,
            last_error: None,
            command_source: "local".to_string(),
            retry_count: 0,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct MaintenanceMode {
    enabled: bool,
    started_at: Option<DateTime<Utc>>,
    duration_minutes: u32,
    reason: String,
    authorized_by: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct ChannelConfig {
    id: String,
    name: String,
    description: String,
    sensor_type: String,
    input_type: Option<String>,
    scaling_min: f64,
    scaling_max: f64,
    units: String,
    enabled: bool,
    alarm_high: Option<f64>,
    alarm_low: Option<f64>,
    calibration_offset: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct BoardConfig {
    board_id: String,
    board_name: String,
    location: String,
    universal_inputs: Vec<ChannelConfig>,
    analog_outputs: Vec<ChannelConfig>,
    relay_outputs: Vec<ChannelConfig>,
    triac_outputs: Vec<ChannelConfig>,
}

impl Default for MaintenanceMode {
    fn default() -> Self {
        Self {
            enabled: false,
            started_at: None,
            duration_minutes: 120, // 2 hours default
            reason: String::new(),
            authorized_by: String::new(),
        }
    }
}

impl AppState {
    async fn new() -> Self {
        // Initialize metrics database
        let db_path = dirs::data_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("automata-nexus")
            .join("metrics.db");
        
        let metrics_db = match MetricsDatabase::new(db_path).await {
            Ok(db) => {
                println!("Metrics database initialized successfully");
                
                // Start cleanup task
                let db_clone = db.clone();
                tokio::spawn(async move {
                    loop {
                        tokio::time::sleep(tokio::time::Duration::from_secs(86400)).await; // 24 hours
                        match db_clone.cleanup_old_data().await {
                            Ok(deleted) => println!("Database cleanup: deleted {} old records", deleted),
                            Err(e) => eprintln!("Database cleanup error: {}", e),
                        }
                    }
                });
                
                Some(db)
            }
            Err(e) => {
                eprintln!("Failed to initialize metrics database: {}", e);
                None
            }
        };

        Self {
            boards: Mutex::new(HashMap::new()),
            io_states: Mutex::new(HashMap::new()),
            bms_configs: Mutex::new(HashMap::new()),
            processing_configs: Mutex::new(HashMap::new()),
            logic_engine: Mutex::new(LogicEngine::new()),
            bms_connection_status: Mutex::new(BmsConnectionStatus::default()),
            maintenance_mode: Mutex::new(MaintenanceMode::default()),
            metrics_db: Mutex::new(metrics_db),
        }
    }
}

// Tauri commands
#[tauri::command]
async fn scan_boards(state: tauri::State<'_, AppState>) -> Result<Vec<BoardInfo>, String> {
    let mut boards = Vec::new();
    let mut board_map = HashMap::new();

    // Scan for MegaBAS boards (stack 0-7)
    for stack in 0..8 {
        if let Ok(version) = get_megabas_version(stack).await {
            let board = BoardInfo {
                board_type: "SM-I-002 Building Automation".to_string(),
                stack_level: stack,
                firmware_version: version,
                status: "Connected".to_string(),
                capabilities: BoardCapabilities {
                    analog_inputs: 8,
                    analog_outputs: 4,
                    digital_inputs: 4,
                    digital_outputs: 0,
                    relays: 0,
                    triacs: 4,
                    has_rtc: true,
                    has_watchdog: true,
                    has_1wire: true,
                },
            };
            board_map.insert(format!("megabas_{}", stack), board.clone());
            boards.push(board);
        }
    }

    // Check for 8-relay board (only stack 1 as specified)
    if let Ok(_) = check_8relay_board(1).await {
        let board = BoardInfo {
            board_type: "SM8relind 8-Relay".to_string(),
            stack_level: 1,
            firmware_version: "1.0".to_string(),
            status: "Connected".to_string(),
            capabilities: BoardCapabilities {
                analog_inputs: 0,
                analog_outputs: 0,
                digital_inputs: 0,
                digital_outputs: 0,
                relays: 8,
                triacs: 0,
                has_rtc: false,
                has_watchdog: false,
                has_1wire: false,
            },
        };
        board_map.insert("8relay_1".to_string(), board.clone());
        boards.push(board);
    }

    // Scan for 16-relay boards (stack 0-7)
    for stack in 0..8 {
        if let Ok(_) = check_16relay_board(stack).await {
            let board = BoardInfo {
                board_type: "SM16relind 16-Relay".to_string(),
                stack_level: stack,
                firmware_version: "1.0".to_string(),
                status: "Connected".to_string(),
                capabilities: BoardCapabilities {
                    analog_inputs: 0,
                    analog_outputs: 0,
                    digital_inputs: 0,
                    digital_outputs: 0,
                    relays: 16,
                    triacs: 0,
                    has_rtc: false,
                    has_watchdog: true,
                    has_1wire: false,
                },
            };
            board_map.insert(format!("16relay_{}", stack), board.clone());
            boards.push(board);
        }
    }

    // Scan for 16 universal input boards (stack 0-7)
    for stack in 0..8 {
        if let Ok(version) = check_16univin_board(stack).await {
            let board = BoardInfo {
                board_type: "SM16univin 16 Universal Input".to_string(),
                stack_level: stack,
                firmware_version: version,
                status: "Connected".to_string(),
                capabilities: BoardCapabilities {
                    analog_inputs: 16,
                    analog_outputs: 0,
                    digital_inputs: 16,
                    digital_outputs: 0,
                    relays: 0,
                    triacs: 0,
                    has_rtc: true,
                    has_watchdog: true,
                    has_1wire: false,
                },
            };
            board_map.insert(format!("16univin_{}", stack), board.clone());
            boards.push(board);
        }
    }

    // Scan for 16 analog output boards (stack 0-7)
    for stack in 0..8 {
        if let Ok(version) = check_16uout_board(stack).await {
            let board = BoardInfo {
                board_type: "SM16uout 16 Analog Output".to_string(),
                stack_level: stack,
                firmware_version: version,
                status: "Connected".to_string(),
                capabilities: BoardCapabilities {
                    analog_inputs: 0,
                    analog_outputs: 16,
                    digital_inputs: 0,
                    digital_outputs: 0,
                    relays: 0,
                    triacs: 0,
                    has_rtc: false,
                    has_watchdog: true,
                    has_1wire: false,
                },
            };
            board_map.insert(format!("16uout_{}", stack), board.clone());
            boards.push(board);
        }
    }

    // Update state
    let mut boards_state = state.boards.lock().unwrap();
    *boards_state = board_map;

    Ok(boards)
}

#[tauri::command]
async fn read_board_io(
    state: tauri::State<'_, AppState>,
    board_id: String,
) -> Result<IoState, String> {
    let board = {
        let boards = state.boards.lock().unwrap();
        boards.get(&board_id).cloned().ok_or("Board not found")?
    };

    let mut io_state = IoState {
        board_id: board_id.clone(),
        analog_inputs: Vec::new(),
        analog_outputs: Vec::new(),
        digital_inputs: Vec::new(),
        digital_outputs: Vec::new(),
        relay_states: Vec::new(),
        triac_states: Vec::new(),
        timestamp: Utc::now(),
    };

    match board.board_type.as_str() {
        "SM-I-002 Building Automation" => {
            // Read analog inputs (0-10V)
            for ch in 1..=8 {
                match read_megabas_analog_input(board.stack_level, ch).await {
                    Ok(value) => io_state.analog_inputs.push(value),
                    Err(_) => io_state.analog_inputs.push(0.0),
                }
            }

            // Read analog outputs
            for ch in 1..=4 {
                match read_megabas_analog_output(board.stack_level, ch).await {
                    Ok(value) => io_state.analog_outputs.push(value),
                    Err(_) => io_state.analog_outputs.push(0.0),
                }
            }

            // Read digital inputs
            for ch in 1..=4 {
                match read_megabas_digital_input(board.stack_level, ch).await {
                    Ok(value) => io_state.digital_inputs.push(value),
                    Err(_) => io_state.digital_inputs.push(false),
                }
            }

            // Read triac states
            for ch in 1..=4 {
                match read_megabas_triac(board.stack_level, ch).await {
                    Ok(value) => io_state.triac_states.push(value),
                    Err(_) => io_state.triac_states.push(false),
                }
            }
        }
        "SM16univin 16 Universal Input" => {
            // Read 16 universal inputs
            for ch in 1..=16 {
                match read_16univin_analog(board.stack_level, ch).await {
                    Ok(value) => io_state.analog_inputs.push(value),
                    Err(_) => io_state.analog_inputs.push(0.0),
                }
                match read_16univin_digital(board.stack_level, ch).await {
                    Ok(value) => io_state.digital_inputs.push(value),
                    Err(_) => io_state.digital_inputs.push(false),
                }
            }
        }
        "SM16uout 16 Analog Output" => {
            // Read 16 analog outputs
            for ch in 1..=16 {
                match read_16uout_output(board.stack_level, ch).await {
                    Ok(value) => io_state.analog_outputs.push(value),
                    Err(_) => io_state.analog_outputs.push(0.0),
                }
            }
        }
        "SM8relind 8-Relay" => {
            // Read 8 relay states
            for ch in 1..=8 {
                match read_8relay_state(board.stack_level, ch).await {
                    Ok(value) => io_state.relay_states.push(value),
                    Err(_) => io_state.relay_states.push(false),
                }
            }
        }
        "SM16relind 16-Relay" => {
            // Read 16 relay states
            for ch in 1..=16 {
                match read_16relay_state(board.stack_level, ch).await {
                    Ok(value) => io_state.relay_states.push(value),
                    Err(_) => io_state.relay_states.push(false),
                }
            }
        }
        _ => return Err("Unknown board type".to_string()),
    }

    // Store in state
    let mut io_states = state.io_states.lock().unwrap();
    io_states.insert(board_id, io_state.clone());

    Ok(io_state)
}

#[tauri::command]
async fn set_analog_output(
    board_id: String,
    channel: u8,
    value: f64,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let board = {
        let boards = state.boards.lock().unwrap();
        boards.get(&board_id).cloned().ok_or("Board not found")?
    };

    match board.board_type.as_str() {
        "SM-I-002 Building Automation" => {
            set_megabas_analog_output(board.stack_level, channel, value).await
        }
        "SM16uout 16 Analog Output" => {
            set_16uout_output(board.stack_level, channel, value).await
        }
        _ => Err("Board does not support analog outputs".to_string()),
    }
}

#[tauri::command]
async fn set_relay_state(
    board_id: String,
    channel: u8,
    state_val: bool,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let board = {
        let boards = state.boards.lock().unwrap();
        boards.get(&board_id).cloned().ok_or("Board not found")?
    };

    match board.board_type.as_str() {
        "SM8relind 8-Relay" => {
            set_8relay_state(board.stack_level, channel, state_val).await
        }
        "SM16relind 16-Relay" => {
            set_16relay_state(board.stack_level, channel, state_val).await
        }
        _ => Err("Board does not support relays".to_string()),
    }
}

#[tauri::command]
async fn set_triac_state(
    board_id: String,
    channel: u8,
    state_val: bool,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let board = {
        let boards = state.boards.lock().unwrap();
        boards.get(&board_id).cloned().ok_or("Board not found")?
    };

    match board.board_type.as_str() {
        "SM-I-002 Building Automation" => {
            set_megabas_triac(board.stack_level, channel, state_val).await
        }
        _ => Err("Board does not support triacs".to_string()),
    }
}

#[tauri::command]
async fn save_bms_config(
    board_id: String,
    config: BmsConfig,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let mut bms_configs = state.bms_configs.lock().unwrap();
    bms_configs.insert(board_id, config);
    Ok(())
}

#[tauri::command]
async fn save_processing_config(
    board_id: String,
    config: ProcessingConfig,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let mut processing_configs = state.processing_configs.lock().unwrap();
    processing_configs.insert(board_id, config);
    Ok(())
}

#[tauri::command]
async fn get_bms_config(
    board_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<Option<BmsConfig>, String> {
    let bms_configs = state.bms_configs.lock().unwrap();
    Ok(bms_configs.get(&board_id).cloned())
}

#[tauri::command]
async fn get_processing_config(
    board_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<Option<ProcessingConfig>, String> {
    let processing_configs = state.processing_configs.lock().unwrap();
    Ok(processing_configs.get(&board_id).cloned())
}

// BMS Command Integration
#[tauri::command]
async fn query_bms_commands(
    equipment_id: String,
    location_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<BmsCommand>, String> {
    let (enabled, bms_server_url) = {
        let bms_configs = state.bms_configs.lock().unwrap();
        
        // Find BMS config for this equipment
        let bms_config = bms_configs.values()
            .find(|config| config.equipment_id == equipment_id && config.location_id == location_id)
            .ok_or("BMS configuration not found")?;

        if !bms_config.enabled {
            return Err("BMS integration is disabled".to_string());
        }
        
        (bms_config.enabled, bms_config.bms_server_url.clone())
    };

    // Build the InfluxDB query exactly as specified
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

    // Create the request payload exactly as specified
    let request_body = serde_json::json!({
        "q": query,
        "db": "AggregatedProcessingEngineCommands"
    });

    // Use curl to make the HTTP request (matching the Node-RED template)
    let output = Command::new("curl")
        .arg("-s")
        .arg("-X")
        .arg("POST")
        .arg(&bms_server_url)
        .arg("-H")
        .arg("Content-Type: application/json")
        .arg("-H")
        .arg("Accept: application/json")
        .arg("-d")
        .arg(request_body.to_string())
        .arg("--connect-timeout")
        .arg("10")
        .arg("--max-time")
        .arg("30")
        .output()
        .map_err(|e| format!("Failed to execute curl: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        update_bms_connection_error(&state, format!("HTTP request failed: {}", stderr));
        return Err(format!("HTTP request failed: {}", stderr));
    }

    let response_body = String::from_utf8_lossy(&output.stdout);
    
    // Parse the InfluxDB response
    match serde_json::from_str::<InfluxDbResponse>(&response_body) {
        Ok(influx_response) => {
            // Update connection status
            let mut status = state.bms_connection_status.lock().unwrap();
            status.connected = true;
            status.last_successful_query = Some(Utc::now());
            status.last_error = None;
            status.command_source = "bms".to_string();
            status.retry_count = 0;

            // Parse BMS commands from response
            let commands = parse_influx_commands(influx_response, &equipment_id, &location_id)?;
            println!("Successfully retrieved {} BMS commands", commands.len());
            Ok(commands)
        }
        Err(e) => {
            let error_msg = format!("Failed to parse InfluxDB response: {}", e);
            update_bms_connection_error(&state, error_msg.clone());
            Err(error_msg)
        }
    }
}

#[tauri::command]
async fn get_bms_connection_status(
    state: tauri::State<'_, AppState>,
) -> Result<BmsConnectionStatus, String> {
    let status = state.bms_connection_status.lock().unwrap();
    Ok(status.clone())
}

#[tauri::command]
async fn test_bms_connection(
    equipment_id: String,
    location_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    match query_bms_commands(equipment_id, location_id, state).await {
        Ok(commands) => Ok(format!("BMS connection successful! Retrieved {} commands", commands.len())),
        Err(e) => Err(format!("BMS connection failed: {}", e)),
    }
}

// Logic Engine Commands - Updated to support BMS integration
#[tauri::command]
async fn load_logic_file(
    file_path: String,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let mut logic_engine = state.logic_engine.lock().unwrap();
    logic_engine.load_logic_file(&file_path)
}

#[tauri::command]
async fn get_logic_files(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<LogicFile>, String> {
    let logic_engine = state.logic_engine.lock().unwrap();
    Ok(logic_engine.logic_files.values().cloned().collect())
}

#[tauri::command]
async fn activate_logic_file(
    logic_id: String,
    active: bool,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let mut logic_engine = state.logic_engine.lock().unwrap();
    if let Some(logic_file) = logic_engine.logic_files.get_mut(&logic_id) {
        logic_file.is_active = active;
        Ok(())
    } else {
        Err("Logic file not found".to_string())
    }
}

#[tauri::command]
async fn execute_logic_file(
    logic_id: String,
    board_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<LogicOutputs, String> {
    // Check if maintenance mode is active
    {
        let maintenance = state.maintenance_mode.lock().unwrap();
        if maintenance.enabled {
            if let Some(started_at) = maintenance.started_at {
                let elapsed = Utc::now().signed_duration_since(started_at);
                let duration_minutes = maintenance.duration_minutes as i64;
                
                if elapsed.num_minutes() < duration_minutes {
                    return Err("Maintenance mode is active - logic execution disabled".to_string());
                } else {
                    // Maintenance mode expired, disable it
                    drop(maintenance);
                    let mut maintenance = state.maintenance_mode.lock().unwrap();
                    maintenance.enabled = false;
                    maintenance.started_at = None;
                }
            }
        }
    }
    
    // Get current I/O state
    let (io_state, bms_config) = {
        let io_states = state.io_states.lock().unwrap();
        let io_state = io_states.get(&board_id).cloned();
        
        let bms_configs = state.bms_configs.lock().unwrap();
        let bms_config = bms_configs.get(&board_id).cloned();
        
        (io_state, bms_config)
    };

    // Prepare inputs
    let mut metrics = HashMap::new();
    let mut settings = HashMap::new();
    let mut board_io = HashMap::new();

    if let Some(io) = io_state.as_ref() {
        // Map I/O to common HVAC metrics
        if !io.analog_inputs.is_empty() {
            metrics.insert("SupplyTemp".to_string(), io.analog_inputs.get(0).copied().unwrap_or(0.0) * 10.0);
            metrics.insert("ReturnTemp".to_string(), io.analog_inputs.get(1).copied().unwrap_or(0.0) * 10.0);
            metrics.insert("Outdoor_Air".to_string(), io.analog_inputs.get(2).copied().unwrap_or(0.0) * 10.0);
            metrics.insert("MixedAir".to_string(), io.analog_inputs.get(3).copied().unwrap_or(0.0) * 10.0);
        }

        // Add I/O state to board_io
        board_io.insert("analog_inputs".to_string(), serde_json::to_value(&io.analog_inputs).unwrap());
        board_io.insert("digital_inputs".to_string(), serde_json::to_value(&io.digital_inputs).unwrap());
        board_io.insert("relay_states".to_string(), serde_json::to_value(&io.relay_states).unwrap());
    }

    // Check if we should use BMS commands or local logic
    let use_bms = if let Some(config) = bms_config.as_ref() {
        config.enabled && {
            let status = state.bms_connection_status.lock().unwrap();
            status.connected
        }
    } else {
        false
    };

    if use_bms {
        // Try to get commands from BMS first
        if let Some(config) = bms_config.as_ref() {
            match query_bms_commands(config.equipment_id.clone(), config.location_id.clone(), state.clone()).await {
                Ok(bms_commands) => {
                    // Execute BMS commands instead of local logic
                    return execute_bms_commands(bms_commands, board_id, state.clone()).await;
                }
                Err(e) => {
                    // BMS failed, fall back to local logic if enabled
                    if config.fallback_to_local {
                        println!("BMS command query failed, falling back to local logic: {}", e);
                        // Continue to local logic execution below
                    } else {
                        return Err(format!("BMS command execution failed and fallback disabled: {}", e));
                    }
                }
            }
        }
    }

    // Execute local logic file (fallback or primary)
    settings.insert("equipmentId".to_string(), serde_json::Value::String("default".to_string()));
    settings.insert("locationId".to_string(), serde_json::Value::String("1".to_string()));

    let inputs = LogicInputs {
        metrics,
        settings,
        current_temp: 70.0,
        state_storage: HashMap::new(),
        board_io,
    };

    // Update command source
    {
        let mut status = state.bms_connection_status.lock().unwrap();
        status.command_source = "local".to_string();
    }

    // Clone logic engine for async execution
    let result = {
        let mut logic_engine = state.logic_engine.lock().unwrap();
        // Use block-on to execute the async method synchronously within the lock
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(
                logic_engine.execute_logic(&logic_id, inputs)
            )
        })
    };
    
    result
}

#[tauri::command]
async fn apply_logic_outputs(
    outputs: LogicOutputs,
    board_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let board = {
        let boards = state.boards.lock().unwrap();
        boards.get(&board_id).cloned().ok_or("Board not found")?
    };

    // Apply analog outputs
    if let Some(heating_valve) = outputs.heating_valve_position {
        if board.capabilities.analog_outputs > 0 {
            set_megabas_analog_output(board.stack_level, 1, heating_valve / 10.0).await?;
        }
    }

    if let Some(cooling_valve) = outputs.cooling_valve_position {
        if board.capabilities.analog_outputs > 1 {
            set_megabas_analog_output(board.stack_level, 2, cooling_valve / 10.0).await?;
        }
    }

    if let Some(damper_pos) = outputs.outdoor_damper_position {
        if board.capabilities.analog_outputs > 2 {
            set_megabas_analog_output(board.stack_level, 3, damper_pos / 10.0).await?;
        }
    }

    // Apply relay states (fan control, etc.)
    if let Some(fan_enabled) = outputs.fan_enabled {
        if board.capabilities.relays > 0 {
            set_8relay_state(board.stack_level, 1, fan_enabled).await?;
        }
    }

    // Apply custom analog outputs
    for (channel, value) in outputs.analog_outputs {
        if channel <= board.capabilities.analog_outputs {
            set_megabas_analog_output(board.stack_level, channel, value).await?;
        }
    }

    // Apply custom relay states
    for (channel, state) in outputs.relay_states {
        if channel <= board.capabilities.relays {
            set_8relay_state(board.stack_level, channel, state).await?;
        }
    }

    // Apply triac states
    for (channel, state) in outputs.triac_states {
        if channel <= board.capabilities.triacs {
            set_megabas_triac(board.stack_level, channel, state).await?;
        }
    }

    Ok(())
}

#[tauri::command]
async fn get_logic_execution_history(
    logic_id: Option<String>,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<LogicExecution>, String> {
    let logic_engine = state.logic_engine.lock().unwrap();
    let history = logic_engine.get_execution_history(logic_id.as_deref());
    Ok(history.into_iter().cloned().collect())
}

#[tauri::command]
async fn delete_logic_file(
    logic_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let mut logic_engine = state.logic_engine.lock().unwrap();
    logic_engine.logic_files.remove(&logic_id);
    Ok(())
}

// Firmware management commands
#[tauri::command]
async fn check_firmware_repos() -> Result<Vec<FirmwareRepo>, String> {
    let repos = vec![
        FirmwareRepo {
            name: "megabas".to_string(),
            display_name: "SM-I-002 Building Automation".to_string(),
            repo_url: "https://github.com/SequentMicrosystems/megabas-rpi.git".to_string(),
            local_path: "/home/Automata/megabas-rpi".to_string(),
            update_command: "./update".to_string(),
            is_cloned: std::path::Path::new("/home/Automata/megabas-rpi").exists(),
            last_updated: None,
        },
        FirmwareRepo {
            name: "16univin".to_string(),
            display_name: "SM16univin 16 Universal Input".to_string(),
            repo_url: "https://github.com/SequentMicrosystems/16univin-rpi.git".to_string(),
            local_path: "/home/Automata/16univin-rpi".to_string(),
            update_command: "./update".to_string(),
            is_cloned: std::path::Path::new("/home/Automata/16univin-rpi").exists(),
            last_updated: None,
        },
        FirmwareRepo {
            name: "8relind".to_string(),
            display_name: "SM8relind 8-Relay".to_string(),
            repo_url: "https://github.com/SequentMicrosystems/8relind-rpi.git".to_string(),
            local_path: "/home/Automata/8relind-rpi".to_string(),
            update_command: "./update".to_string(),
            is_cloned: std::path::Path::new("/home/Automata/8relind-rpi").exists(),
            last_updated: None,
        },
        FirmwareRepo {
            name: "16relind".to_string(),
            display_name: "SM16relind 16-Relay".to_string(),
            repo_url: "https://github.com/SequentMicrosystems/16relind-rpi.git".to_string(),
            local_path: "/home/Automata/16relind-rpi".to_string(),
            update_command: "./update".to_string(),
            is_cloned: std::path::Path::new("/home/Automata/16relind-rpi").exists(),
            last_updated: None,
        },
        FirmwareRepo {
            name: "16uout".to_string(),
            display_name: "SM16uout 16 Analog Output".to_string(),
            repo_url: "https://github.com/SequentMicrosystems/16uout-rpi.git".to_string(),
            local_path: "/home/Automata/16uout-rpi".to_string(),
            update_command: "./update".to_string(),
            is_cloned: std::path::Path::new("/home/Automata/16uout-rpi").exists(),
            last_updated: None,
        },
    ];
    
    Ok(repos)
}

#[tauri::command]
async fn clone_firmware_repo(repo_name: String) -> Result<String, String> {
    let repos = check_firmware_repos().await?;
    let repo = repos.iter().find(|r| r.name == repo_name)
        .ok_or("Repository not found")?;
    
    // Create Automata directory if it doesn't exist
    let _output = Command::new("mkdir")
        .arg("-p")
        .arg("/home/Automata")
        .output()
        .map_err(|e| format!("Failed to create directory: {}", e))?;
    
    // Clone the repository
    let output = Command::new("git")
        .arg("clone")
        .arg(&repo.repo_url)
        .arg(&repo.local_path)
        .current_dir("/home/Automata")
        .output()
        .map_err(|e| format!("Failed to clone repository: {}", e))?;
    
    if output.status.success() {
        Ok(format!("Successfully cloned {}", repo.display_name))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Failed to clone repository: {}", stderr))
    }
}

#[tauri::command]
async fn install_firmware_drivers(repo_name: String) -> Result<String, String> {
    let repos = check_firmware_repos().await?;
    let repo = repos.iter().find(|r| r.name == repo_name)
        .ok_or("Repository not found")?;
    
    if !std::path::Path::new(&repo.local_path).exists() {
        return Err("Repository not cloned. Please clone first.".to_string());
    }
    
    // Run sudo make install
    let output = Command::new("sudo")
        .arg("make")
        .arg("install")
        .current_dir(&repo.local_path)
        .output()
        .map_err(|e| format!("Failed to install drivers: {}", e))?;
    
    if output.status.success() {
        Ok(format!("Successfully installed {} drivers", repo.display_name))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Failed to install drivers: {}", stderr))
    }
}

#[tauri::command]
async fn update_board_firmware(
    repo_name: String, 
    stack_level: u8
) -> Result<String, String> {
    let repos = check_firmware_repos().await?;
    let repo = repos.iter().find(|r| r.name == repo_name)
        .ok_or("Repository not found")?;
    
    let update_path = format!("{}/update", repo.local_path);
    if !std::path::Path::new(&update_path).exists() {
        return Err("Update directory not found. Please install drivers first.".to_string());
    }
    
    // Prepare the update command based on board type
    let mut cmd = Command::new("sudo");
    cmd.arg("./update").arg(stack_level.to_string());
    
    // Add board type parameter for specific boards
    if repo_name == "16univin" {
        cmd.arg("16univin");
    }
    
    cmd.current_dir(&update_path);
    
    // Use expect to handle the interactive prompt
    let output = Command::new("expect")
        .arg("-c")
        .arg(&format!(
            r#"
            spawn sudo ./update {} {}
            expect "Do you want to continue"
            sleep 3
            send "yes\r"
            expect eof
            "#,
            stack_level,
            if repo_name == "16univin" { "16univin" } else { "" }
        ))
        .current_dir(&update_path)
        .output()
        .map_err(|e| format!("Failed to run update command: {}", e))?;
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    if stdout.contains("no cpuid found") {
        Err("No CPU ID found - board may not be connected or compatible".to_string())
    } else if output.status.success() || stdout.contains("Update completed") {
        Ok(format!("Successfully updated {} firmware on stack {}", repo.display_name, stack_level))
    } else {
        Err(format!("Update failed: {}\n{}", stdout, stderr))
    }
}

#[tauri::command]
async fn pull_firmware_updates(repo_name: String) -> Result<String, String> {
    let repos = check_firmware_repos().await?;
    let repo = repos.iter().find(|r| r.name == repo_name)
        .ok_or("Repository not found")?;
    
    if !std::path::Path::new(&repo.local_path).exists() {
        return Err("Repository not cloned. Please clone first.".to_string());
    }
    
    // Pull latest updates
    let output = Command::new("git")
        .arg("pull")
        .arg("origin")
        .arg("master")
        .current_dir(&repo.local_path)
        .output()
        .map_err(|e| format!("Failed to pull updates: {}", e))?;
    
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains("Already up to date") {
            Ok("Repository is already up to date".to_string())
        } else {
            Ok(format!("Successfully pulled updates for {}", repo.display_name))
        }
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Failed to pull updates: {}", stderr))
    }
}

// Helper functions
fn update_bms_connection_error(state: &tauri::State<AppState>, error: String) {
    let mut status = state.bms_connection_status.lock().unwrap();
    status.connected = false;
    status.last_error = Some(error);
    status.command_source = "local".to_string();
    status.retry_count += 1;
}

fn parse_influx_commands(
    response: InfluxDbResponse,
    equipment_id: &str,
    location_id: &str,
) -> Result<Vec<BmsCommand>, String> {
    let mut commands = Vec::new();
    
    if let Some(series) = response.series {
        for serie in series {
            if let Some(_columns) = serie.columns.get(0) {
                // Find column indices
                let mut time_idx = None;
                let mut command_type_idx = None;
                let mut command_data_idx = None;
                let mut priority_idx = None;
                
                for (i, column) in serie.columns.iter().enumerate() {
                    match column.as_str() {
                        "time" => time_idx = Some(i),
                        "command_type" => command_type_idx = Some(i),
                        "command_data" => command_data_idx = Some(i),
                        "priority" => priority_idx = Some(i),
                        _ => {}
                    }
                }
                
                // Parse each row
                for row in serie.values {
                    let command = BmsCommand {
                        equipment_id: equipment_id.to_string(),
                        location_id: location_id.to_string(),
                        command_type: command_type_idx
                            .and_then(|i| row.get(i))
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown")
                            .to_string(),
                        command_data: command_data_idx
                            .and_then(|i| row.get(i))
                            .cloned()
                            .unwrap_or(serde_json::Value::Null),
                        timestamp: time_idx
                            .and_then(|i| row.get(i))
                            .and_then(|v| v.as_str())
                            .and_then(|s| s.parse::<DateTime<Utc>>().ok())
                            .unwrap_or_else(Utc::now),
                        priority: priority_idx
                            .and_then(|i| row.get(i))
                            .and_then(|v| v.as_i64())
                            .unwrap_or(0) as i32,
                    };
                    commands.push(command);
                }
            }
        }
    }
    
    // Sort by priority (higher first) and then by timestamp (newest first)
    commands.sort_by(|a, b| {
        b.priority.cmp(&a.priority)
            .then_with(|| b.timestamp.cmp(&a.timestamp))
    });
    
    Ok(commands)
}

async fn execute_bms_commands(
    commands: Vec<BmsCommand>,
    board_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<LogicOutputs, String> {
    let mut outputs = LogicOutputs::default();
    
    println!("Executing {} BMS commands for board {}", commands.len(), board_id);
    
    // Process each command in priority order
    for command in commands {
        println!("Processing BMS command: {} with data: {:?}", command.command_type, command.command_data);
        
        match command.command_type.as_str() {
            "heating_valve" => {
                if let Some(value) = command.command_data.as_f64() {
                    outputs.heating_valve_position = Some(value);
                }
            }
            "cooling_valve" => {
                if let Some(value) = command.command_data.as_f64() {
                    outputs.cooling_valve_position = Some(value);
                }
            }
            "fan_enable" => {
                if let Some(value) = command.command_data.as_bool() {
                    outputs.fan_enabled = Some(value);
                }
            }
            "fan_speed" => {
                if let Some(value) = command.command_data.as_f64() {
                    outputs.fan_vfd_speed = Some(value);
                }
            }
            "damper_position" => {
                if let Some(value) = command.command_data.as_f64() {
                    outputs.outdoor_damper_position = Some(value);
                }
            }
            "supply_temp_setpoint" => {
                if let Some(value) = command.command_data.as_f64() {
                    outputs.supply_air_temp_setpoint = Some(value);
                }
            }
            "unit_enable" => {
                if let Some(value) = command.command_data.as_bool() {
                    outputs.unit_enable = Some(value);
                }
            }
            "occupied_mode" => {
                if let Some(value) = command.command_data.as_bool() {
                    outputs.is_occupied = Some(value);
                }
            }
            _ => {
                println!("Unknown BMS command type: {}", command.command_type);
            }
        }
    }
    
    // Update command source
    {
        let mut status = state.bms_connection_status.lock().unwrap();
        status.command_source = "bms".to_string();
    }
    
    Ok(outputs)
}

// Hardware interface functions
async fn get_megabas_version(stack: u8) -> Result<String, String> {
    let output = Command::new("python3")
        .arg("-c")
        .arg(&format!("import megabas; print(megabas.getVer({}))", stack))
        .output()
        .map_err(|_| "Board not found".to_string())?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err("Board not found".to_string())
    }
}

async fn read_megabas_analog_input(stack: u8, channel: u8) -> Result<f64, String> {
    let output = Command::new("python3")
        .arg("-c")
        .arg(&format!("import megabas; print(megabas.getUIn({}, {}))", stack, channel))
        .output()
        .map_err(|e| format!("Failed to execute command: {}", e))?;

    if output.status.success() {
        String::from_utf8_lossy(&output.stdout)
            .trim()
            .parse::<f64>()
            .map_err(|e| format!("Failed to parse value: {}", e))
    } else {
        Err("Failed to read input".to_string())
    }
}

async fn read_megabas_analog_output(stack: u8, channel: u8) -> Result<f64, String> {
    let output = Command::new("python3")
        .arg("-c")
        .arg(&format!("import megabas; print(megabas.getUOut({}, {}))", stack, channel))
        .output()
        .map_err(|e| format!("Failed to execute command: {}", e))?;

    if output.status.success() {
        String::from_utf8_lossy(&output.stdout)
            .trim()
            .parse::<f64>()
            .map_err(|e| format!("Failed to parse value: {}", e))
    } else {
        Err("Failed to read output".to_string())
    }
}

async fn read_megabas_digital_input(stack: u8, channel: u8) -> Result<bool, String> {
    let output = Command::new("python3")
        .arg("-c")
        .arg(&format!("import megabas; print(megabas.getOptIn({}, {}))", stack, channel))
        .output()
        .map_err(|e| format!("Failed to execute command: {}", e))?;

    if output.status.success() {
        let value: u8 = String::from_utf8_lossy(&output.stdout)
            .trim()
            .parse()
            .map_err(|e| format!("Failed to parse value: {}", e))?;
        Ok(value == 1)
    } else {
        Err("Failed to read input".to_string())
    }
}

async fn read_megabas_triac(stack: u8, channel: u8) -> Result<bool, String> {
    let output = Command::new("python3")
        .arg("-c")
        .arg(&format!("import megabas; print(megabas.getTriac({}, {}))", stack, channel))
        .output()
        .map_err(|e| format!("Failed to execute command: {}", e))?;

    if output.status.success() {
        let value: u8 = String::from_utf8_lossy(&output.stdout)
            .trim()
            .parse()
            .map_err(|e| format!("Failed to parse value: {}", e))?;
        Ok(value == 1)
    } else {
        Err("Failed to read triac".to_string())
    }
}

async fn check_8relay_board(stack: u8) -> Result<(), String> {
    let output = Command::new("python3")
        .arg("-c")
        .arg(&format!("import lib8relind; print(lib8relind.get_all({}))", stack))
        .output()
        .map_err(|_| "Board not found".to_string())?;

    if output.status.success() {
        Ok(())
    } else {
        Err("Board not found".to_string())
    }
}

async fn check_16relay_board(stack: u8) -> Result<(), String> {
    let output = Command::new("python3")
        .arg("-c")
        .arg(&format!("import SM16relind; rel = SM16relind.SM16relind({}); print(rel.get_all())", stack))
        .output()
        .map_err(|_| "Board not found".to_string())?;

    if output.status.success() {
        Ok(())
    } else {
        Err("Board not found".to_string())
    }
}

async fn check_16univin_board(stack: u8) -> Result<String, String> {
    let output = Command::new("python3")
        .arg("-c")
        .arg(&format!("import lib16univin; card = lib16univin.SM16univin({}); print(card.get_version())", stack))
        .output()
        .map_err(|_| "Board not found".to_string())?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err("Board not found".to_string())
    }
}

async fn check_16uout_board(stack: u8) -> Result<String, String> {
    let output = Command::new("python3")
        .arg("-c")
        .arg(&format!("import SM16uout.SM16uout as m; card = m({}); print(card.get_version())", stack))
        .output()
        .map_err(|_| "Board not found".to_string())?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err("Board not found".to_string())
    }
}

async fn read_16univin_analog(stack: u8, channel: u8) -> Result<f64, String> {
    let output = Command::new("python3")
        .arg("-c")
        .arg(&format!("import lib16univin; card = lib16univin.SM16univin({}); print(card.get_u_in({}))", stack, channel))
        .output()
        .map_err(|e| format!("Failed to execute command: {}", e))?;

    if output.status.success() {
        String::from_utf8_lossy(&output.stdout)
            .trim()
            .parse::<f64>()
            .map_err(|e| format!("Failed to parse value: {}", e))
    } else {
        Err("Failed to read input".to_string())
    }
}

async fn read_16univin_digital(stack: u8, channel: u8) -> Result<bool, String> {
    let output = Command::new("python3")
        .arg("-c")
        .arg(&format!("import lib16univin; card = lib16univin.SM16univin({}); print(card.get_dig_in({}))", stack, channel))
        .output()
        .map_err(|e| format!("Failed to execute command: {}", e))?;

    if output.status.success() {
        let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(value == "True" || value == "1")
    } else {
        Err("Failed to read input".to_string())
    }
}

async fn read_16uout_output(stack: u8, channel: u8) -> Result<f64, String> {
    let output = Command::new("python3")
        .arg("-c")
        .arg(&format!("import SM16uout.SM16uout as m; card = m({}); print(card.get_u_out({}))", stack, channel))
        .output()
        .map_err(|e| format!("Failed to execute command: {}", e))?;

    if output.status.success() {
        String::from_utf8_lossy(&output.stdout)
            .trim()
            .parse::<f64>()
            .map_err(|e| format!("Failed to parse value: {}", e))
    } else {
        Err("Failed to read output".to_string())
    }
}

async fn set_16uout_output(stack: u8, channel: u8, value: f64) -> Result<(), String> {
    let output = Command::new("python3")
        .arg("-c")
        .arg(&format!("import SM16uout.SM16uout as m; card = m({}); card.set_u_out({}, {})", stack, channel, value))
        .output()
        .map_err(|e| format!("Failed to execute command: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        Err("Failed to set output".to_string())
    }
}

async fn read_8relay_state(stack: u8, channel: u8) -> Result<bool, String> {
    let output = Command::new("python3")
        .arg("-c")
        .arg(&format!("import lib8relind; print(lib8relind.get({}, {}))", stack, channel))
        .output()
        .map_err(|e| format!("Failed to execute command: {}", e))?;

    if output.status.success() {
        let value: u8 = String::from_utf8_lossy(&output.stdout)
            .trim()
            .parse()
            .map_err(|e| format!("Failed to parse value: {}", e))?;
        Ok(value == 1)
    } else {
        Err("Failed to read relay".to_string())
    }
}

async fn set_8relay_state(stack: u8, channel: u8, state: bool) -> Result<(), String> {
    let value = if state { 1 } else { 0 };
    let output = Command::new("python3")
        .arg("-c")
        .arg(&format!("import lib8relind; lib8relind.set({}, {}, {})", stack, channel, value))
        .output()
        .map_err(|e| format!("Failed to execute command: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        Err("Failed to set relay".to_string())
    }
}

async fn read_16relay_state(stack: u8, channel: u8) -> Result<bool, String> {
    let output = Command::new("python3")
        .arg("-c")
        .arg(&format!("import SM16relind; rel = SM16relind.SM16relind({}); print(rel.get({}))", stack, channel))
        .output()
        .map_err(|e| format!("Failed to execute command: {}", e))?;

    if output.status.success() {
        let value: u8 = String::from_utf8_lossy(&output.stdout)
            .trim()
            .parse()
            .map_err(|e| format!("Failed to parse value: {}", e))?;
        Ok(value == 1)
    } else {
        Err("Failed to read relay".to_string())
    }
}

async fn set_16relay_state(stack: u8, channel: u8, state: bool) -> Result<(), String> {
    let value = if state { 1 } else { 0 };
    let output = Command::new("python3")
        .arg("-c")
        .arg(&format!("import SM16relind; rel = SM16relind.SM16relind({}); rel.set({}, {})", stack, channel, value))
        .output()
        .map_err(|e| format!("Failed to execute command: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        Err("Failed to set relay".to_string())
    }
}

async fn set_megabas_analog_output(stack: u8, channel: u8, value: f64) -> Result<(), String> {
    let output = Command::new("python3")
        .arg("-c")
        .arg(&format!("import megabas; megabas.setUOut({}, {}, {})", stack, channel, value))
        .output()
        .map_err(|e| format!("Failed to execute command: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        Err("Failed to set output".to_string())
    }
}

async fn set_megabas_triac(stack: u8, channel: u8, state: bool) -> Result<(), String> {
    let value = if state { 1 } else { 0 };
    let output = Command::new("python3")
        .arg("-c")
        .arg(&format!("import megabas; megabas.setTriac({}, {}, {})", stack, channel, value))
        .output()
        .map_err(|e| format!("Failed to execute command: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        Err("Failed to set triac".to_string())
    }
}

// MegaBAS input configuration functions
async fn configure_megabas_input_type(stack: u8, channel: u8, input_type: u8) -> Result<(), String> {
    // input_type: 0 = 0-10V, 1 = 1K Thermistor/Dry contact, 2 = 10K Thermistor
    let output = Command::new("megabas")
        .arg(stack.to_string())
        .arg("incfgwr")
        .arg(channel.to_string())
        .arg(input_type.to_string())
        .output()
        .map_err(|e| format!("Failed to execute command: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Failed to configure input type: {}", stderr))
    }
}

async fn read_megabas_input_type(stack: u8, channel: u8) -> Result<u8, String> {
    let output = Command::new("megabas")
        .arg(stack.to_string())
        .arg("incfgrd")
        .arg(channel.to_string())
        .output()
        .map_err(|e| format!("Failed to execute command: {}", e))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Parse the output to get the input type value
        stdout.trim()
            .split_whitespace()
            .last()
            .and_then(|s| s.parse::<u8>().ok())
            .ok_or_else(|| "Failed to parse input type".to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Failed to read input type: {}", stderr))
    }
}

// Input configuration commands
#[tauri::command]
async fn configure_input_type(
    board_id: String,
    channel: u8,
    input_type: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let board = {
        let boards = state.boards.lock().unwrap();
        boards.get(&board_id).cloned().ok_or("Board not found")?
    };

    // Only MegaBAS supports software configurable inputs
    if board.board_type != "SM-I-002 Building Automation" {
        return Err("Board does not support software configurable inputs".to_string());
    }

    // Convert input type string to value
    let type_value = match input_type.as_str() {
        "0-10V" => 0,
        "1K_thermistor" => 1,
        "dry_contact" => 1,
        "10K_thermistor" => 2,
        "thermistor_10k_type2" => 2,
        _ => return Err(format!("Unknown input type: {}", input_type)),
    };

    configure_megabas_input_type(board.stack_level, channel, type_value).await
}

#[tauri::command]
async fn get_input_type(
    board_id: String,
    channel: u8,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let board = {
        let boards = state.boards.lock().unwrap();
        boards.get(&board_id).cloned().ok_or("Board not found")?
    };

    // Only MegaBAS supports software configurable inputs
    if board.board_type != "SM-I-002 Building Automation" {
        return Err("Board does not support software configurable inputs".to_string());
    }

    let type_value = read_megabas_input_type(board.stack_level, channel).await?;
    
    let input_type = match type_value {
        0 => "0-10V",
        1 => "1K_thermistor",
        2 => "10K_thermistor",
        _ => "unknown",
    };

    Ok(input_type.to_string())
}

// Maintenance Mode Commands
#[tauri::command]
async fn enable_maintenance_mode(
    reason: String,
    authorized_by: String,
    duration_minutes: Option<u32>,
    state: tauri::State<'_, AppState>,
) -> Result<MaintenanceMode, String> {
    let mut maintenance = state.maintenance_mode.lock().unwrap();
    
    // Check if already in maintenance mode
    if maintenance.enabled {
        return Err("Maintenance mode is already active".to_string());
    }
    
    maintenance.enabled = true;
    maintenance.started_at = Some(Utc::now());
    maintenance.duration_minutes = duration_minutes.unwrap_or(120); // Default 2 hours
    maintenance.reason = reason;
    maintenance.authorized_by = authorized_by;
    
    println!("Maintenance mode enabled by {} for {} minutes. Reason: {}", 
             maintenance.authorized_by, maintenance.duration_minutes, maintenance.reason);
    
    Ok(maintenance.clone())
}

#[tauri::command]
async fn disable_maintenance_mode(
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let mut maintenance = state.maintenance_mode.lock().unwrap();
    
    if !maintenance.enabled {
        return Err("Maintenance mode is not active".to_string());
    }
    
    maintenance.enabled = false;
    maintenance.started_at = None;
    
    println!("Maintenance mode disabled");
    
    Ok(())
}

#[tauri::command]
async fn get_maintenance_status(
    state: tauri::State<'_, AppState>,
) -> Result<MaintenanceMode, String> {
    let mut maintenance = state.maintenance_mode.lock().unwrap();
    
    // Check if maintenance mode has expired
    if maintenance.enabled {
        if let Some(started_at) = maintenance.started_at {
            let elapsed = Utc::now().signed_duration_since(started_at);
            let duration_minutes = maintenance.duration_minutes as i64;
            
            if elapsed.num_minutes() >= duration_minutes {
                // Maintenance mode expired, disable it
                maintenance.enabled = false;
                maintenance.started_at = None;
                println!("Maintenance mode auto-expired after {} minutes", duration_minutes);
            }
        }
    }
    
    Ok(maintenance.clone())
}

// Metrics database commands
#[tauri::command]
async fn store_board_metrics(
    state: tauri::State<'_, AppState>,
    board_id: String,
    board_config: BoardConfig,
) -> Result<usize, String> {
    // Get IO state first
    let io_state = {
        let io_states = state.io_states.lock().unwrap();
        io_states.get(&board_id).cloned().ok_or("Board IO state not found")?
    };
    
    // Prepare metrics
    let mut metrics = Vec::new();
    
    // Store universal inputs
    for (i, value) in io_state.analog_inputs.iter().enumerate() {
        if let Some(channel_config) = board_config.universal_inputs.get(i) {
            if channel_config.enabled {
                let scaled_value = calculate_scaled_value(channel_config, *value);
                metrics.push((
                    board_id.clone(),
                    "universal_input".to_string(),
                    i as i32,
                    channel_config.name.clone(),
                    *value,
                    Some(scaled_value),
                    Some(channel_config.units.clone()),
                ));
            }
        }
    }
    
    // Store analog outputs
    for (i, value) in io_state.analog_outputs.iter().enumerate() {
        if let Some(channel_config) = board_config.analog_outputs.get(i) {
            if channel_config.enabled {
                metrics.push((
                    board_id.clone(),
                    "analog_output".to_string(),
                    i as i32,
                    channel_config.name.clone(),
                    *value,
                    None,
                    Some(channel_config.units.clone()),
                ));
            }
        }
    }
    
    // Store metrics batch
    let db = {
        let metrics_db = state.metrics_db.lock().unwrap();
        metrics_db.as_ref().cloned().ok_or("Metrics database not initialized")?
    };
    
    match db.insert_metrics_batch(metrics).await {
        Ok(count) => Ok(count),
        Err(e) => Err(format!("Failed to store metrics: {}", e)),
    }
}

#[tauri::command]
async fn get_trend_data(
    state: tauri::State<'_, AppState>,
    board_id: String,
    channel_type: String,
    channel_index: i32,
    hours: i32,
) -> Result<TrendData, String> {
    let db = {
        let metrics_db = state.metrics_db.lock().unwrap();
        metrics_db.as_ref().cloned().ok_or("Metrics database not initialized")?
    };
    
    match db.get_trend_data(&board_id, &channel_type, channel_index, hours).await {
        Ok(data) => Ok(data),
        Err(e) => Err(format!("Failed to get trend data: {}", e)),
    }
}

#[tauri::command]
async fn query_metrics(
    state: tauri::State<'_, AppState>,
    query: MetricQuery,
) -> Result<Vec<metrics_db::Metric>, String> {
    let db = {
        let metrics_db = state.metrics_db.lock().unwrap();
        metrics_db.as_ref().cloned()
    };
    
    if let Some(db) = db {
        match db.query_metrics(query).await {
            Ok(metrics) => Ok(metrics),
            Err(e) => Err(format!("Failed to query metrics: {}", e)),
        }
    } else {
        Err("Metrics database not initialized".to_string())
    }
}

#[tauri::command]
async fn get_channel_list(
    state: tauri::State<'_, AppState>,
    board_id: String,
) -> Result<Vec<(String, String, i32, String)>, String> {
    let db = {
        let metrics_db = state.metrics_db.lock().unwrap();
        metrics_db.as_ref().cloned()
    };
    
    if let Some(db) = db {
        match db.get_channel_list(&board_id).await {
            Ok(channels) => Ok(channels),
            Err(e) => Err(format!("Failed to get channel list: {}", e)),
        }
    } else {
        Err("Metrics database not initialized".to_string())
    }
}

// Helper function to calculate scaled value
fn calculate_scaled_value(channel_config: &ChannelConfig, raw_value: f64) -> f64 {
    match channel_config.input_type.as_deref() {
        Some("digital") => {
            if raw_value > 5.0 { 1.0 } else { 0.0 }
        }
        Some("thermistor_10k_type2") => {
            // Convert voltage to resistance for 10K thermistor with 10K pull-up
            // Voltage divider: Vout = Vin * (Rtherm / (Rpullup + Rtherm))
            // Rearranged: Rtherm = Rpullup * (Vout / (Vin - Vout))
            let vin = 10.0; // 10V supply
            let rpullup = 10000.0; // 10K pull-up resistor
            
            if raw_value >= vin {
                return -999.0; // Error value for open circuit
            }
            
            let rtherm = rpullup * (raw_value / (vin - raw_value));
            
            // Steinhart-Hart equation coefficients for 10K Type 2 thermistor
            let a = 0.001468069;
            let b = 0.00023887;
            let c = 0.00000010792;
            
            // Steinhart-Hart equation: 1/T = A + B*ln(R) + C*(ln(R))^3
            let ln_r = rtherm.ln();
            let inv_t = a + b * ln_r + c * ln_r.powi(3);
            let temp_k = 1.0 / inv_t;
            let temp_c = temp_k - 273.15;
            let temp_f = temp_c * 9.0 / 5.0 + 32.0;
            
            // Apply calibration offset to final temperature
            temp_f + channel_config.calibration_offset
        }
        Some("current") => {
            // 4-20mA scaling
            let scaled = ((raw_value - 4.0) / 16.0) * (channel_config.scaling_max - channel_config.scaling_min) + channel_config.scaling_min;
            scaled + channel_config.calibration_offset
        }
        _ => {
            // Default 0-10V scaling
            let scaled = (raw_value / 10.0) * (channel_config.scaling_max - channel_config.scaling_min) + channel_config.scaling_min;
            scaled + channel_config.calibration_offset
        }
    }
}

#[tokio::main]
async fn main() {
    let app_state = AppState::new().await;
    
    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            // Logic Engine commands
            load_logic_file,
            get_logic_files,
            activate_logic_file,
            execute_logic_file,
            apply_logic_outputs,
            get_logic_execution_history,
            delete_logic_file,
            // Board control commands
            scan_boards,
            read_board_io,
            set_analog_output,
            set_relay_state,
            set_triac_state,
            save_bms_config,
            save_processing_config,
            get_bms_config,
            get_processing_config,
            // Input configuration
            configure_input_type,
            get_input_type,
            // Firmware management
            check_firmware_repos,
            clone_firmware_repo,
            install_firmware_drivers,
            update_board_firmware,
            pull_firmware_updates,
            // BMS command integration
            query_bms_commands,
            get_bms_connection_status,
            test_bms_connection,
            // Maintenance mode
            enable_maintenance_mode,
            disable_maintenance_mode,
            get_maintenance_status,
            // Metrics database
            store_board_metrics,
            get_trend_data,
            query_metrics,
            get_channel_list
        ])
        .setup(|app| {
            println!("Building Automation Control Center with BMS Integration starting up!");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
