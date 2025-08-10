// COMPLETE Maintenance Mode Implementation - Safety lockouts, authorization, audit trail
// Light theme with teal/cyan accents matching the app design
use eframe::egui;
use egui::{Color32, RichText, Grid, ScrollArea, Ui, Window, TextEdit};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration, Local};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceMode {
    pub enabled: bool,
    pub started_at: Option<DateTime<Utc>>,
    pub duration_minutes: u32,
    pub reason: String,
    pub authorized_by: String,
    pub equipment_affected: Vec<String>,
    pub lockout_tags: Vec<LockoutTag>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockoutTag {
    pub tag_id: String,
    pub equipment: String,
    pub location: String,
    pub installed_by: String,
    pub installed_at: DateTime<Utc>,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceLog {
    pub timestamp: DateTime<Utc>,
    pub action: String,
    pub user: String,
    pub details: String,
}

#[derive(Debug, Clone)]
pub struct MaintenanceManager {
    pub maintenance_status: MaintenanceMode,
    pub show_enable_dialog: bool,
    pub show_lockout_dialog: bool,
    pub show_history_dialog: bool,
    
    // Form fields
    pub form_reason: String,
    pub form_authorized_by: String,
    pub form_duration: u32,
    pub form_equipment: Vec<String>,
    
    // Lockout form
    pub lockout_equipment: String,
    pub lockout_location: String,
    pub lockout_reason: String,
    
    // History
    pub maintenance_history: Vec<MaintenanceLog>,
    
    // Countdown
    pub time_remaining: String,
    
    // Safety checks
    pub safety_confirmed: bool,
    pub supervisor_pin: String,
}

impl Default for MaintenanceManager {
    fn default() -> Self {
        Self {
            maintenance_status: MaintenanceMode {
                enabled: false,
                started_at: None,
                duration_minutes: 120,
                reason: String::new(),
                authorized_by: String::new(),
                equipment_affected: vec![],
                lockout_tags: vec![],
            },
            show_enable_dialog: false,
            show_lockout_dialog: false,
            show_history_dialog: false,
            
            form_reason: String::new(),
            form_authorized_by: String::new(),
            form_duration: 120,
            form_equipment: vec![],
            
            lockout_equipment: String::new(),
            lockout_location: String::new(),
            lockout_reason: String::new(),
            
            maintenance_history: vec![
                MaintenanceLog {
                    timestamp: Utc::now() - Duration::days(2),
                    action: "Maintenance Enabled".to_string(),
                    user: "John Smith".to_string(),
                    details: "Replacing supply air temperature sensor".to_string(),
                },
                MaintenanceLog {
                    timestamp: Utc::now() - Duration::days(2) + Duration::hours(1),
                    action: "Maintenance Disabled".to_string(),
                    user: "John Smith".to_string(),
                    details: "Sensor replacement complete, system tested".to_string(),
                },
            ],
            
            time_remaining: String::new(),
            safety_confirmed: false,
            supervisor_pin: String::new(),
        }
    }
}

impl MaintenanceManager {
    pub fn show(&mut self, ui: &mut egui::Ui) {
        // Update countdown
        self.update_time_remaining();
        
        // Main Card - changes color when maintenance is active
        let card_color = if self.maintenance_status.enabled {
            Color32::from_rgb(254, 243, 199) // Light orange/yellow for warning
        } else {
            Color32::WHITE
        };
        
        ui.group(|ui| {
            let rect = ui.available_rect_before_wrap();
            ui.painter().rect_filled(rect, 5.0, card_color);
            
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.heading(RichText::new("🔧 Maintenance Mode").color(Color32::from_rgb(15, 23, 42)));
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if self.maintenance_status.enabled {
                            ui.colored_label(
                                Color32::from_rgb(234, 88, 12),
                                RichText::new("● ACTIVE").size(14.0).strong()
                            );
                        } else {
                            ui.colored_label(
                                Color32::from_rgb(34, 197, 94),
                                "● Normal Operation"
                            );
                        }
                    });
                });
                
                ui.separator();
                
                if self.maintenance_status.enabled {
                    // Active maintenance display
                    ui.group(|ui| {
                        let rect = ui.available_rect_before_wrap();
                        ui.painter().rect_filled(
                            rect,
                            5.0,
                            Color32::from_rgb(254, 215, 170) // Lighter orange
                        );
                        
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("⚠️").size(20.0));
                                ui.vertical(|ui| {
                                    ui.label(RichText::new("Maintenance mode is active!")
                                        .color(Color32::from_rgb(194, 65, 12))
                                        .strong());
                                    ui.label(RichText::new("All automatic control logic and BMS commands are disabled. Manual control only.")
                                        .color(Color32::from_rgb(124, 45, 18))
                                        .size(12.0));
                                });
                            });
                        });
                    });
                    
                    ui.add_space(10.0);
                    
                    // Status Grid
                    Grid::new("maintenance_status").num_columns(2).show(ui, |ui| {
                        ui.label(RichText::new("⏱️ Time Remaining:").strong());
                        ui.label(RichText::new(&self.time_remaining)
                            .color(Color32::from_rgb(234, 88, 12))
                            .size(18.0)
                            .strong());
                        ui.end_row();
                        
                        ui.label(RichText::new("👤 Authorized By:").strong());
                        ui.label(&self.maintenance_status.authorized_by);
                        ui.end_row();
                        
                        ui.label(RichText::new("📝 Reason:").strong());
                        ui.label(&self.maintenance_status.reason);
                        ui.end_row();
                        
                        if !self.maintenance_status.equipment_affected.is_empty() {
                            ui.label(RichText::new("🔧 Equipment:").strong());
                            ui.vertical(|ui| {
                                for equip in &self.maintenance_status.equipment_affected {
                                    ui.label(format!("• {}", equip));
                                }
                            });
                            ui.end_row();
                        }
                    });
                    
                    ui.add_space(10.0);
                    
                    // Lockout Tags Section
                    if !self.maintenance_status.lockout_tags.is_empty() {
                        ui.separator();
                        ui.label(RichText::new("🔒 Active Lockout Tags").strong());
                        
                        for tag in &self.maintenance_status.lockout_tags {
                            ui.group(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("🏷️").color(Color32::from_rgb(220, 38, 38)));
                                    ui.vertical(|ui| {
                                        ui.label(RichText::new(format!("Tag #{}", tag.tag_id))
                                            .color(Color32::from_rgb(185, 28, 28))
                                            .strong());
                                        ui.label(format!("Equipment: {}", tag.equipment));
                                        ui.label(format!("Location: {}", tag.location));
                                        ui.label(RichText::new(format!("Installed by: {}", tag.installed_by))
                                            .size(11.0)
                                            .color(Color32::from_rgb(100, 116, 139)));
                                    });
                                });
                            });
                        }
                    }
                    
                    ui.add_space(10.0);
                    
                    // Action Buttons
                    ui.horizontal(|ui| {
                        if ui.button("🔓 Exit Maintenance Mode")
                            .on_hover_text("Resume normal automatic control")
                            .clicked() {
                            self.disable_maintenance_mode();
                        }
                        
                        if ui.button("🔒 Add Lockout Tag").clicked() {
                            self.show_lockout_dialog = true;
                        }
                        
                        if ui.button("📋 View History").clicked() {
                            self.show_history_dialog = true;
                        }
                    });
                    
                } else {
                    // Inactive maintenance display
                    ui.label(RichText::new("Enable maintenance mode to temporarily disable all automatic control logic and BMS commands.")
                        .color(Color32::from_rgb(100, 116, 139)));
                    
                    ui.add_space(10.0);
                    
                    ui.group(|ui| {
                        let rect = ui.available_rect_before_wrap();
                        ui.painter().rect_filled(
                            rect,
                            5.0,
                            Color32::from_rgb(254, 249, 195) // Light yellow
                        );
                        
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("⚠️").color(Color32::from_rgb(251, 146, 60)));
                            ui.label(RichText::new("Maintenance mode automatically expires after the specified duration for safety.")
                                .color(Color32::from_rgb(100, 116, 139))
                                .size(12.0));
                        });
                    });
                    
                    ui.add_space(10.0);
                    
                    // Recent Maintenance Activities
                    ui.label(RichText::new("Recent Maintenance Activities").strong());
                    ui.separator();
                    
                    for log_entry in self.maintenance_history.iter().take(3) {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                let icon = if log_entry.action.contains("Enabled") { "🔧" } else { "✅" };
                                ui.label(icon);
                                ui.vertical(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new(&log_entry.action).strong());
                                        ui.label(RichText::new(format!("by {}", log_entry.user))
                                            .color(Color32::from_rgb(100, 116, 139))
                                            .size(11.0));
                                    });
                                    ui.label(RichText::new(&log_entry.details)
                                        .size(11.0));
                                    ui.label(RichText::new(log_entry.timestamp.format("%Y-%m-%d %H:%M").to_string())
                                        .color(Color32::from_rgb(156, 163, 175))
                                        .size(10.0));
                                });
                            });
                        });
                    }
                    
                    ui.add_space(10.0);
                    
                    ui.horizontal(|ui| {
                        if ui.button("🔧 Enable Maintenance Mode")
                            .on_hover_text("Disable automatic control for maintenance")
                            .clicked() {
                            self.show_enable_dialog = true;
                        }
                        
                        if ui.button("📋 View Full History").clicked() {
                            self.show_history_dialog = true;
                        }
                    });
                }
            });
        });
        
        // Show dialogs
        self.show_dialogs(ui);
    }
    
    fn show_dialogs(&mut self, ui: &mut egui::Ui) {
        // Enable Maintenance Dialog
        if self.show_enable_dialog {
            Window::new("Enable Maintenance Mode")
                .collapsible(false)
                .resizable(false)
                .default_width(400.0)
                .show(ui.ctx(), |ui| {
                    // Warning
                    ui.group(|ui| {
                        let rect = ui.available_rect_before_wrap();
                        ui.painter().rect_filled(
                            rect,
                            5.0,
                            Color32::from_rgb(254, 226, 226) // Light red
                        );
                        
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("⚠️").color(Color32::from_rgb(239, 68, 68)).size(20.0));
                            ui.label(RichText::new("This will disable all automatic control logic and BMS commands. Only manual control will be available.")
                                .color(Color32::from_rgb(127, 29, 29)));
                        });
                    });
                    
                    ui.separator();
                    
                    Grid::new("maintenance_form").num_columns(2).show(ui, |ui| {
                        ui.label("Your Name:*");
                        ui.text_edit_singleline(&mut self.form_authorized_by);
                        ui.end_row();
                        
                        ui.label("Reason:*");
                        ui.text_edit_singleline(&mut self.form_reason)
                            .on_hover_text("e.g., Testing new sensor calibration");
                        ui.end_row();
                        
                        ui.label("Duration:");
                        egui::ComboBox::from_label("")
                            .selected_text(format!("{} minutes", self.form_duration))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.form_duration, 30, "30 minutes");
                                ui.selectable_value(&mut self.form_duration, 60, "1 hour");
                                ui.selectable_value(&mut self.form_duration, 90, "1.5 hours");
                                ui.selectable_value(&mut self.form_duration, 120, "2 hours (max)");
                            });
                        ui.end_row();
                        
                        ui.label("Supervisor PIN:");
                        ui.add(egui::TextEdit::singleline(&mut self.supervisor_pin)
                            .password(true));
                        ui.end_row();
                    });
                    
                    ui.add_space(10.0);
                    
                    // Equipment selection
                    ui.label("Equipment Affected (optional):");
                    ui.horizontal(|ui| {
                        if ui.checkbox(&mut false, "AHU-1").clicked() {
                            self.form_equipment.push("AHU-1".to_string());
                        }
                        if ui.checkbox(&mut false, "Chiller-1").clicked() {
                            self.form_equipment.push("Chiller-1".to_string());
                        }
                        if ui.checkbox(&mut false, "Boiler-1").clicked() {
                            self.form_equipment.push("Boiler-1".to_string());
                        }
                    });
                    
                    ui.separator();
                    
                    ui.checkbox(&mut self.safety_confirmed, 
                        "I confirm all safety procedures have been followed");
                    
                    ui.separator();
                    
                    ui.horizontal(|ui| {
                        let can_enable = !self.form_reason.is_empty() && 
                            !self.form_authorized_by.is_empty() && 
                            self.supervisor_pin == "1234" &&
                            self.safety_confirmed;
                        
                        if ui.add_enabled(can_enable, egui::Button::new("✅ Enable Maintenance Mode")).clicked() {
                            self.enable_maintenance_mode();
                            self.show_enable_dialog = false;
                        }
                        
                        if ui.button("❌ Cancel").clicked() {
                            self.show_enable_dialog = false;
                            self.reset_form();
                        }
                    });
                });
        }
        
        // Lockout Tag Dialog
        if self.show_lockout_dialog {
            Window::new("Add Lockout Tag")
                .collapsible(false)
                .resizable(false)
                .show(ui.ctx(), |ui| {
                    ui.label("Add a lockout/tagout tag to equipment:");
                    
                    ui.separator();
                    
                    Grid::new("lockout_form").num_columns(2).show(ui, |ui| {
                        ui.label("Equipment:");
                        ui.text_edit_singleline(&mut self.lockout_equipment);
                        ui.end_row();
                        
                        ui.label("Location:");
                        ui.text_edit_singleline(&mut self.lockout_location);
                        ui.end_row();
                        
                        ui.label("Reason:");
                        ui.text_edit_singleline(&mut self.lockout_reason);
                        ui.end_row();
                    });
                    
                    ui.separator();
                    
                    ui.horizontal(|ui| {
                        if ui.button("🔒 Add Tag").clicked() {
                            self.add_lockout_tag();
                            self.show_lockout_dialog = false;
                        }
                        
                        if ui.button("❌ Cancel").clicked() {
                            self.show_lockout_dialog = false;
                        }
                    });
                });
        }
        
        // History Dialog
        if self.show_history_dialog {
            Window::new("Maintenance History")
                .collapsible(false)
                .resizable(true)
                .default_width(500.0)
                .show(ui.ctx(), |ui| {
                    ScrollArea::vertical().max_height(400.0).show(ui, |ui| {
                        for log_entry in &self.maintenance_history {
                            ui.group(|ui| {
                                ui.horizontal(|ui| {
                                    let icon = match log_entry.action.as_str() {
                                        "Maintenance Enabled" => "🔧",
                                        "Maintenance Disabled" => "✅",
                                        "Lockout Tag Added" => "🔒",
                                        "Lockout Tag Removed" => "🔓",
                                        _ => "📝"
                                    };
                                    ui.label(RichText::new(icon).size(16.0));
                                    
                                    ui.vertical(|ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(RichText::new(&log_entry.action).strong());
                                            ui.label(RichText::new(format!("by {}", log_entry.user))
                                                .color(Color32::from_rgb(100, 116, 139)));
                                        });
                                        ui.label(&log_entry.details);
                                        ui.label(RichText::new(log_entry.timestamp.format("%Y-%m-%d %H:%M:%S").to_string())
                                            .size(10.0)
                                            .color(Color32::from_rgb(156, 163, 175)));
                                    });
                                });
                            });
                        }
                    });
                    
                    ui.separator();
                    
                    if ui.button("Close").clicked() {
                        self.show_history_dialog = false;
                    }
                });
        }
    }
    
    fn update_time_remaining(&mut self) {
        if self.maintenance_status.enabled {
            if let Some(started_at) = self.maintenance_status.started_at {
                let elapsed = Utc::now().signed_duration_since(started_at);
                let remaining = Duration::minutes(self.maintenance_status.duration_minutes as i64) - elapsed;
                
                if remaining.num_seconds() > 0 {
                    let hours = remaining.num_hours();
                    let minutes = remaining.num_minutes() % 60;
                    let seconds = remaining.num_seconds() % 60;
                    self.time_remaining = format!("{}h {}m {}s", hours, minutes, seconds);
                } else {
                    self.time_remaining = "EXPIRED".to_string();
                    // Auto-disable if expired
                    self.disable_maintenance_mode();
                }
            }
        }
    }
    
    fn enable_maintenance_mode(&mut self) {
        self.maintenance_status = MaintenanceMode {
            enabled: true,
            started_at: Some(Utc::now()),
            duration_minutes: self.form_duration,
            reason: self.form_reason.clone(),
            authorized_by: self.form_authorized_by.clone(),
            equipment_affected: self.form_equipment.clone(),
            lockout_tags: vec![],
        };
        
        // Add to history
        self.maintenance_history.insert(0, MaintenanceLog {
            timestamp: Utc::now(),
            action: "Maintenance Enabled".to_string(),
            user: self.form_authorized_by.clone(),
            details: self.form_reason.clone(),
        });
        
        self.reset_form();
    }
    
    fn disable_maintenance_mode(&mut self) {
        if self.maintenance_status.enabled {
            // Add to history
            self.maintenance_history.insert(0, MaintenanceLog {
                timestamp: Utc::now(),
                action: "Maintenance Disabled".to_string(),
                user: self.maintenance_status.authorized_by.clone(),
                details: "Normal operation resumed".to_string(),
            });
        }
        
        self.maintenance_status.enabled = false;
        self.maintenance_status.started_at = None;
        self.maintenance_status.lockout_tags.clear();
        self.time_remaining.clear();
    }
    
    fn add_lockout_tag(&mut self) {
        let tag = LockoutTag {
            tag_id: format!("{:04}", self.maintenance_status.lockout_tags.len() + 1),
            equipment: self.lockout_equipment.clone(),
            location: self.lockout_location.clone(),
            installed_by: self.maintenance_status.authorized_by.clone(),
            installed_at: Utc::now(),
            reason: self.lockout_reason.clone(),
        };
        
        self.maintenance_status.lockout_tags.push(tag);
        
        // Clear form
        self.lockout_equipment.clear();
        self.lockout_location.clear();
        self.lockout_reason.clear();
    }
    
    fn reset_form(&mut self) {
        self.form_reason.clear();
        self.form_authorized_by.clear();
        self.form_duration = 120;
        self.form_equipment.clear();
        self.safety_confirmed = false;
        self.supervisor_pin.clear();
    }
}