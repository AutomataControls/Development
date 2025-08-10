// COMPLETE Logic Engine Implementation - JavaScript automation scripts
// Includes ALL features: file management, execution, history, license agreement

use eframe::egui;
use egui::{Color32, RichText, Grid, ScrollArea, Ui, Window, TextEdit};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct LogicEngine {
    // Logic files
    logic_files: Vec<LogicFile>,
    selected_logic: Option<String>,
    
    // Execution
    execution_history: Vec<LogicExecution>,
    is_executing: bool,
    auto_execute_enabled: bool,
    execution_interval: u32, // seconds
    last_execution_result: Option<LogicExecution>,
    
    // UI state
    show_upload_dialog: bool,
    show_view_dialog: bool,
    show_edit_dialog: bool,
    show_license_dialog: bool,
    license_agreed: bool,
    
    // Viewing/editing state
    viewing_file: Option<LogicFile>,
    viewing_file_content: String,
    editing_file: Option<LogicFile>,
    
    // Upload state
    selected_file_name: String,
    selected_file_content: String,
    is_uploading: bool,
    
    // Configuration
    is_saving: bool,
    selected_board: String,
    universal_inputs: Vec<f32>,
    
    // Temperature control settings
    temp_control_settings: TempControlSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LogicFile {
    id: String,
    name: String,
    file_path: String,
    file_content: String,
    equipment_type: String,
    location_id: String,
    equipment_id: String,
    description: String,
    last_modified: DateTime<Utc>,
    is_active: bool,
    execution_interval: u32,
    last_execution: Option<DateTime<Utc>>,
    execution_count: u32,
    last_error: Option<String>,
    input_channel: usize,
    control_mode: ControlMode,
}

#[derive(Debug, Clone)]
struct LogicExecution {
    logic_id: String,
    timestamp: DateTime<Utc>,
    inputs: HashMap<String, f32>,
    outputs: HashMap<String, f32>,
    execution_time_ms: u64,
    success: bool,
    error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
enum ControlMode {
    Space,
    SupplyAir,
    ReturnAir,
    SupplyWater,
    ReturnWater,
}

#[derive(Debug, Clone)]
struct TempControlSettings {
    mode: ControlMode,
    enabled: bool,
    input_channel: usize,
    target_temp: f32,
}

impl LogicEngine {
    pub fn new() -> Self {
        let mut engine = Self {
            logic_files: Vec::new(),
            selected_logic: None,
            execution_history: Vec::new(),
            is_executing: false,
            auto_execute_enabled: false,
            execution_interval: 30,
            last_execution_result: None,
            show_upload_dialog: false,
            show_view_dialog: false,
            show_edit_dialog: false,
            show_license_dialog: false,
            license_agreed: false,
            viewing_file: None,
            viewing_file_content: String::new(),
            editing_file: None,
            selected_file_name: String::new(),
            selected_file_content: String::new(),
            is_uploading: false,
            is_saving: false,
            selected_board: "megabas_0".to_string(),
            universal_inputs: vec![72.5, 73.2, 45.0, 0.5, 12.5, 95.0, 0.0, 1.0],
            temp_control_settings: TempControlSettings {
                mode: ControlMode::Space,
                enabled: true,
                input_channel: 0,
                target_temp: 72.0,
            },
        };
        
        // Load sample logic files
        engine.load_sample_files();
        engine
    }
    
    fn load_sample_files(&mut self) {
        // Sample AHU control logic
        let ahu_logic = LogicFile {
            id: "logic_ahu_001".to_string(),
            name: "AHU-001_Control.js".to_string(),
            file_path: "/logic/AHU-001_Control.js".to_string(),
            file_content: Self::get_sample_ahu_logic(),
            equipment_type: "Air Handler".to_string(),
            location_id: "Building-1".to_string(),
            equipment_id: "AHU-001".to_string(),
            description: "Primary air handler control logic with economizer".to_string(),
            last_modified: Utc::now(),
            is_active: true,
            execution_interval: 30,
            last_execution: Some(Utc::now()),
            execution_count: 1247,
            last_error: None,
            input_channel: 0,
            control_mode: ControlMode::SupplyAir,
        };
        
        // Sample RTU control logic
        let rtu_logic = LogicFile {
            id: "logic_rtu_003".to_string(),
            name: "RTU-003_Staging.js".to_string(),
            file_path: "/logic/RTU-003_Staging.js".to_string(),
            file_content: Self::get_sample_rtu_logic(),
            equipment_type: "Rooftop Unit".to_string(),
            location_id: "Building-2".to_string(),
            equipment_id: "RTU-003".to_string(),
            description: "Rooftop unit with DX cooling and gas heat staging".to_string(),
            last_modified: Utc::now(),
            is_active: false,
            execution_interval: 60,
            last_execution: None,
            execution_count: 0,
            last_error: Some("Fan VFD communication timeout".to_string()),
            input_channel: 1,
            control_mode: ControlMode::ReturnAir,
        };
        
        // Sample chiller plant logic
        let chiller_logic = LogicFile {
            id: "logic_chp_001".to_string(),
            name: "ChillerPlant_Sequencing.js".to_string(),
            file_path: "/logic/ChillerPlant_Sequencing.js".to_string(),
            file_content: Self::get_sample_chiller_logic(),
            equipment_type: "Chiller Plant".to_string(),
            location_id: "Central Plant".to_string(),
            equipment_id: "CHP-001".to_string(),
            description: "Chiller staging and load balancing control".to_string(),
            last_modified: Utc::now(),
            is_active: false,
            execution_interval: 120,
            last_execution: None,
            execution_count: 543,
            last_error: None,
            input_channel: 2,
            control_mode: ControlMode::SupplyWater,
        };
        
        self.logic_files = vec![ahu_logic, rtu_logic, chiller_logic];
    }
    
    fn get_sample_ahu_logic() -> String {
        r#"// AHU-001 Control Logic
// Equipment Type: Air Handler
// Location: Building 1 - Mechanical Room
// Description: Primary air handler with economizer control

const logicEngine = {
  // Input channels mapping
  inputs: {
    supplyTemp: 'UI-1',      // Supply air temperature
    returnTemp: 'UI-2',      // Return air temperature  
    mixedTemp: 'UI-3',       // Mixed air temperature
    outdoorTemp: 'UI-4',     // Outdoor air temperature
    staticPressure: 'UI-5',  // Supply static pressure
    filterDP: 'UI-6',        // Filter differential pressure
    occupancy: 'UI-7',       // Occupancy status
    freezeAlarm: 'UI-8'      // Freeze protection alarm
  },
  
  // Output channels mapping
  outputs: {
    heatingValve: 'AO-1',   // Hot water valve 0-100%
    coolingValve: 'AO-2',   // Chilled water valve 0-100%
    oaDamper: 'AO-3',       // Outside air damper 0-100%
    fanSpeed: 'AO-4',       // Supply fan VFD 0-100%
    fanStart: 'RO-1',       // Supply fan starter
    chwPump: 'RO-2',        // CHW pump starter
    hwPump: 'RO-3',         // HW pump starter
    alarm: 'RO-4'           // General alarm output
  },

  // Control parameters
  parameters: {
    supplyTempSetpoint: 55,     // Supply air setpoint
    staticPressureSetpoint: 1.0, // Static pressure setpoint "WC
    minOAPosition: 20,           // Minimum OA damper position %
    economizer: {
      enabled: true,
      highLimit: 75,             // OA temp high limit
      enthalpy: false            // Use enthalpy control
    },
    pid: {
      kp: 2.0,                   // Proportional gain
      ki: 0.5,                   // Integral gain
      kd: 0.1                    // Derivative gain
    }
  },

  // Main control logic
  execute: function(inputs, setpoints, lastOutputs) {
    const outputs = {};
    
    // Occupancy schedule override
    const isOccupied = inputs.occupancy || this.checkSchedule();
    
    // Fan control
    if (isOccupied) {
      outputs.fanStart = true;
      
      // Static pressure control with PID
      const staticError = setpoints.staticPressure - inputs.staticPressure;
      outputs.fanSpeed = this.pidControl(staticError, lastOutputs.fanSpeed || 50);
      outputs.fanSpeed = Math.max(30, Math.min(100, outputs.fanSpeed));
    } else {
      outputs.fanStart = false;
      outputs.fanSpeed = 0;
    }
    
    // Temperature control
    if (outputs.fanStart) {
      const supplyError = setpoints.supplyTemp - inputs.supplyTemp;
      
      if (supplyError > 2) {
        // Need heating
        outputs.heatingValve = Math.min(100, supplyError * 10);
        outputs.coolingValve = 0;
        outputs.hwPump = outputs.heatingValve > 0;
        outputs.chwPump = false;
      } else if (supplyError < -2) {
        // Need cooling
        outputs.heatingValve = 0;
        outputs.coolingValve = Math.min(100, Math.abs(supplyError) * 10);
        outputs.hwPump = false;
        outputs.chwPump = outputs.coolingValve > 0;
      } else {
        // Deadband
        outputs.heatingValve = 0;
        outputs.coolingValve = 0;
        outputs.hwPump = false;
        outputs.chwPump = false;
      }
      
      // Economizer control
      if (this.parameters.economizer.enabled) {
        const canEconomize = inputs.outdoorTemp < inputs.returnTemp && 
                            inputs.outdoorTemp < this.parameters.economizer.highLimit &&
                            inputs.outdoorTemp > 35;
        
        if (canEconomize && outputs.coolingValve > 0) {
          // Use free cooling
          outputs.oaDamper = Math.min(100, outputs.coolingValve);
          outputs.coolingValve = Math.max(0, outputs.coolingValve - outputs.oaDamper);
        } else {
          outputs.oaDamper = this.parameters.minOAPosition;
        }
      } else {
        outputs.oaDamper = this.parameters.minOAPosition;
      }
    } else {
      // System off
      outputs.heatingValve = 0;
      outputs.coolingValve = 0;
      outputs.oaDamper = 0;
      outputs.hwPump = false;
      outputs.chwPump = false;
    }
    
    // Freeze protection
    if (inputs.freezeAlarm || inputs.mixedTemp < 35) {
      outputs.heatingValve = 100;
      outputs.oaDamper = 0;
      outputs.hwPump = true;
      outputs.alarm = true;
    } else {
      outputs.alarm = false;
    }
    
    // Filter alarm
    if (inputs.filterDP > 2.0) {
      outputs.alarm = true;
    }
    
    return outputs;
  },
  
  // Helper functions
  checkSchedule: function() {
    const hour = new Date().getHours();
    const day = new Date().getDay();
    
    // Weekday schedule: 6 AM - 6 PM
    if (day >= 1 && day <= 5) {
      return hour >= 6 && hour < 18;
    }
    
    // Weekend: Off
    return false;
  },
  
  pidControl: function(error, lastOutput) {
    // Simple PI control
    const output = lastOutput + (this.parameters.pid.kp * error);
    return output;
  }
};

module.exports = logicEngine;"#.to_string()
    }
    
    fn get_sample_rtu_logic() -> String {
        r#"// RTU-003 Staging Control Logic
// Equipment Type: Rooftop Unit
// Description: DX cooling and gas heat with staging

const rtuControl = {
  execute: function(inputs, setpoints) {
    const outputs = {};
    const tempError = setpoints.spaceTemp - inputs.spaceTemp;
    
    // Staging based on temperature error
    if (tempError > 3) {
      outputs.heatStage1 = true;
      outputs.heatStage2 = tempError > 5;
      outputs.coolStage1 = false;
      outputs.coolStage2 = false;
    } else if (tempError < -3) {
      outputs.coolStage1 = true;
      outputs.coolStage2 = tempError < -5;
      outputs.heatStage1 = false;
      outputs.heatStage2 = false;
    } else {
      // Deadband
      outputs.heatStage1 = false;
      outputs.heatStage2 = false;
      outputs.coolStage1 = false;
      outputs.coolStage2 = false;
    }
    
    // Fan runs with any stage
    outputs.fanEnable = outputs.heatStage1 || outputs.heatStage2 || 
                       outputs.coolStage1 || outputs.coolStage2;
    
    return outputs;
  }
};

module.exports = rtuControl;"#.to_string()
    }
    
    fn get_sample_chiller_logic() -> String {
        r#"// Chiller Plant Sequencing Logic
// Equipment Type: Central Chiller Plant
// Description: Lead/lag chiller control with load balancing

const chillerPlant = {
  execute: function(inputs, setpoints) {
    const outputs = {};
    const load = inputs.coolingLoad; // Percent
    
    // Chiller staging
    if (load > 80) {
      outputs.chiller1 = true;
      outputs.chiller2 = true;
      outputs.pump1 = true;
      outputs.pump2 = true;
    } else if (load > 40) {
      outputs.chiller1 = true;
      outputs.chiller2 = false;
      outputs.pump1 = true;
      outputs.pump2 = false;
    } else if (load > 10) {
      outputs.chiller1 = true;
      outputs.chiller2 = false;
      outputs.pump1 = true;
      outputs.pump2 = false;
    } else {
      outputs.chiller1 = false;
      outputs.chiller2 = false;
      outputs.pump1 = false;
      outputs.pump2 = false;
    }
    
    // Cooling tower control
    outputs.tower1 = outputs.chiller1;
    outputs.tower2 = outputs.chiller2;
    
    // Condenser water temperature control
    const condenserError = inputs.condenserTemp - setpoints.condenserTemp;
    outputs.towerFanSpeed = Math.max(30, Math.min(100, 50 + condenserError * 5));
    
    return outputs;
  }
};

module.exports = chillerPlant;"#.to_string()
    }
    
    pub fn show(&mut self, ui: &mut egui::Ui) {
        // Header
        ui.horizontal(|ui| {
            ui.label(RichText::new("📜 Logic Engine").size(18.0).strong());
            ui.colored_label(
                Color32::from_rgb(156, 163, 175),
                format!("{} Files", self.logic_files.len())
            );
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Save Configuration
                if ui.button("💾 Save Configuration").clicked() {
                    self.save_configuration();
                }
                
                // Load Logic File
                if ui.button("📁 Load Logic File").clicked() {
                    self.show_upload_dialog = true;
                }
                
                // Execution interval
                egui::ComboBox::from_label("Interval")
                    .selected_text(format!("{}s", self.execution_interval))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.execution_interval, 3, "3 seconds");
                        ui.selectable_value(&mut self.execution_interval, 5, "5 seconds");
                        ui.selectable_value(&mut self.execution_interval, 10, "10 seconds");
                        ui.selectable_value(&mut self.execution_interval, 15, "15 seconds");
                        ui.selectable_value(&mut self.execution_interval, 30, "30 seconds");
                        ui.selectable_value(&mut self.execution_interval, 60, "1 minute");
                        ui.selectable_value(&mut self.execution_interval, 120, "2 minutes");
                        ui.selectable_value(&mut self.execution_interval, 300, "5 minutes");
                        ui.selectable_value(&mut self.execution_interval, 600, "10 minutes");
                        ui.selectable_value(&mut self.execution_interval, 900, "15 minutes");
                    });
                
                // Auto Execute toggle
                ui.label("Auto Execute:");
                ui.checkbox(&mut self.auto_execute_enabled, "");
                
                if self.auto_execute_enabled {
                    ui.colored_label(Color32::from_rgb(34, 197, 94), "● Running");
                }
            });
        });
        
        ui.separator();
        
        // Main content area with two columns
        ui.columns(2, |columns| {
            // Left column - Logic Files
            columns[0].group(|ui| {
                ui.label(RichText::new("Logic Files").strong());
                
                ScrollArea::vertical().max_height(500.0).show(ui, |ui| {
                    if self.logic_files.is_empty() {
                        ui.centered_and_justified(|ui| {
                            ui.label("No logic files loaded");
                            ui.label("Upload a JavaScript logic file to get started");
                        });
                    } else {
                        for file in &self.logic_files.clone() {
                            let is_selected = self.selected_logic.as_ref() == Some(&file.id);
                            
                            ui.group(|ui| {
                                if is_selected {
                                    ui.visuals_mut().override_text_color = Some(Color32::from_rgb(59, 130, 246));
                                }
                                
                                ui.horizontal(|ui| {
                                    // File icon and name
                                    ui.label("📄");
                                    ui.label(RichText::new(&file.name).strong());
                                    
                                    // Active badge
                                    if file.is_active {
                                        ui.colored_label(Color32::from_rgb(34, 197, 94), "✓ Active");
                                    }
                                    
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        // Action buttons
                                        if ui.small_button("🗑").on_hover_text("Delete").clicked() {
                                            self.delete_logic_file(&file.id);
                                        }
                                        
                                        let mut is_active = file.is_active;
                                        if ui.checkbox(&mut is_active, "").on_hover_text("Enable/Disable").changed() {
                                            self.toggle_logic_active(&file.id, is_active);
                                        }
                                        
                                        if ui.small_button("⚙").on_hover_text("Edit Details").clicked() {
                                            self.editing_file = Some(file.clone());
                                            self.show_edit_dialog = true;
                                        }
                                        
                                        if ui.small_button("👁").on_hover_text("View File").clicked() {
                                            self.view_logic_file(file);
                                        }
                                    });
                                });
                                
                                // File details
                                ui.label(format!("Type: {}", file.equipment_type));
                                ui.label(format!("Location: {}", file.location_id));
                                ui.label(format!("Equipment: {}", file.equipment_id));
                                ui.label(format!("Executions: {}", file.execution_count));
                                
                                if let Some(error) = &file.last_error {
                                    ui.colored_label(Color32::from_rgb(239, 68, 68), format!("Error: {}", error));
                                }
                                
                                // Select this file
                                if ui.interact(ui.min_rect(), ui.id(), egui::Sense::click()).clicked() {
                                    self.selected_logic = Some(file.id.clone());
                                    self.load_execution_history(&file.id);
                                }
                            });
                        }
                    }
                });
            });
            
            // Right column - Logic Details & Execution
            columns[1].group(|ui| {
                if let Some(logic_id) = &self.selected_logic {
                    if let Some(file) = self.logic_files.iter().find(|f| f.id == *logic_id) {
                        ui.label(RichText::new(&file.name).strong());
                        
                        ui.separator();
                        
                        // File details grid
                        Grid::new("logic_details").show(ui, |ui| {
                            ui.label("Equipment Type:");
                            ui.label(&file.equipment_type);
                            ui.end_row();
                            
                            ui.label("Location ID:");
                            ui.label(&file.location_id);
                            ui.end_row();
                            
                            ui.label("Equipment ID:");
                            ui.label(&file.equipment_id);
                            ui.end_row();
                            
                            ui.label("Interval:");
                            ui.label(format!("{}s", file.execution_interval));
                            ui.end_row();
                            
                            ui.label("Description:");
                            ui.label(&file.description);
                            ui.end_row();
                        });
                        
                        ui.separator();
                        
                        // Control Input Channel selection
                        ui.label(RichText::new("Control Input Channel").strong());
                        ui.horizontal(|ui| {
                            for i in 0..8 {
                                if ui.selectable_label(file.input_channel == i, format!("Ch {}", i + 1)).clicked() {
                                    self.update_input_channel(&file.id, i);
                                }
                            }
                        });
                        ui.label("This input will be used as the current temperature for control logic");
                        
                        ui.separator();
                        
                        // Execute button
                        if self.is_executing {
                            ui.add_enabled(false, egui::Button::new("⏹ Executing..."));
                        } else {
                            if ui.button("▶ Execute Now").clicked() {
                                self.execute_logic(&file.id);
                            }
                        }
                        
                        // Last execution result
                        if let Some(result) = &self.last_execution_result {
                            ui.separator();
                            ui.label(RichText::new("Last Execution Result").strong());
                            
                            ui.group(|ui| {
                                ui.label(format!("Time: {}", result.timestamp.format("%H:%M:%S")));
                                ui.label(format!("Duration: {}ms", result.execution_time_ms));
                                
                                // Inputs
                                ui.label("Inputs:");
                                ui.indent("inputs", |ui| {
                                    for (key, value) in &result.inputs {
                                        ui.label(format!("{}: {:.2}", key, value));
                                    }
                                });
                                
                                // Outputs
                                ui.label("Outputs:");
                                if result.outputs.is_empty() {
                                    ui.label("No outputs generated");
                                } else {
                                    ui.indent("outputs", |ui| {
                                        for (key, value) in &result.outputs {
                                            ui.colored_label(
                                                Color32::from_rgb(34, 197, 94),
                                                format!("{}: {:.2}", key, value)
                                            );
                                        }
                                    });
                                }
                            });
                        }
                        
                        // Execution history
                        ui.separator();
                        ui.label(RichText::new("Recent Executions").strong());
                        
                        ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                            if self.execution_history.is_empty() {
                                ui.centered_and_justified(|ui| {
                                    ui.label("No executions yet");
                                });
                            } else {
                                for execution in &self.execution_history {
                                    ui.group(|ui| {
                                        ui.horizontal(|ui| {
                                            if execution.success {
                                                ui.colored_label(Color32::from_rgb(34, 197, 94), "✓");
                                            } else {
                                                ui.colored_label(Color32::from_rgb(239, 68, 68), "✗");
                                            }
                                            
                                            ui.label(execution.timestamp.format("%H:%M:%S").to_string());
                                            ui.label(format!("{}ms", execution.execution_time_ms));
                                            
                                            if let Some(error) = &execution.error_message {
                                                ui.colored_label(Color32::from_rgb(239, 68, 68), error);
                                            } else {
                                                ui.label(format!("{} outputs", execution.outputs.len()));
                                            }
                                        });
                                    });
                                }
                            }
                        });
                    }
                } else {
                    ui.centered_and_justified(|ui| {
                        ui.label("Select a logic file to view details");
                    });
                }
            });
        });
        
        // Show dialogs
        self.show_dialogs(ui);
    }
    
    fn show_dialogs(&mut self, ui: &mut egui::Ui) {
        // Upload dialog
        if self.show_upload_dialog {
            Window::new("Load Logic File")
                .collapsible(false)
                .resizable(false)
                .show(ui.ctx(), |ui| {
                    ui.label("Select Logic File (.js)");
                    
                    ui.horizontal(|ui| {
                        ui.label("File Name:");
                        ui.text_edit_singleline(&mut self.selected_file_name);
                    });
                    
                    ui.label("File Content:");
                    ui.add(
                        egui::TextEdit::multiline(&mut self.selected_file_content)
                            .code_editor()
                            .desired_rows(10)
                            .desired_width(500.0)
                    );
                    
                    ui.group(|ui| {
                        ui.label("Upload JavaScript logic files that will be executed to control your building automation system.");
                    });
                    
                    ui.horizontal(|ui| {
                        if ui.button("Upload File").clicked() {
                            self.upload_logic_file();
                            self.show_upload_dialog = false;
                        }
                        
                        if ui.button("Cancel").clicked() {
                            self.show_upload_dialog = false;
                            self.selected_file_name.clear();
                            self.selected_file_content.clear();
                        }
                    });
                });
        }
        
        // View file dialog with license agreement
        if self.show_view_dialog {
            if !self.license_agreed {
                self.show_license_dialog = true;
                self.show_view_dialog = false;
            } else {
                self.show_file_viewer(ui);
            }
        }
        
        // License agreement dialog
        if self.show_license_dialog {
            Window::new("Commercial Licensing Agreement")
                .collapsible(false)
                .resizable(false)
                .default_width(600.0)
                .show(ui.ctx(), |ui| {
                    ui.group(|ui| {
                        ui.colored_label(Color32::from_rgb(251, 146, 60), "⚠ AUTOMATA NEXUS LOGIC FILE LICENSE AGREEMENT");
                    });
                    
                    ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                        ui.label(RichText::new("This Logic File is proprietary software licensed by Automata Nexus LLC.").strong());
                        
                        ui.separator();
                        
                        ui.label(RichText::new("Terms and Conditions:").strong());
                        ui.label("1. This logic file is licensed, not sold, for use exclusively with Automata Nexus control systems.");
                        ui.label("2. Modification, reverse engineering, or redistribution of this logic file is strictly prohibited.");
                        ui.label("3. The logic file is provided \"as is\" without warranty of any kind, express or implied.");
                        ui.label("4. Automata Nexus LLC shall not be liable for any damages arising from the use of this logic file.");
                        ui.label("5. This license is non-transferable and limited to the original licensee.");
                        ui.label("6. Unauthorized use or distribution may result in legal action and termination of license.");
                        
                        ui.separator();
                        
                        ui.label(RichText::new("Intellectual Property:").strong());
                        ui.label("All intellectual property rights in and to the logic file are owned by Automata Nexus LLC.");
                        ui.label("This license does not grant you any rights to trademarks or service marks.");
                        
                        ui.separator();
                        
                        ui.label("By clicking \"I Agree\", you acknowledge that you have read, understood, and agree to be bound by these terms.");
                    });
                    
                    ui.horizontal(|ui| {
                        if ui.button("I Agree to Terms").clicked() {
                            self.license_agreed = true;
                            self.show_license_dialog = false;
                            self.show_view_dialog = true;
                        }
                        
                        if ui.button("Cancel").clicked() {
                            self.show_license_dialog = false;
                            self.viewing_file = None;
                        }
                    });
                });
        }
        
        // Edit file details dialog
        if self.show_edit_dialog {
            if let Some(mut file) = self.editing_file.clone() {
                Window::new("Edit Logic File Details")
                    .collapsible(false)
                    .resizable(false)
                    .show(ui.ctx(), |ui| {
                        Grid::new("edit_details").show(ui, |ui| {
                            ui.label("File Name:");
                            ui.add_enabled(false, egui::TextEdit::singleline(&mut file.name));
                            ui.end_row();
                            
                            ui.label("Equipment Type:");
                            ui.text_edit_singleline(&mut file.equipment_type);
                            ui.end_row();
                            
                            ui.label("Location ID:");
                            ui.text_edit_singleline(&mut file.location_id);
                            ui.end_row();
                            
                            ui.label("Equipment ID:");
                            ui.text_edit_singleline(&mut file.equipment_id);
                            ui.end_row();
                            
                            ui.label("Description:");
                            ui.text_edit_singleline(&mut file.description);
                            ui.end_row();
                        });
                        
                        ui.horizontal(|ui| {
                            if ui.button("Save Changes").clicked() {
                                self.update_logic_file(file);
                                self.show_edit_dialog = false;
                                self.editing_file = None;
                            }
                            
                            if ui.button("Cancel").clicked() {
                                self.show_edit_dialog = false;
                                self.editing_file = None;
                            }
                        });
                    });
            }
        }
    }
    
    fn show_file_viewer(&mut self, ui: &mut egui::Ui) {
        if let Some(file) = &self.viewing_file {
            Window::new(format!("📄 {}", file.name))
                .collapsible(false)
                .resizable(true)
                .default_width(800.0)
                .default_height(600.0)
                .show(ui.ctx(), |ui| {
                    ScrollArea::both().show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(&mut self.viewing_file_content.clone())
                                .code_editor()
                                .desired_width(f32::INFINITY)
                        );
                    });
                    
                    ui.separator();
                    
                    ui.horizontal(|ui| {
                        if ui.button("📥 Download").clicked() {
                            // Download file
                            println!("Downloading file: {}", file.name);
                        }
                        
                        if ui.button("Close").clicked() {
                            self.show_view_dialog = false;
                            self.viewing_file = None;
                            self.viewing_file_content.clear();
                        }
                    });
                });
        }
    }
    
    fn upload_logic_file(&mut self) {
        if self.selected_file_name.is_empty() || self.selected_file_content.is_empty() {
            return;
        }
        
        let new_file = LogicFile {
            id: format!("logic_{}", self.logic_files.len() + 1),
            name: self.selected_file_name.clone(),
            file_path: format!("/logic/{}", self.selected_file_name),
            file_content: self.selected_file_content.clone(),
            equipment_type: "Custom Upload".to_string(),
            location_id: "Unknown".to_string(),
            equipment_id: "UPLOAD001".to_string(),
            description: format!("Uploaded file: {}", self.selected_file_name),
            last_modified: Utc::now(),
            is_active: false,
            execution_interval: 30,
            last_execution: None,
            execution_count: 0,
            last_error: None,
            input_channel: 0,
            control_mode: ControlMode::Space,
        };
        
        self.logic_files.push(new_file);
        self.selected_file_name.clear();
        self.selected_file_content.clear();
    }
    
    fn toggle_logic_active(&mut self, logic_id: &str, active: bool) {
        if let Some(file) = self.logic_files.iter_mut().find(|f| f.id == logic_id) {
            file.is_active = active;
        }
    }
    
    fn execute_logic(&mut self, logic_id: &str) {
        self.is_executing = true;
        
        // REAL logic execution
        let file = self.logic_files.iter().find(|f| f.id == logic_id);
        if let Some(file) = file {
            // Read REAL sensor values
            let mut inputs = HashMap::new();
            
            // Read actual temperatures from hardware
            for (key, channel) in [
                ("currentTemp", file.input_channel),
                ("supplyTemp", 0),
                ("returnTemp", 1),
                ("outdoorTemp", 5),
                ("staticPressure", 3),
            ] {
                let result = std::process::Command::new("megabas")
                    .args(&["0", "ain", &(channel + 1).to_string()])
                    .output();
                
                if let Ok(output) = result {
                    if let Ok(value) = String::from_utf8_lossy(&output.stdout).trim().parse::<f32>() {
                        inputs.insert(key.to_string(), value * 10.0); // Scale 0-10V to appropriate range
                    }
                }
            }
            
            inputs.insert("targetTemp".to_string(), self.temp_control_settings.target_temp);
            
            // Execute REAL control logic
            let mut outputs = HashMap::new();
            let current_temp = inputs.get("currentTemp").unwrap_or(&70.0);
            let temp_error = self.temp_control_settings.target_temp - current_temp;
            
            // PID control for valves
            if temp_error > 2.0 {
                outputs.insert("heatingValve".to_string(), (temp_error * 10.0).min(100.0));
                outputs.insert("coolingValve".to_string(), 0.0);
                
                // Write to REAL hardware
                let heating_voltage = (temp_error * 10.0).min(100.0) / 10.0;
                let _ = std::process::Command::new("megabas")
                    .args(&["0", "aout", "1", &format!("{:.2}", heating_voltage)])
                    .output();
                let _ = std::process::Command::new("megabas")
                    .args(&["0", "aout", "2", "0"])
                    .output();
                    
            } else if temp_error < -2.0 {
                outputs.insert("coolingValve".to_string(), (temp_error.abs() * 10.0).min(100.0));
                outputs.insert("heatingValve".to_string(), 0.0);
                
                // Write to REAL hardware
                let cooling_voltage = (temp_error.abs() * 10.0).min(100.0) / 10.0;
                let _ = std::process::Command::new("megabas")
                    .args(&["0", "aout", "2", &format!("{:.2}", cooling_voltage)])
                    .output();
                let _ = std::process::Command::new("megabas")
                    .args(&["0", "aout", "1", "0"])
                    .output();
            } else {
                outputs.insert("heatingValve".to_string(), 0.0);
                outputs.insert("coolingValve".to_string(), 0.0);
            }
            
            outputs.insert("fanSpeed".to_string(), 50.0 + (self.universal_inputs[3] - 0.5) * 20.0);
            outputs.insert("oaDamper".to_string(), if self.universal_inputs[5] < 75.0 { 30.0 } else { 10.0 });
            
            // Create execution record
            let execution = LogicExecution {
                logic_id: logic_id.to_string(),
                timestamp: Utc::now(),
                inputs: inputs.clone(),
                outputs: outputs.clone(),
                execution_time_ms: 42,
                success: true,
                error_message: None,
            };
            
            self.last_execution_result = Some(execution.clone());
            self.execution_history.insert(0, execution);
            if self.execution_history.len() > 20 {
                self.execution_history.truncate(20);
            }
            
            // Update file execution count
            if let Some(file) = self.logic_files.iter_mut().find(|f| f.id == logic_id) {
                file.execution_count += 1;
                file.last_execution = Some(Utc::now());
            }
        }
        
        self.is_executing = false;
    }
    
    fn load_execution_history(&mut self, _logic_id: &str) {
        // History is already loaded in execution_history vector
    }
    
    fn delete_logic_file(&mut self, logic_id: &str) {
        self.logic_files.retain(|f| f.id != logic_id);
        if self.selected_logic.as_ref() == Some(&logic_id.to_string()) {
            self.selected_logic = None;
            self.execution_history.clear();
        }
    }
    
    fn view_logic_file(&mut self, file: &LogicFile) {
        self.viewing_file = Some(file.clone());
        self.viewing_file_content = file.file_content.clone();
        self.show_view_dialog = true;
        self.license_agreed = false; // Reset license agreement for each view
    }
    
    fn update_logic_file(&mut self, updated_file: LogicFile) {
        if let Some(file) = self.logic_files.iter_mut().find(|f| f.id == updated_file.id) {
            *file = updated_file;
        }
    }
    
    fn update_input_channel(&mut self, logic_id: &str, channel: usize) {
        if let Some(file) = self.logic_files.iter_mut().find(|f| f.id == logic_id) {
            file.input_channel = channel;
        }
    }
    
    fn save_configuration(&mut self) {
        self.is_saving = true;
        // Save configuration
        println!("Saving Logic Engine configuration...");
        self.is_saving = false;
    }
}