// Native UI Module for Automata Nexus
// Using egui for cross-platform native UI

use eframe::egui;
use egui::{Color32, Pos2, Rect, Rounding, Stroke, Vec2};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::state::AppState;

mod login;
mod dashboard;
mod io_control;
mod vibration;
mod weather;
mod settings;
mod complete_tabs;

use login::LoginScreen;
use dashboard::Dashboard;
use complete_tabs::CompleteUI;

// Teal/Cyan color scheme matching the original
pub struct ColorScheme {
    pub primary: Color32,        // Teal 500: #14b8a6
    pub primary_dark: Color32,   // Teal 700: #0f766e
    pub primary_light: Color32,  // Teal 300: #5eead4
    pub secondary: Color32,       // Cyan 500: #06b6d4
    pub secondary_dark: Color32, // Cyan 700: #0e7490
    pub secondary_light: Color32, // Cyan 300: #67e8f9
    pub background: Color32,
    pub surface: Color32,
    pub text: Color32,
    pub text_secondary: Color32,
    pub success: Color32,
    pub warning: Color32,
    pub error: Color32,
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self {
            primary: Color32::from_rgb(20, 184, 166),        // #14b8a6
            primary_dark: Color32::from_rgb(15, 118, 110),   // #0f766e
            primary_light: Color32::from_rgb(94, 234, 212),  // #5eead4
            secondary: Color32::from_rgb(6, 182, 212),       // #06b6d4
            secondary_dark: Color32::from_rgb(14, 116, 144), // #0e7490
            secondary_light: Color32::from_rgb(103, 232, 249), // #67e8f9
            background: Color32::from_rgb(248, 250, 252),    // #f8fafc
            surface: Color32::WHITE,
            text: Color32::from_rgb(15, 23, 42),            // #0f172a
            text_secondary: Color32::from_rgb(100, 116, 139), // #64748b
            success: Color32::from_rgb(34, 197, 94),        // #22c55e
            warning: Color32::from_rgb(251, 146, 60),       // #fb923c
            error: Color32::from_rgb(248, 113, 113),        // #f87171
        }
    }
}

pub enum Screen {
    Login,
    Dashboard,
    IOControl,
    VibrationMonitor,
    Protocols,
    Weather,
    Settings,
}

pub struct NexusApp {
    app_state: Arc<Mutex<AppState>>,
    current_screen: Screen,
    color_scheme: ColorScheme,
    authenticated: bool,
    username: String,
    
    // Sub-screens
    login_screen: LoginScreen,
    dashboard: Dashboard,
    complete_ui: CompleteUI,
}

impl NexusApp {
    pub fn new(cc: &eframe::CreationContext<'_>, app_state: Arc<Mutex<AppState>>) -> Self {
        // Configure fonts
        setup_fonts(&cc.egui_ctx);
        
        // Configure style
        configure_style(&cc.egui_ctx);
        
        Self {
            app_state: app_state.clone(),
            current_screen: Screen::Login,
            color_scheme: ColorScheme::default(),
            authenticated: false,
            username: String::new(),
            login_screen: LoginScreen::new(),
            dashboard: Dashboard::new(app_state.clone()),
            complete_ui: CompleteUI::new(app_state.clone()),
        }
    }
    
    fn draw_header(&self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                // Logo and title
                ui.label(
                    egui::RichText::new("🏭")
                        .size(32.0)
                        .color(self.color_scheme.primary)
                );
                ui.label(
                    egui::RichText::new("Automata Nexus AI")
                        .size(24.0)
                        .strong()
                        .color(self.color_scheme.primary)
                );
                
                ui.separator();
                
                // Navigation buttons
                if self.authenticated {
                    if ui.button("Dashboard").clicked() {
                        self.current_screen = Screen::Dashboard;
                    }
                    if ui.button("I/O Control").clicked() {
                        self.current_screen = Screen::IOControl;
                    }
                    if ui.button("Vibration").clicked() {
                        self.current_screen = Screen::VibrationMonitor;
                    }
                    if ui.button("Protocols").clicked() {
                        self.current_screen = Screen::Protocols;
                    }
                    if ui.button("Weather").clicked() {
                        self.current_screen = Screen::Weather;
                    }
                    if ui.button("Settings").clicked() {
                        self.current_screen = Screen::Settings;
                    }
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Logout").clicked() {
                            self.authenticated = false;
                            self.current_screen = Screen::Login;
                        }
                        ui.label(format!("User: {}", self.username));
                    });
                }
            });
            ui.add_space(8.0);
        });
    }
    
    fn draw_footer(&self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("footer").show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("© 2025 Automata Nexus AI | Automata Controls")
                        .color(self.color_scheme.text_secondary)
                );
                ui.separator();
                ui.label(
                    egui::RichText::new("Developed by Andrew Jewell Sr.")
                        .color(self.color_scheme.text_secondary)
                );
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // System status indicators
                    ui.colored_label(self.color_scheme.success, "● System OK");
                    ui.separator();
                    ui.label(format!("v2.0.0"));
                });
            });
            ui.add_space(4.0);
        });
    }
}

impl eframe::App for NexusApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Set background color
        let mut style = ctx.style().as_ref().clone();
        style.visuals.panel_fill = self.color_scheme.background;
        style.visuals.window_fill = self.color_scheme.surface;
        ctx.set_style(style);
        
        // Draw header
        self.draw_header(ctx);
        
        // Draw main content
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_screen {
                Screen::Login => {
                    let (authenticated, username) = self.login_screen.show(ui, &self.color_scheme);
                    if authenticated {
                        self.authenticated = true;
                        self.username = username;
                        self.current_screen = Screen::Dashboard;
                    }
                }
                Screen::Dashboard => {
                    // Use complete UI implementation with all tabs
                    self.complete_ui.show(ui, &self.color_scheme);
                }
                Screen::IOControl | Screen::VibrationMonitor | Screen::Protocols | 
                Screen::Weather | Screen::Settings => {
                    // All other screens are now handled by complete_ui
                    self.complete_ui.show(ui, &self.color_scheme);
                }
            }
        });
        
        // Draw footer
        if self.authenticated {
            self.draw_footer(ctx);
        }
        
        // Request repaint for animations
        ctx.request_repaint_after(std::time::Duration::from_millis(100));
    }
}

fn setup_fonts(ctx: &egui::Context) {
    // Use default system fonts for now
    let mut style = (*ctx.style()).clone();
    style.text_styles = [
        (egui::TextStyle::Small, egui::FontId::proportional(12.0)),
        (egui::TextStyle::Body, egui::FontId::proportional(14.0)),
        (egui::TextStyle::Button, egui::FontId::proportional(14.0)),
        (egui::TextStyle::Heading, egui::FontId::proportional(20.0)),
        (egui::TextStyle::Monospace, egui::FontId::monospace(14.0)),
    ].into();
    ctx.set_style(style);
}

fn configure_style(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    
    // Configure rounding
    style.visuals.window_rounding = Rounding::same(8.0);
    style.visuals.button_frame = true;
    style.visuals.widgets.noninteractive.rounding = Rounding::same(4.0);
    style.visuals.widgets.interactive.rounding = Rounding::same(4.0);
    style.visuals.widgets.hovered.rounding = Rounding::same(4.0);
    
    ctx.set_style(style);
}