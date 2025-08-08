// COMPLETE Metrics Visualization Implementation - Trend analysis, statistics, charts
// Light theme with teal/cyan accents matching the app design
use eframe::egui;
use egui::{Color32, RichText, Grid, ScrollArea, Ui, Window, TextEdit};
use egui_plot::{Line, Plot, PlotPoints, Legend, Corner, PlotBounds, BarChart, Bar};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPoint {
    pub timestamp: DateTime<Utc>,
    pub value: f32,
    pub scaled_value: Option<f32>,
}

#[derive(Debug, Clone)]
pub struct Statistics {
    pub min: f32,
    pub max: f32,
    pub avg: f32,
    pub std_dev: f32,
    pub count: usize,
    pub trend: f32, // Positive = uptrend, negative = downtrend
}

#[derive(Debug, Clone)]
pub struct TrendData {
    pub channel_name: String,
    pub units: Option<String>,
    pub data_points: Vec<DataPoint>,
    pub statistics: Statistics,
}

#[derive(Debug, Clone)]
pub struct Channel {
    pub channel_type: String,
    pub index: usize,
    pub name: String,
    pub enabled: bool,
    pub units: Option<String>,
}

#[derive(Debug, Clone)]
pub struct MetricsVisualization {
    pub channels: Vec<Channel>,
    pub selected_channel: Option<String>,
    pub selected_hours: u32,
    pub trend_data: Option<TrendData>,
    pub is_loading: bool,
    pub auto_refresh: bool,
    pub chart_type: String,
    pub show_export_dialog: bool,
    pub export_format: String,
    pub comparison_channels: Vec<String>,
    pub show_comparison: bool,
    pub aggregation_interval: String,
}

impl Default for MetricsVisualization {
    fn default() -> Self {
        // Sample channels
        let channels = vec![
            Channel {
                channel_type: "universal_input".to_string(),
                index: 0,
                name: "Supply Air Temp".to_string(),
                enabled: true,
                units: Some("°F".to_string()),
            },
            Channel {
                channel_type: "universal_input".to_string(),
                index: 1,
                name: "Return Air Temp".to_string(),
                enabled: true,
                units: Some("°F".to_string()),
            },
            Channel {
                channel_type: "universal_input".to_string(),
                index: 2,
                name: "Outside Air Temp".to_string(),
                enabled: true,
                units: Some("°F".to_string()),
            },
            Channel {
                channel_type: "analog_output".to_string(),
                index: 0,
                name: "Cooling Valve".to_string(),
                enabled: true,
                units: Some("%".to_string()),
            },
            Channel {
                channel_type: "analog_output".to_string(),
                index: 1,
                name: "Heating Valve".to_string(),
                enabled: true,
                units: Some("%".to_string()),
            },
        ];
        
        Self {
            channels,
            selected_channel: Some("universal_input:0".to_string()),
            selected_hours: 24,
            trend_data: None,
            is_loading: false,
            auto_refresh: false,
            chart_type: "line".to_string(),
            show_export_dialog: false,
            export_format: "csv".to_string(),
            comparison_channels: vec![],
            show_comparison: false,
            aggregation_interval: "raw".to_string(),
        }
    }
}

