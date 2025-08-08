// Database Retention Service - Manages data retention policies
use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use sqlx::SqlitePool;
use tokio::time;

pub struct RetentionService {
    db: SqlitePool,
    retention_days: i64,
}

impl RetentionService {
    pub fn new(db: SqlitePool, retention_days: i64) -> Self {
        Self {
            db,
            retention_days,
        }
    }
    
    pub async fn start(&self) {
        let db = self.db.clone();
        let retention_days = self.retention_days;
        
        tokio::spawn(async move {
            loop {
                // Run cleanup every hour
                time::sleep(time::Duration::from_secs(3600)).await;
                
                let cutoff = Utc::now() - Duration::days(retention_days);
                
                // Clean sensor data
                let _ = sqlx::query("DELETE FROM sensor_data WHERE timestamp < ?")
                    .bind(cutoff)
                    .execute(&db)
                    .await;
                
                // Clean alarm history
                let _ = sqlx::query("DELETE FROM alarm_history WHERE cleared_at < ?")
                    .bind(cutoff)
                    .execute(&db)
                    .await;
                
                // Clean audit log (keep longer)
                let audit_cutoff = Utc::now() - Duration::days(retention_days * 3);
                let _ = sqlx::query("DELETE FROM audit_log WHERE timestamp < ?")
                    .bind(audit_cutoff)
                    .execute(&db)
                    .await;
                
                // Vacuum database
                let _ = sqlx::query("VACUUM").execute(&db).await;
            }
        });
    }
}