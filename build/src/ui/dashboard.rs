use eframe::egui;
use egui_plot::{Line, Plot, PlotPoints};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::state::AppState;
use super::ColorScheme;

pub struct Dashboard {
    app_state: Arc<Mutex<AppState>>,
}

impl Dashboard {
    pub fn new(app_state: Arc<Mutex<AppState>>) -> Self {
        Self { app_state }
    }
    
    pub fn show(&mut self, ui: &mut egui::Ui, colors: &ColorScheme) {
        ui.heading(
            egui::RichText::new("System Dashboard")
                .size(24.0)
                .color(colors.primary)
        );
        
        ui.separator();
        ui.add_space(8.0);
        
        // Create grid layout for cards
        egui::Grid::new("dashboard_grid")
            .num_columns(3)
            .spacing([16.0, 16.0])
            .show(ui, |ui| {
                // System Status Card
                self.draw_card(ui, colors, "System Status", |ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("●").color(colors.success));
                        ui.label("All Systems Operational");
                    });
                    ui.separator();
                    ui.label(format!("CPU Usage: {}%", 45));
                    ui.label(format!("Memory: {} MB / {} MB", 1024, 4096));
                    ui.label(format!("Temperature: {}°C", 52));
                });
                
                // Board Status Card
                self.draw_card(ui, colors, "Connected Boards", |ui| {
                    ui.label("Megabas I/O: Connected");
                    ui.label("8-Relay Board: Connected");
                    ui.label("Building Automation: Active");
                    ui.separator();
                    ui.label(egui::RichText::new("4 Boards Online").color(colors.success));
                });
                
                // Sensor Status Card
                self.draw_card(ui, colors, "Vibration Sensors", |ui| {
                    ui.label("WTVB01-485 #1: Active");
                    ui.label("WTVB01-485 #2: Active");
                    ui.separator();
                    ui.label(format!("Last Reading: {} mm/s", 1.2));
                    ui.label(egui::RichText::new("Status: Good").color(colors.success));
                });
                
                ui.end_row();
                
                // Protocol Status Card
                self.draw_card(ui, colors, "BMS Protocols", |ui| {
                    ui.label("BACnet: 2 devices");
                    ui.label("Modbus TCP: 3 devices");
                    ui.label("Modbus RTU: 1 device");
                    ui.separator();
                    ui.label(egui::RichText::new("6 Devices Total").color(colors.primary));
                });
                
                // Weather Card
                self.draw_card(ui, colors, "Weather", |ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("☀").size(32.0));
                        ui.vertical(|ui| {
                            ui.label("72°F");
                            ui.label("Partly Cloudy");
                        });
                    });
                    ui.separator();
                    ui.label("Humidity: 65%");
                    ui.label("Wind: 8 mph");
                });
                
                // Alarms Card
                self.draw_card(ui, colors, "Active Alarms", |ui| {
                    ui.colored_label(colors.success, "No Active Alarms");
                    ui.separator();
                    ui.label("Last Alarm: 2 days ago");
                    ui.label("Total Today: 0");
                });
            });
        
        ui.add_space(16.0);
        
        // Performance Graph
        egui::Frame::none()
            .fill(colors.surface)
            .rounding(8.0)
            .inner_margin(16.0)
            .show(ui, |ui| {
                ui.label(
                    egui::RichText::new("System Performance")
                        .size(18.0)
                        .color(colors.primary)
                );
                
                ui.separator();
                
                let plot = Plot::new("performance_plot")
                    .height(200.0)
                    .show_axes([true, true])
                    .show_grid([true, true]);
                
                plot.show(ui, |plot_ui| {
                    // Generate sample data
                    let points: PlotPoints = (0..100)
                        .map(|i| {
                            let x = i as f64;
                            let y = 50.0 + 20.0 * (x * 0.1).sin();
                            [x, y]
                        })
                        .collect();
                    
                    plot_ui.line(Line::new(points).color(colors.primary));
                });
            });
    }
    
    fn draw_card<F>(&self, ui: &mut egui::Ui, colors: &ColorScheme, title: &str, content: F)
    where
        F: FnOnce(&mut egui::Ui),
    {
        egui::Frame::none()
            .fill(colors.surface)
            .rounding(8.0)
            .inner_margin(16.0)
            .shadow(egui::epaint::Shadow::small_light())
            .show(ui, |ui| {
                ui.set_min_size(egui::Vec2::new(250.0, 150.0));
                
                ui.label(
                    egui::RichText::new(title)
                        .size(16.0)
                        .strong()
                        .color(colors.primary)
                );
                
                ui.separator();
                ui.add_space(8.0);
                
                content(ui);
            });
    }
}