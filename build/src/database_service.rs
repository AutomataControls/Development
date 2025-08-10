// Database Service - Provides database operations for UI and API
use anyhow::{Result, anyhow};
use sqlx::{SqlitePool, Row};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricRecord {
    pub id: i64,
    pub timestamp: DateTime<Utc>,
    pub board_id: String,
    pub channel_id: String,
    pub channel_name: String,
    pub value: f64,
    pub units: String,
    pub quality: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseStats {
    pub total_records: usize,
    pub database_size_mb: f64,
    pub oldest_record: Option<DateTime<Utc>>,
    pub newest_record: Option<DateTime<Utc>>,
    pub records_today: usize,
    pub records_week: usize,
    pub tables: Vec<TableInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableInfo {
    pub name: String,
    pub row_count: usize,
    pub size_kb: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionConfig {
    pub enabled: bool,
    pub retention_days: u32,
    pub archive_enabled: bool,
    pub archive_path: String,
}

pub struct DatabaseService {
    pool: SqlitePool,
}

impl DatabaseService {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
    
    // Get database statistics
    pub async fn get_stats(&self) -> Result<DatabaseStats> {
        // Get total records
        let total_records: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sensor_readings"
        )
        .fetch_one(&self.pool)
        .await?;
        
        // Get database file size
        let db_path = "/var/lib/nexus/nexus.db";
        let metadata = tokio::fs::metadata(db_path).await?;
        let database_size_mb = metadata.len() as f64 / (1024.0 * 1024.0);
        
        // Get date range
        let oldest: Option<(DateTime<Utc>,)> = sqlx::query_as(
            "SELECT MIN(created_at) FROM sensor_readings"
        )
        .fetch_optional(&self.pool)
        .await?;
        
        let newest: Option<(DateTime<Utc>,)> = sqlx::query_as(
            "SELECT MAX(created_at) FROM sensor_readings"
        )
        .fetch_optional(&self.pool)
        .await?;
        
        // Count recent records
        let records_today: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sensor_readings WHERE created_at > datetime('now', '-1 day')"
        )
        .fetch_one(&self.pool)
        .await?;
        
        let records_week: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sensor_readings WHERE created_at > datetime('now', '-7 days')"
        )
        .fetch_one(&self.pool)
        .await?;
        
        // Get table information
        let tables = self.get_table_info().await?;
        
        Ok(DatabaseStats {
            total_records: total_records as usize,
            database_size_mb,
            oldest_record: oldest.map(|r| r.0),
            newest_record: newest.map(|r| r.0),
            records_today: records_today as usize,
            records_week: records_week as usize,
            tables,
        })
    }
    
    // Get information about all tables
    async fn get_table_info(&self) -> Result<Vec<TableInfo>> {
        let rows = sqlx::query(
            "SELECT name, (SELECT COUNT(*) FROM sqlite_master AS m2 WHERE m2.name = m1.name) as count 
             FROM sqlite_master AS m1 
             WHERE type='table' AND name NOT LIKE 'sqlite_%'"
        )
        .fetch_all(&self.pool)
        .await?;
        
        let mut tables = Vec::new();
        for row in rows {
            let name: String = row.get(0);
            
            // Get row count for each table
            let count_query = format!("SELECT COUNT(*) FROM {}", name);
            let count: i64 = sqlx::query_scalar(&count_query)
                .fetch_one(&self.pool)
                .await
                .unwrap_or(0);
            
            tables.push(TableInfo {
                name,
                row_count: count as usize,
                size_kb: 0.0, // Would need to calculate actual size
            });
        }
        
        Ok(tables)
    }
    
    // Query sensor readings with filters
    pub async fn query_metrics(
        &self,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
        board_id: Option<String>,
        channel_id: Option<String>,
        limit: Option<usize>,
    ) -> Result<Vec<MetricRecord>> {
        let mut query = String::from(
            "SELECT id, created_at, sensor_id, sensor_id, sensor_id, 
                    vibration_x, 'mm/s', 'Good' 
             FROM sensor_readings WHERE 1=1"
        );
        
        if let Some(start) = start_time {
            query.push_str(&format!(" AND created_at >= '{}'", start.to_rfc3339()));
        }
        
        if let Some(end) = end_time {
            query.push_str(&format!(" AND created_at <= '{}'", end.to_rfc3339()));
        }
        
        if let Some(board) = board_id {
            query.push_str(&format!(" AND sensor_id LIKE '{}%'", board));
        }
        
        query.push_str(" ORDER BY created_at DESC");
        
        if let Some(lim) = limit {
            query.push_str(&format!(" LIMIT {}", lim));
        }
        
        let rows = sqlx::query(&query)
            .fetch_all(&self.pool)
            .await?;
        
        let mut metrics = Vec::new();
        for row in rows {
            metrics.push(MetricRecord {
                id: row.get(0),
                timestamp: row.get(1),
                board_id: row.get(2),
                channel_id: row.get(3),
                channel_name: row.get(4),
                value: row.get(5),
                units: row.get(6),
                quality: row.get(7),
            });
        }
        
        Ok(metrics)
    }
    
    // Execute custom SQL query (read-only)
    pub async fn execute_sql(&self, query: &str) -> Result<Vec<HashMap<String, String>>> {
        // Only allow SELECT queries
        if !query.trim().to_uppercase().starts_with("SELECT") {
            return Err(anyhow!("Only SELECT queries are allowed"));
        }
        
        let rows = sqlx::query(query)
            .fetch_all(&self.pool)
            .await?;
        
        let mut results = Vec::new();
        for row in rows {
            let mut record = HashMap::new();
            for (i, column) in row.columns().iter().enumerate() {
                let value: String = row.try_get(i).unwrap_or_else(|_| "NULL".to_string());
                record.insert(column.name().to_string(), value);
            }
            results.push(record);
        }
        
        Ok(results)
    }
    
    // Clean old data based on retention policy
    pub async fn clean_old_data(&self, retention_days: u32) -> Result<usize> {
        let cutoff = Utc::now() - chrono::Duration::days(retention_days as i64);
        
        let result = sqlx::query(&format!(
            "DELETE FROM sensor_readings WHERE created_at < '{}'",
            cutoff.to_rfc3339()
        ))
        .execute(&self.pool)
        .await?;
        
        // Vacuum to reclaim space
        sqlx::query("VACUUM")
            .execute(&self.pool)
            .await?;
        
        Ok(result.rows_affected() as usize)
    }
    
    // Archive old data to file
    pub async fn archive_data(&self, days_to_archive: u32) -> Result<String> {
        let cutoff = Utc::now() - chrono::Duration::days(days_to_archive as i64);
        let archive_file = format!(
            "/var/backups/nexus/archive_{}.db",
            Utc::now().format("%Y%m%d_%H%M%S")
        );
        
        // Create archive directory
        tokio::fs::create_dir_all("/var/backups/nexus").await?;
        
        // Attach archive database and copy old data
        sqlx::query(&format!("ATTACH DATABASE '{}' AS archive", archive_file))
            .execute(&self.pool)
            .await?;
        
        // Create tables in archive
        sqlx::query(
            "CREATE TABLE archive.sensor_readings AS 
             SELECT * FROM main.sensor_readings WHERE created_at < ?1"
        )
        .bind(cutoff)
        .execute(&self.pool)
        .await?;
        
        // Delete archived data from main database
        sqlx::query("DELETE FROM main.sensor_readings WHERE created_at < ?1")
            .bind(cutoff)
            .execute(&self.pool)
            .await?;
        
        // Detach archive
        sqlx::query("DETACH DATABASE archive")
            .execute(&self.pool)
            .await?;
        
        Ok(archive_file)
    }
    
    // Get retention configuration
    pub async fn get_retention_config(&self) -> Result<RetentionConfig> {
        let row = sqlx::query(
            "SELECT value FROM system_config WHERE key IN ('retention_enabled', 'retention_days', 'archive_path')"
        )
        .fetch_all(&self.pool)
        .await?;
        
        // Parse configuration from database or use defaults
        Ok(RetentionConfig {
            enabled: true,
            retention_days: 30,
            archive_enabled: true,
            archive_path: "/var/backups/nexus".to_string(),
        })
    }
    
    // Update retention configuration
    pub async fn update_retention_config(&self, config: &RetentionConfig) -> Result<()> {
        sqlx::query(
            "INSERT OR REPLACE INTO system_config (key, value) VALUES 
             ('retention_enabled', ?1),
             ('retention_days', ?2),
             ('archive_enabled', ?3),
             ('archive_path', ?4)"
        )
        .bind(config.enabled.to_string())
        .bind(config.retention_days.to_string())
        .bind(config.archive_enabled.to_string())
        .bind(&config.archive_path)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
}