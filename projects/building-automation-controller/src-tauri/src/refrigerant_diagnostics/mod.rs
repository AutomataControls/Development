// Refrigerant Diagnostics Module
// Professional HVAC/Refrigeration diagnostic system integration

pub mod refrigerants;
pub mod p499_transducer;
pub mod diagnostics;

use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};

pub use refrigerants::{RefrigerantDatabase, PressureTemperatureData};
pub use p499_transducer::{P499Interface, P499Configuration, TransducerReading};
pub use diagnostics::{DiagnosticEngine, SystemConfiguration, DiagnosticReading, DiagnosticResult};

// Re-export common types for easier access
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefrigerantInfo {
    pub designation: String,
    pub chemical_name: String,
    pub safety_class: String,
    pub gwp: i32,
    pub odp: f32,
    pub critical_temp_f: f32,
    pub boiling_point_f: f32,
    pub applications: Vec<String>,
}

pub struct RefrigerantDiagnosticsManager {
    pub refrigerant_db: Arc<RefrigerantDatabase>,
    pub p499_interface: Arc<Mutex<P499Interface>>,
    pub diagnostic_engine: Arc<DiagnosticEngine>,
}

impl RefrigerantDiagnosticsManager {
    pub fn new() -> Self {
        let refrigerant_db = Arc::new(RefrigerantDatabase::new());
        let diagnostic_engine = Arc::new(DiagnosticEngine::new(refrigerant_db.clone()));
        
        RefrigerantDiagnosticsManager {
            refrigerant_db,
            p499_interface: Arc::new(Mutex::new(P499Interface::new())),
            diagnostic_engine,
        }
    }
}