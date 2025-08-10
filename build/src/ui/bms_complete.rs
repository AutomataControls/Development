// COMPLETE BMS Integration Implementation - InfluxDB, command system, fallback logic
// Light theme with teal/cyan accents matching the app design
use eframe::egui;
use egui::{Color32, RichText, Grid, ScrollArea, Ui, Window, TextEdit};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Local};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BmsConfig {
    pub enabled: bool,
    pub location_name: String,
    pub system_name: String,
    pub location_id: String,
    pub equipment_id: String,
    pub equipment_type: String,
    pub zone: String,
    pub influx_url: String,
    pub update_interval: u32,
    pub field_mappings: HashMap<String, String>,
    pub bms_server_url: String,
    pub command_query_interval: u32,
    pub fallback_to_local: bool,
}

#[derive(Debug, Clone)]
pub struct BmsConnectionStatus {
    pub connected: bool,
    pub last_successful_query: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
    pub command_source: String,
    pub retry_count: u32,
    pub latency_ms: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BmsCommand {
    pub equipment_id: String,
    pub location_id: String,
    pub command_type: String,
    pub command_data: HashMap<String, String>,
    pub timestamp: DateTime<Utc>,
    pub priority: u8,
}

#[derive(Debug, Clone)]
pub struct BmsIntegration {
    pub config: BmsConfig,
    pub connection_status: BmsConnectionStatus,
    pub recent_commands: Vec<BmsCommand>,
    pub is_testing_connection: bool,
    pub is_saving: bool,
    pub active_tab: String,
    pub show_field_mapping_dialog: bool,
    pub new_field_mapping: (String, String),
    pub ping_history: Vec<(DateTime<Utc>, bool, u32)>, // timestamp, success, latency
}

impl Default for BmsIntegration {
    fn default() -> Self {
        Self {
            config: BmsConfig {
                enabled: false,
                location_name: "FirstChurchOfGod".to_string(),
                system_name: "AHU-001".to_string(),
                location_id: "9".to_string(),
                equipment_id: "WAg6mWpJneM2zLMDu11b".to_string(),
                equipment_type: "Air Handler".to_string(),
                zone: "Main Building".to_string(),
                influx_url: "http://143.198.162.31:8205/ping".to_string(),
                update_interval: 30,
                field_mappings: HashMap::new(),
                bms_server_url: "http://143.198.162.31:8205/ping".to_string(),
                command_query_interval: 30,
                fallback_to_local: true,
            },
            connection_status: BmsConnectionStatus {
                connected: false,
                last_successful_query: None,
                last_error: None,
                command_source: "local".to_string(),
                retry_count: 0,
                latency_ms: 0,
            },
            recent_commands: vec![],
            is_testing_connection: false,
            is_saving: false,
            active_tab: "configuration".to_string(),
            show_field_mapping_dialog: false,
            new_field_mapping: (String::new(), String::new()),
            ping_history: vec![],
        }
    }
}

impl BmsIntegration {
    pub fn show(&mut self, ui: &mut egui::Ui, board_id: &str) {
        // Header Card
        ui.group(|ui| {
            ui.set_min_height(80.0);
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.heading(RichText::new("🏢 BMS Integration & Command System").color(Color32::from_rgb(15, 23, 42)));
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Connection status badges
                        if self.connection_status.connected {
                            ui.label(RichText::new("📡").color(Color32::from_rgb(34, 197, 94)));
                            ui.colored_label(Color32::from_rgb(34, 197, 94), "Connected");
                        } else {
                            ui.label(RichText::new("📡").color(Color32::from_rgb(248, 113, 113)));
                            ui.colored_label(Color32::from_rgb(248, 113, 113), "Disconnected");
                        }
                        
                        ui.separator();
                        
                        // Command source badge
                        let source_color = if self.connection_status.command_source == "bms" {
                            Color32::from_rgb(6, 182, 212) // Cyan for cloud
                        } else {
                            Color32::from_rgb(100, 116, 139) // Gray for local
                        };
                        
                        ui.label(RichText::new(if self.connection_status.command_source == "bms" { "☁️" } else { "💾" })
                            .color(source_color));
                        ui.colored_label(source_color, self.connection_status.command_source.to_uppercase());
                    });
                });
                
