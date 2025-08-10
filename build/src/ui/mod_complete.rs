// Complete UI Module Exports
// This file should replace or be integrated with the existing mod.rs

// Core UI modules
pub mod login;
pub mod dashboard;
pub mod io_control;
pub mod vibration;
pub mod weather;
pub mod settings;

// COMPLETE UI implementations (all 13 modules)
pub mod io_control_complete;
pub mod admin_complete;
pub mod live_monitor_complete;
pub mod vibration_complete;
pub mod refrigerant_complete;
pub mod database_complete;
pub mod board_config_complete;
pub mod logic_engine_complete;
pub mod firmware_complete;
pub mod bms_complete;
pub mod processing_complete;
pub mod metrics_complete;
pub mod maintenance_complete;

// Integration module
pub mod complete_ui_integration;

// Re-export the main integration
pub use complete_ui_integration::CompleteUI;

// Re-export individual complete modules for direct access if needed
pub use io_control_complete::IOControlComplete;
pub use admin_complete::AdminPanelComplete;
pub use live_monitor_complete::LiveMonitor;
pub use vibration_complete::VibrationMonitor as VibrationComplete;
pub use refrigerant_complete::RefrigerantDiagnostics;
pub use database_complete::DatabaseViewer;
pub use board_config_complete::BoardConfiguration;
pub use logic_engine_complete::LogicEngine;
pub use firmware_complete::FirmwareManager;
pub use bms_complete::BmsIntegration;
pub use processing_complete::ProtocolManager;
pub use metrics_complete::MetricsVisualization;
pub use maintenance_complete::MaintenanceManager;