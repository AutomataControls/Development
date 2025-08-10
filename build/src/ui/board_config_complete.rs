// COMPLETE Board Configuration Implementation - Sequent Microsystems board management
// Includes ALL features: channel configs, manual overrides, CT scaling, relay modes

use eframe::egui;
use egui::{Color32, RichText, Grid, ScrollArea, Ui, Window, TextEdit};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct BoardConfiguration {
    // Board configuration
    config: BoardConfig,
    
    // Available boards
    boards: Vec<BoardInfo>,
    
    // UI state
    active_tab: ConfigTab,
    editing_channel: Option<ChannelConfig>,
    show_edit_dialog: bool,
    is_saving: bool,
    is_loading: bool,
    
    // Board management
    show_board_management: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BoardConfig {
    board_id: String,
    board_name: String,
    location: String,
    universal_inputs: Vec<ChannelConfig>,
    analog_outputs: Vec<ChannelConfig>,
    relay_outputs: Vec<ChannelConfig>,
    triac_outputs: Vec<ChannelConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChannelConfig {
    id: String,
    name: String,
    description: String,
    sensor_type: String,
    input_type: Option<String>,
    voltage_type: Option<String>,      // For 0-10V: 'rh', '10k_transducer', 'ct'
    ct_scaling: Option<String>,         // For CT: '0-20A', '0-50A', '0-100A'
    scaling_min: f32,
    scaling_max: f32,
    units: String,
    enabled: bool,
    alarm_high: Option<f32>,
    alarm_low: Option<f32>,
    calibration_offset: f32,
    min_voltage: Option<f32>,           // For analog outputs
    max_voltage: Option<f32>,           // For analog outputs
    reverse_acting: Option<bool>,       // For control outputs
    normally_closed: Option<bool>,      // For relay outputs (true = N.C., false = N.O.)
    manual_mode: Option<bool>,          // For universal inputs - override with static value
    manual_value: Option<f32>,          // Static value when in manual mode
}

#[derive(Debug, Clone)]
struct BoardInfo {
    id: String,
    name: String,
    board_type: String,
    stack_level: usize,
    enabled: bool,
    capabilities: BoardCapabilities,
}

#[derive(Debug, Clone)]
struct BoardCapabilities {
    universal_inputs: usize,
    analog_outputs: usize,
    relays: usize,
    triacs: usize,
}

#[derive(Debug, Clone, PartialEq)]
enum ConfigTab {
    UniversalInputs,
    AnalogOutputs,
    Relays,
    Triacs,
}

impl BoardConfiguration {
    pub fn new() -> Self {
        let mut config = Self {
            config: Self::create_default_config(),
            boards: Self::init_available_boards(),
            active_tab: ConfigTab::UniversalInputs,
            editing_channel: None,
            show_edit_dialog: false,
            is_saving: false,
            is_loading: false,
            show_board_management: true,
        };
        
        config
    }
    
    fn create_default_config() -> BoardConfig {
        BoardConfig {
            board_id: "megabas_0".to_string(),
            board_name: "MegaBAS Configuration".to_string(),
            location: "Mechanical Room A".to_string(),
            universal_inputs: Self::init_universal_inputs(),
            analog_outputs: Self::init_analog_outputs(),
            relay_outputs: Self::init_relay_outputs(),
            triac_outputs: Self::init_triac_outputs(),
        }
    }
    
    fn init_available_boards() -> Vec<BoardInfo> {
        vec![
            BoardInfo {
                id: "megabas_0".to_string(),
                name: "MegaBAS Board".to_string(),
                board_type: "MegaBAS".to_string(),
                stack_level: 0,
                enabled: true,
                capabilities: BoardCapabilities {
                    universal_inputs: 8,  // 8 configurable inputs (0-10V, 1K, 10K)
                    analog_outputs: 4,    // 4x 0-10V outputs
                    relays: 0,           // MegaBAS has NO relays!
                    triacs: 4,           // 4 triacs for AC control
                },
            },
            BoardInfo {
                id: "16univin_1".to_string(),
                name: "16-Universal Input".to_string(),
                board_type: "16-UNIVIN".to_string(),
                stack_level: 1,
                enabled: true,
                capabilities: BoardCapabilities {
                    universal_inputs: 16,  // 16 INPUTS ONLY
                    analog_outputs: 0,
                    relays: 0,
                    triacs: 0,
                },
            },
            BoardInfo {
                id: "16uout_2".to_string(),
                name: "16-Universal Output".to_string(),
                board_type: "16-UOUT".to_string(),
                stack_level: 2,
                enabled: false,
                capabilities: BoardCapabilities {
                    universal_inputs: 0,
                    analog_outputs: 16,    // 16x 0-10V OUTPUTS ONLY
                    relays: 0,
                    triacs: 0,
                },
            },
            BoardInfo {
                id: "8relay_3".to_string(),
                name: "8-Relay Board".to_string(),
                board_type: "8-RELAY".to_string(),
                stack_level: 3,
                enabled: false,
                capabilities: BoardCapabilities {
                    universal_inputs: 0,
                    analog_outputs: 0,
                    relays: 8,            // 8 RELAY OUTPUTS ONLY
                    triacs: 0,
                },
            },
            BoardInfo {
                id: "16relay_4".to_string(),
                name: "16-Relay Board".to_string(),
                board_type: "16-RELAY".to_string(),
                stack_level: 4,
                enabled: false,
                capabilities: BoardCapabilities {
                    universal_inputs: 0,
                    analog_outputs: 0,
                    relays: 16,           // 16 RELAY OUTPUTS ONLY
                    triacs: 0,
                },
            },
        ]
    }
    
    fn init_universal_inputs() -> Vec<ChannelConfig> {
        vec![
            ChannelConfig {
                id: "ui_1".to_string(),
                name: "Supply Air Temp".to_string(),
                description: "Supply air temperature sensor".to_string(),
                sensor_type: "temperature".to_string(),
                input_type: Some("resistance".to_string()),
                voltage_type: None,
                ct_scaling: None,
                scaling_min: 32.0,
                scaling_max: 122.0,
                units: "°F".to_string(),
                enabled: true,
                alarm_high: Some(85.0),
                alarm_low: Some(45.0),
                calibration_offset: 0.0,
                min_voltage: None,
                max_voltage: None,
                reverse_acting: None,
                normally_closed: None,
                manual_mode: Some(false),
                manual_value: Some(72.0),
            },
            ChannelConfig {
                id: "ui_2".to_string(),
                name: "Return Air Temp".to_string(),
                description: "Return air temperature sensor".to_string(),
                sensor_type: "temperature".to_string(),
                input_type: Some("resistance".to_string()),
                voltage_type: None,
                ct_scaling: None,
                scaling_min: 32.0,
                scaling_max: 122.0,
                units: "°F".to_string(),
                enabled: true,
                alarm_high: Some(90.0),
                alarm_low: Some(50.0),
                calibration_offset: 0.0,
                min_voltage: None,
                max_voltage: None,
                reverse_acting: None,
                normally_closed: None,
                manual_mode: Some(false),
                manual_value: Some(75.0),
            },
            ChannelConfig {
                id: "ui_3".to_string(),
                name: "Space Humidity".to_string(),
                description: "Space relative humidity sensor".to_string(),
                sensor_type: "humidity".to_string(),
                input_type: Some("voltage".to_string()),
                voltage_type: Some("rh".to_string()),
                ct_scaling: None,
                scaling_min: 0.0,
                scaling_max: 99.0,
                units: "%".to_string(),
                enabled: true,
                alarm_high: Some(60.0),
                alarm_low: Some(30.0),
                calibration_offset: 0.0,
                min_voltage: None,
                max_voltage: None,
                reverse_acting: None,
                normally_closed: None,
                manual_mode: Some(false),
                manual_value: Some(45.0),
            },
            ChannelConfig {
                id: "ui_4".to_string(),
                name: "Static Pressure".to_string(),
                description: "Duct static pressure sensor".to_string(),
                sensor_type: "pressure".to_string(),
                input_type: Some("current".to_string()),
                voltage_type: None,
                ct_scaling: None,
                scaling_min: 0.0,
                scaling_max: 2.0,
                units: "\"WC".to_string(),
                enabled: true,
                alarm_high: Some(1.5),
                alarm_low: None,
                calibration_offset: 0.0,
                min_voltage: None,
                max_voltage: None,
                reverse_acting: None,
                normally_closed: None,
                manual_mode: Some(false),
                manual_value: Some(0.5),
            },
            ChannelConfig {
                id: "ui_5".to_string(),
                name: "Motor Current".to_string(),
                description: "Supply fan motor current".to_string(),
                sensor_type: "current".to_string(),
                input_type: Some("voltage".to_string()),
                voltage_type: Some("ct".to_string()),
                ct_scaling: Some("0-50A".to_string()),
                scaling_min: 0.0,
                scaling_max: 50.0,
                units: "A".to_string(),
                enabled: true,
                alarm_high: Some(40.0),
                alarm_low: None,
                calibration_offset: 0.0,
                min_voltage: None,
                max_voltage: None,
                reverse_acting: None,
                normally_closed: None,
                manual_mode: Some(false),
                manual_value: Some(12.5),
            },
            ChannelConfig {
                id: "ui_6".to_string(),
                name: "Outdoor Temp".to_string(),
                description: "Outdoor air temperature".to_string(),
                sensor_type: "temperature".to_string(),
                input_type: Some("voltage".to_string()),
                voltage_type: Some("10k_transducer".to_string()),
                ct_scaling: None,
                scaling_min: 32.0,
                scaling_max: 122.0,
                units: "°F".to_string(),
                enabled: true,
                alarm_high: None,
                alarm_low: None,
                calibration_offset: 0.0,
                min_voltage: None,
                max_voltage: None,
                reverse_acting: None,
                normally_closed: None,
                manual_mode: Some(true), // Example of manual override
                manual_value: Some(95.0),
            },
            ChannelConfig {
                id: "ui_7".to_string(),
                name: "Filter Status".to_string(),
                description: "Filter differential pressure switch".to_string(),
                sensor_type: "status".to_string(),
                input_type: Some("digital".to_string()),
                voltage_type: None,
                ct_scaling: None,
                scaling_min: 0.0,
                scaling_max: 1.0,
                units: "".to_string(),
                enabled: true,
                alarm_high: None,
                alarm_low: None,
                calibration_offset: 0.0,
                min_voltage: None,
                max_voltage: None,
                reverse_acting: None,
                normally_closed: None,
                manual_mode: Some(false),
                manual_value: Some(0.0),
            },
            ChannelConfig {
                id: "ui_8".to_string(),
                name: "Occupancy".to_string(),
                description: "Space occupancy sensor".to_string(),
                sensor_type: "status".to_string(),
                input_type: Some("digital".to_string()),
                voltage_type: None,
                ct_scaling: None,
                scaling_min: 0.0,
                scaling_max: 1.0,
                units: "".to_string(),
                enabled: true,
                alarm_high: None,
                alarm_low: None,
                calibration_offset: 0.0,
                min_voltage: None,
                max_voltage: None,
                reverse_acting: None,
                normally_closed: None,
                manual_mode: Some(false),
                manual_value: Some(1.0),
            },
        ]
    }
    
    fn init_analog_outputs() -> Vec<ChannelConfig> {
        vec![
            ChannelConfig {
                id: "ao_1".to_string(),
                name: "Cooling Valve".to_string(),
                description: "Chilled water valve position".to_string(),
                sensor_type: "valve_position".to_string(),
                input_type: None,
                voltage_type: None,
                ct_scaling: None,
                scaling_min: 0.0,
                scaling_max: 100.0,
                units: "%".to_string(),
                enabled: true,
                alarm_high: None,
                alarm_low: None,
                calibration_offset: 0.0,
                min_voltage: Some(2.0),
                max_voltage: Some(10.0),
                reverse_acting: Some(false),
                normally_closed: None,
                manual_mode: None,
                manual_value: None,
            },
            ChannelConfig {
                id: "ao_2".to_string(),
                name: "Heating Valve".to_string(),
                description: "Hot water valve position".to_string(),
                sensor_type: "valve_position".to_string(),
                input_type: None,
                voltage_type: None,
                ct_scaling: None,
                scaling_min: 0.0,
                scaling_max: 100.0,
                units: "%".to_string(),
                enabled: true,
                alarm_high: None,
                alarm_low: None,
                calibration_offset: 0.0,
                min_voltage: Some(0.0),
                max_voltage: Some(10.0),
                reverse_acting: Some(true),
                normally_closed: None,
                manual_mode: None,
                manual_value: None,
            },
            ChannelConfig {
                id: "ao_3".to_string(),
                name: "Damper Position".to_string(),
                description: "Outside air damper position".to_string(),
                sensor_type: "damper_position".to_string(),
                input_type: None,
                voltage_type: None,
                ct_scaling: None,
                scaling_min: 0.0,
                scaling_max: 100.0,
                units: "%".to_string(),
                enabled: true,
                alarm_high: None,
                alarm_low: None,
                calibration_offset: 0.0,
                min_voltage: Some(2.0),
                max_voltage: Some(10.0),
                reverse_acting: Some(false),
                normally_closed: None,
                manual_mode: None,
                manual_value: None,
            },
            ChannelConfig {
                id: "ao_4".to_string(),
                name: "VFD Speed".to_string(),
                description: "Supply fan VFD speed command".to_string(),
                sensor_type: "vfd_speed".to_string(),
                input_type: None,
                voltage_type: None,
                ct_scaling: None,
                scaling_min: 0.0,
                scaling_max: 100.0,
                units: "%".to_string(),
                enabled: true,
                alarm_high: None,
                alarm_low: None,
                calibration_offset: 0.0,
                min_voltage: Some(0.0),
                max_voltage: Some(10.0),
                reverse_acting: Some(false),
                normally_closed: None,
                manual_mode: None,
                manual_value: None,
            },
        ]
    }
    
    fn init_relay_outputs() -> Vec<ChannelConfig> {
        vec![
            ChannelConfig {
                id: "ro_1".to_string(),
                name: "Supply Fan".to_string(),
                description: "Supply fan enable/disable".to_string(),
                sensor_type: "relay".to_string(),
                input_type: None,
                voltage_type: None,
                ct_scaling: None,
                scaling_min: 0.0,
                scaling_max: 1.0,
                units: "".to_string(),
                enabled: true,
                alarm_high: None,
                alarm_low: None,
                calibration_offset: 0.0,
                min_voltage: None,
                max_voltage: None,
                reverse_acting: None,
                normally_closed: Some(false), // N.O. - Normally Open
                manual_mode: None,
                manual_value: None,
            },
            ChannelConfig {
                id: "ro_2".to_string(),
                name: "Return Fan".to_string(),
                description: "Return fan enable/disable".to_string(),
                sensor_type: "relay".to_string(),
                input_type: None,
                voltage_type: None,
                ct_scaling: None,
                scaling_min: 0.0,
                scaling_max: 1.0,
                units: "".to_string(),
                enabled: true,
                alarm_high: None,
                alarm_low: None,
                calibration_offset: 0.0,
                min_voltage: None,
                max_voltage: None,
                reverse_acting: None,
                normally_closed: Some(false), // N.O.
                manual_mode: None,
                manual_value: None,
            },
            ChannelConfig {
                id: "ro_3".to_string(),
                name: "Compressor 1".to_string(),
                description: "Compressor stage 1".to_string(),
                sensor_type: "relay".to_string(),
                input_type: None,
                voltage_type: None,
                ct_scaling: None,
                scaling_min: 0.0,
                scaling_max: 1.0,
                units: "".to_string(),
                enabled: true,
                alarm_high: None,
                alarm_low: None,
                calibration_offset: 0.0,
                min_voltage: None,
                max_voltage: None,
                reverse_acting: None,
                normally_closed: Some(true), // N.C. - Normally Closed for safety
                manual_mode: None,
                manual_value: None,
            },
            ChannelConfig {
                id: "ro_4".to_string(),
                name: "Alarm Output".to_string(),
                description: "General alarm relay".to_string(),
                sensor_type: "relay".to_string(),
                input_type: None,
                voltage_type: None,
                ct_scaling: None,
                scaling_min: 0.0,
                scaling_max: 1.0,
                units: "".to_string(),
                enabled: true,
                alarm_high: None,
                alarm_low: None,
                calibration_offset: 0.0,
                min_voltage: None,
                max_voltage: None,
                reverse_acting: None,
                normally_closed: Some(true), // N.C. - Active on alarm
                manual_mode: None,
                manual_value: None,
            },
        ]
    }
    
    fn init_triac_outputs() -> Vec<ChannelConfig> {
        vec![
            ChannelConfig {
                id: "to_1".to_string(),
                name: "Stage Heat 1".to_string(),
                description: "Electric heat stage 1".to_string(),
                sensor_type: "triac".to_string(),
                input_type: None,
                voltage_type: None,
                ct_scaling: None,
                scaling_min: 0.0,
                scaling_max: 1.0,
                units: "".to_string(),
                enabled: true,
                alarm_high: None,
                alarm_low: None,
                calibration_offset: 0.0,
                min_voltage: None,
                max_voltage: None,
                reverse_acting: None,
                normally_closed: None,
                manual_mode: None,
                manual_value: None,
            },
            ChannelConfig {
                id: "to_2".to_string(),
                name: "Stage Heat 2".to_string(),
                description: "Electric heat stage 2".to_string(),
                sensor_type: "triac".to_string(),
                input_type: None,
                voltage_type: None,
                ct_scaling: None,
                scaling_min: 0.0,
                scaling_max: 1.0,
                units: "".to_string(),
                enabled: true,
                alarm_high: None,
                alarm_low: None,
                calibration_offset: 0.0,
                min_voltage: None,
                max_voltage: None,
                reverse_acting: None,
                normally_closed: None,
                manual_mode: None,
                manual_value: None,
            },
            ChannelConfig {
                id: "to_3".to_string(),
                name: "Humidifier".to_string(),
                description: "Humidifier control".to_string(),
                sensor_type: "triac".to_string(),
                input_type: None,
                voltage_type: None,
                ct_scaling: None,
                scaling_min: 0.0,
                scaling_max: 1.0,
                units: "".to_string(),
                enabled: false,
                alarm_high: None,
                alarm_low: None,
                calibration_offset: 0.0,
                min_voltage: None,
                max_voltage: None,
                reverse_acting: None,
                normally_closed: None,
                manual_mode: None,
                manual_value: None,
            },
            ChannelConfig {
                id: "to_4".to_string(),
                name: "UV Light".to_string(),
                description: "UV sanitization light".to_string(),
                sensor_type: "triac".to_string(),
                input_type: None,
                voltage_type: None,
                ct_scaling: None,
                scaling_min: 0.0,
                scaling_max: 1.0,
                units: "".to_string(),
                enabled: false,
                alarm_high: None,
                alarm_low: None,
                calibration_offset: 0.0,
                min_voltage: None,
                max_voltage: None,
                reverse_acting: None,
                normally_closed: None,
                manual_mode: None,
                manual_value: None,
            },
        ]
    }
    
    pub fn show(&mut self, ui: &mut egui::Ui) {
        // Board Management Section
        if self.show_board_management {
            self.show_board_management_section(ui);
            ui.separator();
        }
        
        // Configuration Header
        ui.horizontal(|ui| {
            ui.label(RichText::new("⚙️ Board Configuration").size(18.0).strong());
            ui.label(format!("Board: {}", self.config.board_name));
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("💾 Save Config").clicked() {
                    self.save_config();
                }
                
                ui.colored_label(Color32::from_rgb(156, 163, 175), self.config.board_id.clone());
            });
        });
        
        ui.separator();
        
        // Basic Configuration
        ui.horizontal(|ui| {
            ui.group(|ui| {
                ui.label("Board Name:");
                ui.text_edit_singleline(&mut self.config.board_name);
            });
            
            ui.group(|ui| {
                ui.label("Location (from BMS):");
                ui.add_enabled(false, egui::TextEdit::singleline(&mut self.config.location));
            });
        });
        
        ui.separator();
        
        // Tab selector
        ui.horizontal(|ui| {
            let ui_count = self.config.universal_inputs.len();
            let ao_count = self.config.analog_outputs.len();
            let relay_count = self.config.relay_outputs.len();
            let triac_count = self.config.triac_outputs.len();
            
            if ui.selectable_label(
                self.active_tab == ConfigTab::UniversalInputs,
                format!("📊 Universal Inputs ({})", ui_count)
            ).clicked() {
                self.active_tab = ConfigTab::UniversalInputs;
            }
            
            if ui.selectable_label(
                self.active_tab == ConfigTab::AnalogOutputs,
                format!("📈 Analog Outputs ({})", ao_count)
            ).clicked() {
                self.active_tab = ConfigTab::AnalogOutputs;
            }
            
            if ui.selectable_label(
                self.active_tab == ConfigTab::Relays,
                format!("⚡ Relays ({})", relay_count)
            ).clicked() {
                self.active_tab = ConfigTab::Relays;
            }
            
            if ui.selectable_label(
                self.active_tab == ConfigTab::Triacs,
                format!("🔌 Triacs ({})", triac_count)
            ).clicked() {
                self.active_tab = ConfigTab::Triacs;
            }
        });
        
        ui.separator();
        
        // Tab content
        ScrollArea::vertical().max_height(400.0).show(ui, |ui| {
            match self.active_tab {
                ConfigTab::UniversalInputs => self.show_universal_inputs(ui),
                ConfigTab::AnalogOutputs => self.show_analog_outputs(ui),
                ConfigTab::Relays => self.show_relay_outputs(ui),
                ConfigTab::Triacs => self.show_triac_outputs(ui),
            }
        });
        
        // Edit dialog
        self.show_edit_dialog(ui);
    }
    
    fn show_board_management_section(&mut self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.label(RichText::new("🔧 Board Management").strong());
            ui.label("Enable or disable boards based on your installation. Only enabled boards will appear in the Select Board dropdown.");
            
            ui.separator();
            
            for board in &mut self.boards {
                let is_enabled = board.enabled;
                
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        // Board info
                        ui.vertical(|ui| {
                            ui.label(RichText::new(&board.name).strong());
                            ui.label(format!("{} • Stack Level {}", board.board_type, board.stack_level));
                            ui.label(format!("ID: {}", board.id));
                        });
                        
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            // Status badge
                            if is_enabled {
                                ui.colored_label(Color32::from_rgb(34, 197, 94), "✓ Enabled");
                            } else {
                                ui.colored_label(Color32::from_rgb(156, 163, 175), "✗ Disabled");
                            }
                            
                            // Enable/disable toggle
                            if ui.checkbox(&mut board.enabled, "").changed() {
                                println!("Board {} {}", board.id, if board.enabled { "enabled" } else { "disabled" });
                            }
                        });
                    });
                });
            }
        });
    }
    
    fn show_universal_inputs(&mut self, ui: &mut egui::Ui) {
        if self.config.universal_inputs.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label("No universal inputs available on this board");
            });
            return;
        }
        
        for channel in &self.config.universal_inputs.clone() {
            let is_manual = channel.manual_mode.unwrap_or(false);
            
            ui.group(|ui| {
                if is_manual {
                    ui.visuals_mut().override_text_color = Some(Color32::from_rgb(251, 146, 60));
                }
                
                ui.horizontal(|ui| {
                    // Status indicator
                    let color = if channel.enabled {
                        Color32::from_rgb(94, 234, 212)
                    } else {
                        Color32::from_rgb(156, 163, 175)
                    };
                    ui.painter().circle_filled(ui.cursor().min + egui::vec2(5.0, 10.0), 3.0, color);
                    ui.add_space(15.0);
                    
                    // Channel info
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(&channel.name).strong());
                            
                            if is_manual {
                                ui.colored_label(
                                    Color32::from_rgb(251, 146, 60),
                                    format!("MANUAL: {:.1} {}", channel.manual_value.unwrap_or(0.0), channel.units)
                                );
                            }
                        });
                        
                        ui.label(format!(
                            "{} • {} • {}",
                            channel.sensor_type,
                            channel.input_type.as_ref().unwrap_or(&"unknown".to_string()),
                            channel.units
                        ));
                        
                        // Special indicators for voltage types
                        if let Some(input_type) = &channel.input_type {
                            if input_type == "voltage" {
                                if let Some(voltage_type) = &channel.voltage_type {
                                    match voltage_type.as_str() {
                                        "rh" => ui.label("RH% (0-99%)"),
                                        "10k_transducer" => ui.label("10K Transducer (32-122°F)"),
                                        "ct" => {
                                            if let Some(ct_scaling) = &channel.ct_scaling {
                                                ui.label(format!("CT: {}", ct_scaling));
                                            }
                                        },
                                        _ => {}
                                    };
                                }
                            }
                        }
                    });
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Edit button
                        if ui.small_button("✏️").clicked() {
                            self.editing_channel = Some(channel.clone());
                            self.show_edit_dialog = true;
                        }
                        
                        // Alarm badges
                        if let Some(high) = channel.alarm_high {
                            ui.colored_label(Color32::from_rgb(239, 68, 68), format!("H: {:.1}", high));
                        }
                        if let Some(low) = channel.alarm_low {
                            ui.colored_label(Color32::from_rgb(59, 130, 246), format!("L: {:.1}", low));
                        }
                    });
                });
            });
        }
    }
    
    fn show_analog_outputs(&mut self, ui: &mut egui::Ui) {
        if self.config.analog_outputs.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label("No analog outputs available on this board");
            });
            return;
        }
        
        for channel in &self.config.analog_outputs.clone() {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    // Status indicator
                    let color = if channel.enabled {
                        Color32::from_rgb(59, 130, 246)
                    } else {
                        Color32::from_rgb(156, 163, 175)
                    };
                    ui.painter().circle_filled(ui.cursor().min + egui::vec2(5.0, 10.0), 3.0, color);
                    ui.add_space(15.0);
                    
                    // Channel info
                    ui.vertical(|ui| {
                        ui.label(RichText::new(&channel.name).strong());
                        ui.label(format!(
                            "{:.1}-{:.1}V • {:.0}-{:.0} {}",
                            channel.min_voltage.unwrap_or(0.0),
                            channel.max_voltage.unwrap_or(10.0),
                            channel.scaling_min,
                            channel.scaling_max,
                            channel.units
                        ));
                        
                        if channel.reverse_acting.unwrap_or(false) {
                            ui.colored_label(Color32::from_rgb(251, 146, 60), "REVERSE ACTING");
                        }
                    });
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("✏️").clicked() {
                            self.editing_channel = Some(channel.clone());
                            self.show_edit_dialog = true;
                        }
                    });
                });
            });
        }
    }
    
    fn show_relay_outputs(&mut self, ui: &mut egui::Ui) {
        if self.config.relay_outputs.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label("No relay outputs available on this board");
            });
            return;
        }
        
        for channel in &self.config.relay_outputs.clone() {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    // Status indicator
                    let color = if channel.enabled {
                        Color32::from_rgb(59, 130, 246)
                    } else {
                        Color32::from_rgb(156, 163, 175)
                    };
                    ui.painter().circle_filled(ui.cursor().min + egui::vec2(5.0, 10.0), 3.0, color);
                    ui.add_space(15.0);
                    
                    // Channel info
                    ui.vertical(|ui| {
                        ui.label(RichText::new(&channel.name).strong());
                        ui.label(&channel.description);
                    });
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("✏️").clicked() {
                            self.editing_channel = Some(channel.clone());
                            self.show_edit_dialog = true;
                        }
                        
                        // N.O./N.C. indicator
                        if let Some(normally_closed) = channel.normally_closed {
                            if normally_closed {
                                ui.colored_label(Color32::from_rgb(239, 68, 68), "N.C.");
                            } else {
                                ui.colored_label(Color32::from_rgb(34, 197, 94), "N.O.");
                            }
                        }
                    });
                });
            });
        }
    }
    
    fn show_triac_outputs(&mut self, ui: &mut egui::Ui) {
        if self.config.triac_outputs.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label("No triac outputs available on this board");
            });
            return;
        }
        
        for channel in &self.config.triac_outputs.clone() {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    // Status indicator
                    let color = if channel.enabled {
                        Color32::from_rgb(59, 130, 246)
                    } else {
                        Color32::from_rgb(156, 163, 175)
                    };
                    ui.painter().circle_filled(ui.cursor().min + egui::vec2(5.0, 10.0), 3.0, color);
                    ui.add_space(15.0);
                    
                    // Channel info
                    ui.vertical(|ui| {
                        ui.label(RichText::new(&channel.name).strong());
                        ui.label(&channel.description);
                    });
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("✏️").clicked() {
                            self.editing_channel = Some(channel.clone());
                            self.show_edit_dialog = true;
                        }
                    });
                });
            });
        }
    }
    
    fn show_edit_dialog(&mut self, ui: &mut egui::Ui) {
        if !self.show_edit_dialog {
            return;
        }
        
        if let Some(mut channel) = self.editing_channel.clone() {
            Window::new("Edit Channel Configuration")
                .collapsible(false)
                .resizable(false)
                .default_width(500.0)
                .show(ui.ctx(), |ui| {
                    ScrollArea::vertical().max_height(500.0).show(ui, |ui| {
                        // Channel Name
                        ui.horizontal(|ui| {
                            ui.label("Channel Name:");
                            ui.text_edit_singleline(&mut channel.name);
                        });
                        
                        // Description
                        ui.horizontal(|ui| {
                            ui.label("Description:");
                            ui.text_edit_singleline(&mut channel.description);
                        });
                        
                        ui.separator();
                        
                        // Input Type (for universal inputs)
                        if channel.input_type.is_some() {
                            ui.horizontal(|ui| {
                                ui.label("Input Type:");
                                let mut input_type = channel.input_type.clone().unwrap_or_default();
                                egui::ComboBox::from_label("")
                                    .selected_text(&input_type)
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(&mut input_type, "resistance".to_string(), "10K Thermistor");
                                        ui.selectable_value(&mut input_type, "voltage".to_string(), "0-10V");
                                        ui.selectable_value(&mut input_type, "current".to_string(), "4-20mA");
                                        ui.selectable_value(&mut input_type, "digital".to_string(), "Digital");
                                    });
                                channel.input_type = Some(input_type);
                            });
                            
                            // 0-10V Type Selection
                            if channel.input_type == Some("voltage".to_string()) {
                                ui.horizontal(|ui| {
                                    ui.label("0-10V Type:");
                                    let mut voltage_type = channel.voltage_type.clone().unwrap_or("generic".to_string());
                                    egui::ComboBox::from_id_source("voltage_type")
                                        .selected_text(&voltage_type)
                                        .show_ui(ui, |ui| {
                                            ui.selectable_value(&mut voltage_type, "generic".to_string(), "Generic 0-10V");
                                            ui.selectable_value(&mut voltage_type, "rh".to_string(), "RH% (0-99%)");
                                            ui.selectable_value(&mut voltage_type, "10k_transducer".to_string(), "10K Transducer (32-122°F)");
                                            ui.selectable_value(&mut voltage_type, "ct".to_string(), "CT (Current Transformer)");
                                        });
                                    channel.voltage_type = Some(voltage_type);
                                });
                                
                                // CT Scaling
                                if channel.voltage_type == Some("ct".to_string()) {
                                    ui.horizontal(|ui| {
                                        ui.label("CT Scaling:");
                                        let mut ct_scaling = channel.ct_scaling.clone().unwrap_or("0-20A".to_string());
                                        egui::ComboBox::from_id_source("ct_scaling")
                                            .selected_text(&ct_scaling)
                                            .show_ui(ui, |ui| {
                                                ui.selectable_value(&mut ct_scaling, "0-20A".to_string(), "0-20 Amps");
                                                ui.selectable_value(&mut ct_scaling, "0-50A".to_string(), "0-50 Amps");
                                                ui.selectable_value(&mut ct_scaling, "0-100A".to_string(), "0-100 Amps");
                                            });
                                        channel.ct_scaling = Some(ct_scaling);
                                    });
                                }
                            }
                        }
                        
                        ui.separator();
                        
                        // Scaling
                        ui.horizontal(|ui| {
                            ui.label("Scaling:");
                            ui.add(egui::DragValue::new(&mut channel.scaling_min).prefix("Min: ").speed(1.0));
                            ui.add(egui::DragValue::new(&mut channel.scaling_max).prefix("Max: ").speed(1.0));
                            ui.label(&channel.units);
                        });
                        
                        // Units
                        ui.horizontal(|ui| {
                            ui.label("Units:");
                            ui.text_edit_singleline(&mut channel.units);
                        });
                        
                        // Calibration Offset
                        ui.horizontal(|ui| {
                            ui.label("Calibration Offset:");
                            ui.add(egui::DragValue::new(&mut channel.calibration_offset).speed(0.1));
                        });
                        
                        ui.separator();
                        
                        // Alarms (for temperature sensors)
                        if channel.sensor_type == "temperature" {
                            ui.label(RichText::new("Alarm Limits").strong());
                            ui.horizontal(|ui| {
                                ui.label("Low:");
                                if let Some(mut low) = channel.alarm_low {
                                    ui.add(egui::DragValue::new(&mut low).speed(1.0));
                                    channel.alarm_low = Some(low);
                                }
                                
                                ui.label("High:");
                                if let Some(mut high) = channel.alarm_high {
                                    ui.add(egui::DragValue::new(&mut high).speed(1.0));
                                    channel.alarm_high = Some(high);
                                }
                            });
                        }
                        
                        // Manual Mode (for universal inputs)
                        if channel.id.starts_with("ui_") {
                            ui.separator();
                            ui.group(|ui| {
                                ui.label(RichText::new("Manual Override Mode").strong().color(Color32::from_rgb(251, 146, 60)));
                                
                                let mut manual_mode = channel.manual_mode.unwrap_or(false);
                                ui.checkbox(&mut manual_mode, "Enable Manual Override");
                                channel.manual_mode = Some(manual_mode);
                                
                                if manual_mode {
                                    ui.horizontal(|ui| {
                                        ui.label("Manual Value:");
                                        let mut manual_value = channel.manual_value.unwrap_or(0.0);
                                        ui.add(egui::DragValue::new(&mut manual_value).speed(0.1));
                                        ui.label(&channel.units);
                                        channel.manual_value = Some(manual_value);
                                    });
                                    ui.label("This static value will override the sensor reading when in manual mode");
                                }
                            });
                        }
                        
                        // Analog Output Configuration
                        if channel.id.starts_with("ao_") {
                            ui.separator();
                            ui.label(RichText::new("Voltage Configuration").strong());
                            
                            ui.horizontal(|ui| {
                                ui.label("Voltage Range:");
                                let mut min_v = channel.min_voltage.unwrap_or(0.0);
                                let mut max_v = channel.max_voltage.unwrap_or(10.0);
                                ui.add(egui::DragValue::new(&mut min_v).suffix("V").speed(0.1).clamp_range(0.0..=10.0));
                                ui.label("to");
                                ui.add(egui::DragValue::new(&mut max_v).suffix("V").speed(0.1).clamp_range(0.0..=10.0));
                                channel.min_voltage = Some(min_v);
                                channel.max_voltage = Some(max_v);
                            });
                            
                            let mut reverse = channel.reverse_acting.unwrap_or(false);
                            ui.checkbox(&mut reverse, "Reverse Acting (100% = Min Voltage)");
                            channel.reverse_acting = Some(reverse);
                        }
                        
                        // Relay Configuration
                        if channel.id.starts_with("ro_") {
                            ui.separator();
                            ui.label(RichText::new("Relay Configuration").strong());
                            
                            let mut nc = channel.normally_closed.unwrap_or(false);
                            ui.checkbox(&mut nc, "Normally Closed (N.C.) - Default state is ON");
                            channel.normally_closed = Some(nc);
                        }
                        
                        ui.separator();
                        
                        // Channel Enable
                        ui.checkbox(&mut channel.enabled, "Channel Enabled");
                        
                        ui.separator();
                        
                        // Dialog buttons
                        ui.horizontal(|ui| {
                            if ui.button("💾 Save Changes").clicked() {
                                self.save_channel_edit(channel);
                                self.show_edit_dialog = false;
                                self.editing_channel = None;
                            }
                            
                            if ui.button("Cancel").clicked() {
                                self.show_edit_dialog = false;
                                self.editing_channel = None;
                            }
                        });
                    });
                });
        }
    }
    
    fn save_channel_edit(&mut self, channel: ChannelConfig) {
        // Find and update the channel in the appropriate list
        if channel.id.starts_with("ui_") {
            if let Some(idx) = self.config.universal_inputs.iter().position(|c| c.id == channel.id) {
                self.config.universal_inputs[idx] = channel;
            }
        } else if channel.id.starts_with("ao_") {
            if let Some(idx) = self.config.analog_outputs.iter().position(|c| c.id == channel.id) {
                self.config.analog_outputs[idx] = channel;
            }
        } else if channel.id.starts_with("ro_") {
            if let Some(idx) = self.config.relay_outputs.iter().position(|c| c.id == channel.id) {
                self.config.relay_outputs[idx] = channel;
            }
        } else if channel.id.starts_with("to_") {
            if let Some(idx) = self.config.triac_outputs.iter().position(|c| c.id == channel.id) {
                self.config.triac_outputs[idx] = channel;
            }
        }
        
        // Auto-save
        self.save_config();
    }
    
    fn save_config(&mut self) {
        self.is_saving = true;
        
        // Save REAL configuration to database
        let config_json = serde_json::to_string(&self.config).unwrap_or_default();
        let board_id = self.config.board_id.clone();
        
        // Write to database
        let result = std::process::Command::new("sqlite3")
            .arg("/var/lib/nexus/nexus.db")
            .arg(&format!(
                "INSERT OR REPLACE INTO board_states (board_id, state_json, updated_at) VALUES ('{}', '{}', datetime('now'))",
                board_id, config_json
            ))
            .output();
        
        match result {
            Ok(output) if output.status.success() => {
                println!("Board configuration saved for {}", self.config.board_id);
            }
            _ => {
                println!("Failed to save board configuration");
            }
        }
        
        self.is_saving = false;
    }
}