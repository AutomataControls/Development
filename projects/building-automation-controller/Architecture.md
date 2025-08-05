# Automata Nexus Automation Control Center - Architecture

## Overview

The Automata Nexus Automation Control Center is a professional building automation system designed for Raspberry Pi, built using Tauri (Rust + Next.js) to compete with industry leaders like Niagara, Automated Logic, and Johnson Controls.

## System Architecture

### Technology Stack

- **Frontend**: Next.js 14 with TypeScript
- **UI Components**: shadcn/ui (Radix UI + Tailwind CSS)
- **Backend**: Rust with Tauri v1.5
- **Database**: SQLite with SQLx (7-day retention)
- **BMS Integration**: InfluxDB v3 (SQL queries)
- **Hardware**: Sequent Microsystems boards via Python bindings
- **Logic Engine**: JavaScript (user-configurable)

### Directory Structure

```
building-automation-controller/
├── app/                      # Next.js application
│   ├── page.tsx             # Main dashboard with tabs
│   ├── globals.css          # Global styles
│   └── layout.tsx           # Root layout
├── components/              
│   ├── ui/                  # shadcn/ui components
│   ├── dashboard.tsx        # Main dashboard component
│   ├── metrics-chart.tsx    # Real-time trend visualization
│   ├── io-control.tsx       # Universal I/O control
│   ├── relay-control.tsx    # Relay management
│   ├── analog-control.tsx   # Analog I/O control
│   ├── maintenance-mode.tsx # Time-limited manual control
│   ├── bms-integration.tsx  # BMS command interface
│   ├── logic-engine.tsx     # JavaScript logic editor
│   └── diagnostic-panel.tsx # System diagnostics
├── lib/
│   ├── utils.ts            # Utility functions
│   └── hooks.ts            # React hooks
├── src-tauri/              # Rust backend
│   ├── src/
│   │   ├── main.rs         # Application entry point
│   │   ├── hardware.rs     # Hardware interface
│   │   ├── database.rs     # SQLite operations
│   │   ├── bms.rs          # BMS integration
│   │   ├── logic_engine.rs # JavaScript executor
│   │   └── api.rs          # Tauri command handlers
│   └── Cargo.toml          # Rust dependencies
└── installer/              # Installation scripts

```

## Core Components

### 1. Hardware Interface Layer

The system interfaces with Sequent Microsystems boards through Python bindings:

- **MegaBAS HAT**: Primary interface board
- **16 Universal Input**: 0-10V analog inputs (16 channels)
- **16 Analog Output**: 0-10V outputs (16 channels)
- **8/16 Relay**: Digital relay control

**Communication Flow**:
```
Rust Backend → Python Sidecar → I2C Bus → Sequent Boards
```

### 2. Data Management

#### Local Metrics Database (SQLite)
- Stores all sensor readings with timestamps
- 7-day automatic retention policy
- Optimized for time-series queries
- Schema:
  ```sql
  CREATE TABLE metrics (
    id TEXT PRIMARY KEY,
    sensor_name TEXT NOT NULL,
    value REAL NOT NULL,
    unit TEXT NOT NULL,
    timestamp INTEGER NOT NULL,
    created_at INTEGER NOT NULL
  );
  ```

#### BMS Integration (InfluxDB)
- Fetches commands via SQL queries
- Falls back to local logic on connection failure
- Command structure:
  ```typescript
  interface ProcessingEngineCommand {
    equipment: string
    command: string
    value: number
    priority: number
  }
  ```

### 3. Logic Engine

JavaScript-based logic execution with:
- User-defined control logic
- Access to all I/O points
- BMS command integration
- Local fallback capability
- Maintenance mode override

**Execution Context**:
```javascript
// Available in user scripts:
const inputs = await getUniversalInputs()
const outputs = await getAnalogOutputs()
const relays = await getRelays()
const bmsCommands = await getBMSCommands()
```

### 4. Frontend Architecture

#### Component Hierarchy
```
App
├── Dashboard
│   ├── IOControl
│   │   ├── UniversalInputs
│   │   ├── AnalogOutputs
│   │   └── RelayControl
│   ├── MetricsVisualization
│   │   ├── TrendChart
│   │   └── DataTable
│   ├── LogicEngine
│   │   ├── CodeEditor
│   │   └── StatusPanel
│   └── MaintenanceMode
│       └── CountdownTimer
```

