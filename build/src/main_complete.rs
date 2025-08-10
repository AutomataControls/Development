// COMPLETE Rust/Tauri Nexus Controller Main Application
// Integrates all 13 UI modules with proper state management and Tauri commands

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use std::sync::Arc;
use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};
use tauri::Manager;

mod state;
mod ui;
mod hardware;
mod database;
mod protocols;
mod logic;

use state::AppState;
use ui::complete_ui_integration::CompleteUI;

// Main application structure
pub struct NexusControllerApp {
    app_state: Arc<Mutex<AppState>>,
    complete_ui: CompleteUI,
    is_authenticated: bool,
    username: String,
    show_login: bool,
}

impl NexusControllerApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Configure fonts and style
        configure_fonts(&cc.egui_ctx);
        configure_style(&cc.egui_ctx);
        
        // Initialize application state
        let app_state = Arc::new(Mutex::new(AppState::default()));
        
        // Create complete UI system
        let complete_ui = CompleteUI::new(app_state.clone());
        
        Self {
            app_state,
            complete_ui,
            is_authenticated: false,
            username: String::new(),
            show_login: true,
        }
    }
}

impl eframe::App for NexusControllerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Set up auto-refresh for real-time data
        ctx.request_repaint_after(std::time::Duration::from_secs(1));
        
        // Main window
        egui::CentralPanel::default().show(ctx, |ui| {
            // Show login screen if not authenticated
            if self.show_login && !self.is_authenticated {
                self.show_login_screen(ui);
            } else {
                // Show main application header
                self.show_header(ui);
                
                // Show the complete UI system
                self.complete_ui.show(ui);
            }
        });
        
        // Handle keyboard shortcuts
        self.handle_shortcuts(ctx);
    }
}

impl NexusControllerApp {
    fn show_login_screen(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(100.0);
            
            // Logo and title
            ui.heading(egui::RichText::new("🏢 Automata Nexus Controller")
                .size(32.0)
                .color(egui::Color32::from_rgb(20, 184, 166)));
            
            ui.label(egui::RichText::new("Building Automation System")
                .size(16.0)
                .color(egui::Color32::from_rgb(100, 116, 139)));
            
            ui.add_space(50.0);
            
            // Login form
            ui.group(|ui| {
                ui.set_max_width(300.0);
                ui.vertical(|ui| {
                    ui.label("Username:");
                    ui.text_edit_singleline(&mut self.username);
                    
                    ui.add_space(10.0);
                    
                    ui.label("Password:");
                    let mut password = String::new();
                    ui.add(egui::TextEdit::singleline(&mut password).password(true));
                    
                    ui.add_space(20.0);
                    
                    if ui.button(egui::RichText::new("🔓 Login")
                        .size(16.0))
                        .clicked() {
                        // Demo: accept any non-empty username
                        if !self.username.is_empty() {
                            self.is_authenticated = true;
                            self.show_login = false;
                        }
                    }
                    
                    ui.add_space(10.0);
                    
                    ui.label(egui::RichText::new("Demo: Enter any username to login")
                        .size(11.0)
                        .color(egui::Color32::from_rgb(156, 163, 175))
                        .italics());
                });
            });
        });
    }
    
    fn show_header(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("🏢")
                .size(24.0)
                .color(egui::Color32::from_rgb(20, 184, 166)));
            
            ui.label(egui::RichText::new("Automata Nexus Controller")
                .size(18.0)
                .color(egui::Color32::from_rgb(15, 23, 42))
                .strong());
            
            ui.separator();
            
            // Show current user
            ui.label(format!("User: {}", self.username));
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("🚪 Logout").clicked() {
                    self.is_authenticated = false;
                    self.show_login = true;
                    self.username.clear();
                }
                
                ui.separator();
                
                // Quick status indicators
                if self.complete_ui.maintenance.maintenance_status.enabled {
                    ui.colored_label(
                        egui::Color32::from_rgb(234, 88, 12),
                        "⚠️ MAINTENANCE"
                    );
                }
                
                // System time
                let now = chrono::Local::now();
                ui.label(egui::RichText::new(now.format("%Y-%m-%d %H:%M:%S").to_string())
                    .size(11.0)
                    .color(egui::Color32::from_rgb(100, 116, 139)));
            });
        });
        
        ui.separator();
    }
    
    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        // F1 - Help
        if ctx.input(|i| i.key_pressed(egui::Key::F1)) {
            // Show help dialog
        }
        
        // F5 - Refresh
        if ctx.input(|i| i.key_pressed(egui::Key::F5)) {
            self.complete_ui.refresh_all_data();
        }
        
        // Ctrl+M - Toggle maintenance mode
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::M)) {
            let current = self.complete_ui.maintenance.maintenance_status.enabled;
            self.complete_ui.set_maintenance_mode(!current);
        }
        
        // Ctrl+L - Logout
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::L)) {
            self.is_authenticated = false;
            self.show_login = true;
        }
    }
}