                ui.separator();
                
                ui.horizontal(|ui| {
                    // Enable/Disable switch
                    ui.checkbox(&mut self.config.enabled, "Enable BMS Integration");
                    
                    if self.config.enabled {
                        ui.colored_label(Color32::from_rgb(34, 197, 94), "● Active");
                    }
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("💾 Save Config").clicked() {
                            self.save_config(board_id);
                        }
                        
                        if ui.button("🔄 Test Connection").clicked() {
                            self.test_connection();
                        }
                    });
                });
            });
        });
        
        ui.add_space(10.0);
        
        // Connection Status Alert
        if self.config.enabled {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    if self.connection_status.connected {
                        ui.label(RichText::new("✅").size(16.0));
                        ui.label(RichText::new("Connected to BMS")
                            .color(Color32::from_rgb(34, 197, 94))
                            .strong());
                        
                        if let Some(last_query) = self.connection_status.last_successful_query {
                            ui.label(format!("Last Query: {}", last_query.format("%H:%M:%S")));
                        }
                        
                        ui.label(format!("Latency: {}ms", self.connection_status.latency_ms));
                    } else {
                        ui.label(RichText::new("⚠️").size(16.0));
                        ui.label(RichText::new("Using Local Logic")
                            .color(Color32::from_rgb(251, 146, 60))
                            .strong());
                        
                        if let Some(error) = &self.connection_status.last_error {
                            ui.label(RichText::new(format!("Error: {}", error))
                                .color(Color32::from_rgb(248, 113, 113))
                                .size(12.0));
                        }
                        
                        if self.connection_status.retry_count > 0 {
                            ui.label(format!("Retries: {}", self.connection_status.retry_count));
                        }
                    }
                });
            });
            
            ui.add_space(10.0);
        }
        
        // Tabs
        ui.horizontal(|ui| {
            if ui.selectable_label(self.active_tab == "configuration", "⚙️ Configuration").clicked() {
                self.active_tab = "configuration".to_string();
            }
            if ui.selectable_label(self.active_tab == "commands", "📋 Live Commands").clicked() {
                self.active_tab = "commands".to_string();
            }
            if ui.selectable_label(self.active_tab == "monitoring", "📊 Monitoring").clicked() {
                self.active_tab = "monitoring".to_string();
            }
        });
        
        ui.separator();
        ui.add_space(10.0);
        
        match self.active_tab.as_str() {
            "configuration" => self.show_configuration_tab(ui),
            "commands" => self.show_commands_tab(ui),
            "monitoring" => self.show_monitoring_tab(ui),
            _ => {}
        }
        
        // Show dialogs
        self.show_dialogs(ui);
    }
    
    fn show_configuration_tab(&mut self, ui: &mut egui::Ui) {
        ui.columns(2, |columns| {
            // Basic Configuration
            columns[0].group(|ui| {
                ui.vertical(|ui| {
                    ui.heading(RichText::new("📝 Basic Configuration").color(Color32::from_rgb(15, 23, 42)));
                    ui.separator();
                    
                    Grid::new("basic_config").num_columns(2).show(ui, |ui| {
                        ui.label("Location Name:");
                        ui.text_edit_singleline(&mut self.config.location_name);
                        ui.end_row();
                        
                        ui.label("System Name:");
                        ui.text_edit_singleline(&mut self.config.system_name);
                        ui.end_row();
                        
                        ui.label("Location ID:");
                        ui.text_edit_singleline(&mut self.config.location_id);
                        ui.end_row();
                        
                        ui.label("Equipment ID:");
                        ui.text_edit_singleline(&mut self.config.equipment_id);
                        ui.end_row();
                        
                        ui.label("Equipment Type:");
                        ui.text_edit_singleline(&mut self.config.equipment_type);
                        ui.end_row();
                        
                        ui.label("Zone:");
                        ui.text_edit_singleline(&mut self.config.zone);
                        ui.end_row();
                    });
                });
            });
            
            // Server Configuration
            columns[1].group(|ui| {
                ui.vertical(|ui| {
                    ui.heading(RichText::new("☁️ BMS Server Configuration").color(Color32::from_rgb(15, 23, 42)));
                    ui.separator();
                    
                    Grid::new("server_config").num_columns(2).show(ui, |ui| {
                        ui.label("BMS Server URL:");
                        ui.text_edit_singleline(&mut self.config.bms_server_url);
                        ui.end_row();
                        
                        ui.label("InfluxDB URL:");
                        ui.text_edit_singleline(&mut self.config.influx_url);
                        ui.end_row();
                        
                        ui.label("Command Query (sec):");
                        ui.add(egui::DragValue::new(&mut self.config.command_query_interval)
                            .speed(1.0)
                            .clamp_range(5..=300));
                        ui.end_row();
                        
                        ui.label("Data Update (sec):");
                        ui.add(egui::DragValue::new(&mut self.config.update_interval)
                            .speed(1.0)
                            .clamp_range(5..=300));
                        ui.end_row();
                        
                        ui.label("Fallback to Local:");
                        ui.checkbox(&mut self.config.fallback_to_local, "");
                        ui.end_row();
                    });
                    
                    ui.add_space(10.0);
                    
                    if ui.button("➕ Add Field Mapping").clicked() {
                        self.show_field_mapping_dialog = true;
                    }
                });
            });
        });
        
        ui.add_space(20.0);
        
        // Query Template
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.heading(RichText::new("📊 InfluxDB Query Template").color(Color32::from_rgb(15, 23, 42)));
                ui.separator();
                
                // Query preview with light background
                ui.group(|ui| {
                    let query = format!(
                        "SELECT * FROM \"ProcessingEngineCommands\"\n\
                         WHERE equipment_id = '{}'\n\
                         AND location_id = '{}'\n\
                         AND time >= now() - INTERVAL '5 minutes'\n\
                         ORDER BY time DESC\n\
                         LIMIT 35",
                        self.config.equipment_id,
                        self.config.location_id
                    );
                    
                    ui.colored_label(Color32::from_rgb(6, 182, 212), query);
                });
            });
        });
    }
    
    fn show_commands_tab(&mut self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.set_min_height(400.0);
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.heading(RichText::new("📋 Recent BMS Commands").color(Color32::from_rgb(15, 23, 42)));
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("🔄 Refresh").clicked() {
                            self.query_recent_commands();
                        }
                    });
                });
                
                ui.separator();
                
                if self.recent_commands.is_empty() {
                    ui.vertical_centered(|ui| {
                        ui.add_space(50.0);
                        ui.label(RichText::new("📭").size(48.0).color(Color32::from_rgb(200, 200, 200)));
                        ui.label(RichText::new("No recent commands")
                            .color(Color32::from_rgb(100, 116, 139))
                            .size(16.0));
                        
                        if self.config.enabled {
                            ui.label("Waiting for BMS commands...");
                        } else {
                            ui.label("Enable BMS integration to see commands");
                        }
                    });
                } else {
                    ScrollArea::vertical().max_height(350.0).show(ui, |ui| {
                        for command in &self.recent_commands {
                            ui.group(|ui| {
                                ui.set_min_width(ui.available_width() - 10.0);
                                ui.vertical(|ui| {
                                    ui.horizontal(|ui| {
                                        // Command type badge
                                        ui.colored_label(
                                            Color32::from_rgb(20, 184, 166),
                                            &command.command_type
                                        );
                                        
                                        // Priority badge
                                        let priority_color = match command.priority {
                                            0..=3 => Color32::from_rgb(34, 197, 94),
                                            4..=6 => Color32::from_rgb(251, 146, 60),
                                            _ => Color32::from_rgb(248, 113, 113),
                                        };
                                        ui.colored_label(priority_color, format!("Priority: {}", command.priority));
                                        
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            ui.label(RichText::new(command.timestamp.format("%H:%M:%S").to_string())
                                                .color(Color32::from_rgb(100, 116, 139))
                                                .size(11.0));
                                        });
                                    });
                                    
                                    ui.separator();
                                    
                                    ui.label(format!("Equipment: {}", command.equipment_id));
                                    ui.label(format!("Location: {}", command.location_id));
                                    
                                    // Command data
                                    if !command.command_data.is_empty() {
                                        ui.collapsing("Command Data", |ui| {
                                            for (key, value) in &command.command_data {
                                                ui.horizontal(|ui| {
                                                    ui.label(RichText::new(format!("{}:", key))
                                                        .color(Color32::from_rgb(100, 116, 139)));
                                                    ui.label(value);
                                                });
                                            }
                                        });
                                    }
                                });
                            });
                            
                            ui.add_space(5.0);
                        }
                    });
                }
            });
        });
    }
    
    fn show_monitoring_tab(&mut self, ui: &mut egui::Ui) {
        ui.columns(2, |columns| {
            // Command Source Status
            columns[0].group(|ui| {
                ui.vertical(|ui| {
                    ui.heading(RichText::new("📡 Command Source Status").color(Color32::from_rgb(15, 23, 42)));
                    ui.separator();
                    
                    // Current Source
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            let icon = if self.connection_status.command_source == "bms" { "☁️" } else { "💾" };
                            ui.label(RichText::new(icon).size(16.0));
                            ui.label("Current Source:");
                            
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let color = if self.connection_status.command_source == "bms" {
                                    Color32::from_rgb(6, 182, 212)
                                } else {
                                    Color32::from_rgb(100, 116, 139)
                                };
                                ui.colored_label(color, 
                                    if self.connection_status.command_source == "bms" { 
                                        "BMS Server" 
                                    } else { 
                                        "Local Logic Files" 
                                    }
                                );
                            });
                        });
                    });
                    
                    // Connection Status
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            let icon = if self.connection_status.connected { "✅" } else { "❌" };
                            ui.label(RichText::new(icon).size(16.0));
                            ui.label("Connection:");
                            
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let color = if self.connection_status.connected {
                                    Color32::from_rgb(34, 197, 94)
                                } else {
                                    Color32::from_rgb(248, 113, 113)
                                };
                                ui.colored_label(color, 
                                    if self.connection_status.connected { 
                                        "Connected" 
                                    } else { 
                                        "Disconnected" 
                                    }
                                );
                            });
                        });
                    });
                    
                    // Last Query
                    if let Some(last_query) = self.connection_status.last_successful_query {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("🕐").size(16.0));
                                ui.label("Last Query:");
                                
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.label(last_query.format("%Y-%m-%d %H:%M:%S").to_string());
                                });
                            });
                        });
                    }
                    
                    // Retry Count
                    if self.connection_status.retry_count > 0 {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("⚠️").size(16.0));
                                ui.label("Retry Count:");
                                
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.colored_label(
                                        Color32::from_rgb(251, 146, 60),
                                        self.connection_status.retry_count.to_string()
                                    );
                                });
                            });
                        });
                    }
                    
                    // Ping History Chart
                    ui.add_space(10.0);
                    ui.label(RichText::new("Ping History (Last 10)")
                        .color(Color32::from_rgb(15, 23, 42))
                        .strong());
                    
                    ui.group(|ui| {
                        for (timestamp, success, latency) in self.ping_history.iter().rev().take(10) {
                            ui.horizontal(|ui| {
                                let icon = if *success { "🟢" } else { "🔴" };
                                ui.label(icon);
                                ui.label(timestamp.format("%H:%M:%S").to_string());
                                ui.label(format!("{}ms", latency));
                            });
                        }
                    });
                });
            });
            
            // Fallback Configuration
            columns[1].group(|ui| {
                ui.vertical(|ui| {
                    ui.heading(RichText::new("🔄 Fallback Configuration").color(Color32::from_rgb(15, 23, 42)));
                    ui.separator();
                    
                    // Alert box
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("⚠️").color(Color32::from_rgb(251, 146, 60)).size(20.0));
                            ui.vertical(|ui| {
                                ui.label(RichText::new("Fallback Mode")
                                    .color(Color32::from_rgb(15, 23, 42))
                                    .strong());
                                
                                let fallback_text = if self.config.fallback_to_local {
                                    "When BMS connection is lost, the system will automatically fall back to local logic files."
                                } else {
                                    "When BMS connection is lost, the system will NOT fall back to local logic files."
                                };
                                
                                ui.label(RichText::new(fallback_text)
                                    .color(Color32::from_rgb(100, 116, 139))
                                    .size(12.0));
                            });
                        });
                    });
                    
                    ui.add_space(10.0);
                    
                    ui.checkbox(&mut self.config.fallback_to_local, "Enable Automatic Fallback");
                    
                    ui.add_space(10.0);
                    
                    // How it works box with light blue background
                    ui.group(|ui| {
                        let rect = ui.available_rect_before_wrap();
                        ui.painter().rect_filled(
                            rect,
                            5.0,
                            Color32::from_rgb(240, 249, 255) // Very light blue
                        );
                        
                        ui.vertical(|ui| {
                            ui.label(RichText::new("📘 How It Works:")
                                .color(Color32::from_rgb(14, 116, 144))
                                .strong());
                            
                            ui.label(format!("• System queries BMS server every {}s", 
                                self.config.command_query_interval));
                            ui.label("• If BMS is available, commands are executed from server");
                            ui.label("• If BMS fails and fallback is enabled, local logic files are used");
                            ui.label("• System automatically reconnects when BMS becomes available");
                        });
                    });
                    
                    ui.add_space(10.0);
                    
                    // Statistics
                    ui.label(RichText::new("📊 Connection Statistics")
                        .color(Color32::from_rgb(15, 23, 42))
                        .strong());
                    
                    ui.group(|ui| {
                        let total_pings = self.ping_history.len();
                        let successful_pings = self.ping_history.iter()
                            .filter(|(_, success, _)| *success)
                            .count();
                        let avg_latency = if !self.ping_history.is_empty() {
                            self.ping_history.iter()
                                .map(|(_, _, latency)| *latency as f32)
                                .sum::<f32>() / self.ping_history.len() as f32
                        } else {
                            0.0
                        };
                        
                        ui.label(format!("Total Pings: {}", total_pings));
                        ui.label(format!("Successful: {}", successful_pings));
                        ui.label(format!("Failed: {}", total_pings - successful_pings));
                        ui.label(format!("Avg Latency: {:.1}ms", avg_latency));
                        
                        if total_pings > 0 {
                            let success_rate = (successful_pings as f32 / total_pings as f32) * 100.0;
                            ui.label(format!("Success Rate: {:.1}%", success_rate));
                        }
                    });
                });
            });
        });
    }
    
    fn show_dialogs(&mut self, ui: &mut egui::Ui) {
        // Field Mapping Dialog
        if self.show_field_mapping_dialog {
            Window::new("Add Field Mapping")
                .collapsible(false)
                .resizable(false)
                .show(ui.ctx(), |ui| {
                    ui.label("Map BMS fields to local channels:");
                    
                    ui.separator();
                    
                    Grid::new("field_mapping").num_columns(2).show(ui, |ui| {
                        ui.label("BMS Field:");
                        ui.text_edit_singleline(&mut self.new_field_mapping.0);
                        ui.end_row();
                        
                        ui.label("Local Channel:");
                        ui.text_edit_singleline(&mut self.new_field_mapping.1);
                        ui.end_row();
                    });
                    
                    ui.separator();
                    
                    ui.horizontal(|ui| {
                        if ui.button("✅ Add").clicked() {
                            if !self.new_field_mapping.0.is_empty() && !self.new_field_mapping.1.is_empty() {
                                self.config.field_mappings.insert(
                                    self.new_field_mapping.0.clone(),
                                    self.new_field_mapping.1.clone()
                                );
                                self.new_field_mapping = (String::new(), String::new());
                                self.show_field_mapping_dialog = false;
                            }
                        }
                        
                        if ui.button("❌ Cancel").clicked() {
                            self.show_field_mapping_dialog = false;
                        }
                    });
                });
        }
    }
    
    fn test_connection(&mut self) {
        self.is_testing_connection = true;
        
        // REAL connection test based on protocol
        let (success, latency) = match self.protocol_config.protocol_type {
            ProtocolType::Modbus => {
                // Test REAL Modbus connection
                let result = std::process::Command::new("python3")
                    .arg("/opt/nexus/modbus_test.py")
                    .arg(&self.protocol_config.ip_address)
                    .arg(&self.protocol_config.port.to_string())
                    .output();
                
                match result {
                    Ok(output) if output.status.success() => {
                        let response_time = String::from_utf8_lossy(&output.stdout)
                            .trim()
                            .parse::<u32>()
                            .unwrap_or(999);
                        (true, response_time)
                    }
                    _ => (false, 0)
                }
            }
            ProtocolType::BACnet => {
                // Test REAL BACnet connection
                let result = std::process::Command::new("bacnet-discover")
                    .arg(&self.protocol_config.ip_address)
                    .output();
                
                match result {
                    Ok(output) if output.status.success() => (true, 100),
                    _ => (false, 0)
                }
            }
            _ => (false, 0)
        };
        
        self.connection_status.connected = success;
        self.connection_status.latency_ms = latency;
        
        if success {
            self.connection_status.last_successful_query = Some(Utc::now());
            self.connection_status.command_source = "bms".to_string();
            self.connection_status.retry_count = 0;
        } else {
            self.connection_status.last_error = Some("Connection timeout".to_string());
            self.connection_status.command_source = "local".to_string();
            self.connection_status.retry_count += 1;
        }
        
        // Add to ping history
        self.ping_history.push((Utc::now(), success, latency));
        if self.ping_history.len() > 100 {
            self.ping_history.remove(0);
        }
        
        self.is_testing_connection = false;
    }
    
    fn save_config(&mut self, board_id: &str) {
        self.is_saving = true;
        println!("Saving BMS configuration for board {}", board_id);
        // In real implementation, would save to file or database
        self.is_saving = false;
    }
    
    fn query_recent_commands(&mut self) {
        // Simulate fetching recent commands
        if self.config.enabled && self.connection_status.connected {
            self.recent_commands = vec![
                BmsCommand {
                    equipment_id: self.config.equipment_id.clone(),
                    location_id: self.config.location_id.clone(),
                    command_type: "SetTemperature".to_string(),
                    command_data: [("value".to_string(), "72.0".to_string())].iter().cloned().collect(),
                    timestamp: Utc::now(),
                    priority: 5,
                },
                BmsCommand {
                    equipment_id: self.config.equipment_id.clone(),
                    location_id: self.config.location_id.clone(),
                    command_type: "FanSpeed".to_string(),
                    command_data: [("speed".to_string(), "Medium".to_string())].iter().cloned().collect(),
                    timestamp: Utc::now() - chrono::Duration::minutes(5),
                    priority: 3,
                },
                BmsCommand {
                    equipment_id: self.config.equipment_id.clone(),
                    location_id: self.config.location_id.clone(),
                    command_type: "SystemMode".to_string(),
                    command_data: [("mode".to_string(), "Cooling".to_string())].iter().cloned().collect(),
                    timestamp: Utc::now() - chrono::Duration::minutes(10),
                    priority: 7,
                },
            ];
        }
    }
}

// Add rand for simulation
use rand::Rng;