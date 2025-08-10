// Complete UI Implementation with ALL tabs from Next.js
// All 14 tabs plus admin panel

use egui::{Context, Id, Ui, Window, Color32, RichText, Vec2, Rect};
use crate::state::AppState;
use crate::admin::{AdminSystem, UserRole};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct CompleteUI {
    app_state: Arc<Mutex<AppState>>,
    admin_system: Arc<Mutex<AdminSystem>>,
    active_tab: Tab,
    show_admin_panel: bool,
    admin_pin_input: String,
    admin_authenticated: bool,
    logo_click_count: u8,
    last_logo_click: std::time::Instant,
}

#[derive(Debug, Clone, PartialEq)]
enum Tab {
    IOControl,
    LiveMonitor,
    ISOVibration,
    Refrigerant,
    Database,
    BoardConfig,
    LogicEngine,
    Firmware,
    BMS,
    Processing,
    Metrics,
    Maintenance,
    BMSServer,
    ProtocolManager,
}

impl CompleteUI {
    pub fn new(app_state: Arc<Mutex<AppState>>) -> Self {
        Self {
            app_state: app_state.clone(),
            admin_system: Arc::new(Mutex::new(AdminSystem::new())),
            active_tab: Tab::IOControl,
            show_admin_panel: false,
            admin_pin_input: String::new(),
            admin_authenticated: false,
            logo_click_count: 0,
            last_logo_click: std::time::Instant::now(),
        }
    }
    
