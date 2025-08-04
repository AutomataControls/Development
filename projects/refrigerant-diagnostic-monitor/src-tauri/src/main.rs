// AutomataNexus Multi-Refrigerant Diagnostic Monitor
// Professional HVAC/Refrigeration diagnostic system
// (c) 2025 AutomataControls

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod refrigerants;
mod p499_transducer;
mod diagnostics;

use refrigerants::{RefrigerantDatabase, RefrigerantProperties};
use p499_transducer::{P499Interface, P499Configuration, TransducerReading};
use diagnostics::{DiagnosticEngine, SystemConfiguration, DiagnosticReading, DiagnosticResult};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tauri::State;
use std::collections::HashMap;

struct AppState {
    refrigerant_db: Arc<RefrigerantDatabase>,
    p499_interface: Arc<Mutex<P499Interface>>,
    diagnostic_engine: Arc<DiagnosticEngine>,
    current_readings: Arc<Mutex<HashMap<String, DiagnosticReading>>>,
    system_configs: Arc<Mutex<HashMap<String, SystemConfiguration>>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct RefrigerantInfo {
    designation: String,
    chemical_name: String,
    safety_class: String,
    gwp: i32,
    applications: Vec<String>,
}

// List all available refrigerants
#[tauri::command]
fn list_refrigerants(state: State<AppState>) -> Vec<RefrigerantInfo> {
    let refrigerants = state.refrigerant_db.list_all_refrigerants();
    
    refrigerants.into_iter()
        .filter_map(|designation| {
            state.refrigerant_db.get_refrigerant(&designation).map(|props| {
                RefrigerantInfo {
                    designation: props.designation.clone(),
                    chemical_name: props.chemical_name.clone(),
                    safety_class: props.safety_class.clone(),
                    gwp: props.gwp,
                    applications: props.applications.clone(),
                }
            })
        })
        .collect()
}

// Get refrigerant properties
#[tauri::command]
fn get_refrigerant_properties(
    designation: String,
    state: State<AppState>
) -> Result<RefrigerantProperties, String> {
    state.refrigerant_db
        .get_refrigerant(&designation)
        .cloned()
        .ok_or_else(|| format!("Refrigerant {} not found", designation))
}

// Calculate saturation temperature from pressure
#[tauri::command]
fn calculate_saturation_temp(
    refrigerant: String,
    pressure_psi: f32,
    state: State<AppState>
) -> Result<f32, String> {
    state.refrigerant_db
        .calculate_saturation_temperature(&refrigerant, pressure_psi)
        .ok_or_else(|| "Cannot calculate saturation temperature".to_string())
}

// Configure P499 transducer
#[tauri::command]
fn configure_transducer(
    config: P499Configuration,
    state: State<AppState>
) -> Result<String, String> {
    let mut interface = state.p499_interface.lock().unwrap();
    interface.add_transducer(config);
    Ok("Transducer configured successfully".to_string())
}

// Read pressure from P499 transducer
#[tauri::command]
fn read_transducer(
    channel: u8,
    state: State<AppState>
) -> Result<TransducerReading, String> {
    let interface = state.p499_interface.lock().unwrap();
    interface.read_transducer(channel)
        .map_err(|e| e.to_string())
}

// Read all configured transducers
#[tauri::command]
fn read_all_transducers(state: State<AppState>) -> Vec<TransducerReading> {
    let interface = state.p499_interface.lock().unwrap();
    interface.read_all_transducers()
        .into_iter()
        .filter_map(|result| result.ok())
        .collect()
}

// Configure system for diagnostics
#[tauri::command]
fn configure_system(
    system_id: String,
    config: SystemConfiguration,
    state: State<AppState>
) -> Result<String, String> {
    let mut configs = state.system_configs.lock().unwrap();
    configs.insert(system_id, config);
    Ok("System configured successfully".to_string())
}

// Perform diagnostic analysis
#[tauri::command]
fn analyze_system(
    system_id: String,
    reading: DiagnosticReading,
    state: State<AppState>
) -> Result<DiagnosticResult, String> {
    let configs = state.system_configs.lock().unwrap();
    let config = configs.get(&system_id)
        .ok_or("System configuration not found")?;
    
    let result = state.diagnostic_engine.analyze_system(config, &reading)?;
    
    // Store reading for trend analysis
    let mut readings = state.current_readings.lock().unwrap();
    readings.insert(system_id.clone(), reading);
    
    Ok(result)
}

// Get pressure-temperature data for a refrigerant
#[tauri::command]
fn get_pt_data(
    refrigerant: String,
    state: State<AppState>
) -> Result<Vec<(f32, f32)>, String> {
    state.refrigerant_db
        .get_pt_data(&refrigerant)
        .map(|data| {
            data.iter()
                .map(|pt| (pt.temperature_f, pt.pressure_psi))
                .collect()
        })
        .ok_or_else(|| format!("No P-T data for refrigerant {}", refrigerant))
}

// Search refrigerants by GWP
#[tauri::command]
fn search_by_gwp(
    max_gwp: i32,
    state: State<AppState>
) -> Vec<RefrigerantInfo> {
    let refrigerants = state.refrigerant_db.search_by_gwp(max_gwp);
    
    refrigerants.into_iter()
        .filter_map(|designation| {
            state.refrigerant_db.get_refrigerant(&designation).map(|props| {
                RefrigerantInfo {
                    designation: props.designation.clone(),
                    chemical_name: props.chemical_name.clone(),
                    safety_class: props.safety_class.clone(),
                    gwp: props.gwp,
                    applications: props.applications.clone(),
                }
            })
        })
        .collect()
}

// Search refrigerants by safety class
#[tauri::command]
fn search_by_safety_class(
    safety_class: String,
    state: State<AppState>
) -> Vec<RefrigerantInfo> {
    let refrigerants = state.refrigerant_db.search_by_safety_class(&safety_class);
    
    refrigerants.into_iter()
        .filter_map(|designation| {
            state.refrigerant_db.get_refrigerant(&designation).map(|props| {
                RefrigerantInfo {
                    designation: props.designation.clone(),
                    chemical_name: props.chemical_name.clone(),
                    safety_class: props.safety_class.clone(),
                    gwp: props.gwp,
                    applications: props.applications.clone(),
                }
            })
        })
        .collect()
}

// Initialize P499 HAT interface
#[tauri::command]
fn initialize_hat(stack_level: u8) -> Result<String, String> {
    // Test Python interface
    use std::process::Command;
    
    let output = Command::new("python3")
        .arg("-c")
        .arg("import sm_4_20ma; print('HAT interface available')")
        .output()
        .map_err(|e| format!("Failed to access HAT: {}", e))?;
    
    if output.status.success() {
        Ok(format!("HAT initialized at stack level {}", stack_level))
    } else {
        Err(format!("HAT initialization failed: {}", 
            String::from_utf8_lossy(&output.stderr)))
    }
}

// Test simulated system
#[tauri::command]
fn simulate_reading() -> DiagnosticReading {
    use std::time::{SystemTime, UNIX_EPOCH};
    
    DiagnosticReading {
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        suction_pressure: 125.0,  // Typical R-410A
        discharge_pressure: 385.0,
        suction_temperature: 55.0,
        discharge_temperature: 120.0,
        liquid_line_temperature: 105.0,
        ambient_temperature: 95.0,
        indoor_wet_bulb: Some(65.0),
        indoor_dry_bulb: Some(75.0),
    }
}

fn main() {
    println!("====================================");
    println!("AutomataNexus Refrigerant Monitor");
    println!("Supporting 100+ Refrigerants");
    println!("====================================");
    
    let app_state = AppState {
        refrigerant_db: Arc::new(RefrigerantDatabase::new()),
        p499_interface: Arc::new(Mutex::new(P499Interface::new(0))),
        diagnostic_engine: Arc::new(DiagnosticEngine::new()),
        current_readings: Arc::new(Mutex::new(HashMap::new())),
        system_configs: Arc::new(Mutex::new(HashMap::new())),
    };
    
    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            list_refrigerants,
            get_refrigerant_properties,
            calculate_saturation_temp,
            configure_transducer,
            read_transducer,
            read_all_transducers,
            configure_system,
            analyze_system,
            get_pt_data,
            search_by_gwp,
            search_by_safety_class,
            initialize_hat,
            simulate_reading,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}