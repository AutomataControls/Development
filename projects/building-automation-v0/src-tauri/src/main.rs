#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod logic_engine;

use tauri::Manager;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::collections::HashMap;
use std::process::Command;
use chrono::{DateTime, Utc};
use logic_engine::{LogicEngine, LogicFile, LogicInputs, LogicOutputs, LogicExecution};

#[derive(Default)]
struct AppState {
    boards: Mutex<HashMap<String, BoardInfo>>,
    io_states: Mutex<HashMap<String, IoState>>,
    bms_configs: Mutex<HashMap<String, BmsConfig>>,
    processing_configs: Mutex<HashMap<String, ProcessingConfig>>,
    logic_engine: Mutex<LogicEngine>,
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

// Tauri commands
#[tauri::command]
async fn scan_boards(state: tauri::State<'_, AppState>) -> Result<Vec<BoardInfo>, String> {
    let mut boards = Vec::new();
    let mut board_map = HashMap::new();

    // Scan for different board types
    for stack in 0..8 {
        // Check for megabas (Building Automation HAT)
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

        // Check for 8-relay board
        if let Ok(_) = check_8relay_board(stack).await {
            let board = BoardInfo {
                board_type: "SM8relind 8-Relay".to_string(),
                stack_level: stack,
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
            board_map.insert(format!("8relay_{}", stack), board.clone());
            boards.push(board);
        }

        // Check for 16-relay board
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

        // Check for 16 universal input board
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

        // Check for 16 analog output board
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
    let boards = state.boards.lock().unwrap();
    let board = boards.get(&board_id).ok_or("Board not found")?;

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
    let boards = state.boards.lock().unwrap();
    let board = boards.get(&board_id).ok_or("Board not found")?;

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
    let boards = state.boards.lock().unwrap();
    let board = boards.get(&board_id).ok_or("Board not found")?;

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
    let boards = state.boards.lock().unwrap();
    let board = boards.get(&board_id).ok_or("Board not found")?;

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

// Logic Engine Commands
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
    // Get current I/O state
    let io_states = state.io_states.lock().unwrap();
    let io_state = io_states.get(&board_id);

    // Prepare inputs
    let mut metrics = HashMap::new();
    let mut settings = HashMap::new();
    let mut board_io = HashMap::new();

    if let Some(io) = io_state {
        // Map I/O to common HVAC metrics
        if !io.analog_inputs.is_empty() {
            metrics.insert("SupplyTemp".to_string(), io.analog_inputs.get(0).copied().unwrap_or(0.0) * 10.0); // Convert 0-10V to temp
            metrics.insert("ReturnTemp".to_string(), io.analog_inputs.get(1).copied().unwrap_or(0.0) * 10.0);
            metrics.insert("Outdoor_Air".to_string(), io.analog_inputs.get(2).copied().unwrap_or(0.0) * 10.0);
            metrics.insert("MixedAir".to_string(), io.analog_inputs.get(3).copied().unwrap_or(0.0) * 10.0);
        }

        // Add I/O state to board_io
        board_io.insert("analog_inputs".to_string(), serde_json::to_value(&io.analog_inputs).unwrap());
        board_io.insert("digital_inputs".to_string(), serde_json::to_value(&io.digital_inputs).unwrap());
        board_io.insert("relay_states".to_string(), serde_json::to_value(&io.relay_states).unwrap());
    }

    // Add default settings
    settings.insert("equipmentId".to_string(), serde_json::Value::String("default".to_string()));
    settings.insert("locationId".to_string(), serde_json::Value::String("1".to_string()));

    let inputs = LogicInputs {
        metrics,
        settings,
        current_temp: 70.0, // Default
        state_storage: HashMap::new(),
        board_io,
    };

    // Execute logic
    let mut logic_engine = state.logic_engine.lock().unwrap();
    logic_engine.execute_logic(&logic_id, inputs).await
}

#[tauri::command]
async fn apply_logic_outputs(
    outputs: LogicOutputs,
    board_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let boards = state.boards.lock().unwrap();
    let board = boards.get(&board_id).ok_or("Board not found")?;

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

// Additional helper functions for other boards...
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
        let value = String::from_utf8_lossy(&output.stdout).trim();
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

fn main() {
    tauri::Builder::default()
        .manage(AppState::default())
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
            get_processing_config
        ])
        .setup(|app| {
            println!("Building Automation Control Center with Logic Engine starting up!");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
