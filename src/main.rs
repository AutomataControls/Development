// Automata Nexus AI - Native Rust Building Automation Controller
// Copyright (c) 2025 Automata Controls
// Developed by Andrew Jewell Sr.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod api;
mod api_complete;
mod api_all_routes;
mod auth;
mod admin;
mod alarms;
mod boards;
mod database;
mod config;
mod logic_engine;
mod sensors;
mod vibration;
mod refrigerant;
mod protocols;
mod weather;
mod state;
mod utils;
mod serial_manager;
mod ui;
mod email;
mod audit;
mod database_retention;
mod system_commands;
mod firmware_manager;
mod processing_rules;

use state::AppState;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("nexus_controller=debug,tauri=info")
        .init();

    info!("Starting Automata Nexus AI Controller v2.0.0");

    // Initialize application state
    let app_state = Arc::new(Mutex::new(
        AppState::new().await.expect("Failed to initialize app state")
    ));

    // Initialize database
    let db_state = app_state.clone();
    tokio::spawn(async move {
        if let Err(e) = database::init().await {
            error!("Failed to initialize database: {}", e);
        }
        
        // Start database retention service
        if let Ok(pool) = sqlx::SqlitePool::connect("sqlite:///var/lib/nexus/nexus.db").await {
            let retention = database_retention::RetentionService::new(pool, 30);
            retention.start().await;
        }
    });

    // Start background services
    let state_clone = app_state.clone();
    tokio::spawn(async move {
        // Start board monitoring
        boards::start_monitoring(state_clone.clone()).await;
        
        // Start sensor monitoring  
        sensors::start_monitoring(state_clone.clone()).await;
        
        // Start weather updates
        weather::start_updates(state_clone.clone()).await;
        
        // Start processing rules engine
        let engine = processing_rules::ProcessingEngine::new();
        loop {
            engine.evaluate_rules().await;
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    });
    
    // Start API server in background
    let api_state = app_state.clone();
    tokio::spawn(async move {
        let app = api_all_routes::create_all_routes(api_state);
        let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
        axum::serve(listener, app).await.unwrap();
    });

    // Launch native UI using egui
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Automata Nexus AI Controller")
            .with_inner_size([1920.0, 1080.0])
            .with_min_inner_size([1024.0, 768.0])
            .with_icon(load_icon()),
        ..Default::default()
    };

    eframe::run_native(
        "Automata Nexus AI",
        native_options,
        Box::new(|cc| Box::new(ui::NexusApp::new(cc, app_state)))
    )?;

    Ok(())
}

fn load_icon() -> egui::IconData {
    // Load icon from embedded bytes
    let icon_bytes = include_bytes!("../public/favicon.ico");
    
    // For now, return a default icon
    egui::IconData {
        rgba: vec![14, 184, 166, 255; 32 * 32], // Teal color
        width: 32,
        height: 32,
    }
}