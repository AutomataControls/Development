// COMPLETE Live Monitor Implementation - Real-time monitoring with trends, charts, statistics
// Includes all features from original: live values, trends, sparklines, min/max/avg, alarms

use eframe::egui;
use egui::{Color32, RichText, Grid, ScrollArea, Ui};
use egui_plot::{Line, Plot, PlotPoints, Legend, Corner};
use std::collections::{HashMap, VecDeque};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};

#[derive(Debug, Clone)]
pub struct LiveMonitor {
    // Channel data
    channels: Vec<ChannelMonitor>,
    
    // Historical data for charts (last 1000 points per channel)
    history: HashMap<String, VecDeque<DataPoint>>,
    
    // Statistics
    statistics: HashMap<String, ChannelStatistics>,
    
    // Display settings
    show_charts: bool,
    show_statistics: bool,
    auto_scale: bool,
    update_rate: UpdateRate,
    
    // Selected channels for detailed view
    selected_channels: Vec<usize>,
    
    // Alarm states
    active_alarms: Vec<AlarmState>,
    
    // Last update time
    last_update: std::time::Instant,
}

#[derive(Debug, Clone)]
struct ChannelMonitor {
    id: String,
    name: String,
    channel_type: ChannelType,
    index: usize,
    current_value: f32,
    scaled_value: f32,
    units: String,
    
    // Limits
    min_limit: f32,
    max_limit: f32,
    alarm_high: Option<f32>,
    alarm_low: Option<f32>,
    
    // Trend
    trend: Trend,
    previous_value: f32,
    
    // Display
    color: Color32,
    enabled: bool,
}

#[derive(Debug, Clone, PartialEq)]
enum ChannelType {
    UniversalInput,
    AnalogOutput,
    DigitalInput,
    DigitalOutput,
    VirtualPoint,
}

#[derive(Debug, Clone)]
struct DataPoint {
    timestamp: DateTime<Utc>,
    value: f32,
    scaled_value: f32,
}

#[derive(Debug, Clone)]
struct ChannelStatistics {
    min: f32,
    max: f32,
    avg: f32,
    std_dev: f32,
    count: usize,
    last_reset: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq)]
enum Trend {
    Rising,
    Falling,
    Stable,
    RapidRise,
    RapidFall,
}

#[derive(Debug, Clone, PartialEq)]
enum UpdateRate {
    Fast,    // 100ms
    Normal,  // 1s
    Slow,    // 5s
}

#[derive(Debug, Clone)]
struct AlarmState {
    channel_id: String,
    channel_name: String,
    alarm_type: AlarmType,
    value: f32,
    threshold: f32,
    timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq)]
enum AlarmType {
    High,
    Low,
    RateOfChange,
    Deviation,
}

impl LiveMonitor {
    pub fn new() -> Self {
        let mut monitor = Self {
            channels: Vec::new(),
            history: HashMap::new(),
            statistics: HashMap::new(),
            show_charts: true,
            show_statistics: true,
            auto_scale: true,
            update_rate: UpdateRate::Normal,
            selected_channels: Vec::new(),
            active_alarms: Vec::new(),
            last_update: std::time::Instant::now(),
        };
        
        // Initialize with sample channels
        monitor.init_channels();
        monitor
    }
    
