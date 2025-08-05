#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod hardware;
mod data_export;
mod logic_engine;

use tauri::Manager;
use std::sync::Arc;

fn main() {
    // Create logic engine
    let engine = Arc::new(logic_engine::LogicEngine::new());
    let engine_clone = engine.clone();
    
    tauri::Builder::default()
        .setup(move |app| {
            // Start logic engine
            engine_clone.start();
            
            // Store engine in app state
            app.manage(engine_clone);
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Hardware control
            hardware::scan_boards,
            hardware::get_board_status,
            hardware::set_output,
            hardware::set_relay,
            hardware::get_all_status,
            
            // Data export
            data_export::save_bms_config,
            data_export::save_processing_config,
            data_export::get_export_configs,
            data_export::send_metrics,
            data_export::test_connection,
            
            // Logic control
            upload_logic_file,
            get_logic_files,
            remove_logic_file,
            toggle_logic_file,
            get_control_status,
            set_control_interval,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// Logic control commands
#[tauri::command]
async fn upload_logic_file(
    engine: tauri::State<'_, Arc<logic_engine::LogicEngine>>,
    name: String,
    content: String,
    equipment_type: String,
    equipment_id: String,
    location_id: String,
    description: String,
) -> Result<(), String> {
    // Save logic file
    let logic_dir = dirs::config_dir()
        .ok_or("Failed to get config directory")?
        .join("building-automation-controller")
        .join("logic");
    
    std::fs::create_dir_all(&logic_dir)
        .map_err(|e| format!("Failed to create logic directory: {}", e))?;
    
    let file_id = format!("{}_{}", equipment_id, chrono::Utc::now().timestamp());
    let file_path = logic_dir.join(format!("{}.js", file_id));
    
    std::fs::write(&file_path, content)
        .map_err(|e| format!("Failed to write logic file: {}", e))?;
    
    // Add to engine
    let logic_file = logic_engine::LogicFile {
        id: file_id,
        name,
        path: file_path.to_str().unwrap().to_string(),
        enabled: true,
        equipment_type,
        equipment_id,
        location_id,
        description,
    };
    
    engine.add_logic_file(logic_file)?;
    
    Ok(())
}

#[tauri::command]
async fn get_logic_files(
    engine: tauri::State<'_, Arc<logic_engine::LogicEngine>>,
) -> Result<Vec<logic_engine::LogicFile>, String> {
    Ok(engine.get_logic_files())
}

#[tauri::command]
async fn remove_logic_file(
    engine: tauri::State<'_, Arc<logic_engine::LogicEngine>>,
    id: String,
) -> Result<(), String> {
    engine.remove_logic_file(&id)
}

#[tauri::command]
async fn toggle_logic_file(
    engine: tauri::State<'_, Arc<logic_engine::LogicEngine>>,
    id: String,
    enabled: bool,
) -> Result<(), String> {
    engine.set_logic_enabled(&id, enabled)
}

#[tauri::command]
async fn get_control_status(
    engine: tauri::State<'_, Arc<logic_engine::LogicEngine>>,
) -> Result<Vec<logic_engine::ControlLoop>, String> {
    Ok(engine.get_control_status())
}

#[tauri::command]
async fn set_control_interval(
    engine: tauri::State<'_, Arc<logic_engine::LogicEngine>>,
    id: String,
    interval: u64,
) -> Result<(), String> {
    // This would need to be implemented in the engine
    Ok(())
}