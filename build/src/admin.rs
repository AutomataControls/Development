// Complete Admin Panel Implementation
// User management, audit logging, email system, demo mode

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use bcrypt::{hash, verify, DEFAULT_COST};
use uuid::Uuid;

// CRITICAL: Admin PIN for access
const ADMIN_PIN: &str = "Invertedskynet2$";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserRole {
    Admin,
    Operator,
    Viewer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub role: UserRole,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub active: bool,
    pub api_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub user: String,
    pub action: String,
    pub target: String,
    pub details: String,
    pub ip_address: Option<String>,
    pub success: bool,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditAction {
    Login,
    Logout,
    CreateUser,
    UpdateUser,
    DeleteUser,
    ChangePassword,
    ConfigChange,
    BoardControl,
    ManualOverride,
    AlarmAcknowledge,
    EmailSent,
    FirmwareUpdate,
    SystemReboot,
    DatabaseBackup,
    LogicExecution,
    ProtocolConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailTemplate {
    pub name: String,
    pub subject: String,
    pub body: String,
    pub body_html: Option<String>,
    pub variables: Vec<String>,
    pub category: String,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailLog {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub to_address: String,
    pub cc_address: Option<String>,
    pub subject: String,
    pub template_used: Option<String>,
    pub sent: bool,
    pub error: Option<String>,
    pub attempts: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailSettings {
    pub local_email_enabled: bool,
    pub bms_email_fallback: bool,
    pub alarm_recipients: Vec<String>,
    pub email_on_critical: bool,
    pub email_on_high: bool,
    pub email_on_medium: bool,
    pub email_on_low: bool,
    pub resend_api_key: String,
    pub from_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemoModeSettings {
    pub enabled: bool,
    pub mock_io_values: bool,
    pub mock_alarms: bool,
    pub mock_metrics: bool,
    pub mock_bms_connection: bool,
    pub simulated_values: HashMap<String, f32>,
}

pub struct AdminSystem {
    users: HashMap<String, User>,
    audit_logs: Vec<AuditLog>,
    email_templates: HashMap<String, EmailTemplate>,
    email_logs: Vec<EmailLog>,
    email_settings: EmailSettings,
    demo_mode: DemoModeSettings,
    authenticated_sessions: HashMap<String, AuthSession>,
}

#[derive(Debug, Clone)]
struct AuthSession {
    user_id: String,
    username: String,
    role: UserRole,
    created_at: DateTime<Utc>,
    last_activity: DateTime<Utc>,
    ip_address: String,
}

impl AdminSystem {
    pub fn new() -> Self {
        let mut system = Self {
            users: HashMap::new(),
            audit_logs: Vec::new(),
            email_templates: HashMap::new(),
            email_logs: Vec::new(),
            email_settings: EmailSettings {
                local_email_enabled: true,
                bms_email_fallback: true,
                alarm_recipients: vec!["devops@automatacontrols.com".to_string()],
                email_on_critical: true,
                email_on_high: true,
                email_on_medium: false,
                email_on_low: false,
                resend_api_key: "re_YoQNcv4n_5DUt4XCbBGMze8njxVR6uZ9Q".to_string(),
                from_address: "devops@automatacontrols.com".to_string(),
            },
            demo_mode: DemoModeSettings {
                enabled: false,
                mock_io_values: false,
                mock_alarms: false,
                mock_metrics: false,
                mock_bms_connection: false,
                simulated_values: HashMap::new(),
            },
            authenticated_sessions: HashMap::new(),
        };
        
        // Create default admin user
        system.create_default_admin();
        system.load_email_templates();
        
        system
    }
    
    fn create_default_admin(&mut self) {
        let admin = User {
            id: Uuid::new_v4().to_string(),
            username: "admin".to_string(),
            email: "admin@automatacontrols.com".to_string(),
            password_hash: hash("Nexus", DEFAULT_COST).unwrap(),
            role: UserRole::Admin,
            created_at: Utc::now(),
            last_login: None,
            active: true,
            api_key: Some("automata-nexus-api-key".to_string()),
        };
        self.users.insert(admin.id.clone(), admin);
    }
    
    fn load_email_templates(&mut self) {
        // Alarm notification template
        self.email_templates.insert("alarm_notification".to_string(), EmailTemplate {
            name: "alarm_notification".to_string(),
            subject: "🚨 Nexus Controller Alarm: {{severity}} - {{equipment}}".to_string(),
            body: r#"
Alarm Notification from Automata Nexus Controller

Severity: {{severity}}
Equipment: {{equipment}}
Point: {{point}}
Value: {{value}} {{units}}
Threshold: {{threshold}}
Timestamp: {{timestamp}}
Location: {{location}}
Controller: {{controller_serial}}

Message: {{message}}

Please log in to the controller to acknowledge this alarm.
Access URL: {{access_url}}
            "#.to_string(),
            body_html: None,
            variables: vec![
                "severity".to_string(),
                "equipment".to_string(),
                "point".to_string(),
                "value".to_string(),
                "units".to_string(),
                "threshold".to_string(),
                "timestamp".to_string(),
                "location".to_string(),
                "controller_serial".to_string(),
                "message".to_string(),
                "access_url".to_string(),
            ],
            category: "alarm".to_string(),
            active: true,
        });
        
        // System status report template
        self.email_templates.insert("status_report".to_string(), EmailTemplate {
            name: "status_report".to_string(),
            subject: "Nexus Controller Status Report - {{date}}".to_string(),
            body: r#"
Daily Status Report
Controller: {{controller_serial}}
Date: {{date}}

System Health:
- CPU Usage: {{cpu_usage}}%
- Memory Usage: {{memory_usage}}%
- Disk Usage: {{disk_usage}}%
- Uptime: {{uptime}}

Active Alarms: {{active_alarms}}
Points Monitored: {{points_monitored}}
BMS Connections: {{bms_connections}}

Last 24 Hours:
- Total Alarms: {{total_alarms}}
- Logic Executions: {{logic_executions}}
- API Calls: {{api_calls}}
            "#.to_string(),
            body_html: None,
            variables: vec![
                "controller_serial".to_string(),
                "date".to_string(),
                "cpu_usage".to_string(),
                "memory_usage".to_string(),
                "disk_usage".to_string(),
                "uptime".to_string(),
                "active_alarms".to_string(),
                "points_monitored".to_string(),
                "bms_connections".to_string(),
                "total_alarms".to_string(),
                "logic_executions".to_string(),
                "api_calls".to_string(),
            ],
            category: "report".to_string(),
            active: true,
        });
        
        // Maintenance reminder template
        self.email_templates.insert("maintenance_reminder".to_string(), EmailTemplate {
            name: "maintenance_reminder".to_string(),
            subject: "Maintenance Due: {{equipment}} - {{task}}".to_string(),
            body: r#"
Maintenance Reminder

Equipment: {{equipment}}
Task: {{task}}
Due Date: {{due_date}}
Priority: {{priority}}
Assigned To: {{assigned_to}}

Notes: {{notes}}

Please complete this maintenance task and update the system.
            "#.to_string(),
            body_html: None,
            variables: vec![
                "equipment".to_string(),
                "task".to_string(),
                "due_date".to_string(),
                "priority".to_string(),
                "assigned_to".to_string(),
                "notes".to_string(),
            ],
            category: "maintenance".to_string(),
            active: true,
        });
    }
    
    // Authentication with admin PIN
    pub fn verify_admin_pin(&self, pin: &str) -> bool {
        pin == ADMIN_PIN
    }
    
    // User management
    pub async fn create_user(&mut self, username: String, email: String, password: String, role: UserRole) -> Result<User> {
        // Check if username exists
        if self.users.values().any(|u| u.username == username) {
            return Err(anyhow!("Username already exists"));
        }
        
        let user = User {
            id: Uuid::new_v4().to_string(),
            username: username.clone(),
            email,
            password_hash: hash(password, DEFAULT_COST)?,
            role,
            created_at: Utc::now(),
            last_login: None,
            active: true,
            api_key: Some(Uuid::new_v4().to_string()),
        };
        
        self.users.insert(user.id.clone(), user.clone());
        
        self.audit_log(
            "system",
            AuditAction::CreateUser,
            &username,
            &format!("Created user with role {:?}", role),
            None,
            true,
            None,
        );
        
        Ok(user)
    }
    
    pub async fn authenticate(&mut self, username: &str, password: &str) -> Result<String> {
        let user = self.users.values_mut()
            .find(|u| u.username == username && u.active)
            .ok_or_else(|| anyhow!("Invalid credentials"))?;
        
        if !verify(password, &user.password_hash)? {
            self.audit_log(
                username,
                AuditAction::Login,
                "login",
                "Failed login attempt",
                None,
                false,
                Some("Invalid password"),
            );
            return Err(anyhow!("Invalid credentials"));
        }
        
        user.last_login = Some(Utc::now());
        let session_id = Uuid::new_v4().to_string();
        
        self.authenticated_sessions.insert(session_id.clone(), AuthSession {
            user_id: user.id.clone(),
            username: user.username.clone(),
            role: user.role.clone(),
            created_at: Utc::now(),
            last_activity: Utc::now(),
            ip_address: "127.0.0.1".to_string(),
        });
        
        self.audit_log(
            username,
            AuditAction::Login,
            "login",
            "Successful login",
            Some("127.0.0.1"),
            true,
            None,
        );
        
        Ok(session_id)
    }
    
    pub fn verify_session(&self, session_id: &str) -> Result<AuthSession> {
        self.authenticated_sessions.get(session_id)
            .cloned()
            .ok_or_else(|| anyhow!("Invalid session"))
    }
    
    pub fn delete_user(&mut self, user_id: &str, deleted_by: &str) -> Result<()> {
        let user = self.users.remove(user_id)
            .ok_or_else(|| anyhow!("User not found"))?;
        
        self.audit_log(
            deleted_by,
            AuditAction::DeleteUser,
            &user.username,
            &format!("Deleted user {}", user.username),
            None,
            true,
            None,
        );
        
        Ok(())
    }
    
    pub fn update_user_role(&mut self, user_id: &str, new_role: UserRole, updated_by: &str) -> Result<()> {
        let user = self.users.get_mut(user_id)
            .ok_or_else(|| anyhow!("User not found"))?;
        
        let old_role = user.role.clone();
        user.role = new_role.clone();
        
        self.audit_log(
            updated_by,
            AuditAction::UpdateUser,
            &user.username,
            &format!("Changed role from {:?} to {:?}", old_role, new_role),
            None,
            true,
            None,
        );
        
        Ok(())
    }
    
    pub fn change_password(&mut self, user_id: &str, new_password: String) -> Result<()> {
        let user = self.users.get_mut(user_id)
            .ok_or_else(|| anyhow!("User not found"))?;
        
        user.password_hash = hash(new_password, DEFAULT_COST)?;
        
        self.audit_log(
            &user.username,
            AuditAction::ChangePassword,
            "password",
            "Password changed",
            None,
            true,
            None,
        );
        
        Ok(())
    }
    
    // Audit logging
    pub fn audit_log(
        &mut self,
        user: &str,
        action: AuditAction,
        target: &str,
        details: &str,
        ip_address: Option<&str>,
        success: bool,
        error_message: Option<&str>,
    ) {
        let log = AuditLog {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            user: user.to_string(),
            action: format!("{:?}", action),
            target: target.to_string(),
            details: details.to_string(),
            ip_address: ip_address.map(|s| s.to_string()),
            success,
            error_message: error_message.map(|s| s.to_string()),
        };
        
        self.audit_logs.push(log);
        
        // Keep only last 10000 logs
        if self.audit_logs.len() > 10000 {
            self.audit_logs.remove(0);
        }
    }
    
    pub fn get_audit_logs(&self, limit: usize) -> Vec<AuditLog> {
        let start = if self.audit_logs.len() > limit {
            self.audit_logs.len() - limit
        } else {
            0
        };
        
        self.audit_logs[start..].to_vec()
    }
    
    pub fn get_audit_logs_by_user(&self, username: &str, limit: usize) -> Vec<AuditLog> {
        self.audit_logs.iter()
            .filter(|log| log.user == username)
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }
    
    // Email system
    pub async fn send_email(&mut self, to: &str, subject: &str, body: &str) -> Result<()> {
        if !self.email_settings.local_email_enabled {
            return Ok(());
        }
        
        let email_log = EmailLog {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            to_address: to.to_string(),
            cc_address: None,
            subject: subject.to_string(),
            template_used: None,
            sent: false,
            error: None,
            attempts: 1,
        };
        
        // Use Resend API
        let client = reqwest::Client::new();
        let response = client
            .post("https://api.resend.com/emails")
            .header("Authorization", format!("Bearer {}", self.email_settings.resend_api_key))
            .json(&serde_json::json!({
                "from": self.email_settings.from_address,
                "to": to,
                "subject": subject,
                "text": body,
            }))
            .send()
            .await?;
        
        if response.status().is_success() {
            let mut log = email_log;
            log.sent = true;
            self.email_logs.push(log);
            
            self.audit_log(
                "system",
                AuditAction::EmailSent,
                to,
                &format!("Email sent: {}", subject),
                None,
                true,
                None,
            );
            
            Ok(())
        } else {
            let error = response.text().await?;
            let mut log = email_log;
            log.error = Some(error.clone());
            self.email_logs.push(log);
            
            Err(anyhow!("Failed to send email: {}", error))
        }
    }
    
    pub async fn send_template_email(
        &mut self,
        template_name: &str,
        to: &str,
        variables: HashMap<String, String>,
    ) -> Result<()> {
        let template = self.email_templates.get(template_name)
            .ok_or_else(|| anyhow!("Template not found"))?
            .clone();
        
        if !template.active {
            return Ok(());
        }
        
        let mut subject = template.subject.clone();
        let mut body = template.body.clone();
        
        // Replace variables
        for (key, value) in variables {
            subject = subject.replace(&format!("{{{{{}}}}}", key), &value);
            body = body.replace(&format!("{{{{{}}}}}", key), &value);
        }
        
        self.send_email(to, &subject, &body).await?;
        
        Ok(())
    }
    
    pub fn update_email_template(&mut self, name: &str, template: EmailTemplate) -> Result<()> {
        self.email_templates.insert(name.to_string(), template);
        Ok(())
    }
    
    pub fn get_email_logs(&self, limit: usize) -> Vec<EmailLog> {
        let start = if self.email_logs.len() > limit {
            self.email_logs.len() - limit
        } else {
            0
        };
        
        self.email_logs[start..].to_vec()
    }
    
    // Demo mode
    pub fn set_demo_mode(&mut self, enabled: bool) {
        self.demo_mode.enabled = enabled;
        
        if enabled {
            // Generate some mock data
            self.demo_mode.simulated_values.insert("temperature".to_string(), 72.5);
            self.demo_mode.simulated_values.insert("humidity".to_string(), 45.0);
            self.demo_mode.simulated_values.insert("pressure".to_string(), 14.7);
            self.demo_mode.simulated_values.insert("flow".to_string(), 250.0);
        }
    }
    
    pub fn is_demo_mode(&self) -> bool {
        self.demo_mode.enabled
    }
    
    pub fn get_demo_value(&self, key: &str) -> Option<f32> {
        if self.demo_mode.enabled {
            self.demo_mode.simulated_values.get(key).copied()
        } else {
            None
        }
    }
    
    pub fn update_demo_value(&mut self, key: String, value: f32) {
        if self.demo_mode.enabled {
            self.demo_mode.simulated_values.insert(key, value);
        }
    }
    
    // User list
    pub fn list_users(&self) -> Vec<User> {
        self.users.values().cloned().collect()
    }
    
    pub fn get_user(&self, user_id: &str) -> Option<User> {
        self.users.get(user_id).cloned()
    }
    
    pub fn get_user_by_username(&self, username: &str) -> Option<User> {
        self.users.values().find(|u| u.username == username).cloned()
    }
    
    // Session management
    pub fn logout(&mut self, session_id: &str) {
        if let Some(session) = self.authenticated_sessions.remove(session_id) {
            self.audit_log(
                &session.username,
                AuditAction::Logout,
                "logout",
                "User logged out",
                Some(&session.ip_address),
                true,
                None,
            );
        }
    }
    
    pub fn cleanup_sessions(&mut self) {
        let cutoff = Utc::now() - chrono::Duration::hours(24);
        self.authenticated_sessions.retain(|_, session| {
            session.last_activity > cutoff
        });
    }
}