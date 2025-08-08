// COMPLETE UI Integration - Combines all 13 completed UI modules
// This file integrates all the complete UI implementations into the main app

use eframe::egui;
use egui::{Color32, RichText, ScrollArea, Ui};
use crate::state::AppState;
use std::sync::Arc;
use tokio::sync::Mutex;

// Import all complete UI modules
use super::io_control_complete::IOControlComplete;
use super::admin_complete::AdminPanelComplete;
use super::live_monitor_complete::LiveMonitor;
use super::vibration_complete::VibrationMonitor;
use super::refrigerant_complete::RefrigerantDiagnostics;
use super::database_complete::DatabaseViewer;
use super::board_config_complete::BoardConfiguration;
use super::logic_engine_complete::LogicEngine;
use super::firmware_complete::FirmwareManager;
use super::bms_complete::BmsIntegration;
use super::processing_complete::ProtocolManager;
use super::metrics_complete::MetricsVisualization;
use super::maintenance_complete::MaintenanceManager;

pub struct CompleteUISystem {
    // Core modules
    pub io_control: IOControlComplete,
    pub admin_panel: AdminPanelComplete,
    pub live_monitor: LiveMonitor,
    
    // Specialized modules
    pub vibration: VibrationMonitor,
    pub refrigerant: RefrigerantDiagnostics,
    pub database: DatabaseViewer,
    pub board_config: BoardConfiguration,
    pub logic_engine: LogicEngine,
    pub firmware: FirmwareManager,
    pub bms: BmsIntegration,
    pub protocols: ProtocolManager,
    pub metrics: MetricsVisualization,
    pub maintenance: MaintenanceManager,
    
    // UI state
    pub selected_tab: String,
    pub board_id: String,
    pub is_admin_mode: bool,
}

impl CompleteUISystem {
    pub fn new(app_state: Arc<Mutex<AppState>>) -> Self {
        Self {
            // Initialize all modules with defaults
            io_control: IOControlComplete::default(),
            admin_panel: AdminPanelComplete::default(),
            live_monitor: LiveMonitor::default(),
            vibration: VibrationMonitor::default(),
            refrigerant: RefrigerantDiagnostics::default(),
            database: DatabaseViewer::default(),
            board_config: BoardConfiguration::default(),
            logic_engine: LogicEngine::default(),
            firmware: FirmwareManager::default(),
            bms: BmsIntegration::default(),
            protocols: ProtocolManager::default(),
            metrics: MetricsVisualization::default(),
            maintenance: MaintenanceManager::default(),
            
            selected_tab: "io_control".to_string(),
            board_id: "megabas_0".to_string(),
            is_admin_mode: false,
        }
    }
    