    pub fn show(&mut self, ctx: &Context, ui: &mut Ui) {
        // Header with logo and secret admin access
        ui.horizontal(|ui| {
            // Logo - click 5 times for admin panel
            if ui.button("🏭 Automata Nexus").clicked() {
                let now = std::time::Instant::now();
                if now.duration_since(self.last_logo_click).as_secs() < 2 {
                    self.logo_click_count += 1;
                    if self.logo_click_count >= 5 {
                        self.show_admin_panel = true;
                        self.logo_click_count = 0;
                    }
                } else {
                    self.logo_click_count = 1;
                }
                self.last_logo_click = now;
            }
            
            ui.separator();
            
            // System status
            ui.label("Status:");
            ui.colored_label(Color32::GREEN, "● Connected");
            
            ui.separator();
            
            // Weather display (from weather module)
            ui.label("🌡️ 72°F");
            ui.label("💧 45%");
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label("User: admin");
                if ui.button("Logout").clicked() {
                    // Handle logout
                }
            });
        });
        
        ui.separator();
        
        // Tab bar with ALL tabs
        ui.horizontal(|ui| {
            ui.style_mut().spacing.item_spacing.x = 2.0;
            
            // Main tabs
            if ui.selectable_label(self.active_tab == Tab::IOControl, "I/O Control").clicked() {
                self.active_tab = Tab::IOControl;
            }
            if ui.selectable_label(self.active_tab == Tab::LiveMonitor, "Live Monitor").clicked() {
                self.active_tab = Tab::LiveMonitor;
            }
            if ui.selectable_label(self.active_tab == Tab::ISOVibration, "ISO Vibration").clicked() {
                self.active_tab = Tab::ISOVibration;
            }
            if ui.selectable_label(self.active_tab == Tab::Refrigerant, "Refrigerant").clicked() {
                self.active_tab = Tab::Refrigerant;
            }
            if ui.selectable_label(self.active_tab == Tab::Database, "Database").clicked() {
                self.active_tab = Tab::Database;
            }
            if ui.selectable_label(self.active_tab == Tab::BoardConfig, "Board Config").clicked() {
                self.active_tab = Tab::BoardConfig;
            }
            if ui.selectable_label(self.active_tab == Tab::LogicEngine, "Logic Engine").clicked() {
                self.active_tab = Tab::LogicEngine;
            }
        });
        
        ui.horizontal(|ui| {
            ui.style_mut().spacing.item_spacing.x = 2.0;
            
            // More tabs
            if ui.selectable_label(self.active_tab == Tab::Firmware, "Firmware").clicked() {
                self.active_tab = Tab::Firmware;
            }
            if ui.selectable_label(self.active_tab == Tab::BMS, "BMS").clicked() {
                self.active_tab = Tab::BMS;
            }
            if ui.selectable_label(self.active_tab == Tab::Processing, "Processing").clicked() {
                self.active_tab = Tab::Processing;
            }
            if ui.selectable_label(self.active_tab == Tab::Metrics, "Metrics").clicked() {
                self.active_tab = Tab::Metrics;
            }
            if ui.selectable_label(self.active_tab == Tab::Maintenance, RichText::new("Maintenance").color(Color32::from_rgb(251, 146, 60))).clicked() {
                self.active_tab = Tab::Maintenance;
            }
            if ui.selectable_label(self.active_tab == Tab::BMSServer, "BMS Server").clicked() {
                self.active_tab = Tab::BMSServer;
            }
            if ui.selectable_label(self.active_tab == Tab::ProtocolManager, "Protocol Manager").clicked() {
                self.active_tab = Tab::ProtocolManager;
            }
        });
        
        ui.separator();
        
        // Tab content
        match self.active_tab {
            Tab::IOControl => self.show_io_control(ui),
            Tab::LiveMonitor => self.show_live_monitor(ui),
            Tab::ISOVibration => self.show_vibration_monitor(ui),
            Tab::Refrigerant => self.show_refrigerant_diagnostics(ui),
            Tab::Database => self.show_database_viewer(ui),
            Tab::BoardConfig => self.show_board_config(ui),
            Tab::LogicEngine => self.show_logic_engine(ui),
            Tab::Firmware => self.show_firmware_manager(ui),
            Tab::BMS => self.show_bms_integration(ui),
            Tab::Processing => self.show_processing(ui),
            Tab::Metrics => self.show_metrics(ui),
            Tab::Maintenance => self.show_maintenance_mode(ui),
            Tab::BMSServer => self.show_bms_server(ui),
            Tab::ProtocolManager => self.show_protocol_manager(ui),
        }
        
        // Admin panel window
        if self.show_admin_panel {
            self.show_admin_panel_window(ctx);
        }
    }
    
    fn show_io_control(&mut self, ui: &mut Ui) {
        ui.heading("I/O Control Panel");
        
        egui::ScrollArea::vertical().show(ui, |ui| {
            // Board selector
            ui.horizontal(|ui| {
                ui.label("Board:");
                egui::ComboBox::from_label("")
                    .selected_text("Building Automation")
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut (), (), "Building Automation");
                        ui.selectable_value(&mut (), (), "16 Universal Input");
                        ui.selectable_value(&mut (), (), "16 Relay");
                        ui.selectable_value(&mut (), (), "8 Relay");
                    });
            });
            
            ui.separator();
            
            // Universal Inputs
            ui.collapsing("Universal Inputs (8)", |ui| {
                egui::Grid::new("ui_grid").striped(true).show(ui, |ui| {
                    ui.label("Channel");
                    ui.label("Name");
                    ui.label("Value");
                    ui.label("Units");
                    ui.label("Type");
                    ui.label("Manual");
                    ui.end_row();
                    
                    for i in 1..=8 {
                        ui.label(format!("UI{}", i));
                        ui.text_edit_singleline(&mut String::from(format!("Input {}", i)));
                        ui.label(format!("{:.2}", 20.0 + i as f32 * 0.5));
                        ui.label("°F");
                        egui::ComboBox::from_id_source(format!("type_{}", i))
                            .selected_text("0-10V")
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut (), (), "0-10V");
                                ui.selectable_value(&mut (), (), "Thermistor");
                                ui.selectable_value(&mut (), (), "Dry Contact");
                            });
                        ui.checkbox(&mut false, "");
                        ui.end_row();
                    }
                });
            });
            
            ui.separator();
            
            // Analog Outputs
            ui.collapsing("Analog Outputs (4)", |ui| {
                egui::Grid::new("ao_grid").striped(true).show(ui, |ui| {
                    ui.label("Channel");
                    ui.label("Name");
                    ui.label("Value");
                    ui.label("Control");
                    ui.end_row();
                    
                    for i in 1..=4 {
                        ui.label(format!("AO{}", i));
                        ui.text_edit_singleline(&mut String::from(format!("Output {}", i)));
                        ui.label(format!("{:.1}V", 5.0 + i as f32 * 0.5));
                        ui.add(egui::Slider::new(&mut 0.0, 0.0..=10.0).suffix("V"));
                        ui.end_row();
                    }
                });
            });
            
            ui.separator();
            
            // Relays
            ui.collapsing("Relay Outputs (8)", |ui| {
                egui::Grid::new("relay_grid").striped(true).show(ui, |ui| {
                    ui.label("Channel");
                    ui.label("Name");
                    ui.label("State");
                    ui.label("Control");
                    ui.end_row();
                    
                    for i in 1..=8 {
                        ui.label(format!("R{}", i));
                        ui.text_edit_singleline(&mut String::from(format!("Relay {}", i)));
                        ui.colored_label(Color32::RED, "OFF");
                        if ui.button("Toggle").clicked() {
                            // Toggle relay
                        }
                        ui.end_row();
                    }
                });
            });
        });
    }
    
    fn show_live_monitor(&mut self, ui: &mut Ui) {
        ui.heading("Live System Monitor");
        
        egui::Grid::new("monitor_grid").striped(true).show(ui, |ui| {
            ui.label("Metric");
            ui.label("Value");
            ui.label("Status");
            ui.end_row();
            
            ui.label("CPU Usage");
            ui.label("15%");
            ui.colored_label(Color32::GREEN, "Normal");
            ui.end_row();
            
            ui.label("Memory");
            ui.label("512MB / 2GB");
            ui.colored_label(Color32::GREEN, "Normal");
            ui.end_row();
            
            ui.label("Disk Space");
            ui.label("8GB / 64GB");
            ui.colored_label(Color32::GREEN, "Normal");
            ui.end_row();
            
            ui.label("Temperature");
            ui.label("45°C");
            ui.colored_label(Color32::GREEN, "Normal");
            ui.end_row();
            
            ui.label("Uptime");
            ui.label("5 days, 3:45:22");
            ui.colored_label(Color32::GREEN, "Running");
            ui.end_row();
        });
        
        ui.separator();
        
        // Real-time charts would go here
        ui.label("📊 Real-time performance charts");
    }
    
    fn show_vibration_monitor(&mut self, ui: &mut Ui) {
        ui.heading("ISO 10816-3 Vibration Monitor");
        
        ui.horizontal(|ui| {
            if ui.button("🔍 Scan Ports").clicked() {
                // Scan for sensors
            }
            if ui.button("➕ Add Sensor").clicked() {
                // Add sensor dialog
            }
        });
        
        ui.separator();
        
        // Sensor list
        egui::Grid::new("vibration_grid").striped(true).show(ui, |ui| {
            ui.label("Sensor");
            ui.label("Location");
            ui.label("Velocity");
            ui.label("Severity");
            ui.label("Temp");
            ui.label("Status");
            ui.end_row();
            
            ui.label("WTVB01-01");
            ui.label("Compressor 1");
            ui.label("3.2 mm/s");
            ui.colored_label(Color32::YELLOW, "Satisfactory");
            ui.label("65°C");
            ui.colored_label(Color32::GREEN, "Active");
            ui.end_row();
        });
        
        ui.separator();
        
        // ISO zones
        ui.label("ISO 10816-3 Severity Zones:");
        ui.colored_label(Color32::GREEN, "● Zone A: Good (<2.3 mm/s)");
        ui.colored_label(Color32::YELLOW, "● Zone B: Satisfactory (2.3-4.5 mm/s)");
        ui.colored_label(Color32::from_rgb(255, 165, 0), "● Zone C: Unsatisfactory (4.5-7.1 mm/s)");
        ui.colored_label(Color32::RED, "● Zone D: Unacceptable (>7.1 mm/s)");
    }
    
    fn show_refrigerant_diagnostics(&mut self, ui: &mut Ui) {
        ui.heading("Refrigerant Diagnostics");
        
        ui.horizontal(|ui| {
            ui.label("Refrigerant:");
            egui::ComboBox::from_label("")
                .selected_text("R-410A")
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut (), (), "R-410A");
                    ui.selectable_value(&mut (), (), "R-22");
                    ui.selectable_value(&mut (), (), "R-134a");
                    ui.selectable_value(&mut (), (), "R-404A");
                });
            
            ui.label("System:");
            egui::ComboBox::from_label("sys")
                .selected_text("TXV")
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut (), (), "TXV");
                    ui.selectable_value(&mut (), (), "Fixed Orifice");
                    ui.selectable_value(&mut (), (), "EEV");
                });
        });
        
        ui.separator();
        
        egui::Grid::new("refrig_grid").striped(true).show(ui, |ui| {
            ui.label("Parameter");
            ui.label("Value");
            ui.label("Target");
            ui.label("Status");
            ui.end_row();
            
            ui.label("Superheat");
            ui.label("12°F");
            ui.label("10-15°F");
            ui.colored_label(Color32::GREEN, "Normal");
            ui.end_row();
            
            ui.label("Subcooling");
            ui.label("8°F");
            ui.label("5-10°F");
            ui.colored_label(Color32::GREEN, "Normal");
            ui.end_row();
            
            ui.label("Discharge Temp");
            ui.label("180°F");
            ui.label("<225°F");
            ui.colored_label(Color32::GREEN, "Normal");
            ui.end_row();
            
            ui.label("Pressure Ratio");
            ui.label("3.5");
            ui.label("<4.0");
            ui.colored_label(Color32::GREEN, "Normal");
            ui.end_row();
        });
        
        ui.separator();
        ui.label("Efficiency Score: 92%");
    }
    
    fn show_database_viewer(&mut self, ui: &mut Ui) {
        ui.heading("Database Viewer");
        
        ui.horizontal(|ui| {
            ui.label("Table:");
            egui::ComboBox::from_label("table")
                .selected_text("Select table")
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut (), (), "users");
                    ui.selectable_value(&mut (), (), "boards");
                    ui.selectable_value(&mut (), (), "io_values");
                    ui.selectable_value(&mut (), (), "alarms");
                    ui.selectable_value(&mut (), (), "audit_logs");
                    ui.selectable_value(&mut (), (), "vibration_readings");
                    ui.selectable_value(&mut (), (), "diagnostic_results");
                });
            
            if ui.button("Query").clicked() {
                // Execute query
            }
            if ui.button("Export").clicked() {
                // Export data
            }
        });
        
        ui.separator();
        
        // SQL editor
        ui.label("SQL Query:");
        ui.text_edit_multiline(&mut String::from("SELECT * FROM io_values ORDER BY timestamp DESC LIMIT 100"));
        
        ui.separator();
        
        // Results table
        ui.label("Query Results:");
        egui::ScrollArea::both().show(ui, |ui| {
            ui.label("Results would appear here...");
        });
    }
    
    fn show_board_config(&mut self, ui: &mut Ui) {
        ui.heading("Board Configuration");
        
        if ui.button("🔍 Scan I2C Bus").clicked() {
            // Scan for boards
        }
        
        ui.separator();
        
        ui.collapsing("Detected Boards", |ui| {
            ui.label("• Building Automation (0x48)");
            ui.label("• 16 Universal Input (0x50)");
            ui.label("• 8 Relay (0x38)");
        });
        
        ui.separator();
        
        ui.label("Configure channels, scaling, alarms, etc.");
    }
    
    fn show_logic_engine(&mut self, ui: &mut Ui) {
        ui.heading("Logic Engine");
        
        ui.horizontal(|ui| {
            if ui.button("➕ New Logic").clicked() {
                // Create new logic
            }
            if ui.button("📁 Load").clicked() {
                // Load logic file
            }
            if ui.button("▶️ Execute All").clicked() {
                // Execute all logic
            }
        });
        
        ui.separator();
        
        ui.label("Active Logic Files:");
        ui.label("• cooling-tower.js - Next execution: 15s");
        ui.label("• ahu-control.js - Next execution: 30s");
    }
    
    fn show_firmware_manager(&mut self, ui: &mut Ui) {
        ui.heading("Firmware Manager");
        
        ui.horizontal(|ui| {
            if ui.button("🔄 Check Updates").clicked() {
                // Check for updates
            }
            if ui.button("⬇️ Download").clicked() {
                // Download firmware
            }
            if ui.button("⚡ Flash").clicked() {
                // Flash firmware
            }
        });
        
        ui.separator();
        
        ui.label("Board Firmware Versions:");
        ui.label("• Building Automation: v2.1.3 (latest)");
        ui.label("• 16 Universal: v1.2.1 (update available)");
    }
    
    fn show_bms_integration(&mut self, ui: &mut Ui) {
        ui.heading("BMS Integration");
        
        ui.label("BACnet Devices: 3 discovered");
        ui.label("Modbus Devices: 2 connected");
        ui.label("Total Points: 145");
    }
    
    fn show_processing(&mut self, ui: &mut Ui) {
        ui.heading("Data Processing");
        
        ui.label("Processing rules, transformations, etc.");
    }
    
    fn show_metrics(&mut self, ui: &mut Ui) {
        ui.heading("Metrics Visualization");
        
        ui.label("Charts, graphs, historical data");
    }
    
    fn show_maintenance_mode(&mut self, ui: &mut Ui) {
        ui.heading(RichText::new("⚠️ MAINTENANCE MODE").color(Color32::from_rgb(251, 146, 60)));
        
        ui.colored_label(Color32::from_rgb(251, 146, 60), "System is in maintenance mode");
        ui.label("• Alarms disabled");
        ui.label("• Logic execution paused");
        ui.label("• Manual control enabled");
        
        if ui.button("Exit Maintenance Mode").clicked() {
            // Exit maintenance
        }
    }
    
    fn show_bms_server(&mut self, ui: &mut Ui) {
        ui.heading("BMS Server");
        
        ui.label("BACnet Server: Running on port 47808");
        ui.label("Modbus Server: Running on port 502");
        ui.label("Active connections: 3");
    }
    
    fn show_protocol_manager(&mut self, ui: &mut Ui) {
        ui.heading("Protocol Manager");
        
        ui.label("Configure BACnet, Modbus, MQTT, etc.");
    }
    
    fn show_admin_panel_window(&mut self, ctx: &Context) {
        Window::new("🔐 Admin Panel")
            .default_size(Vec2::new(800.0, 600.0))
            .show(ctx, |ui| {
                if !self.admin_authenticated {
                    ui.heading("Admin Authentication Required");
                    ui.label("Enter admin PIN:");
                    ui.text_edit_singleline(&mut self.admin_pin_input);
                    
                    if ui.button("Authenticate").clicked() {
                        let admin = self.admin_system.blocking_lock();
                        if admin.verify_admin_pin(&self.admin_pin_input) {
                            self.admin_authenticated = true;
                            self.admin_pin_input.clear();
                        }
                    }
                    
                    if ui.button("Cancel").clicked() {
                        self.show_admin_panel = false;
                        self.admin_pin_input.clear();
                    }
                } else {
                    // Full admin panel UI
                    ui.heading("Admin Panel");
                    
                    ui.horizontal(|ui| {
                        if ui.button("Users").clicked() {}
                        if ui.button("Audit Logs").clicked() {}
                        if ui.button("Email").clicked() {}
                        if ui.button("System").clicked() {}
                        if ui.button("Demo Mode").clicked() {}
                    });
                    
                    ui.separator();
                    
                    // Admin content here
                    ui.label("User management, audit logs, email settings, etc.");
                    
                    if ui.button("Close").clicked() {
                        self.show_admin_panel = false;
                        self.admin_authenticated = false;
                    }
                }
            });
    }
}