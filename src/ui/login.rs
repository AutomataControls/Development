use eframe::egui;
use super::ColorScheme;

pub struct LoginScreen {
    username: String,
    password: String,
    error_message: Option<String>,
}

impl LoginScreen {
    pub fn new() -> Self {
        Self {
            username: String::new(),
            password: String::new(),
            error_message: None,
        }
    }
    
    pub fn show(&mut self, ui: &mut egui::Ui, colors: &ColorScheme) -> (bool, String) {
        let mut authenticated = false;
        
        // Center the login panel
        ui.vertical_centered(|ui| {
            ui.add_space(100.0);
            
            // Logo
            ui.label(
                egui::RichText::new("🏭")
                    .size(80.0)
                    .color(colors.primary)
            );
            
            ui.add_space(20.0);
            
            ui.label(
                egui::RichText::new("Automata Nexus AI Controller")
                    .size(28.0)
                    .strong()
                    .color(colors.primary)
            );
            
            ui.label(
                egui::RichText::new("Building Automation System")
                    .size(16.0)
                    .color(colors.text_secondary)
            );
            
            ui.add_space(40.0);
            
            // Login form
            egui::Frame::none()
                .fill(colors.surface)
                .rounding(8.0)
                .shadow(egui::epaint::Shadow::small_dark())
                .inner_margin(32.0)
                .show(ui, |ui| {
                    ui.set_max_width(400.0);
                    
                    ui.label("Username");
                    let username_response = ui.add(
                        egui::TextEdit::singleline(&mut self.username)
                            .desired_width(350.0)
                            .font(egui::TextStyle::Body)
                    );
                    
                    ui.add_space(16.0);
                    
                    ui.label("Password");
                    let password_response = ui.add(
                        egui::TextEdit::singleline(&mut self.password)
                            .password(true)
                            .desired_width(350.0)
                            .font(egui::TextStyle::Body)
                    );
                    
                    ui.add_space(24.0);
                    
                    // Error message
                    if let Some(error) = &self.error_message {
                        ui.colored_label(colors.error, error);
                        ui.add_space(16.0);
                    }
                    
                    // Login button
                    let button = egui::Button::new(
                        egui::RichText::new("Login")
                            .size(16.0)
                            .color(egui::Color32::WHITE)
                    )
                    .fill(colors.primary)
                    .min_size(egui::Vec2::new(350.0, 40.0));
                    
                    let response = ui.add(button);
                    
                    // Handle Enter key
                    if (username_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                        || (password_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                        || response.clicked()
                    {
                        // Validate credentials
                        if self.username == "admin" && self.password == "Nexus" {
                            authenticated = true;
                            self.error_message = None;
                        } else {
                            self.error_message = Some("Invalid username or password".to_string());
                        }
                    }
                    
                    ui.add_space(16.0);
                    
                    ui.separator();
                    
                    ui.add_space(16.0);
                    
                    ui.label(
                        egui::RichText::new("Default: admin / Nexus")
                            .size(12.0)
                            .color(colors.text_secondary)
                    );
                });
        });
        
        (authenticated, self.username.clone())
    }
}