    pub fn show(&mut self, ui: &mut egui::Ui) {
        // Top Navigation Bar
        ui.horizontal(|ui| {
            ui.add_space(10.0);
            
            // Main tabs
            self.show_tab_button(ui, "io_control", "🎛️ I/O Control");
            self.show_tab_button(ui, "live_monitor", "📊 Live Monitor");
            self.show_tab_button(ui, "vibration", "📳 Vibration");
            self.show_tab_button(ui, "refrigerant", "❄️ Refrigerant");
            self.show_tab_button(ui, "database", "💾 Database");
            self.show_tab_button(ui, "board_config", "⚙️ Board Config");
            
            ui.separator();
            
            // Advanced tabs
            self.show_tab_button(ui, "logic", "🔧 Logic Engine");
            self.show_tab_button(ui, "firmware", "📦 Firmware");
            self.show_tab_button(ui, "bms", "🏢 BMS");
            self.show_tab_button(ui, "protocols", "🔌 Protocols");
            self.show_tab_button(ui, "metrics", "📈 Metrics");
            self.show_tab_button(ui, "maintenance", "🔧 Maintenance");
            
            ui.separator();
            
            // Admin panel (always visible as a button)
            if ui.selectable_label(
                self.selected_tab == "admin",
                RichText::new("👤 Admin").color(Color32::from_rgb(234, 88, 12))
            ).clicked() {
                self.selected_tab = "admin".to_string();
            }
            
            // Right-aligned status
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if self.maintenance.maintenance_status.enabled {
                    ui.colored_label(
                        Color32::from_rgb(234, 88, 12),
                        "⚠️ MAINTENANCE MODE"
                    );
                }
                
                if self.bms.connection_status.connected {
                    ui.colored_label(
                        Color32::from_rgb(34, 197, 94),
                        "● BMS Connected"
                    );
                } else {
                    ui.colored_label(
                        Color32::from_rgb(100, 116, 139),
                        "● Local Control"
                    );
                }
            });
        });
        
        ui.separator();
        ui.add_space(10.0);
        
        // Main content area with scroll
        ScrollArea::vertical().show(ui, |ui| {
            match self.selected_tab.as_str() {
                "io_control" => {
                    self.io_control.show(ui);
                }
                "admin" => {
                    self.admin_panel.show(ui);
                }
                "live_monitor" => {
                    self.live_monitor.show(ui);
                }
                "vibration" => {
                    self.vibration.show(ui);
                }
                "refrigerant" => {
                    self.refrigerant.show(ui);
                }
                "database" => {
                    self.database.show(ui, &self.board_id);
                }
                "board_config" => {
                    self.board_config.show(ui);
                }
                "logic" => {
                    self.logic_engine.show(ui);
                }
                "firmware" => {
                    self.firmware.show(ui);
                }
                "bms" => {
                    self.bms.show(ui, &self.board_id);
                }
                "protocols" => {
                    self.protocols.show(ui);
                }
                "metrics" => {
                    self.metrics.show(ui, &self.board_id);
                }
                "maintenance" => {
                    self.maintenance.show(ui);
                }
                _ => {
                    ui.vertical_centered(|ui| {
                        ui.add_space(50.0);
                        ui.label(RichText::new("Select a tab to view")
                            .color(Color32::from_rgb(100, 116, 139))
                            .size(16.0));
                    });
                }
            }
        });
        
        // Footer status bar
        ui.add_space(10.0);
        ui.separator();
        ui.horizontal(|ui| {
            ui.label(RichText::new(format!("Board: {}", self.board_id))
                .color(Color32::from_rgb(100, 116, 139))
                .size(11.0));
            
            ui.separator();
            
            ui.label(RichText::new(format!("Location: {}", self.bms.config.location_name))
                .color(Color32::from_rgb(100, 116, 139))
                .size(11.0));
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(RichText::new(format!("© 2024 Automata Nexus Controller"))
                    .color(Color32::from_rgb(156, 163, 175))
                    .size(10.0));
            });
        });
    }
    
    fn show_tab_button(&mut self, ui: &mut egui::Ui, tab_id: &str, label: &str) {
        let is_selected = self.selected_tab == tab_id;
        
        let button_color = if is_selected {
            Color32::from_rgb(20, 184, 166) // Teal for selected
        } else {
            Color32::from_rgb(15, 23, 42) // Dark for unselected
        };
        
        if ui.selectable_label(
            is_selected,
            RichText::new(label).color(button_color)
        ).clicked() {
            self.selected_tab = tab_id.to_string();
        }
    }
    
    // Public methods for external control
    pub fn set_maintenance_mode(&mut self, enabled: bool) {
        if enabled {
            self.maintenance.maintenance_status.enabled = true;
        } else {
            self.maintenance.disable_maintenance_mode();
        }
    }
    
    pub fn get_bms_status(&self) -> bool {
        self.bms.connection_status.connected
    }
    
    pub fn refresh_all_data(&mut self) {
        // Trigger refresh on all modules that support it
        self.live_monitor.refresh_data();
        self.metrics.fetch_trend_data();
        self.vibration.update_sensor_data();
        self.refrigerant.update_diagnostics();
        self.protocols.refresh_protocols();
    }
}

// Re-export for use in main app
pub use CompleteUISystem as CompleteUI;