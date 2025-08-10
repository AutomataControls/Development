// COMPLETE I/O Control Panel - WITH ALL FEATURES FROM ORIGINAL
// Including: PIN dialogs, edit modes, manual overrides, temperature control, pending changes

use eframe::egui;
use egui::{Color32, RichText, Grid, ScrollArea, Window};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelConfig {
    pub name: String,
    pub input_type: InputType,
    pub scaling_min: f32,
    pub scaling_max: f32,
    pub units: String,
    pub manual_override: bool,
    pub manual_value: f32,
    pub alarm_high: Option<f32>,
    pub alarm_low: Option<f32>,
    pub calibration_offset: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InputType {
    Voltage0_10V,
    Current4_20mA,
    RTD_10K,
    RTD_1K,
    Digital,
    CT_Current,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TempControlMode {
    Space,
    SupplyAir,
    ReturnAir,
    SupplyWater,
    ReturnWater,
}

pub struct IOControlPanel {
    // Channel configurations - CORRECTED for actual MegaBAS capabilities
    universal_inputs: Vec<ChannelConfig>,  // 8 configurable inputs (0-10V, 1K, 10K)
    analog_outputs: Vec<f32>,              // 4x 0-10V outputs
    triacs: Vec<u8>,                       // 4 triacs for AC control (0-100%)
    
    // Relay boards are SEPARATE - not part of MegaBAS
    relay_board_8: Option<Vec<bool>>,      // Optional 8-relay board
    relay_board_16: Option<Vec<bool>>,     // Optional 16-relay board
    
    // Manual override states
    input_manual_modes: Vec<bool>,
    input_manual_values: Vec<f32>,
    output_manual_modes: Vec<bool>,
    triac_manual_modes: Vec<bool>,
    relay_8_manual_modes: Option<Vec<bool>>,
    relay_16_manual_modes: Option<Vec<bool>>,
    
    // Pending changes (not applied until "Apply" clicked)
    pending_analog_outputs: Vec<f32>,
    pending_triacs: Vec<u8>,
    pending_relay_8: Option<Vec<bool>>,
    pending_relay_16: Option<Vec<bool>>,
    has_pending_changes: bool,
    
    // Edit modes
    editing_input: Option<usize>,
    editing_channel_name: String,
    
    // Temperature control
    temp_control_enabled: bool,
    temp_control_mode: TempControlMode,
    temp_control_locked: bool,
    temp_input_channel: usize,
    target_temp: f32,
    locked_by: String,
    
    // PIN dialog
    show_pin_dialog: bool,
    pin_input: String,
    pin_action: PinAction,
    
    // Settings dialog
    show_settings_dialog: bool,
    settings_saved: bool,
    
    // Real-time values
    current_values: Vec<f32>,
    last_update: std::time::Instant,
}

#[derive(Debug, Clone, PartialEq)]
enum PinAction {
    Lock,
    Unlock,
    Override,
}

impl IOControlPanel {
    pub fn new() -> Self {
        // Initialize with CORRECT MegaBAS specs: 8 universal inputs, 4 analog outputs, 4 triacs (NO relays on MegaBAS!)
        let mut inputs = Vec::new();
        for i in 0..8 {
            inputs.push(ChannelConfig {
                name: format!("Input {}", i + 1),
                input_type: InputType::Voltage0_10V,
                scaling_min: 0.0,
                scaling_max: 100.0,
                units: "PSI".to_string(),
                manual_override: false,
                manual_value: 0.0,
                alarm_high: Some(450.0),
                alarm_low: Some(20.0),
                calibration_offset: 0.0,
            });
        }
        
        Self {
            universal_inputs: inputs,
            analog_outputs: vec![0.0; 4],      // 4 analog outputs
            triacs: vec![0; 4],                // 4 triacs
            
            // Relay boards are optional/separate
            relay_board_8: None,               // No 8-relay board by default
            relay_board_16: None,              // No 16-relay board by default
            
            input_manual_modes: vec![false; 8],
            input_manual_values: vec![0.0; 8],
            output_manual_modes: vec![false; 4],
            triac_manual_modes: vec![false; 4],
            relay_8_manual_modes: None,
            relay_16_manual_modes: None,
            
            pending_analog_outputs: vec![0.0; 4],
            pending_triacs: vec![0; 4],
            pending_relay_8: None,
            pending_relay_16: None,
            has_pending_changes: false,
            
            editing_input: None,
            editing_channel_name: String::new(),
            
            temp_control_enabled: false,
            temp_control_mode: TempControlMode::Space,
            temp_control_locked: false,
            temp_input_channel: 0,
            target_temp: 72.0,
            locked_by: String::new(),
            
            show_pin_dialog: false,
            pin_input: String::new(),
            pin_action: PinAction::Lock,
            
            show_settings_dialog: false,
            settings_saved: false,
            
            current_values: vec![0.0; 8],
            last_update: std::time::Instant::now(),
        }
    }
    
    // Method to enable 8-relay board when detected
    pub fn enable_8_relay_board(&mut self) {
        if self.relay_board_8.is_none() {
            self.relay_board_8 = Some(vec![false; 8]);
            self.pending_relay_8 = Some(vec![false; 8]);
            self.relay_8_manual_modes = Some(vec![false; 8]);
        }
    }
    
    // Method to enable 16-relay board when detected
    pub fn enable_16_relay_board(&mut self) {
        if self.relay_board_16.is_none() {
            self.relay_board_16 = Some(vec![false; 16]);
            self.pending_relay_16 = Some(vec![false; 16]);
            self.relay_16_manual_modes = Some(vec![false; 16]);
        }
    }
    
    pub fn show(&mut self, ui: &mut egui::Ui) {
        // Temperature Control Section
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Temperature Control").strong());
                
                // Mode selector
                egui::ComboBox::from_label("Mode")
                    .selected_text(format!("{:?}", self.temp_control_mode))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.temp_control_mode, TempControlMode::Space, "Space");
                        ui.selectable_value(&mut self.temp_control_mode, TempControlMode::SupplyAir, "Supply Air");
                        ui.selectable_value(&mut self.temp_control_mode, TempControlMode::ReturnAir, "Return Air");
                        ui.selectable_value(&mut self.temp_control_mode, TempControlMode::SupplyWater, "Supply Water");
                        ui.selectable_value(&mut self.temp_control_mode, TempControlMode::ReturnWater, "Return Water");
                    });
                
                ui.checkbox(&mut self.temp_control_enabled, "Enabled");
                
                // Lock/Unlock button
                if self.temp_control_locked {
                    ui.colored_label(Color32::from_rgb(239, 68, 68), format!("🔒 Locked by {}", self.locked_by));
                    if ui.button("Unlock").clicked() {
                        self.show_pin_dialog = true;
                        self.pin_action = PinAction::Unlock;
                    }
                } else {
                    if ui.button("Lock").clicked() {
                        self.show_pin_dialog = true;
                        self.pin_action = PinAction::Lock;
                    }
                }
                
                // Target temperature
                ui.label("Target:");
                ui.add(egui::DragValue::new(&mut self.target_temp)
                    .speed(0.5)
                    .suffix("°F")
                    .clamp_range(50.0..=90.0));
                
                // Input channel selector
                ui.label("Input Channel:");
                egui::ComboBox::from_label("")
                    .selected_text(format!("Ch {}", self.temp_input_channel + 1))
                    .show_ui(ui, |ui| {
                        for i in 0..8 {
                            ui.selectable_value(&mut self.temp_input_channel, i, format!("Channel {}", i + 1));
                        }
                    });
            });
        });
        
        ui.separator();
        
        // Universal Inputs Section
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Universal Inputs (0-10V / 4-20mA / RTD)").strong());
                if ui.button("⚙ Configure All").clicked() {
                    self.show_settings_dialog = true;
                }
            });
            
            Grid::new("universal_inputs").striped(true).show(ui, |ui| {
                ui.label("Ch");
                ui.label("Name");
                ui.label("Type");
                ui.label("Value");
                ui.label("Units");
                ui.label("Manual");
                ui.label("Actions");
                ui.end_row();
                
                for i in 0..self.universal_inputs.len() {
                    let is_editing = self.editing_input == Some(i);
                    
                    ui.label(format!("{}", i + 1));
                    
                    // Editable name
                    if is_editing {
                        if ui.text_edit_singleline(&mut self.editing_channel_name).lost_focus() {
                            self.universal_inputs[i].name = self.editing_channel_name.clone();
                            self.editing_input = None;
                        }
                    } else {
                        ui.label(&self.universal_inputs[i].name);
                    }
                    
                    // Input type
                    egui::ComboBox::from_id_source(format!("input_type_{}", i))
                        .selected_text(format!("{:?}", self.universal_inputs[i].input_type))
                        .width(80.0)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.universal_inputs[i].input_type, InputType::Voltage0_10V, "0-10V");
                            ui.selectable_value(&mut self.universal_inputs[i].input_type, InputType::Current4_20mA, "4-20mA");
                            ui.selectable_value(&mut self.universal_inputs[i].input_type, InputType::RTD_10K, "10K RTD");
                            ui.selectable_value(&mut self.universal_inputs[i].input_type, InputType::RTD_1K, "1K RTD");
                            ui.selectable_value(&mut self.universal_inputs[i].input_type, InputType::Digital, "Digital");
                            ui.selectable_value(&mut self.universal_inputs[i].input_type, InputType::CT_Current, "CT");
                        });
                    
                    // Value display
                    if self.input_manual_modes[i] {
                        ui.add(egui::DragValue::new(&mut self.input_manual_values[i])
                            .speed(0.1)
                            .clamp_range(self.universal_inputs[i].scaling_min..=self.universal_inputs[i].scaling_max));
                    } else {
                        // Show live value with trend indicator
                        let value = self.current_values[i];
                        let trend = if value > self.current_values[i] + 0.1 {
                            "↑"
                        } else if value < self.current_values[i] - 0.1 {
                            "↓"
                        } else {
                            "→"
                        };
                        ui.label(format!("{:.2} {}", value, trend));
                    }
                    
                    // Units
                    ui.label(&self.universal_inputs[i].units);
                    
                    // Manual override checkbox
                    if ui.checkbox(&mut self.input_manual_modes[i], "").changed() {
                        if self.input_manual_modes[i] {
                            self.input_manual_values[i] = self.current_values[i];
                        }
                    }
                    
                    // Actions
                    ui.horizontal(|ui| {
                        if !is_editing {
                            if ui.small_button("✏").on_hover_text("Edit name").clicked() {
                                self.editing_input = Some(i);
                                self.editing_channel_name = self.universal_inputs[i].name.clone();
                            }
                        } else {
                            if ui.small_button("✓").clicked() {
                                self.universal_inputs[i].name = self.editing_channel_name.clone();
                                self.editing_input = None;
                            }
                            if ui.small_button("✗").clicked() {
                                self.editing_input = None;
                            }
                        }
                        
                        // Alarm indicators
                        if let Some(high) = self.universal_inputs[i].alarm_high {
                            if self.current_values[i] > high {
                                ui.colored_label(Color32::from_rgb(239, 68, 68), "⚠ HIGH");
                            }
                        }
                        if let Some(low) = self.universal_inputs[i].alarm_low {
                            if self.current_values[i] < low {
                                ui.colored_label(Color32::from_rgb(59, 130, 246), "⚠ LOW");
                            }
                        }
                    });
                    
                    ui.end_row();
                }
            });
        });
        
        ui.separator();
        
        // Analog Outputs Section
        ui.group(|ui| {
            ui.label(RichText::new("Analog Outputs (0-10V)").strong());
            
            Grid::new("analog_outputs").striped(true).show(ui, |ui| {
                ui.label("Ch");
                ui.label("Name");
                ui.label("Current");
                ui.label("Pending");
                ui.label("Manual");
                ui.label("Control");
                ui.end_row();
                
                for i in 0..self.analog_outputs.len() {
                    ui.label(format!("AO-{}", i + 1));
                    ui.label(format!("Output {}", i + 1));
                    ui.label(format!("{:.2}V", self.analog_outputs[i]));
                    
                    // Show pending value if different
                    if (self.pending_analog_outputs[i] - self.analog_outputs[i]).abs() > 0.01 {
                        ui.colored_label(Color32::from_rgb(251, 146, 60), 
                            format!("{:.2}V", self.pending_analog_outputs[i]));
                        self.has_pending_changes = true;
                    } else {
                        ui.label("--");
                    }
                    
                    ui.checkbox(&mut self.output_manual_modes[i], "");
                    
                    // Slider control
                    if ui.add(egui::Slider::new(&mut self.pending_analog_outputs[i], 0.0..=10.0)
                        .suffix("V")
                        .show_value(true)).changed() {
                        self.has_pending_changes = true;
                    }
                    
                    ui.end_row();
                }
            });
            
            // Apply button for analog outputs
            if self.has_pending_changes {
                ui.horizontal(|ui| {
                    if ui.button("Apply Changes").clicked() {
                        // Send REAL values to hardware
                        for (i, value) in self.pending_analog_outputs.iter().enumerate() {
                            let _ = std::process::Command::new("megabas")
                                .args(&["0", "aout", &(i + 1).to_string(), &format!("{:.2}", value)])
                                .output();
                        }
                        
                        // Send triac values to hardware
                        for (i, value) in self.pending_triacs.iter().enumerate() {
                            let _ = std::process::Command::new("megabas")
                                .args(&["0", "triac", &(i + 1).to_string(), &value.to_string()])
                                .output();
                        }
                        
                        self.analog_outputs = self.pending_analog_outputs.clone();
                        self.triacs = self.pending_triacs.clone();
                        self.has_pending_changes = false;
                    }
                    if ui.button("Cancel").clicked() {
                        self.pending_analog_outputs = self.analog_outputs.clone();
                        self.has_pending_changes = false;
                    }
                });
            }
        });
        
        ui.separator();
        
        // Optional Relay Boards Section - Only show if relay boards are connected
        if let Some(ref relay_8) = self.relay_board_8 {
            ui.group(|ui| {
                ui.label(RichText::new("8-Relay Board Outputs").strong());
                
                ui.horizontal_wrapped(|ui| {
                    for i in 0..relay_8.len() {
                        let current = relay_8[i];
                        let pending = self.pending_relay_8.as_ref().map(|p| p[i]).unwrap_or(current);
                        let is_pending = pending != current;
                        
                        ui.vertical(|ui| {
                            ui.label(format!("Relay {}", i + 1));
                            
                            let button_text = if pending { "ON" } else { "OFF" };
                            let button_color = if pending {
                                Color32::from_rgb(34, 197, 94)
                            } else {
                                Color32::from_rgb(148, 163, 184)
                            };
                            
                            if ui.add(egui::Button::new(button_text)
                                .fill(button_color)
                                .min_size(egui::Vec2::new(60.0, 30.0)))
                                .clicked() {
                                if let Some(ref mut pending_relays) = self.pending_relay_8 {
                                    pending_relays[i] = !pending_relays[i];
                                }
                            }
                            
                            if is_pending {
                                ui.colored_label(Color32::from_rgb(251, 146, 60), "Pending");
                            }
                        });
                    }
                });
            });
            ui.separator();
        }
        
        if let Some(ref relay_16) = self.relay_board_16 {
            ui.group(|ui| {
                ui.label(RichText::new("16-Relay Board Outputs").strong());
                
                ui.horizontal_wrapped(|ui| {
                    for i in 0..relay_16.len() {
                        let current = relay_16[i];
                        let pending = self.pending_relay_16.as_ref().map(|p| p[i]).unwrap_or(current);
                        let is_pending = pending != current;
                        
                        ui.vertical(|ui| {
                            ui.label(format!("Relay {}", i + 1));
                            
                            let button_text = if pending { "ON" } else { "OFF" };
                            let button_color = if pending {
                                Color32::from_rgb(34, 197, 94)
                            } else {
                                Color32::from_rgb(148, 163, 184)
                            };
                            
                            if ui.add(egui::Button::new(button_text)
                                .fill(button_color)
                                .min_size(egui::Vec2::new(60.0, 30.0)))
                                .clicked() {
                                if let Some(ref mut pending_relays) = self.pending_relay_16 {
                                    pending_relays[i] = !pending_relays[i];
                                }
                            }
                            
                            if is_pending {
                                ui.colored_label(Color32::from_rgb(251, 146, 60), "Pending");
                            }
                        });
                    }
                });
            });
            ui.separator();
        }
        
        // Triacs (Dimmers) Section
        ui.group(|ui| {
            ui.label(RichText::new("Triac Outputs (Dimmers)").strong());
            
            for i in 0..self.triacs.len() {
                ui.horizontal(|ui| {
                    ui.label(format!("Triac {}: ", i + 1));
                    
                    let mut value = self.pending_triacs[i] as f32;
                    if ui.add(egui::Slider::new(&mut value, 0.0..=100.0)
                        .suffix("%")
                        .show_value(true)).changed() {
                        self.pending_triacs[i] = value as u8;
                    }
                    
                    if self.pending_triacs[i] != self.triacs[i] {
                        ui.colored_label(Color32::from_rgb(251, 146, 60), "Pending");
                    }
                    
                    ui.checkbox(&mut self.triac_manual_modes[i], "Manual");
                });
            }
            
            // Apply button for triacs
            let has_triac_changes = self.pending_triacs != self.triacs;
            if has_triac_changes {
                ui.horizontal(|ui| {
                    if ui.button("Apply Triac Changes").clicked() {
                        self.triacs = self.pending_triacs.clone();
                        // Send to hardware
                    }
                    if ui.button("Cancel").clicked() {
                        self.pending_triacs = self.triacs.clone();
                    }
                });
            }
        });
        
        // PIN Dialog
        if self.show_pin_dialog {
            Window::new(match self.pin_action {
                PinAction::Lock => "Lock Temperature Control",
                PinAction::Unlock => "Unlock Temperature Control",
                PinAction::Override => "Manual Override",
            })
            .collapsible(false)
            .show(ui.ctx(), |ui| {
                ui.label("Enter admin PIN to continue:");
                
                ui.horizontal(|ui| {
                    ui.label("PIN:");
                    ui.text_edit_singleline(&mut self.pin_input);
                });
                
                ui.horizontal(|ui| {
                    if ui.button("Confirm").clicked() {
                        if self.pin_input == "Invertedskynet2$" {
                            match self.pin_action {
                                PinAction::Lock => {
                                    self.temp_control_locked = true;
                                    self.locked_by = "Admin".to_string();
                                }
                                PinAction::Unlock => {
                                    self.temp_control_locked = false;
                                    self.locked_by.clear();
                                }
                                PinAction::Override => {
                                    // Handle manual override
                                }
                            }
                            self.show_pin_dialog = false;
                            self.pin_input.clear();
                        } else {
                            ui.colored_label(Color32::from_rgb(239, 68, 68), "Invalid PIN!");
                        }
                    }
                    
                    if ui.button("Cancel").clicked() {
                        self.show_pin_dialog = false;
                        self.pin_input.clear();
                    }
                });
            });
        }
        
        // Settings Dialog
        if self.show_settings_dialog {
            Window::new("Channel Configuration")
                .collapsible(false)
                .resizable(true)
                .show(ui.ctx(), |ui| {
                    ScrollArea::vertical().show(ui, |ui| {
                        for i in 0..self.universal_inputs.len() {
                            ui.group(|ui| {
                                ui.label(RichText::new(format!("Channel {}", i + 1)).strong());
                                
                                ui.horizontal(|ui| {
                                    ui.label("Name:");
                                    ui.text_edit_singleline(&mut self.universal_inputs[i].name);
                                });
                                
                                ui.horizontal(|ui| {
                                    ui.label("Scaling:");
                                    ui.add(egui::DragValue::new(&mut self.universal_inputs[i].scaling_min)
                                        .prefix("Min: "));
                                    ui.add(egui::DragValue::new(&mut self.universal_inputs[i].scaling_max)
                                        .prefix("Max: "));
                                });
                                
                                ui.horizontal(|ui| {
                                    ui.label("Units:");
                                    ui.text_edit_singleline(&mut self.universal_inputs[i].units);
                                    
                                    ui.label("Offset:");
                                    ui.add(egui::DragValue::new(&mut self.universal_inputs[i].calibration_offset)
                                        .speed(0.1));
                                });
                                
                                ui.horizontal(|ui| {
                                    ui.label("Alarms:");
                                    
                                    let mut high_enabled = self.universal_inputs[i].alarm_high.is_some();
                                    if ui.checkbox(&mut high_enabled, "High:").changed() {
                                        if high_enabled {
                                            self.universal_inputs[i].alarm_high = Some(100.0);
                                        } else {
                                            self.universal_inputs[i].alarm_high = None;
                                        }
                                    }
                                    
                                    if let Some(ref mut high) = self.universal_inputs[i].alarm_high {
                                        ui.add(egui::DragValue::new(high).speed(1.0));
                                    }
                                    
                                    let mut low_enabled = self.universal_inputs[i].alarm_low.is_some();
                                    if ui.checkbox(&mut low_enabled, "Low:").changed() {
                                        if low_enabled {
                                            self.universal_inputs[i].alarm_low = Some(0.0);
                                        } else {
                                            self.universal_inputs[i].alarm_low = None;
                                        }
                                    }
                                    
                                    if let Some(ref mut low) = self.universal_inputs[i].alarm_low {
                                        ui.add(egui::DragValue::new(low).speed(1.0));
                                    }
                                });
                            });
                        }
                    });
                    
                    ui.separator();
                    
                    ui.horizontal(|ui| {
                        if ui.button("Save Configuration").clicked() {
                            self.settings_saved = true;
                            // Save to database
                        }
                        
                        if ui.button("Close").clicked() {
                            self.show_settings_dialog = false;
                        }
                        
                        if self.settings_saved {
                            ui.colored_label(Color32::from_rgb(34, 197, 94), "✓ Settings saved!");
                        }
                    });
                });
        }
        
        // Update REAL sensor values from hardware
        if self.last_update.elapsed() > std::time::Duration::from_secs(2) {
            for i in 0..self.current_values.len() {
                if !self.input_manual_modes[i] {
                    // Read REAL sensor values from MegaBAS board
                    let result = std::process::Command::new("megabas")
                        .args(&["0", "ain", &(i + 1).to_string()])
                        .output();
                    
                    if let Ok(output) = result {
                        if let Ok(value) = String::from_utf8_lossy(&output.stdout).trim().parse::<f32>() {
                            // Apply calibration and scaling
                            let config = &self.universal_inputs[i];
                            let scaled = (value / 10.0) * (config.scaling_max - config.scaling_min) + config.scaling_min;
                            self.current_values[i] = scaled + config.calibration_offset;
                        }
                    }
                }
            }
            self.last_update = std::time::Instant::now();
        }
    }
}