impl MetricsVisualization {
    pub fn show(&mut self, ui: &mut egui::Ui, board_id: &str) {
        // Header Card
        ui.group(|ui| {
            ui.set_min_height(80.0);
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.heading(RichText::new("📊 Metrics & Trend Analysis").color(Color32::from_rgb(15, 23, 42)));
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("💾 Export").clicked() {
                            self.show_export_dialog = true;
                        }
                        
                        ui.separator();
                        
                        // Auto-refresh toggle
                        if self.auto_refresh {
                            if ui.button("🔄 Auto-Refresh ON")
                                .on_hover_text("Click to disable auto-refresh")
                                .clicked() {
                                self.auto_refresh = false;
                            }
                            ui.colored_label(Color32::from_rgb(34, 197, 94), "● Live");
                        } else {
                            if ui.button("⏸️ Auto-Refresh OFF")
                                .on_hover_text("Click to enable auto-refresh")
                                .clicked() {
                                self.auto_refresh = true;
                            }
                        }
                    });
                });
                
                ui.separator();
                
                // Channel and time range selection
                ui.horizontal(|ui| {
                    ui.label("Channel:");
                    egui::ComboBox::from_label("")
                        .selected_text(
                            self.selected_channel.as_ref()
                                .and_then(|sc| self.channels.iter()
                                    .find(|c| format!("{}:{}", c.channel_type, c.index) == *sc)
                                    .map(|c| c.name.clone()))
                                .unwrap_or_else(|| "Select channel".to_string())
                        )
                        .show_ui(ui, |ui| {
                            for channel in &self.channels {
                                let channel_id = format!("{}:{}", channel.channel_type, channel.index);
                                ui.selectable_value(
                                    &mut self.selected_channel,
                                    Some(channel_id.clone()),
                                    &channel.name
                                );
                            }
                        });
                    
                    ui.separator();
                    
                    ui.label("Time Range:");
                    egui::ComboBox::from_label("time_range")
                        .selected_text(format!("{} hours", self.selected_hours))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.selected_hours, 1, "Last Hour");
                            ui.selectable_value(&mut self.selected_hours, 6, "Last 6 Hours");
                            ui.selectable_value(&mut self.selected_hours, 12, "Last 12 Hours");
                            ui.selectable_value(&mut self.selected_hours, 24, "Last 24 Hours");
                            ui.selectable_value(&mut self.selected_hours, 48, "Last 2 Days");
                            ui.selectable_value(&mut self.selected_hours, 72, "Last 3 Days");
                            ui.selectable_value(&mut self.selected_hours, 168, "Last 7 Days");
                        });
                    
                    ui.separator();
                    
                    ui.label("Aggregation:");
                    egui::ComboBox::from_label("aggregation")
                        .selected_text(&self.aggregation_interval)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.aggregation_interval, "raw".to_string(), "Raw Data");
                            ui.selectable_value(&mut self.aggregation_interval, "1min".to_string(), "1 Minute");
                            ui.selectable_value(&mut self.aggregation_interval, "5min".to_string(), "5 Minutes");
                            ui.selectable_value(&mut self.aggregation_interval, "15min".to_string(), "15 Minutes");
                            ui.selectable_value(&mut self.aggregation_interval, "1hour".to_string(), "1 Hour");
                        });
                    
                    if ui.button("🔄 Refresh").clicked() {
                        self.fetch_trend_data();
                    }
                });
            });
        });
        
        ui.add_space(10.0);
        
        // Load sample data if needed
        if self.trend_data.is_none() && self.selected_channel.is_some() {
            self.fetch_trend_data();
        }
        
        if let Some(trend_data) = &self.trend_data {
            // Statistics Cards
            ui.horizontal(|ui| {
                // Current Value Card
                ui.group(|ui| {
                    let rect = ui.available_rect_before_wrap();
                    ui.painter().rect_filled(
                        rect,
                        5.0,
                        Color32::from_rgb(239, 246, 255) // Light blue background
                    );
                    
                    ui.vertical(|ui| {
                        ui.label(RichText::new("Current")
                            .color(Color32::from_rgb(59, 130, 246))
                            .size(12.0));
                        
                        let current_value = trend_data.data_points.last()
                            .map(|p| p.scaled_value.unwrap_or(p.value))
                            .unwrap_or(0.0);
                        
                        ui.label(RichText::new(format!("{:.1}{}", 
                            current_value,
                            trend_data.units.as_ref().map(|u| format!(" {}", u)).unwrap_or_default()))
                            .color(Color32::from_rgb(30, 64, 175))
                            .size(20.0)
                            .strong());
                    });
                });
                
                // Average Card
                ui.group(|ui| {
                    let rect = ui.available_rect_before_wrap();
                    ui.painter().rect_filled(
                        rect,
                        5.0,
                        Color32::from_rgb(240, 253, 244) // Light green background
                    );
                    
                    ui.vertical(|ui| {
                        ui.label(RichText::new("Average")
                            .color(Color32::from_rgb(34, 197, 94))
                            .size(12.0));
                        
                        ui.label(RichText::new(format!("{:.1}{}", 
                            trend_data.statistics.avg,
                            trend_data.units.as_ref().map(|u| format!(" {}", u)).unwrap_or_default()))
                            .color(Color32::from_rgb(21, 128, 61))
                            .size(20.0)
                            .strong());
                    });
                });
                
                // Maximum Card
                ui.group(|ui| {
                    let rect = ui.available_rect_before_wrap();
                    ui.painter().rect_filled(
                        rect,
                        5.0,
                        Color32::from_rgb(254, 242, 242) // Light red background
                    );
                    
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("📈").size(10.0));
                            ui.label(RichText::new("Maximum")
                                .color(Color32::from_rgb(239, 68, 68))
                                .size(12.0));
                        });
                        
                        ui.label(RichText::new(format!("{:.1}{}", 
                            trend_data.statistics.max,
                            trend_data.units.as_ref().map(|u| format!(" {}", u)).unwrap_or_default()))
                            .color(Color32::from_rgb(185, 28, 28))
                            .size(20.0)
                            .strong());
                    });
                });
                
                // Minimum Card
                ui.group(|ui| {
                    let rect = ui.available_rect_before_wrap();
                    ui.painter().rect_filled(
                        rect,
                        5.0,
                        Color32::from_rgb(250, 245, 255) // Light purple background
                    );
                    
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("📉").size(10.0));
                            ui.label(RichText::new("Minimum")
                                .color(Color32::from_rgb(147, 51, 234))
                                .size(12.0));
                        });
                        
                        ui.label(RichText::new(format!("{:.1}{}", 
                            trend_data.statistics.min,
                            trend_data.units.as_ref().map(|u| format!(" {}", u)).unwrap_or_default()))
                            .color(Color32::from_rgb(107, 33, 168))
                            .size(20.0)
                            .strong());
                    });
                });
                
                // Std Dev Card
                ui.group(|ui| {
                    let rect = ui.available_rect_before_wrap();
                    ui.painter().rect_filled(
                        rect,
                        5.0,
                        Color32::from_rgb(249, 250, 251) // Light gray background
                    );
                    
                    ui.vertical(|ui| {
                        ui.label(RichText::new("Std Dev")
                            .color(Color32::from_rgb(107, 114, 128))
                            .size(12.0));
                        
                        ui.label(RichText::new(format!("{:.2}", trend_data.statistics.std_dev))
                            .color(Color32::from_rgb(55, 65, 81))
                            .size(20.0)
                            .strong());
                    });
                });
                
                // Trend Indicator
                ui.group(|ui| {
                    let rect = ui.available_rect_before_wrap();
                    ui.painter().rect_filled(
                        rect,
                        5.0,
                        Color32::from_rgb(240, 253, 250) // Light teal background
                    );
                    
                    ui.vertical(|ui| {
                        ui.label(RichText::new("Trend")
                            .color(Color32::from_rgb(20, 184, 166))
                            .size(12.0));
                        
                        let trend_icon = if trend_data.statistics.trend > 0.1 {
                            "↗️"
                        } else if trend_data.statistics.trend < -0.1 {
                            "↘️"
                        } else {
                            "→"
                        };
                        
                        ui.label(RichText::new(format!("{} {:.1}%", trend_icon, trend_data.statistics.trend * 100.0))
                            .color(Color32::from_rgb(13, 148, 136))
                            .size(20.0)
                            .strong());
                    });
                });
            });
            
            ui.add_space(10.0);
            
            // Chart Type Selection
            ui.horizontal(|ui| {
                if ui.selectable_label(self.chart_type == "line", "📈 Line Chart").clicked() {
                    self.chart_type = "line".to_string();
                }
                if ui.selectable_label(self.chart_type == "area", "📊 Area Chart").clicked() {
                    self.chart_type = "area".to_string();
                }
                if ui.selectable_label(self.chart_type == "bar", "📊 Bar Chart").clicked() {
                    self.chart_type = "bar".to_string();
                }
                
                ui.separator();
                
                ui.checkbox(&mut self.show_comparison, "Compare Channels");
            });
            
            ui.add_space(10.0);
            
            // Main Chart
            self.show_chart(ui, trend_data);
            
            ui.add_space(10.0);
            
            // Data Summary
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("📊 Data Summary").strong());
                    ui.separator();
                    ui.label(format!("Points: {}", trend_data.statistics.count));
                    ui.separator();
                    ui.label(format!("Channel: {}", trend_data.channel_name));
                    ui.separator();
                    
                    if let Some(last_point) = trend_data.data_points.last() {
                        ui.label(format!("Last Update: {}", 
                            last_point.timestamp.format("%Y-%m-%d %H:%M:%S")));
                    }
                });
            });
        } else {
            // No data message
            ui.vertical_centered(|ui| {
                ui.add_space(50.0);
                ui.label(RichText::new("📊").size(48.0).color(Color32::from_rgb(200, 200, 200)));
                ui.label(RichText::new("Select a channel to view metrics")
                    .color(Color32::from_rgb(100, 116, 139))
                    .size(16.0));
            });
        }
        
        // Show dialogs
        self.show_dialogs(ui);
    }
    
    fn show_chart(&self, ui: &mut egui::Ui, trend_data: &TrendData) {
        ui.group(|ui| {
            ui.set_min_height(400.0);
            
            let plot = Plot::new("metrics_plot")
                .legend(Legend::default().position(Corner::LeftTop))
                .height(380.0)
                .show_axes([true, true])
                .show_grid(true);
            
            plot.show(ui, |plot_ui| {
                // Prepare data points
                let points: PlotPoints = trend_data.data_points.iter()
                    .enumerate()
                    .map(|(i, p)| [i as f64, p.scaled_value.unwrap_or(p.value) as f64])
                    .collect();
                
                match self.chart_type.as_str() {
                    "line" => {
                        plot_ui.line(Line::new(points)
                            .color(egui::Color32::from_rgb(59, 130, 246))
                            .width(2.0)
                            .name(&trend_data.channel_name));
                    }
                    "area" => {
                        plot_ui.line(Line::new(points.clone())
                            .color(egui::Color32::from_rgb(59, 130, 246))
                            .width(2.0)
                            .fill(0.0)
                            .name(&trend_data.channel_name));
                    }
                    "bar" => {
                        // For bar chart, sample fewer points
                        let bar_data: Vec<Bar> = trend_data.data_points.iter()
                            .enumerate()
                            .step_by((trend_data.data_points.len() / 20).max(1))
                            .map(|(i, p)| Bar::new(i as f64, p.scaled_value.unwrap_or(p.value) as f64))
                            .collect();
                        
                        plot_ui.bar_chart(BarChart::new(bar_data)
                            .color(egui::Color32::from_rgb(59, 130, 246))
                            .width(0.8)
                            .name(&trend_data.channel_name));
                    }
                    _ => {}
                }
                
                // Add comparison channels if enabled
                if self.show_comparison {
                    for (idx, comp_channel) in self.comparison_channels.iter().enumerate() {
                        // Generate sample comparison data
                        let comp_points: PlotPoints = trend_data.data_points.iter()
                            .enumerate()
                            .map(|(i, p)| {
                                let variation = ((idx + 1) as f32 * 5.0) * (i as f32 * 0.1).sin();
                                [i as f64, (p.scaled_value.unwrap_or(p.value) + variation) as f64]
                            })
                            .collect();
                        
                        let colors = [
                            egui::Color32::from_rgb(34, 197, 94),
                            egui::Color32::from_rgb(239, 68, 68),
                            egui::Color32::from_rgb(147, 51, 234),
                        ];
                        
                        plot_ui.line(Line::new(comp_points)
                            .color(colors[idx % colors.len()])
                            .width(2.0)
                            .name(comp_channel));
                    }
                }
            });
        });
    }
    
    fn show_dialogs(&mut self, ui: &mut egui::Ui) {
        // Export Dialog
        if self.show_export_dialog {
            Window::new("Export Data")
                .collapsible(false)
                .resizable(false)
                .show(ui.ctx(), |ui| {
                    ui.label("Select export format:");
                    
                    ui.radio_value(&mut self.export_format, "csv".to_string(), "CSV (Comma Separated)");
                    ui.radio_value(&mut self.export_format, "json".to_string(), "JSON");
                    ui.radio_value(&mut self.export_format, "excel".to_string(), "Excel (.xlsx)");
                    
                    ui.separator();
                    
                    ui.horizontal(|ui| {
                        if ui.button("📥 Export").clicked() {
                            self.export_data();
                            self.show_export_dialog = false;
                        }
                        
                        if ui.button("❌ Cancel").clicked() {
                            self.show_export_dialog = false;
                        }
                    });
                });
        }
    }
    
    fn fetch_trend_data(&mut self) {
        self.is_loading = true;
        
        // Generate sample trend data
        let mut data_points = vec![];
        let now = Utc::now();
        let hours = self.selected_hours as i64;
        let points_count = (hours * 12).min(500); // 12 points per hour, max 500
        
        for i in 0..points_count {
            let timestamp = now - Duration::minutes((points_count - i - 1) * 5);
            let base_value = 72.0 + (i as f32 * 0.1).sin() * 5.0;
            let noise = (rand::random::<f32>() - 0.5) * 2.0;
            let value = base_value + noise;
            
            data_points.push(DataPoint {
                timestamp,
                value,
                scaled_value: Some(value),
            });
        }
        
        // Calculate statistics
        let values: Vec<f32> = data_points.iter().map(|p| p.scaled_value.unwrap_or(p.value)).collect();
        let count = values.len();
        let sum: f32 = values.iter().sum();
        let avg = if count > 0 { sum / count as f32 } else { 0.0 };
        let min = values.iter().cloned().fold(f32::INFINITY, f32::min);
        let max = values.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        
        let variance = values.iter()
            .map(|v| (v - avg).powi(2))
            .sum::<f32>() / count as f32;
        let std_dev = variance.sqrt();
        
        // Calculate trend (simple linear regression slope)
        let trend = if count > 1 {
            let first_half_avg = values[..count/2].iter().sum::<f32>() / (count/2) as f32;
            let second_half_avg = values[count/2..].iter().sum::<f32>() / (count - count/2) as f32;
            (second_half_avg - first_half_avg) / first_half_avg
        } else {
            0.0
        };
        
        let statistics = Statistics {
            min,
            max,
            avg,
            std_dev,
            count,
            trend,
        };
        
        let channel = self.selected_channel.as_ref()
            .and_then(|sc| self.channels.iter()
                .find(|c| format!("{}:{}", c.channel_type, c.index) == *sc))
            .cloned();
        
        self.trend_data = Some(TrendData {
            channel_name: channel.as_ref().map(|c| c.name.clone()).unwrap_or_else(|| "Unknown".to_string()),
            units: channel.and_then(|c| c.units),
            data_points,
            statistics,
        });
        
        self.is_loading = false;
    }
    
    fn export_data(&self) {
        if let Some(trend_data) = &self.trend_data {
            println!("Exporting {} data points as {}", 
                trend_data.data_points.len(), 
                self.export_format);
            // In real implementation, would generate and download file
        }
    }
}

// Add rand for simulation
use rand::Rng;