#### State Management
- React hooks for local state
- Tauri commands for backend communication
- Real-time updates via polling (1s intervals)

### 5. Communication Protocol

#### Tauri IPC Commands
```rust
#[tauri::command]
async fn get_universal_inputs() -> Result<Vec<InputReading>, String>

#[tauri::command]
async fn set_analog_output(channel: u8, value: f32) -> Result<(), String>

#[tauri::command]
async fn toggle_relay(board: u8, relay: u8) -> Result<(), String>

#[tauri::command]
async fn execute_logic(script: String) -> Result<LogicResult, String>
```

### 6. Security & Permissions

- Runs as dedicated user (Automata)
- Systemd service with restart policy
- I2C group permissions for hardware access
- API authentication for BMS integration
- Maintenance mode time limits (2 hours max)

## Data Flow

### Normal Operation
1. Hardware boards read sensors (1s interval)
2. Backend stores readings in SQLite
3. BMS commands fetched (30s interval)
4. Logic engine processes inputs + commands
5. Outputs written to hardware
6. Frontend updates display

### Maintenance Mode
1. User activates maintenance mode
2. 2-hour countdown starts
3. Logic engine suspended
4. Manual control enabled
5. All changes logged
6. Auto-revert on timeout

## Scaling Considerations

### Hardware Scaling
- Independent stack addressing (0-7) for each board type
- Board addressing scheme:
  - MegaBAS HAT: Stack 0-7 (ID: megabas_0 to megabas_7)
  - SM16univin: Stack 0-7 (ID: 16univin_0 to 16univin_7)
  - SM16uout: Stack 0-7 (ID: 16uout_0 to 16uout_7)
  - SM16relind: Stack 0-7 (ID: 16relay_0 to 16relay_7)
  - SM8relind: Stack 0-7 (ID: 8relay_0 to 8relay_7)
- Maximum I/O points:
  - 64 universal inputs from MegaBAS + 128 from 16univin = 192 total
  - 32 analog outputs from MegaBAS + 128 from 16uout = 160 total
  - 64 relays from 8relind + 128 from 16relind = 192 total relay outputs
  - 32 triac outputs from MegaBAS boards

### RS485 Vibration Monitoring (Optional)
- WIT-Motion WTVB01-485 sensors via Modbus RTU
- High-speed vibration monitoring (up to 1000Hz)
- ISO 10816-3 compliance for equipment health monitoring
- Features:
  - RMS velocity measurement (mm/s)
  - Temperature monitoring
  - Automatic alert thresholds
  - Equipment classification (A/B/C/D zones)

### Performance Optimization
- Rust backend for speed
- SQLite with proper indexing
- Batch I2C operations
- Frontend virtualization for large datasets
- Compiled JavaScript logic

## Integration Points

### Current Integrations
- InfluxDB v3 for BMS commands
- JavaScript for custom logic
- Web UI at http://localhost:1420

### Future Integration Options
- Modbus TCP/RTU
- BACnet IP/MS/TP
- MQTT broker
- REST API endpoints
- WebSocket real-time updates

## Deployment

### System Requirements
- Raspberry Pi 4/5 (8GB recommended)
- 64-bit Raspberry Pi OS
- 32GB+ storage
- Network connectivity

### Installation Process
1. System updates & timezone
2. I2C enablement
3. Dependency installation
4. Cross-compilation for ARM64
5. Systemd service setup
6. Permission configuration

### Service Management
```bash
# Start service
sudo systemctl start automata-nexus

# View logs
sudo journalctl -u automata-nexus -f

# Check status
sudo systemctl status automata-nexus
```

## Error Handling

### Hardware Failures
- Graceful degradation
- Error logging
- Fallback to safe states
- Alert generation

### Network Failures
- Local logic takeover
- Command caching
- Automatic reconnection
- Status indicators

### Logic Errors
- JavaScript sandboxing
- Error boundaries
- Execution timeouts
- Debug logging

## Maintenance

### Database Maintenance
- Automatic 7-day cleanup
- Vacuum operations
- Index optimization
- Backup procedures

### System Updates
- Git-based deployment
- Hot-reload development
- Staged rollouts
- Rollback capability

## Performance Metrics

### Target Specifications
- 1s sensor update rate
- <100ms UI response
- <500ms logic execution
- 99.9% uptime target
- 7-day data retention