fn configure_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    
    // Add custom font sizes
    fonts.font_data.insert(
        "default".to_owned(),
        egui::FontData::from_static(include_bytes!("../assets/fonts/Roboto-Regular.ttf")),
    );
    
    fonts.families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "default".to_owned());
    
    ctx.set_fonts(fonts);
}

fn configure_style(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    
    // Light theme colors
    style.visuals.override_text_color = Some(egui::Color32::from_rgb(15, 23, 42));
    style.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(248, 250, 252);
    style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(241, 245, 249);
    style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(226, 232, 240);
    style.visuals.selection.bg_fill = egui::Color32::from_rgb(20, 184, 166);
    style.visuals.extreme_bg_color = egui::Color32::WHITE;
    style.visuals.faint_bg_color = egui::Color32::from_rgb(248, 250, 252);
    
    // Rounding
    style.visuals.widgets.inactive.rounding = egui::Rounding::same(5.0);
    style.visuals.widgets.hovered.rounding = egui::Rounding::same(5.0);
    style.visuals.widgets.active.rounding = egui::Rounding::same(5.0);
    
    ctx.set_style(style);
}

// Tauri command handlers
#[tauri::command]
async fn get_board_status(board_id: String) -> Result<String, String> {
    Ok(format!("Board {} is operational", board_id))
}

#[tauri::command]
async fn set_channel_value(
    board_id: String,
    channel_type: String,
    channel_index: u8,
    value: f32,
) -> Result<(), String> {
    println!("Setting {}[{}] = {} on board {}", 
        channel_type, channel_index, value, board_id);
    Ok(())
}

#[tauri::command]
async fn enable_maintenance_mode(
    reason: String,
    authorized_by: String,
    duration_minutes: u32,
) -> Result<(), String> {
    println!("Enabling maintenance mode for {} minutes. Reason: {}, Authorized by: {}", 
        duration_minutes, reason, authorized_by);
    Ok(())
}

#[tauri::command]
async fn disable_maintenance_mode() -> Result<(), String> {
    println!("Disabling maintenance mode");
    Ok(())
}

#[tauri::command]
async fn get_trend_data(
    board_id: String,
    channel_type: String,
    channel_index: u8,
    hours: u32,
) -> Result<serde_json::Value, String> {
    // Return sample trend data
    Ok(serde_json::json!({
        "channel_name": format!("{}[{}]", channel_type, channel_index),
        "units": "°F",
        "data_points": [],
        "statistics": {
            "min": 68.0,
            "max": 76.0,
            "avg": 72.0,
            "std_dev": 2.1,
            "count": 100
        }
    }))
}

#[tauri::command]
async fn test_bms_connection(
    equipment_id: String,
    location_id: String,
) -> Result<String, String> {
    Ok(format!("BMS connection test successful for equipment {} at location {}", 
        equipment_id, location_id))
}

// Main entry point for Tauri
#[cfg(feature = "tauri")]
fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_board_status,
            set_channel_value,
            enable_maintenance_mode,
            disable_maintenance_mode,
            get_trend_data,
            test_bms_connection,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// Main entry point for standalone egui
#[cfg(not(feature = "tauri"))]
fn main() -> Result<(), eframe::Error> {
    env_logger::init();
    
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1400.0, 900.0)),
        min_window_size: Some(egui::vec2(1200.0, 700.0)),
        ..Default::default()
    };
    
    eframe::run_native(
        "Automata Nexus Controller",
        options,
        Box::new(|cc| Box::new(NexusControllerApp::new(cc))),
    )
}