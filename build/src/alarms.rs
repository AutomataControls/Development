// Alarm Monitoring System for Critical HVAC Equipment
// Real-time alarm detection, notification, and acknowledgment

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlarmSeverity {
    Critical,   // Immediate action required - equipment damage risk
    High,       // Action required soon - performance degradation
    Medium,     // Attention needed - efficiency issue
    Low,        // Informational - minor issue
    Info,       // Status update only
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlarmType {
    HighPressure,
    LowPressure,
    HighTemperature,
    LowTemperature,
    SensorFailure,
    CommunicationLoss,
    EquipmentFault,
    SafetyInterlock,
    MaintenanceDue,
    VibrationHigh,
    RefrigerantLeak,
    PowerFailure,
    ControlFailure,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlarmDefinition {
    pub id: String,
    pub name: String,
    pub alarm_type: AlarmType,
    pub severity: AlarmSeverity,
    pub enabled: bool,
    pub point_name: String,
    pub condition: AlarmCondition,
    pub delay_seconds: u32,        // Delay before triggering
    pub auto_reset: bool,           // Auto-reset when condition clears
    pub require_acknowledgment: bool,
    pub notification_enabled: bool,
    pub email_recipients: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlarmCondition {
    HighLimit { value: f32 },
    LowLimit { value: f32 },
    OutOfRange { min: f32, max: f32 },
    RateOfChange { max_change_per_min: f32 },
    Boolean { expected: bool },
    Deviation { from_setpoint: f32 },
    Equipment { fault_code: u32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveAlarm {
    pub id: String,
    pub definition_id: String,
    pub name: String,
    pub alarm_type: AlarmType,
    pub severity: AlarmSeverity,
    pub triggered_at: DateTime<Utc>,
    pub value: f32,
    pub threshold: f32,
    pub message: String,
    pub location: String,
    pub equipment: Option<String>,
    pub acknowledged: bool,
    pub acknowledged_by: Option<String>,
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub cleared: bool,
    pub cleared_at: Option<DateTime<Utc>>,
    pub duration_seconds: u64,
    pub occurrence_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlarmHistory {
    pub alarm_id: String,
    pub name: String,
    pub severity: AlarmSeverity,
    pub triggered_at: DateTime<Utc>,
    pub cleared_at: Option<DateTime<Utc>>,
    pub duration_seconds: u64,
    pub max_value: f32,
    pub acknowledged: bool,
    pub acknowledged_by: Option<String>,
    pub notes: Option<String>,
}

pub struct AlarmMonitor {
    definitions: Arc<RwLock<HashMap<String, AlarmDefinition>>>,
    active_alarms: Arc<RwLock<HashMap<String, ActiveAlarm>>>,
    alarm_history: Arc<RwLock<Vec<AlarmHistory>>>,
    alarm_counts: Arc<RwLock<HashMap<AlarmSeverity, u32>>>,
    monitoring_enabled: Arc<RwLock<bool>>,
    alarm_callback: Option<Arc<dyn Fn(ActiveAlarm) + Send + Sync>>,
}

impl AlarmMonitor {
    pub fn new() -> Self {
        let mut monitor = Self {
            definitions: Arc::new(RwLock::new(HashMap::new())),
            active_alarms: Arc::new(RwLock::new(HashMap::new())),
            alarm_history: Arc::new(RwLock::new(Vec::new())),
            alarm_counts: Arc::new(RwLock::new(HashMap::new())),
            monitoring_enabled: Arc::new(RwLock::new(true())),
            alarm_callback: None,
        };
        
        monitor.init_default_alarms();
        monitor
    }
    
    fn init_default_alarms(&self) {
        let alarms = vec![
            // Critical alarms
            AlarmDefinition {
                id: "high_discharge_pressure".to_string(),
                name: "High Discharge Pressure".to_string(),
                alarm_type: AlarmType::HighPressure,
                severity: AlarmSeverity::Critical,
                enabled: true,
                point_name: "discharge_pressure".to_string(),
                condition: AlarmCondition::HighLimit { value: 450.0 },
                delay_seconds: 5,
                auto_reset: false,
                require_acknowledgment: true,
                notification_enabled: true,
                email_recipients: vec!["devops@automatacontrols.com".to_string()],
            },
            AlarmDefinition {
                id: "low_suction_pressure".to_string(),
                name: "Low Suction Pressure".to_string(),
                alarm_type: AlarmType::LowPressure,
                severity: AlarmSeverity::Critical,
                enabled: true,
                point_name: "suction_pressure".to_string(),
                condition: AlarmCondition::LowLimit { value: 20.0 },
                delay_seconds: 10,
                auto_reset: false,
                require_acknowledgment: true,
                notification_enabled: true,
                email_recipients: vec!["devops@automatacontrols.com".to_string()],
            },
            AlarmDefinition {
                id: "high_discharge_temp".to_string(),
                name: "High Discharge Temperature".to_string(),
                alarm_type: AlarmType::HighTemperature,
                severity: AlarmSeverity::Critical,
                enabled: true,
                point_name: "discharge_temp".to_string(),
                condition: AlarmCondition::HighLimit { value: 225.0 },
                delay_seconds: 5,
                auto_reset: false,
                require_acknowledgment: true,
                notification_enabled: true,
                email_recipients: vec!["devops@automatacontrols.com".to_string()],
            },
            
            // High severity alarms
            AlarmDefinition {
                id: "vibration_high".to_string(),
                name: "High Vibration".to_string(),
                alarm_type: AlarmType::VibrationHigh,
                severity: AlarmSeverity::High,
                enabled: true,
                point_name: "vibration_velocity".to_string(),
                condition: AlarmCondition::HighLimit { value: 7.1 }, // ISO 10816-3 Zone D
                delay_seconds: 30,
                auto_reset: true,
                require_acknowledgment: true,
                notification_enabled: true,
                email_recipients: vec!["devops@automatacontrols.com".to_string()],
            },
            AlarmDefinition {
                id: "low_superheat".to_string(),
                name: "Low Superheat".to_string(),
                alarm_type: AlarmType::LowTemperature,
                severity: AlarmSeverity::High,
                enabled: true,
                point_name: "superheat".to_string(),
                condition: AlarmCondition::LowLimit { value: 5.0 },
                delay_seconds: 60,
                auto_reset: true,
                require_acknowledgment: false,
                notification_enabled: true,
                email_recipients: vec!["devops@automatacontrols.com".to_string()],
            },
            
            // Medium severity alarms
            AlarmDefinition {
                id: "high_runtime".to_string(),
                name: "Extended Runtime".to_string(),
                alarm_type: AlarmType::MaintenanceDue,
                severity: AlarmSeverity::Medium,
                enabled: true,
                point_name: "compressor_runtime".to_string(),
                condition: AlarmCondition::HighLimit { value: 86400.0 }, // 24 hours
                delay_seconds: 0,
                auto_reset: false,
                require_acknowledgment: true,
                notification_enabled: false,
                email_recipients: vec![],
            },
            
            // Low severity alarms
            AlarmDefinition {
                id: "efficiency_low".to_string(),
                name: "Low Efficiency".to_string(),
                alarm_type: AlarmType::EquipmentFault,
                severity: AlarmSeverity::Low,
                enabled: true,
                point_name: "efficiency_score".to_string(),
                condition: AlarmCondition::LowLimit { value: 70.0 },
                delay_seconds: 300,
                auto_reset: true,
                require_acknowledgment: false,
                notification_enabled: false,
                email_recipients: vec![],
            },
        ];
        
        let mut defs = self.definitions.blocking_write();
        for alarm in alarms {
            defs.insert(alarm.id.clone(), alarm);
        }
    }
    
    // Check if value triggers alarm condition
    fn check_condition(condition: &AlarmCondition, value: f32) -> bool {
        match condition {
            AlarmCondition::HighLimit { value: limit } => value > *limit,
            AlarmCondition::LowLimit { value: limit } => value < *limit,
            AlarmCondition::OutOfRange { min, max } => value < *min || value > *max,
            AlarmCondition::Deviation { from_setpoint } => value.abs() > *from_setpoint,
            _ => false,
        }
    }
    
    // Process point value for alarms
    pub async fn process_point(&self, point_name: &str, value: f32) -> Result<Vec<ActiveAlarm>> {
        if !*self.monitoring_enabled.read().await {
            return Ok(vec![]);
        }
        
        let mut triggered_alarms = Vec::new();
        let definitions = self.definitions.read().await;
        
        for (def_id, definition) in definitions.iter() {
            if !definition.enabled || definition.point_name != point_name {
                continue;
            }
            
            let is_triggered = Self::check_condition(&definition.condition, value);
            let mut active_alarms = self.active_alarms.write().await;
            
            if is_triggered {
                // Check if alarm already active
                if !active_alarms.contains_key(def_id) {
                    // Get threshold value for display
                    let threshold = match &definition.condition {
                        AlarmCondition::HighLimit { value } => *value,
                        AlarmCondition::LowLimit { value } => *value,
                        _ => 0.0,
                    };
                    
                    // Create new active alarm
                    let alarm = ActiveAlarm {
                        id: format!("{}_{}", def_id, Utc::now().timestamp()),
                        definition_id: def_id.clone(),
                        name: definition.name.clone(),
                        alarm_type: definition.alarm_type.clone(),
                        severity: definition.severity.clone(),
                        triggered_at: Utc::now(),
                        value,
                        threshold,
                        message: format!("{}: {} = {:.2} (threshold: {:.2})", 
                            definition.name, point_name, value, threshold),
                        location: "Main Building".to_string(),
                        equipment: Some("HVAC System".to_string()),
                        acknowledged: false,
                        acknowledged_by: None,
                        acknowledged_at: None,
                        cleared: false,
                        cleared_at: None,
                        duration_seconds: 0,
                        occurrence_count: 1,
                    };
                    
                    // Log alarm
                    self.log_alarm(&alarm);
                    
                    // Call notification callback if set
                    if let Some(callback) = &self.alarm_callback {
                        callback(alarm.clone());
                    }
                    
                    // Send email notification if enabled
                    if definition.notification_enabled {
                        self.send_alarm_notification(&alarm).await?;
                    }
                    
                    active_alarms.insert(def_id.clone(), alarm.clone());
                    triggered_alarms.push(alarm);
                    
                    // Update alarm counts
                    let mut counts = self.alarm_counts.write().await;
                    *counts.entry(definition.severity.clone()).or_insert(0) += 1;
                }
            } else if definition.auto_reset {
                // Check if alarm should clear
                if let Some(mut alarm) = active_alarms.get_mut(def_id) {
                    if !alarm.cleared {
                        alarm.cleared = true;
                        alarm.cleared_at = Some(Utc::now());
                        alarm.duration_seconds = Utc::now()
                            .signed_duration_since(alarm.triggered_at)
                            .num_seconds() as u64;
                        
                        // Move to history
                        let history = AlarmHistory {
                            alarm_id: alarm.id.clone(),
                            name: alarm.name.clone(),
                            severity: alarm.severity.clone(),
                            triggered_at: alarm.triggered_at,
                            cleared_at: alarm.cleared_at,
                            duration_seconds: alarm.duration_seconds,
                            max_value: alarm.value,
                            acknowledged: alarm.acknowledged,
                            acknowledged_by: alarm.acknowledged_by.clone(),
                            notes: None,
                        };
                        
                        let mut alarm_history = self.alarm_history.write().await;
                        alarm_history.push(history);
                        
                        // Remove from active if acknowledged or not required
                        if !definition.require_acknowledgment || alarm.acknowledged {
                            active_alarms.remove(def_id);
                        }
                    }
                }
            }
        }
        
        Ok(triggered_alarms)
    }
    
    // Acknowledge alarm
    pub async fn acknowledge_alarm(&self, alarm_id: &str, user: &str) -> Result<()> {
        let mut active_alarms = self.active_alarms.write().await;
        
        for alarm in active_alarms.values_mut() {
            if alarm.id == alarm_id {
                alarm.acknowledged = true;
                alarm.acknowledged_by = Some(user.to_string());
                alarm.acknowledged_at = Some(Utc::now());
                
                // If alarm is cleared and acknowledged, move to history
                if alarm.cleared {
                    let history = AlarmHistory {
                        alarm_id: alarm.id.clone(),
                        name: alarm.name.clone(),
                        severity: alarm.severity.clone(),
                        triggered_at: alarm.triggered_at,
                        cleared_at: alarm.cleared_at,
                        duration_seconds: alarm.duration_seconds,
                        max_value: alarm.value,
                        acknowledged: true,
                        acknowledged_by: Some(user.to_string()),
                        notes: None,
                    };
                    
                    let mut alarm_history = self.alarm_history.write().await;
                    alarm_history.push(history);
                }
                
                return Ok(());
            }
        }
        
        Err(anyhow!("Alarm not found"))
    }
    
    // Get active alarms
    pub async fn get_active_alarms(&self) -> Vec<ActiveAlarm> {
        self.active_alarms.read().await.values().cloned().collect()
    }
    
    // Get active alarms by severity
    pub async fn get_alarms_by_severity(&self, severity: AlarmSeverity) -> Vec<ActiveAlarm> {
        self.active_alarms
            .read()
            .await
            .values()
            .filter(|a| a.severity == severity)
            .cloned()
            .collect()
    }
    
    // Get alarm history
    pub async fn get_alarm_history(&self, limit: usize) -> Vec<AlarmHistory> {
        let history = self.alarm_history.read().await;
        let start = if history.len() > limit {
            history.len() - limit
        } else {
            0
        };
        history[start..].to_vec()
    }
    
    // Get alarm counts by severity
    pub async fn get_alarm_counts(&self) -> HashMap<AlarmSeverity, u32> {
        self.alarm_counts.read().await.clone()
    }
    
    // Clear all alarms (for testing/reset)
    pub async fn clear_all_alarms(&self) {
        self.active_alarms.write().await.clear();
        self.alarm_counts.write().await.clear();
    }
    
    // Enable/disable monitoring
    pub async fn set_monitoring_enabled(&self, enabled: bool) {
        *self.monitoring_enabled.write().await = enabled;
    }
    
    // Add alarm definition
    pub async fn add_alarm_definition(&self, definition: AlarmDefinition) -> Result<()> {
        let mut defs = self.definitions.write().await;
        defs.insert(definition.id.clone(), definition);
        Ok(())
    }
    
    // Remove alarm definition
    pub async fn remove_alarm_definition(&self, id: &str) -> Result<()> {
        let mut defs = self.definitions.write().await;
        defs.remove(id).ok_or_else(|| anyhow!("Definition not found"))?;
        Ok(())
    }
    
    // Set alarm callback
    pub fn set_alarm_callback<F>(&mut self, callback: F)
    where
        F: Fn(ActiveAlarm) + Send + Sync + 'static,
    {
        self.alarm_callback = Some(Arc::new(callback));
    }
    
    // Log alarm to console (would go to database in production)
    fn log_alarm(&self, alarm: &ActiveAlarm) {
        let icon = match alarm.severity {
            AlarmSeverity::Critical => "🚨",
            AlarmSeverity::High => "⚠️",
            AlarmSeverity::Medium => "⚡",
            AlarmSeverity::Low => "ℹ️",
            AlarmSeverity::Info => "📋",
        };
        
        println!("{} ALARM: {} - {}", icon, alarm.name, alarm.message);
    }
    
    // Send alarm notification
    async fn send_alarm_notification(&self, alarm: &ActiveAlarm) -> Result<()> {
        // In production, this would send actual emails
        println!("📧 Sending alarm notification for: {}", alarm.name);
        Ok(())
    }
}