    fn init_channels(&mut self) {
        // Universal Inputs
        for i in 0..8 {
            let channel = ChannelMonitor {
                id: format!("UI_{}", i + 1),
                name: format!("Universal Input {}", i + 1),
                channel_type: ChannelType::UniversalInput,
                index: i,
                current_value: 0.0,
                scaled_value: 0.0,
                units: "PSI".to_string(),
                min_limit: 0.0,
                max_limit: 500.0,
                alarm_high: Some(450.0),
                alarm_low: Some(20.0),
                trend: Trend::Stable,
                previous_value: 0.0,
                color: Color32::from_rgb(59, 130, 246), // Blue
                enabled: true,
            };
            self.channels.push(channel.clone());
            self.history.insert(channel.id.clone(), VecDeque::with_capacity(1000));
            self.statistics.insert(channel.id.clone(), ChannelStatistics {
                min: f32::MAX,
                max: f32::MIN,
                avg: 0.0,
                std_dev: 0.0,
                count: 0,
                last_reset: Utc::now(),
            });
        }
        
        // Analog Outputs
        for i in 0..4 {
            let channel = ChannelMonitor {
                id: format!("AO_{}", i + 1),
                name: format!("Analog Output {}", i + 1),
                channel_type: ChannelType::AnalogOutput,
                index: i,
                current_value: 0.0,
                scaled_value: 0.0,
                units: "V".to_string(),
                min_limit: 0.0,
                max_limit: 10.0,
                alarm_high: None,
                alarm_low: None,
                trend: Trend::Stable,
                previous_value: 0.0,
                color: Color32::from_rgb(34, 197, 94), // Green
                enabled: true,
            };
            self.channels.push(channel.clone());
            self.history.insert(channel.id.clone(), VecDeque::with_capacity(1000));
            self.statistics.insert(channel.id, ChannelStatistics {
                min: f32::MAX,
                max: f32::MIN,
                avg: 0.0,
                std_dev: 0.0,
                count: 0,
                last_reset: Utc::now(),
            });
        }
        
        // Virtual Points for calculated values
        let virtual_points = vec![
            ("Discharge Pressure", "PSI", 325.0, Some(450.0), Some(200.0)),
            ("Suction Pressure", "PSI", 68.0, Some(100.0), Some(20.0)),
            ("Discharge Temp", "°F", 180.0, Some(225.0), Some(100.0)),
            ("Suction Temp", "°F", 45.0, Some(80.0), Some(20.0)),
            ("Superheat", "°F", 12.0, Some(20.0), Some(5.0)),
            ("Subcooling", "°F", 8.0, Some(15.0), Some(3.0)),
        ];
        
        for (i, (name, units, value, high, low)) in virtual_points.iter().enumerate() {
            let channel = ChannelMonitor {
                id: format!("VP_{}", i + 1),
                name: name.to_string(),
                channel_type: ChannelType::VirtualPoint,
                index: i,
                current_value: *value,
                scaled_value: *value,
                units: units.to_string(),
                min_limit: 0.0,
                max_limit: 500.0,
                alarm_high: *high,
                alarm_low: *low,
                trend: Trend::Stable,
                previous_value: *value,
                color: Color32::from_rgb(251, 146, 60), // Orange
                enabled: true,
            };
            self.channels.push(channel.clone());
            self.history.insert(channel.id.clone(), VecDeque::with_capacity(1000));
            self.statistics.insert(channel.id, ChannelStatistics {
                min: f32::MAX,
                max: f32::MIN,
                avg: 0.0,
                std_dev: 0.0,
                count: 0,
                last_reset: Utc::now(),
            });
        }
    }
    
