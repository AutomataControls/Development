// Database Module - SQLite with SQLx

use anyhow::{Result, anyhow};
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use std::time::Duration;

static mut DB_POOL: Option<SqlitePool> = None;

pub async fn init() -> Result<()> {
    // Create database directory
    tokio::fs::create_dir_all("/var/lib/nexus").await?;
    
    let database_url = "sqlite:///var/lib/nexus/nexus.db";
    
    // Create connection pool
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_timeout(Duration::from_secs(3))
        .connect(database_url)
        .await?;
    
    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;
    
    unsafe {
        DB_POOL = Some(pool);
    }
    
    Ok(())
}

pub fn get_pool() -> Result<&'static SqlitePool> {
    unsafe {
        DB_POOL.as_ref()
            .ok_or_else(|| anyhow!("Database not initialized"))
    }
}

// Example database operations
pub async fn log_event(event_type: &str, message: &str, severity: &str) -> Result<()> {
    let pool = get_pool()?;
    
    sqlx::query!(
        r#"
        INSERT INTO event_logs (event_type, message, severity, created_at)
        VALUES (?1, ?2, ?3, datetime('now'))
        "#,
        event_type,
        message,
        severity
    )
    .execute(pool)
    .await?;
    
    Ok(())
}

pub async fn store_sensor_reading(
    sensor_id: &str,
    vibration_x: f32,
    vibration_y: f32,
    vibration_z: f32,
    temperature: f32,
) -> Result<()> {
    let pool = get_pool()?;
    
    sqlx::query!(
        r#"
        INSERT INTO sensor_readings (sensor_id, vibration_x, vibration_y, vibration_z, temperature, created_at)
        VALUES (?1, ?2, ?3, ?4, ?5, datetime('now'))
        "#,
        sensor_id,
        vibration_x,
        vibration_y,
        vibration_z,
        temperature
    )
    .execute(pool)
    .await?;
    
    Ok(())
}

pub async fn store_board_state(
    board_id: &str,
    state_json: &str,
) -> Result<()> {
    let pool = get_pool()?;
    
    sqlx::query!(
        r#"
        INSERT OR REPLACE INTO board_states (board_id, state_json, updated_at)
        VALUES (?1, ?2, datetime('now'))
        "#,
        board_id,
        state_json
    )
    .execute(pool)
    .await?;
    
    Ok(())
}