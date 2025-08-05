use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;
use std::sync::Mutex;
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Board {
    pub board_type: String,
    pub stack: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardStatus {
    pub board_type: String,
    pub stack: u8,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalogInput {
    pub voltage: f32,
    pub r1k: f32,
    pub r10k: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DryContact {
    pub state: bool,
    pub counter: u32,
    pub edge_mode: u8,
}

pub struct HardwareState {
    boards: Mutex<Vec<Board>>,
}

impl Default for HardwareState {
    fn default() -> Self {
        HardwareState {
            boards: Mutex::new(Vec::new()),
        }
    }
}

#[tauri::command]
pub async fn scan_boards() -> Result<Vec<Board>, String> {
    let output = Command::new("python3")
        .arg("../scripts/megabas_interface.py")
        .arg("scan")
        .output()
        .map_err(|e| format!("Failed to run scan: {}", e))?;
    
    if !output.status.success() {
        return Err(format!("Scan failed: {}", String::from_utf8_lossy(&output.stderr)));
    }
    
    let boards: Vec<Board> = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("Failed to parse scan results: {}", e))?;
    
    Ok(boards)
}

#[tauri::command]
pub async fn get_board_status(board_type: String, stack: u8) -> Result<BoardStatus, String> {
    let output = Command::new("python3")
        .arg("../scripts/megabas_interface.py")
        .arg("status")
        .arg(&board_type)
        .arg(stack.to_string())
        .output()
        .map_err(|e| format!("Failed to get status: {}", e))?;
    
    if !output.status.success() {
        return Err(format!("Status failed: {}", String::from_utf8_lossy(&output.stderr)));
    }
    
    let data: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("Failed to parse status: {}", e))?;
    
    Ok(BoardStatus {
        board_type,
        stack,
        data,
    })
}

#[tauri::command]
pub async fn set_output(
    board_type: String,
    stack: u8,
    channel: u8,
    value: f32
) -> Result<(), String> {
    let output = Command::new("python3")
        .arg("../scripts/megabas_interface.py")
        .arg("set")
        .arg(&board_type)
        .arg(stack.to_string())
        .arg(channel.to_string())
        .arg(value.to_string())
        .output()
        .map_err(|e| format!("Failed to set output: {}", e))?;
    
    if !output.status.success() {
        return Err(format!("Set output failed: {}", String::from_utf8_lossy(&output.stderr)));
    }
    
    let result: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("Failed to parse result: {}", e))?;
    
    if let Some(error) = result.get("error") {
        return Err(error.as_str().unwrap_or("Unknown error").to_string());
    }
    
    Ok(())
}

#[tauri::command]
pub async fn set_relay(
    board_type: String,
    stack: u8,
    channel: u8,
    state: bool
) -> Result<(), String> {
    set_output(board_type, stack, channel, if state { 1.0 } else { 0.0 }).await
}

#[tauri::command]
pub async fn get_all_status() -> Result<Vec<BoardStatus>, String> {
    let boards = scan_boards().await?;
    let mut all_status = Vec::new();
    
    for board in boards {
        match get_board_status(board.board_type.clone(), board.stack).await {
            Ok(status) => all_status.push(status),
            Err(e) => eprintln!("Failed to get status for board {:?}: {}", board, e),
        }
    }
    
    Ok(all_status)
}