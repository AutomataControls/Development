// Board Communication Module for Sequent Microsystems Hardware
// Wraps existing Python interface instead of recreating functionality

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use std::time::Duration;
use std::process::Command;
use std::path::Path;

// Python interface paths - using comprehensive firmware interface
const PYTHON_INTERFACE: &str = "/opt/automata-nexus/firmware_interface.py";
const FALLBACK_INTERFACE: &str = "/home/Automata/Development/projects/Rust-SSD-Nexus-Controller/src/firmware_interface.py";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BoardType {
    MegaBAS,
    Relay8,
    Relay16,
    UnivIn16,
    UOut16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardInfo {
    pub board_type: BoardType,
    pub stack_level: u8,
    pub name: String,
    pub version: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalogInput {
    pub channel: u8,
    pub voltage: f32,
    pub resistance_1k: Option<f32>,
    pub resistance_10k: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalogOutput {
    pub channel: u8,
    pub voltage: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayState {
    pub channel: u8,
    pub state: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriacState {
    pub channel: u8,
    pub state: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactState {
    pub channel: u8,
    pub state: bool,
    pub counter: u32,
    pub edge_mode: u8,
}

pub struct BoardManager {
    boards: Arc<RwLock<Vec<BoardInfo>>>,
    board_states: Arc<RwLock<HashMap<String, Value>>>,
    monitoring_enabled: Arc<RwLock<bool>>,
}

impl BoardManager {
    pub fn new() -> Self {
        Self {
            boards: Arc::new(RwLock::new(Vec::new())),
            board_states: Arc::new(RwLock::new(HashMap::new())),
            monitoring_enabled: Arc::new(RwLock::new(true)),
        }
    }
    
    // Get the correct Python interface path
    fn get_interface_path() -> Result<&'static str> {
        if Path::new(PYTHON_INTERFACE).exists() {
            Ok(PYTHON_INTERFACE)
        } else if Path::new(FALLBACK_INTERFACE).exists() {
            Ok(FALLBACK_INTERFACE)
        } else {
            Err(anyhow!("Firmware interface not found at {} or {}", 
                PYTHON_INTERFACE, FALLBACK_INTERFACE))
        }
    }
    
    // Scan for all connected boards using Python interface
    pub async fn scan_boards(&self) -> Result<Vec<BoardInfo>> {
        let interface_path = Self::get_interface_path()?;
        
        let output = Command::new("python3")
            .arg(interface_path)
            .arg("scan")
            .output()?;
        
        if !output.status.success() {
            return Err(anyhow!("Failed to scan boards: {}", 
                String::from_utf8_lossy(&output.stderr)));
        }
        
        let json_str = String::from_utf8(output.stdout)?;
        let boards_json: Vec<Value> = serde_json::from_str(&json_str)?;
        
        let mut boards = Vec::new();
        for board in boards_json {
            let board_type = match board["type"].as_str().unwrap_or("") {
                "megabas" => BoardType::MegaBAS,
                "8relay" => BoardType::Relay8,
                "16relay" => BoardType::Relay16,
                "16univin" => BoardType::UnivIn16,
                "16uout" => BoardType::UOut16,
                _ => continue,
            };
            
            boards.push(BoardInfo {
                board_type,
                stack_level: board["stack"].as_u64().unwrap_or(0) as u8,
                name: board["name"].as_str().unwrap_or("Unknown").to_string(),
                version: board.get("version")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown")
                    .to_string(),
                status: "Online".to_string(),
            });
        }
        
        // Update stored boards list
        *self.boards.write().await = boards.clone();
        
        Ok(boards)
    }
    
    // Read analog input from MegaBAS using Python interface
    pub async fn read_analog_input(&self, stack: u8, channel: u8) -> Result<f32> {
        if channel < 1 || channel > 8 {
            return Err(anyhow!("Invalid channel: {}", channel));
        }
        
        let interface_path = Self::get_interface_path()?;
        
        let output = Command::new("python3")
            .arg(interface_path)
            .arg("status")
            .arg("megabas")
            .arg(stack.to_string())
            .output()?;
        
        if !output.status.success() {
            return Err(anyhow!("Failed to read analog input: {}",
                String::from_utf8_lossy(&output.stderr)));
        }
        
        let json_str = String::from_utf8(output.stdout)?;
        let status: Value = serde_json::from_str(&json_str)?;
        
        let channel_key = format!("ch{}", channel);
        let voltage = status["analog_inputs"][&channel_key]["voltage"]
            .as_f64()
            .ok_or_else(|| anyhow!("Failed to read channel {}", channel))? as f32;
        
        Ok(voltage)
    }
    
    // Write analog output on MegaBAS using Python interface
    pub async fn write_analog_output(&self, stack: u8, channel: u8, voltage: f32) -> Result<()> {
        if channel < 1 || channel > 4 {
            return Err(anyhow!("Invalid channel: {}", channel));
        }
        
        if voltage < 0.0 || voltage > 10.0 {
            return Err(anyhow!("Voltage out of range: {}", voltage));
        }
        
        let output = Command::new("python3")
            .arg(PYTHON_INTERFACE)
            .arg("set_output")
            .arg(stack.to_string())
            .arg("analog")
            .arg(channel.to_string())
            .arg(voltage.to_string())
            .output()?;
        
        if !output.status.success() {
            return Err(anyhow!("Failed to write analog output: {}",
                String::from_utf8_lossy(&output.stderr)));
        }
        
        Ok(())
    }
    
    // Control relay using Python interface
    pub async fn set_relay(&self, board_type: BoardType, stack: u8, relay: u8, state: bool) -> Result<()> {
        let board_type_str = match board_type {
            BoardType::Relay8 => "8relay",
            BoardType::Relay16 => "16relay",
            _ => return Err(anyhow!("Not a relay board")),
        };
        
        let output = Command::new("python3")
            .arg(PYTHON_INTERFACE)
            .arg("set_relay")
            .arg(board_type_str)
            .arg(stack.to_string())
            .arg(relay.to_string())
            .arg(if state { "1" } else { "0" })
            .output()?;
        
        if !output.status.success() {
            return Err(anyhow!("Failed to set relay: {}",
                String::from_utf8_lossy(&output.stderr)));
        }
        
        Ok(())
    }
    
    // Control triac output using Python interface
    pub async fn set_triac(&self, stack: u8, triac: u8, state: bool) -> Result<()> {
        if triac < 1 || triac > 4 {
            return Err(anyhow!("Invalid triac number: {}", triac));
        }
        
        let output = Command::new("python3")
            .arg(PYTHON_INTERFACE)
            .arg("set_output")
            .arg(stack.to_string())
            .arg("triac")
            .arg(triac.to_string())
            .arg(if state { "1" } else { "0" })
            .output()?;
        
        if !output.status.success() {
            return Err(anyhow!("Failed to set triac: {}",
                String::from_utf8_lossy(&output.stderr)));
        }
        
        Ok(())
    }
    
    // Read universal input (16-channel board) using Python interface
    pub async fn read_universal_input(&self, stack: u8, channel: u8) -> Result<f32> {
        if channel < 1 || channel > 16 {
            return Err(anyhow!("Invalid channel: {}", channel));
        }
        
        let output = Command::new("python3")
            .arg(PYTHON_INTERFACE)
            .arg("read_16univin")
            .arg(stack.to_string())
            .arg(channel.to_string())
            .output()?;
        
        if !output.status.success() {
            return Err(anyhow!("Failed to read universal input: {}",
                String::from_utf8_lossy(&output.stderr)));
        }
        
        let json_str = String::from_utf8(output.stdout)?;
        let result: Value = serde_json::from_str(&json_str)?;
        
        let voltage = result["voltage"]
            .as_f64()
            .ok_or_else(|| anyhow!("Failed to read voltage"))? as f32;
        
        Ok(voltage)
    }
    
    // Write universal output (16-channel 0-10V board) using Python interface
    pub async fn write_universal_output(&self, stack: u8, channel: u8, voltage: f32) -> Result<()> {
        if channel < 1 || channel > 16 {
            return Err(anyhow!("Invalid channel: {}", channel));
        }
        
        if voltage < 0.0 || voltage > 10.0 {
            return Err(anyhow!("Voltage out of range: {}", voltage));
        }
        
        let output = Command::new("python3")
            .arg(PYTHON_INTERFACE)
            .arg("set_16uout")
            .arg(stack.to_string())
            .arg(channel.to_string())
            .arg(voltage.to_string())
            .output()?;
        
        if !output.status.success() {
            return Err(anyhow!("Failed to write universal output: {}",
                String::from_utf8_lossy(&output.stderr)));
        }
        
        Ok(())
    }
    
    // Get complete board status using Python interface
    pub async fn get_board_status(&self, board_type: BoardType, stack: u8) -> Result<HashMap<String, Value>> {
        let board_type_str = match board_type {
            BoardType::MegaBAS => "megabas",
            BoardType::Relay8 => "8relay",
            BoardType::Relay16 => "16relay",
            BoardType::UnivIn16 => "16univin",
            BoardType::UOut16 => "16uout",
        };
        
        let output = Command::new("python3")
            .arg(PYTHON_INTERFACE)
            .arg("status")
            .arg(board_type_str)
            .arg(stack.to_string())
            .output()?;
        
        if !output.status.success() {
            return Err(anyhow!("Failed to get board status: {}",
                String::from_utf8_lossy(&output.stderr)));
        }
        
        let json_str = String::from_utf8(output.stdout)?;
        let status: HashMap<String, Value> = serde_json::from_str(&json_str)?;
        
        // Store in cache
        let key = format!("{}_{}", board_type_str, stack);
        self.board_states.write().await.insert(key, Value::Object(status.clone().into_iter().collect()));
        
        Ok(status)
    }
    
    // Read dry contact state
    pub async fn read_contact(&self, stack: u8, channel: u8) -> Result<ContactState> {
        let status = self.get_board_status(BoardType::MegaBAS, stack).await?;
        
        let channel_key = format!("ch{}", channel);
        let contact = &status["contacts"][&channel_key];
        
        Ok(ContactState {
            channel,
            state: contact["state"].as_bool().unwrap_or(false),
            counter: contact["counter"].as_u64().unwrap_or(0) as u32,
            edge_mode: contact["edge_mode"].as_u64().unwrap_or(0) as u8,
        })
    }
    
    // Read system voltages
    pub async fn read_system_voltages(&self, stack: u8) -> Result<(f32, f32, f32)> {
        let status = self.get_board_status(BoardType::MegaBAS, stack).await?;
        
        let power_v = status["sensors"]["power_supply_v"]
            .as_f64()
            .unwrap_or(0.0) as f32;
        let rasp_v = status["sensors"]["raspberry_v"]
            .as_f64()
            .unwrap_or(0.0) as f32;
        let cpu_temp = status["sensors"]["cpu_temp_c"]
            .as_f64()
            .unwrap_or(0.0) as f32;
        
        Ok((power_v, rasp_v, cpu_temp))
    }
    
    // Get all analog inputs
    pub async fn get_all_analog_inputs(&self, stack: u8) -> Result<Vec<AnalogInput>> {
        let status = self.get_board_status(BoardType::MegaBAS, stack).await?;
        let mut inputs = Vec::new();
        
        if let Some(analog_inputs) = status.get("analog_inputs") {
            for ch in 1..=8 {
                let channel_key = format!("ch{}", ch);
                if let Some(input) = analog_inputs.get(&channel_key) {
                    inputs.push(AnalogInput {
                        channel: ch,
                        voltage: input["voltage"].as_f64().unwrap_or(0.0) as f32,
                        resistance_1k: input.get("r1k").and_then(|v| v.as_f64()).map(|v| v as f32),
                        resistance_10k: input.get("r10k").and_then(|v| v.as_f64()).map(|v| v as f32),
                    });
                }
            }
        }
        
        Ok(inputs)
    }
    
    // Get all relay states
    pub async fn get_all_relays(&self, board_type: BoardType, stack: u8) -> Result<Vec<RelayState>> {
        let status = self.get_board_status(board_type, stack).await?;
        let mut relays = Vec::new();
        
        if let Some(relay_states) = status.get("relays") {
            for (key, value) in relay_states.as_object().unwrap_or(&serde_json::Map::new()) {
                if let Some(ch) = key.strip_prefix("ch") {
                    if let Ok(channel) = ch.parse::<u8>() {
                        relays.push(RelayState {
                            channel,
                            state: value.as_bool().unwrap_or(false),
                        });
                    }
                }
            }
        }
        
        Ok(relays)
    }
    
    // Get all triac states
    pub async fn get_all_triacs(&self, stack: u8) -> Result<Vec<TriacState>> {
        let status = self.get_board_status(BoardType::MegaBAS, stack).await?;
        let mut triacs = Vec::new();
        
        if let Some(triac_states) = status.get("triacs") {
            for (key, value) in triac_states.as_object().unwrap_or(&serde_json::Map::new()) {
                if let Some(ch) = key.strip_prefix("ch") {
                    if let Ok(channel) = ch.parse::<u8>() {
                        triacs.push(TriacState {
                            channel,
                            state: value.as_bool().unwrap_or(false),
                        });
                    }
                }
            }
        }
        
        Ok(triacs)
    }
    
    // Emergency stop - turn off all outputs
    pub async fn emergency_stop(&self) -> Result<()> {
        let boards = self.boards.read().await.clone();
        
        for board in boards {
            match board.board_type {
                BoardType::MegaBAS => {
                    // Turn off all triacs
                    for triac in 1..=4 {
                        let _ = self.set_triac(board.stack_level, triac, false).await;
                    }
                    // Set all analog outputs to 0
                    for ch in 1..=4 {
                        let _ = self.write_analog_output(board.stack_level, ch, 0.0).await;
                    }
                }
                BoardType::Relay8 => {
                    // Turn off all relays
                    for relay in 1..=8 {
                        let _ = self.set_relay(BoardType::Relay8, board.stack_level, relay, false).await;
                    }
                }
                BoardType::Relay16 => {
                    // Turn off all relays
                    for relay in 1..=16 {
                        let _ = self.set_relay(BoardType::Relay16, board.stack_level, relay, false).await;
                    }
                }
                BoardType::UOut16 => {
                    // Set all outputs to 0
                    for ch in 1..=16 {
                        let _ = self.write_universal_output(board.stack_level, ch, 0.0).await;
                    }
                }
                _ => {}
            }
        }
        
        Ok(())
    }
}

// Start background monitoring task
pub async fn start_monitoring(state: Arc<Mutex<crate::state::AppState>>) {
    let manager = BoardManager::new();
    
    tokio::spawn(async move {
        loop {
            // Scan boards every 10 seconds
            tokio::time::sleep(Duration::from_secs(10)).await;
            
            if let Ok(boards) = manager.scan_boards().await {
                // Update state with found boards
                let mut app_state = state.lock().await;
                app_state.boards.clear();
                
                for board in boards {
                    let id = format!("{:?}_{}", board.board_type, board.stack_level);
                    app_state.boards.insert(id, crate::state::Board {
                        id: format!("{:?}_{}", board.board_type, board.stack_level),
                        name: board.name.clone(),
                        board_type: format!("{:?}", board.board_type),
                        address: board.stack_level,
                        port: None,
                        is_connected: true,
                        firmware_version: Some(board.version.clone()),
                    });
                }
            }
        }
    });
}