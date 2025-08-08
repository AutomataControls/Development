// Email Service for Automata Nexus
// Handles email notifications, templates, and logging

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use lettre::message::{header, Message, MultiPart, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{SmtpTransport, Transport};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_user: String,
    pub smtp_password: String,
    pub from_email: String,
    pub from_name: String,
    pub use_tls: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailTemplate {
    pub name: String,
    pub subject: String,
    pub html_body: String,
    pub text_body: String,
    pub variables: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailLog {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub to: String,
    pub subject: String,
    pub template: Option<String>,
    pub status: EmailStatus,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmailStatus {
    Pending,
    Sent,
    Failed,
    Retry,
}

pub struct EmailService {
    config: EmailConfig,
    templates: HashMap<String, EmailTemplate>,
    logs: Vec<EmailLog>,
}

impl EmailService {
    pub fn new(config: EmailConfig) -> Self {
        let mut service = Self {
            config,
            templates: HashMap::new(),
            logs: Vec::new(),
        };
        
        service.load_default_templates();
        service
    }
    
    fn load_default_templates(&mut self) {
        // Alarm notification template
        self.templates.insert("alarm_notification".to_string(), EmailTemplate {
            name: "alarm_notification".to_string(),
            subject: "🚨 Nexus Alarm: {{alarm_name}}".to_string(),
            html_body: r#"
<!DOCTYPE html>
<html>
<head>
    <style>
        body { font-family: Arial, sans-serif; }
        .header { background: #14b8a6; color: white; padding: 20px; }
        .content { padding: 20px; }
        .alarm-critical { color: #dc2626; font-weight: bold; }
        .alarm-high { color: #ea580c; font-weight: bold; }
        .alarm-medium { color: #ca8a04; }
        .alarm-low { color: #0284c7; }
        .footer { background: #f1f5f9; padding: 10px; text-align: center; font-size: 12px; }
    </style>
</head>
<body>
    <div class="header">
        <h1>Automata Nexus AI - Alarm Notification</h1>
    </div>
    <div class="content">
        <h2 class="alarm-{{severity}}">{{alarm_name}}</h2>
        <p><strong>Time:</strong> {{timestamp}}</p>
        <p><strong>Location:</strong> {{location}}</p>
        <p><strong>Equipment:</strong> {{equipment}}</p>
        <p><strong>Value:</strong> {{value}} {{units}}</p>
        <p><strong>Threshold:</strong> {{threshold}} {{units}}</p>
        <p><strong>Message:</strong> {{message}}</p>
        <hr>
        <p>Please log in to the Nexus system to acknowledge this alarm.</p>
    </div>
    <div class="footer">
        <p>© 2025 Automata Controls | This is an automated message</p>
    </div>
</body>
</html>"#.to_string(),
            text_body: r#"
AUTOMATA NEXUS AI - ALARM NOTIFICATION

Alarm: {{alarm_name}}
Severity: {{severity}}
Time: {{timestamp}}
Location: {{location}}
Equipment: {{equipment}}
Value: {{value}} {{units}}
Threshold: {{threshold}} {{units}}
Message: {{message}}

Please log in to the Nexus system to acknowledge this alarm.

© 2025 Automata Controls"#.to_string(),
            variables: vec![
                "alarm_name".to_string(),
                "severity".to_string(),
                "timestamp".to_string(),
                "location".to_string(),
                "equipment".to_string(),
                "value".to_string(),
                "threshold".to_string(),
                "units".to_string(),
                "message".to_string(),
            ],
        });
        
        // System report template
        self.templates.insert("system_report".to_string(), EmailTemplate {
            name: "system_report".to_string(),
            subject: "📊 Nexus Daily System Report - {{date}}".to_string(),
            html_body: r#"
<!DOCTYPE html>
<html>
<head>
    <style>
        body { font-family: Arial, sans-serif; }
        .header { background: #14b8a6; color: white; padding: 20px; }
        .content { padding: 20px; }
        .metrics { display: grid; grid-template-columns: repeat(3, 1fr); gap: 15px; }
        .metric { background: #f8fafc; padding: 15px; border-radius: 8px; }
        .metric-value { font-size: 24px; font-weight: bold; color: #0f172a; }
        .metric-label { color: #64748b; font-size: 14px; }
        .good { color: #22c55e; }
        .warning { color: #fb923c; }
        .bad { color: #f87171; }
    </style>
</head>
<body>
    <div class="header">
        <h1>Daily System Report</h1>
        <p>{{date}}</p>
    </div>
    <div class="content">
        <h2>System Metrics</h2>
        <div class="metrics">
            <div class="metric">
                <div class="metric-value">{{runtime_hours}}</div>
                <div class="metric-label">Runtime Hours</div>
            </div>
            <div class="metric">
                <div class="metric-value">{{energy_kwh}}</div>
                <div class="metric-label">Energy Usage (kWh)</div>
            </div>
            <div class="metric">
                <div class="metric-value good">{{efficiency}}%</div>
                <div class="metric-label">System Efficiency</div>
            </div>
            <div class="metric">
                <div class="metric-value">{{alarm_count}}</div>
                <div class="metric-label">Alarms Today</div>
            </div>
            <div class="metric">
                <div class="metric-value">{{cycles}}</div>
                <div class="metric-label">Compressor Cycles</div>
            </div>
            <div class="metric">
                <div class="metric-value good">{{uptime}}%</div>
                <div class="metric-label">System Uptime</div>
            </div>
        </div>
        
        <h2>Active Alarms</h2>
        {{alarm_list}}
        
        <h2>Maintenance Items</h2>
        {{maintenance_list}}
    </div>
    <div class="footer">
        <p>© 2025 Automata Controls | Automated Daily Report</p>
    </div>
</body>
</html>"#.to_string(),
            text_body: r#"
DAILY SYSTEM REPORT - {{date}}

SYSTEM METRICS:
- Runtime Hours: {{runtime_hours}}
- Energy Usage: {{energy_kwh}} kWh
- System Efficiency: {{efficiency}}%
- Alarms Today: {{alarm_count}}
- Compressor Cycles: {{cycles}}
- System Uptime: {{uptime}}%

ACTIVE ALARMS:
{{alarm_list}}

MAINTENANCE ITEMS:
{{maintenance_list}}

© 2025 Automata Controls"#.to_string(),
            variables: vec![
                "date".to_string(),
                "runtime_hours".to_string(),
                "energy_kwh".to_string(),
                "efficiency".to_string(),
                "alarm_count".to_string(),
                "cycles".to_string(),
                "uptime".to_string(),
                "alarm_list".to_string(),
                "maintenance_list".to_string(),
            ],
        });
        
        // User welcome template
        self.templates.insert("user_welcome".to_string(), EmailTemplate {
            name: "user_welcome".to_string(),
            subject: "Welcome to Automata Nexus AI".to_string(),
            html_body: r#"
<!DOCTYPE html>
<html>
<head>
    <style>
        body { font-family: Arial, sans-serif; }
        .header { background: #14b8a6; color: white; padding: 20px; text-align: center; }
        .content { padding: 20px; }
        .button { 
            display: inline-block; 
            padding: 12px 24px; 
            background: #14b8a6; 
            color: white; 
            text-decoration: none; 
            border-radius: 4px; 
            margin: 10px 0;
        }
    </style>
</head>
<body>
    <div class="header">
        <h1>Welcome to Automata Nexus AI</h1>
    </div>
    <div class="content">
        <h2>Hello {{username}}!</h2>
        <p>Your account has been created successfully.</p>
        <p><strong>Username:</strong> {{username}}</p>
        <p><strong>Role:</strong> {{role}}</p>
        <p><strong>Access Level:</strong> {{access_level}}</p>
        
        <p>You can now log in to the Nexus system using your credentials.</p>
        
        <a href="{{login_url}}" class="button">Login to Nexus</a>
        
        <h3>Getting Started:</h3>
        <ul>
            <li>Review the system dashboard for current status</li>
            <li>Check active alarms and acknowledge as needed</li>
            <li>Configure your notification preferences</li>
            <li>Explore the various monitoring and control features</li>
        </ul>
        
        <p>If you have any questions, please contact your system administrator.</p>
    </div>
    <div class="footer">
        <p>© 2025 Automata Controls</p>
    </div>
</body>
</html>"#.to_string(),
            text_body: r#"
Welcome to Automata Nexus AI!

Hello {{username}}!

Your account has been created successfully.

Username: {{username}}
Role: {{role}}
Access Level: {{access_level}}

You can now log in to the Nexus system at: {{login_url}}

Getting Started:
- Review the system dashboard for current status
- Check active alarms and acknowledge as needed
- Configure your notification preferences
- Explore the various monitoring and control features

If you have any questions, please contact your system administrator.

© 2025 Automata Controls"#.to_string(),
            variables: vec![
                "username".to_string(),
                "role".to_string(),
                "access_level".to_string(),
                "login_url".to_string(),
            ],
        });
    }
    
    // Send email with template
    pub async fn send_template(
        &mut self,
        to: &str,
        template_name: &str,
        variables: HashMap<String, String>,
    ) -> Result<()> {
        let template = self.templates.get(template_name)
            .ok_or_else(|| anyhow!("Template not found: {}", template_name))?
            .clone();
        
        let subject = self.replace_variables(&template.subject, &variables);
        let html_body = self.replace_variables(&template.html_body, &variables);
        let text_body = self.replace_variables(&template.text_body, &variables);
        
        self.send_email(to, &subject, &html_body, &text_body).await
    }
    
    // Send raw email
    pub async fn send_email(
        &mut self,
        to: &str,
        subject: &str,
        html_body: &str,
        text_body: &str,
    ) -> Result<()> {
        // Create log entry
        let log_entry = EmailLog {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            to: to.to_string(),
            subject: subject.to_string(),
            template: None,
            status: EmailStatus::Pending,
            error: None,
        };
        self.logs.push(log_entry.clone());
        
        // Build email
        let email = Message::builder()
            .from(format!("{} <{}>", self.config.from_name, self.config.from_email).parse()?)
            .to(to.parse()?)
            .subject(subject)
            .multipart(
                MultiPart::alternative()
                    .singlepart(SinglePart::plain(text_body.to_string()))
                    .singlepart(SinglePart::html(html_body.to_string()))
            )?;
        
        // Create transport
        let creds = Credentials::new(
            self.config.smtp_user.clone(),
            self.config.smtp_password.clone(),
        );
        
        let mailer = if self.config.use_tls {
            SmtpTransport::relay(&self.config.smtp_host)?
                .credentials(creds)
                .port(self.config.smtp_port)
                .build()
        } else {
            SmtpTransport::builder_dangerous(&self.config.smtp_host)
                .port(self.config.smtp_port)
                .credentials(creds)
                .build()
        };
        
        // Send email
        match mailer.send(&email) {
            Ok(_) => {
                // Update log entry
                if let Some(log) = self.logs.iter_mut().find(|l| l.id == log_entry.id) {
                    log.status = EmailStatus::Sent;
                }
                Ok(())
            }
            Err(e) => {
                // Update log entry
                if let Some(log) = self.logs.iter_mut().find(|l| l.id == log_entry.id) {
                    log.status = EmailStatus::Failed;
                    log.error = Some(e.to_string());
                }
                Err(anyhow!("Failed to send email: {}", e))
            }
        }
    }
    
    // Test email configuration
    pub async fn test_email(&mut self) -> Result<()> {
        self.send_email(
            &self.config.from_email,
            "Nexus Email Test",
            "<h1>Test Email</h1><p>This is a test email from Automata Nexus.</p>",
            "Test Email\n\nThis is a test email from Automata Nexus.",
        ).await
    }
    
    // Replace template variables
    fn replace_variables(&self, template: &str, variables: &HashMap<String, String>) -> String {
        let mut result = template.to_string();
        for (key, value) in variables {
            result = result.replace(&format!("{{{{{}}}}}", key), value);
        }
        result
    }
    
    // Get email logs
    pub fn get_logs(&self, limit: usize) -> Vec<EmailLog> {
        let start = if self.logs.len() > limit {
            self.logs.len() - limit
        } else {
            0
        };
        self.logs[start..].to_vec()
    }
    
    // Get template
    pub fn get_template(&self, name: &str) -> Option<&EmailTemplate> {
        self.templates.get(name)
    }
    
    // Update template
    pub fn update_template(&mut self, template: EmailTemplate) {
        self.templates.insert(template.name.clone(), template);
    }
    
    // Get all templates
    pub fn get_templates(&self) -> Vec<EmailTemplate> {
        self.templates.values().cloned().collect()
    }
}