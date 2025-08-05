use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use sqlx::{Pool, Sqlite};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Metric {
    pub id: String,
    pub board_id: String,
    pub channel_type: String, // universal_input, analog_output, relay, triac
    pub channel_index: i32,
    pub channel_name: String,
    pub value: f64,
    pub scaled_value: Option<f64>,
    pub units: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendData {
    pub channel_name: String,
    pub units: Option<String>,
    pub data_points: Vec<DataPoint>,
    pub statistics: Statistics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPoint {
    pub timestamp: DateTime<Utc>,
    pub value: f64,
    pub scaled_value: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Statistics {
    pub min: f64,
    pub max: f64,
    pub avg: f64,
    pub std_dev: f64,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricQuery {
    pub board_id: Option<String>,
    pub channel_type: Option<String>,
    pub channel_name: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
}

#[derive(Clone)]
pub struct MetricsDatabase {
    pool: Pool<Sqlite>,
}

impl MetricsDatabase {
    pub async fn new(db_path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        // Ensure directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let db_url = format!("sqlite:{}", db_path.display());
        
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&db_url)
            .await?;

        let db = Self { pool };
        db.initialize().await?;
        Ok(db)
    }

    async fn initialize(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Create metrics table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS metrics (
                id TEXT PRIMARY KEY,
                board_id TEXT NOT NULL,
                channel_type TEXT NOT NULL,
                channel_index INTEGER NOT NULL,
                channel_name TEXT NOT NULL,
                value REAL NOT NULL,
                scaled_value REAL,
                units TEXT,
                timestamp DATETIME NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create indexes for efficient querying
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_metrics_timestamp ON metrics(timestamp DESC)"
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_metrics_board_channel ON metrics(board_id, channel_type, channel_index)"
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_metrics_channel_name ON metrics(channel_name)"
        )
        .execute(&self.pool)
        .await?;

        // Create aggregated hourly data table for faster trend analysis
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS metrics_hourly (
                id TEXT PRIMARY KEY,
                board_id TEXT NOT NULL,
                channel_type TEXT NOT NULL,
                channel_index INTEGER NOT NULL,
                channel_name TEXT NOT NULL,
                hour_timestamp DATETIME NOT NULL,
                min_value REAL NOT NULL,
                max_value REAL NOT NULL,
                avg_value REAL NOT NULL,
                sample_count INTEGER NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                UNIQUE(board_id, channel_type, channel_index, hour_timestamp)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_metrics_hourly_timestamp ON metrics_hourly(hour_timestamp DESC)"
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn insert_metric(
        &self,
        board_id: &str,
        channel_type: &str,
        channel_index: i32,
        channel_name: &str,
        value: f64,
        scaled_value: Option<f64>,
        units: Option<&str>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let id = Uuid::new_v4().to_string();
        let timestamp = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO metrics (id, board_id, channel_type, channel_index, channel_name, value, scaled_value, units, timestamp)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            "#,
        )
        .bind(&id)
        .bind(board_id)
        .bind(channel_type)
        .bind(channel_index)
        .bind(channel_name)
        .bind(value)
        .bind(scaled_value)
        .bind(units)
        .bind(timestamp)
        .execute(&self.pool)
        .await?;

        Ok(id)
    }

    pub async fn insert_metrics_batch(
        &self,
        metrics: Vec<(String, String, i32, String, f64, Option<f64>, Option<String>)>,
    ) -> Result<usize, Box<dyn std::error::Error>> {
        let timestamp = Utc::now();
        let mut count = 0;

        for (board_id, channel_type, channel_index, channel_name, value, scaled_value, units) in metrics {
            let id = Uuid::new_v4().to_string();
            
            sqlx::query(
                r#"
                INSERT INTO metrics (id, board_id, channel_type, channel_index, channel_name, value, scaled_value, units, timestamp)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                "#,
            )
            .bind(&id)
            .bind(&board_id)
            .bind(&channel_type)
            .bind(channel_index)
            .bind(&channel_name)
            .bind(value)
            .bind(scaled_value)
            .bind(units.as_deref())
            .bind(timestamp)
            .execute(&self.pool)
            .await?;
            
            count += 1;
        }

        Ok(count)
    }

    pub async fn query_metrics(
        &self,
        query: MetricQuery,
    ) -> Result<Vec<Metric>, Box<dyn std::error::Error>> {
        let mut sql = String::from("SELECT * FROM metrics WHERE 1=1");
        let mut bindings = vec![];

        if let Some(board_id) = query.board_id {
            sql.push_str(" AND board_id = ?");
            bindings.push(board_id);
        }

        if let Some(channel_type) = query.channel_type {
            sql.push_str(" AND channel_type = ?");
            bindings.push(channel_type);
        }

        if let Some(channel_name) = query.channel_name {
            sql.push_str(" AND channel_name = ?");
            bindings.push(channel_name);
        }

        let start_time = query.start_time.unwrap_or(Utc::now() - Duration::hours(24));
        sql.push_str(" AND timestamp >= ?");

        let end_time = query.end_time.unwrap_or(Utc::now());
        sql.push_str(" AND timestamp <= ?");

        sql.push_str(" ORDER BY timestamp DESC");

        if let Some(limit) = query.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        let mut query_builder = sqlx::query_as::<_, Metric>(&sql);
        
        for binding in bindings {
            query_builder = query_builder.bind(binding);
        }
        
        query_builder = query_builder.bind(start_time);
        query_builder = query_builder.bind(end_time);

        let metrics = query_builder.fetch_all(&self.pool).await?;
        Ok(metrics)
    }

    pub async fn get_trend_data(
        &self,
        board_id: &str,
        channel_type: &str,
        channel_index: i32,
        hours: i32,
    ) -> Result<TrendData, Box<dyn std::error::Error>> {
        let start_time = Utc::now() - Duration::hours(hours as i64);

        // Get all data points
        let metrics: Vec<Metric> = sqlx::query_as(
            r#"
            SELECT * FROM metrics 
            WHERE board_id = ?1 AND channel_type = ?2 AND channel_index = ?3 
            AND timestamp >= ?4
            ORDER BY timestamp ASC
            "#,
        )
        .bind(board_id)
        .bind(channel_type)
        .bind(channel_index)
        .bind(start_time)
        .fetch_all(&self.pool)
        .await?;

        if metrics.is_empty() {
            return Ok(TrendData {
                channel_name: format!("{} {}", channel_type, channel_index),
                units: None,
                data_points: vec![],
                statistics: Statistics {
                    min: 0.0,
                    max: 0.0,
                    avg: 0.0,
                    std_dev: 0.0,
                    count: 0,
                },
            });
        }

        // Get channel info from first metric
        let channel_name = metrics[0].channel_name.clone();
        let units = metrics[0].units.clone();

        // Convert to data points
        let data_points: Vec<DataPoint> = metrics
            .iter()
            .map(|m| DataPoint {
                timestamp: m.timestamp,
                value: m.value,
                scaled_value: m.scaled_value,
            })
            .collect();

        // Calculate statistics
        let values: Vec<f64> = metrics.iter().map(|m| m.scaled_value.unwrap_or(m.value)).collect();
        let count = values.len() as i64;
        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let sum: f64 = values.iter().sum();
        let avg = sum / count as f64;

        // Calculate standard deviation
        let variance = values.iter()
            .map(|&x| (x - avg).powi(2))
            .sum::<f64>() / count as f64;
        let std_dev = variance.sqrt();

        Ok(TrendData {
            channel_name,
            units,
            data_points,
            statistics: Statistics {
                min,
                max,
                avg,
                std_dev,
                count,
            },
        })
    }

    pub async fn cleanup_old_data(&self) -> Result<usize, Box<dyn std::error::Error>> {
        let cutoff_date = Utc::now() - Duration::days(7);

        // Archive to hourly aggregates before deleting
        self.aggregate_to_hourly(&cutoff_date).await?;

        // Delete old raw data
        let result = sqlx::query("DELETE FROM metrics WHERE timestamp < ?1")
            .bind(cutoff_date)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() as usize)
    }

    async fn aggregate_to_hourly(
        &self,
        before_date: &DateTime<Utc>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Aggregate data older than cutoff to hourly summaries
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO metrics_hourly (id, board_id, channel_type, channel_index, channel_name, hour_timestamp, min_value, max_value, avg_value, sample_count)
            SELECT 
                printf('%s-%s-%s-%d-%s', board_id, channel_type, channel_index, strftime('%Y%m%d%H', timestamp), channel_name) as id,
                board_id,
                channel_type,
                channel_index,
                channel_name,
                datetime(strftime('%Y-%m-%d %H:00:00', timestamp)) as hour_timestamp,
                MIN(value) as min_value,
                MAX(value) as max_value,
                AVG(value) as avg_value,
                COUNT(*) as sample_count
            FROM metrics
            WHERE timestamp < ?1
            GROUP BY board_id, channel_type, channel_index, strftime('%Y%m%d%H', timestamp)
            "#,
        )
        .bind(before_date)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_channel_list(&self, board_id: &str) -> Result<Vec<(String, String, i32, String)>, Box<dyn std::error::Error>> {
        let channels: Vec<(String, String, i32, String)> = sqlx::query_as(
            r#"
            SELECT DISTINCT channel_type, channel_name, channel_index, units
            FROM metrics
            WHERE board_id = ?1
            ORDER BY channel_type, channel_index
            "#,
        )
        .bind(board_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(channels)
    }
}

// Cleanup task that runs periodically
pub async fn start_cleanup_task(db: MetricsDatabase) {
    tokio::spawn(async move {
        loop {
            // Run cleanup once per day
            tokio::time::sleep(tokio::time::Duration::from_secs(86400)).await;
            
            match db.cleanup_old_data().await {
                Ok(deleted) => {
                    println!("Cleaned up {} old metric records", deleted);
                }
                Err(e) => {
                    eprintln!("Error during metrics cleanup: {}", e);
                }
            }
        }
    });
}