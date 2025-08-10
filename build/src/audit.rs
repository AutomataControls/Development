// Audit Service - Tracks ALL user actions and system events
use anyhow::Result;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use sqlx::SqlitePool;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub user: String,
    pub action: String,
    pub resource: String,
    pub details: Option<String>,
    pub ip_address: Option<String>,
    pub success: bool,
    pub error: Option<String>,
}

pub struct AuditService {
    db: SqlitePool,
    entries: Arc<RwLock<Vec<AuditEntry>>>,
}

impl AuditService {
    pub async fn new(db: SqlitePool) -> Result<Self> {
        // Create audit table
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS audit_log (
                id TEXT PRIMARY KEY,
                timestamp DATETIME NOT NULL,
                user TEXT NOT NULL,
                action TEXT NOT NULL,
                resource TEXT NOT NULL,
                details TEXT,
                ip_address TEXT,
                success BOOLEAN NOT NULL,
                error TEXT
            )
        "#).execute(&db).await?;
        
        Ok(Self {
            db,
            entries: Arc::new(RwLock::new(Vec::new())),
        })
    }
    
    pub async fn log(&self, entry: AuditEntry) -> Result<()> {
        // Store in database
        sqlx::query(r#"
            INSERT INTO audit_log (id, timestamp, user, action, resource, details, ip_address, success, error)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
        "#)
        .bind(&entry.id)
        .bind(&entry.timestamp)
        .bind(&entry.user)
        .bind(&entry.action)
        .bind(&entry.resource)
        .bind(&entry.details)
        .bind(&entry.ip_address)
        .bind(&entry.success)
        .bind(&entry.error)
        .execute(&self.db)
        .await?;
        
        // Store in memory cache
        self.entries.write().await.push(entry);
        
        Ok(())
    }
    
    pub async fn get_recent(&self, limit: i64) -> Result<Vec<AuditEntry>> {
        let entries = sqlx::query_as!(
            AuditEntry,
            r#"
            SELECT id, timestamp, user, action, resource, details, ip_address, success, error
            FROM audit_log
            ORDER BY timestamp DESC
            LIMIT ?
            "#,
            limit
        )
        .fetch_all(&self.db)
        .await?;
        
        Ok(entries)
    }
}