    pub fn show(&mut self, ui: &mut egui::Ui) {
        // Update data if needed
        let update_interval = match self.update_rate {
            UpdateRate::Fast => std::time::Duration::from_millis(100),
            UpdateRate::Normal => std::time::Duration::from_secs(1),
            UpdateRate::Slow => std::time::Duration::from_secs(5),
        };
        
        if self.last_update.elapsed() > update_interval {
            self.update_values();
            self.last_update = std::time::Instant::now();
        }
        
        // Header controls
        ui.horizontal(|ui| {
            ui.label(RichText::new("Live Monitor").size(18.0).strong());
            
            ui.separator();
            
            // Update rate selector
            ui.label("Update Rate:");
            egui::ComboBox::from_label("")
                .selected_text(match self.update_rate {
                    UpdateRate::Fast => "Fast (100ms)",
                    UpdateRate::Normal => "Normal (1s)",
                    UpdateRate::Slow => "Slow (5s)",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.update_rate, UpdateRate::Fast, "Fast (100ms)");
                    ui.selectable_value(&mut self.update_rate, UpdateRate::Normal, "Normal (1s)");
                    ui.selectable_value(&mut self.update_rate, UpdateRate::Slow, "Slow (5s)");
                });
            
            ui.checkbox(&mut self.show_charts, "Charts");
            ui.checkbox(&mut self.show_statistics, "Statistics");
            ui.checkbox(&mut self.auto_scale, "Auto Scale");
            
            if ui.button("Reset Stats").clicked() {
                self.reset_statistics();
            }
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Active alarms indicator
                if !self.active_alarms.is_empty() {
                    ui.colored_label(
                        Color32::from_rgb(239, 68, 68),
                        format!("⚠ {} Active Alarms", self.active_alarms.len())
                    );
                } else {
                    ui.colored_label(Color32::from_rgb(34, 197, 94), "✓ No Alarms");
                }
            });
        });
        
        ui.separator();
        
        // Main content area with scroll
        ScrollArea::vertical().show(ui, |ui| {
            // Active Alarms Section
            if !self.active_alarms.is_empty() {
                ui.group(|ui| {
                    ui.label(RichText::new("⚠ Active Alarms").color(Color32::from_rgb(239, 68, 68)).strong());
                    
                    Grid::new("active_alarms").striped(true).show(ui, |ui| {
                        ui.label("Channel");
                        ui.label("Type");
                        ui.label("Value");
                        ui.label("Threshold");
                        ui.label("Duration");
                        ui.end_row();
                        
                        for alarm in &self.active_alarms {
                            ui.label(&alarm.channel_name);
                            
                            let type_color = match alarm.alarm_type {
                                AlarmType::High => Color32::from_rgb(239, 68, 68),
                                AlarmType::Low => Color32::from_rgb(59, 130, 246),
                                AlarmType::RateOfChange => Color32::from_rgb(251, 146, 60),
                                AlarmType::Deviation => Color32::from_rgb(168, 85, 247),
                            };
                            ui.colored_label(type_color, format!("{:?}", alarm.alarm_type));
                            
                            ui.label(format!("{:.2}", alarm.value));
                            ui.label(format!("{:.2}", alarm.threshold));
                            
                            let duration = Utc::now().signed_duration_since(alarm.timestamp);
                            ui.label(format!("{}s", duration.num_seconds()));
                            
                            ui.end_row();
                        }
                    });
                });
                
                ui.separator();
            }
            
            // Channel Groups
            self.show_channel_group(ui, "Universal Inputs", ChannelType::UniversalInput);
            ui.separator();
            self.show_channel_group(ui, "Analog Outputs", ChannelType::AnalogOutput);
            ui.separator();
            self.show_channel_group(ui, "Virtual Points", ChannelType::VirtualPoint);
            
            // Detailed Charts Section
            if self.show_charts && !self.selected_channels.is_empty() {
                ui.separator();
                ui.group(|ui| {
                    ui.label(RichText::new("Trend Charts").strong());
                    
                    for &idx in &self.selected_channels {
                        if let Some(channel) = self.channels.get(idx) {
                            self.show_channel_chart(ui, channel);
                        }
                    }
                });
            }
            
            // Statistics Section
            if self.show_statistics {
                ui.separator();
                ui.group(|ui| {
                    ui.label(RichText::new("Channel Statistics").strong());
                    
                    Grid::new("statistics").striped(true).show(ui, |ui| {
                        ui.label("Channel");
                        ui.label("Min");
                        ui.label("Max");
                        ui.label("Average");
                        ui.label("Std Dev");
                        ui.label("Samples");
                        ui.end_row();
                        
                        for channel in &self.channels {
                            if !channel.enabled {
                                continue;
                            }
                            
                            if let Some(stats) = self.statistics.get(&channel.id) {
                                ui.label(&channel.name);
                                ui.label(format!("{:.2}", stats.min));
                                ui.label(format!("{:.2}", stats.max));
                                ui.label(format!("{:.2}", stats.avg));
                                ui.label(format!("{:.2}", stats.std_dev));
                                ui.label(format!("{}", stats.count));
                                ui.end_row();
                            }
                        }
                    });
                });
            }
        });
    }
    
    fn show_channel_group(&mut self, ui: &mut egui::Ui, title: &str, channel_type: ChannelType) {
        ui.group(|ui| {
            ui.label(RichText::new(title).strong());
            
            Grid::new(format!("{}_grid", title)).striped(true).show(ui, |ui| {
                // Header
                ui.label("#");
                ui.label("Name");
                ui.label("Raw");
                ui.label("Scaled");
                ui.label("Units");
                ui.label("Trend");
                ui.label("Status");
                ui.label("Chart");
                ui.end_row();
                
                for (idx, channel) in self.channels.iter().enumerate() {
                    if channel.channel_type != channel_type || !channel.enabled {
                        continue;
                    }
                    
                    // Channel number
                    ui.label(format!("{}", channel.index + 1));
                    
                    // Name
                    ui.label(&channel.name);
                    
                    // Raw value
                    ui.monospace(format!("{:.3}", channel.current_value));
                    
                    // Scaled value with color based on alarm state
                    let value_color = if let Some(high) = channel.alarm_high {
                        if channel.scaled_value > high {
                            Color32::from_rgb(239, 68, 68) // Red for high alarm
                        } else if let Some(low) = channel.alarm_low {
                            if channel.scaled_value < low {
                                Color32::from_rgb(59, 130, 246) // Blue for low alarm
                            } else {
                                Color32::from_rgb(34, 197, 94) // Green for normal
                            }
                        } else {
                            Color32::from_rgb(34, 197, 94)
                        }
                    } else {
                        Color32::WHITE
                    };
                    
                    ui.colored_label(value_color, format!("{:.2}", channel.scaled_value));
                    
                    // Units
                    ui.label(&channel.units);
                    
                    // Trend indicator
                    let (trend_symbol, trend_color) = match channel.trend {
                        Trend::RapidRise => ("⬆", Color32::from_rgb(239, 68, 68)),
                        Trend::Rising => ("↗", Color32::from_rgb(251, 146, 60)),
                        Trend::Stable => ("→", Color32::from_rgb(148, 163, 184)),
                        Trend::Falling => ("↘", Color32::from_rgb(59, 130, 246)),
                        Trend::RapidFall => ("⬇", Color32::from_rgb(139, 92, 246)),
                    };
                    ui.colored_label(trend_color, trend_symbol);
                    
                    // Status indicator with sparkline
                    ui.horizontal(|ui| {
                        // Mini sparkline (last 20 points)
                        if let Some(history) = self.history.get(&channel.id) {
                            let recent: Vec<f32> = history.iter()
                                .rev()
                                .take(20)
                                .map(|p| p.scaled_value)
                                .collect();
                            
                            if !recent.is_empty() {
                                self.draw_sparkline(ui, &recent, channel.alarm_high, channel.alarm_low);
                            }
                        }
                    });
                    
                    // Chart selection checkbox
                    let mut is_selected = self.selected_channels.contains(&idx);
                    if ui.checkbox(&mut is_selected, "").changed() {
                        if is_selected {
                            self.selected_channels.push(idx);
                        } else {
                            self.selected_channels.retain(|&i| i != idx);
                        }
                    }
                    
                    ui.end_row();
                }
            });
        });
    }
    
    fn draw_sparkline(&self, ui: &mut egui::Ui, values: &[f32], high: Option<f32>, low: Option<f32>) {
        let (response, painter) = ui.allocate_painter(egui::Vec2::new(60.0, 20.0), egui::Sense::hover());
        let rect = response.rect;
        
        if values.len() < 2 {
            return;
        }
        
        let min = values.iter().fold(f32::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
        let range = max - min;
        
        if range == 0.0 {
            return;
        }
        
        // Draw alarm zones
        if let Some(h) = high {
            let y = rect.bottom() - (h - min) / range * rect.height();
            painter.line_segment(
                [rect.left_top() + egui::Vec2::new(0.0, y), rect.right_top() + egui::Vec2::new(0.0, y)],
                egui::Stroke::new(0.5, Color32::from_rgba_premultiplied(239, 68, 68, 100))
            );
        }
        
        if let Some(l) = low {
            let y = rect.bottom() - (l - min) / range * rect.height();
            painter.line_segment(
                [rect.left_top() + egui::Vec2::new(0.0, y), rect.right_top() + egui::Vec2::new(0.0, y)],
                egui::Stroke::new(0.5, Color32::from_rgba_premultiplied(59, 130, 246, 100))
            );
        }
        
        // Draw sparkline
        let points: Vec<egui::Pos2> = values
            .iter()
            .enumerate()
            .map(|(i, &v)| {
                let x = rect.left() + (i as f32 / (values.len() - 1) as f32) * rect.width();
                let y = rect.bottom() - (v - min) / range * rect.height();
                egui::Pos2::new(x, y)
            })
            .collect();
        
        for window in points.windows(2) {
            painter.line_segment(
                [window[0], window[1]],
                egui::Stroke::new(1.0, Color32::from_rgb(94, 234, 212))
            );
        }
        
        // Draw last point
        if let Some(last) = points.last() {
            painter.circle_filled(*last, 2.0, Color32::from_rgb(251, 146, 60));
        }
    }
    
    fn show_channel_chart(&self, ui: &mut egui::Ui, channel: &ChannelMonitor) {
        if let Some(history) = self.history.get(&channel.id) {
            let points: PlotPoints = history
                .iter()
                .enumerate()
                .map(|(i, point)| [i as f64, point.scaled_value as f64])
                .collect();
            
            let line = Line::new(points)
                .color(channel.color)
                .name(&channel.name);
            
            Plot::new(format!("chart_{}", channel.id))
                .height(150.0)
                .legend(Legend::default().position(Corner::RightTop))
                .show(ui, |plot_ui| {
                    plot_ui.line(line);
                    
                    // Draw alarm limits
                    if let Some(high) = channel.alarm_high {
                        let high_line = Line::new(PlotPoints::from_iter(
                            (0..history.len()).map(|i| [i as f64, high as f64])
                        ))
                        .color(Color32::from_rgba_premultiplied(239, 68, 68, 100))
                        .name("High Limit");
                        plot_ui.line(high_line);
                    }
                    
                    if let Some(low) = channel.alarm_low {
                        let low_line = Line::new(PlotPoints::from_iter(
                            (0..history.len()).map(|i| [i as f64, low as f64])
                        ))
                        .color(Color32::from_rgba_premultiplied(59, 130, 246, 100))
                        .name("Low Limit");
                        plot_ui.line(low_line);
                    }
                });
        }
    }
    
    fn update_values(&mut self) {
        // Simulate real-time data updates
        for channel in &mut self.channels {
            channel.previous_value = channel.current_value;
            
            // Generate realistic values based on channel type
            match channel.channel_type {
                ChannelType::UniversalInput => {
                    // Simulate sensor readings with noise
                    let base = 5.0 + channel.index as f32 * 0.5;
                    let noise = (rand::random::<f32>() - 0.5) * 0.2;
                    channel.current_value = base + noise;
                    
                    // Scale to engineering units
                    channel.scaled_value = channel.current_value * 50.0; // Convert to PSI
                }
                ChannelType::AnalogOutput => {
                    // Outputs remain at set values with minimal drift
                    let drift = (rand::random::<f32>() - 0.5) * 0.01;
                    channel.current_value += drift;
                    channel.current_value = channel.current_value.clamp(0.0, 10.0);
                    channel.scaled_value = channel.current_value;
                }
                ChannelType::VirtualPoint => {
                    // Virtual points with realistic HVAC behavior
                    let variation = (rand::random::<f32>() - 0.5) * 2.0;
                    channel.scaled_value += variation;
                    channel.current_value = channel.scaled_value;
                }
                _ => {}
            }
            
            // Calculate trend
            let delta = channel.current_value - channel.previous_value;
            channel.trend = if delta > 0.5 {
                Trend::RapidRise
            } else if delta > 0.1 {
                Trend::Rising
            } else if delta < -0.5 {
                Trend::RapidFall
            } else if delta < -0.1 {
                Trend::Falling
            } else {
                Trend::Stable
            };
            
            // Update history
            if let Some(history) = self.history.get_mut(&channel.id) {
                history.push_back(DataPoint {
                    timestamp: Utc::now(),
                    value: channel.current_value,
                    scaled_value: channel.scaled_value,
                });
                
                // Keep only last 1000 points
                while history.len() > 1000 {
                    history.pop_front();
                }
            }
            
            // Update statistics
            if let Some(stats) = self.statistics.get_mut(&channel.id) {
                stats.min = stats.min.min(channel.scaled_value);
                stats.max = stats.max.max(channel.scaled_value);
                stats.count += 1;
                
                // Update running average
                let n = stats.count as f32;
                stats.avg = (stats.avg * (n - 1.0) + channel.scaled_value) / n;
                
                // Simple standard deviation calculation
                if stats.count > 1 {
                    let variance = ((channel.scaled_value - stats.avg).powi(2)) / n;
                    stats.std_dev = variance.sqrt();
                }
            }
            
            // Check for alarms
            self.check_alarms(channel);
        }
    }
    
    fn check_alarms(&mut self, channel: &ChannelMonitor) {
        // Remove existing alarms for this channel
        self.active_alarms.retain(|a| a.channel_id != channel.id);
        
        // Check high alarm
        if let Some(high) = channel.alarm_high {
            if channel.scaled_value > high {
                self.active_alarms.push(AlarmState {
                    channel_id: channel.id.clone(),
                    channel_name: channel.name.clone(),
                    alarm_type: AlarmType::High,
                    value: channel.scaled_value,
                    threshold: high,
                    timestamp: Utc::now(),
                });
            }
        }
        
        // Check low alarm
        if let Some(low) = channel.alarm_low {
            if channel.scaled_value < low {
                self.active_alarms.push(AlarmState {
                    channel_id: channel.id.clone(),
                    channel_name: channel.name.clone(),
                    alarm_type: AlarmType::Low,
                    value: channel.scaled_value,
                    threshold: low,
                    timestamp: Utc::now(),
                });
            }
        }
        
        // Check rate of change
        let rate = (channel.current_value - channel.previous_value).abs();
        if rate > 1.0 {
            self.active_alarms.push(AlarmState {
                channel_id: channel.id.clone(),
                channel_name: channel.name.clone(),
                alarm_type: AlarmType::RateOfChange,
                value: rate,
                threshold: 1.0,
                timestamp: Utc::now(),
            });
        }
    }
    
    fn reset_statistics(&mut self) {
        for stats in self.statistics.values_mut() {
            *stats = ChannelStatistics {
                min: f32::MAX,
                max: f32::MIN,
                avg: 0.0,
                std_dev: 0.0,
                count: 0,
                last_reset: Utc::now(),
            };
        }
        
        // Clear history
        for history in self.history.values_mut() {
            history.clear();
        }
    }
}

// Add rand for simulation
use rand::Rng;