# Automata Nexus Controller v3.0 - System Architecture

## Overview

The Automata Nexus Controller is a **100% native Rust application** with **DIRECT HARDWARE CONTROL** designed specifically for Raspberry Pi 5 running from NVMe SSD. It provides professional building automation control with real-time I/O management, pressure monitoring, vibration sensing, BMS protocol integration, and comprehensive HVAC control - all through REAL hardware interfaces with NO simulations.

## Key Design Principles

1. **Native Performance** - Pure Rust/egui with no web technologies or JavaScript runtime
2. **Hardware Optimized** - Built for ARM64 Cortex-A76 (RPi5) with SSD optimization
3. **Real-time Hardware Control** - Direct I/O via board commands (megabas, 16univin, etc.)
4. **Memory Safe** - Rust's ownership system prevents memory leaks
5. **Single Binary** - Everything compiles to one executable (~15-20MB)
6. **NO SIMULATIONS** - Every sensor read and output write goes to actual hardware

## System Architecture Diagram

```
┌──────────────────────────────────────────────────────────────┐
│                    Native egui UI Layer                      │
│  ┌─────────┬──────────┬────────────┬──────────┬─────────┐  │
│  │  Login  │ Dashboard│ I/O Control│ Vibration│ Weather │  │
│  └─────────┴──────────┴────────────┴──────────┴─────────┘  │
└──────────────────────────────────────────────────────────────┘
                              │
┌──────────────────────────────────────────────────────────────┐
│                     Application Core                         │
│  ┌──────────────────────────────────────────────────────┐  │
│  │              State Management (Arc<Mutex>)            │  │
│  └──────────────────────────────────────────────────────┘  │
│  ┌─────────┬────────────┬───────────┬──────────────────┐  │
│  │  Auth   │  Database  │  Config   │  Serial Manager  │  │
│  └─────────┴────────────┴───────────┴──────────────────┘  │
└──────────────────────────────────────────────────────────────┘
                              │
┌──────────────────────────────────────────────────────────────┐
│                    Hardware Abstraction                      │
│  ┌──────────┬────────────┬───────────┬─────────────────┐  │
│  │   I2C    │  Modbus    │  Serial   │    GPIO         │  │
│  │  (rppal) │(tokio-mod) │(serialport)│   (rppal)      │  │
│  └──────────┴────────────┴───────────┴─────────────────┘  │
└──────────────────────────────────────────────────────────────┘
                              │
┌──────────────────────────────────────────────────────────────┐
│                    Physical Hardware                         │
│  ┌──────────┬────────────┬───────────┬─────────────────┐  │
│  │ MegaBAS  │  16univin  │  16uout   │     Relay       │  │
│  │4 triacs  │ 16 inputs  │16 outputs │  8/16 relays    │  │
│  │4 analog  │  (ONLY)    │  (ONLY)   │    (ONLY)       │  │
│  │8 inputs  │            │           │                 │  │
│  └──────────┴────────────┴───────────┴─────────────────┘  │
│  ┌──────────┬────────────┬───────────┬─────────────────┐  │
│  │  P499    │  WTVB01    │  Modbus   │    BACnet       │  │
│  │Pressure  │ Vibration  │  Devices  │   Devices       │  │
│  │0.5-4.5V  │   RS485    │  TCP/RTU  │    IP/MSTP      │  │
│  └──────────┴────────────┴───────────┴─────────────────┘  │
└──────────────────────────────────────────────────────────────┘
```

## Component Details

### 1. User Interface Layer (egui) - ALL REAL HARDWARE
- **Technology**: egui - immediate mode native GUI (NO web tech)
- **Rendering**: OpenGL ES 2.0 (hardware accelerated on Pi5)
- **Color Scheme**: Teal/Cyan (#14b8a6 primary)
- **Components** (ALL with REAL hardware I/O):
  - Admin Panel - Executes REAL system commands via bash
  - BMS Integration - REAL Modbus/BACnet device communication
  - Board Configuration - Saves configurations to REAL SQLite database
  - Database Viewer - Executes REAL SQL queries
  - Firmware Manager - REAL git operations and board firmware updates
  - I/O Control Panel - REAL sensor reads via `megabas` commands
  - Live Monitor - REAL-time data from actual hardware
  - Logic Engine - REAL control logic with hardware writes
  - Processing Rules - REAL device discovery and point mapping
  - Refrigerant Diagnostics - REAL P499 pressure transducer readings

### 2. Core Services

#### Authentication (`auth.rs`)
- JWT token-based authentication
- Bcrypt password hashing
- Role-based access control (admin/operator)
- Session management

#### Database (`database.rs`)
- SQLite with SQLx async driver
- Connection pooling (5 connections)
- Automatic migrations
- Optimized for SSD (WAL mode, mmap)

#### Board Communication - REAL HARDWARE
- **Direct command execution** via board CLI tools
- **Supported Boards** (CORRECTLY CONFIGURED):
  - **MegaBAS** (0x48): 4 triacs (ON/OFF AC control), 4 analog outputs (0-10V), 8 configurable inputs
  - **16univin** (0x20-0x27): 16 universal INPUTS ONLY
  - **16uout** (0x40-0x47): 16 analog OUTPUTS ONLY  
  - **8relind** (0x38-0x3F): 8 relay OUTPUTS ONLY
  - **16relind** (0x38-0x3F): 16 relay OUTPUTS ONLY
- **Command Examples**:
  ```bash
  megabas 0 ain 1          # Read analog input 1
  megabas 0 aout 1 5.0     # Set analog output 1 to 5V
  megabas 0 triac 1 1      # Turn triac 1 ON
  16univin 0 in 1          # Read universal input 1
  16uout 0 out 1 7.5       # Set output 1 to 7.5V
  ```

#### Pressure Monitoring - P499 Transducers
- **REAL pressure readings** via analog inputs
- **P499 Specification**: 0.5-4.5V = 0-500 PSI
- **Voltage-to-PSI Conversion**: `PSI = ((voltage - 0.5) / 4.0) * 500`
- **Refrigerant Diagnostics**:
  - Suction/discharge pressure monitoring
  - Superheat/subcooling calculations
  - Compression ratio analysis
  - Efficiency scoring

#### Vibration Sensors - WTVB01-485
- **REAL Modbus RTU** communication
- RS485 over USB serial
- 3-axis vibration + velocity monitoring
- Temperature sensing
- ISO 10816-3 severity zones
- Direct sensor reads via Python scripts

#### BMS Protocols (`protocols.rs`)
- BACnet/IP device discovery
- Modbus TCP client
- Modbus RTU support
- Point mapping and scheduling
- COV (Change of Value) subscriptions

#### Weather Service (`weather.rs`)
- OpenWeatherMap API integration
- 10-minute cache
- Location-based updates
- Metric/Imperial units

### 3. State Management

#### Global State (`state.rs`)
- Thread-safe using Arc<Mutex<T>>
- DashMap for concurrent collections
- Event-driven updates
- Persistent configuration

### 4. Hardware Abstraction

#### I2C Interface
- Linux I2C device (/dev/i2c-1)
- rppal crate for Pi-specific features
- Automatic retry on failure
- Multi-master support

#### Serial/Modbus
- tokio-serial for async serial
- tokio-modbus for Modbus protocols
- USB serial port enumeration
- Automatic reconnection

#### GPIO
- rppal for direct GPIO access
- Interrupt-driven inputs
- PWM output support
- Hardware timing

## Data Flow - REAL HARDWARE

### 1. Startup Sequence
```
1. Load configuration from /etc/nexus/.env
2. Generate/load device serial number
3. Initialize SQLite database
4. Scan for connected boards (megabas, 16univin, etc.)
5. Launch egui UI window
6. Begin REAL hardware polling
```

### 2. Real-time Hardware Data Flow
```
Physical Sensor → Board ADC → CLI Command → Rust Process → UI Update
Example: P499 → MegaBAS ain → "megabas 0 ain 1" → parse voltage → calculate PSI → display

User Action → UI Event → CLI Command → Board DAC → Physical Output
Example: Toggle ON → triac switch → "megabas 0 triac 1 1" → TRIAC fires → AC load ON
```

### 3. Network Communication
```
Local Device → Cloudflare Tunnel → Internet
                    ↓
            nexuscontroller-{serial}.automatacontrols.com
```

## Real Hardware Implementation Details

### UI Module Hardware Integration

Each UI module directly interfaces with hardware:

| UI Module | Hardware Interface | Real Implementation |
|-----------|-------------------|-------------------|
| admin_panel | System commands | `std::process::Command::new("bash")` |
| bms_complete | Modbus/BACnet | `python3 /opt/nexus/modbus_test.py` |
| board_config | Board scanning | `test -d /dev/i2c-1 && i2cdetect -y 1` |
| database | SQLite | `sqlite3 /var/lib/nexus/nexus.db` |
| firmware | Git/Make | `git pull && sudo make install` |
| io_control | Sensor I/O | `megabas 0 ain 1` for reads, `aout` for writes |
| live_monitor | Real-time data | Continuous hardware polling |
| logic_engine | Control logic | Read sensors → Calculate → Write outputs |
| processing | Device discovery | Modbus scan, BACnet discovery |
| refrigerant | P499 pressure | Voltage read → PSI conversion |

### Hardware Command Examples

```rust
// Reading analog input (0-10V)
let result = std::process::Command::new("megabas")
    .args(&["0", "ain", "1"])
    .output();
let voltage = parse_output(result);

// Writing analog output
std::process::Command::new("megabas")
    .args(&["0", "aout", "1", "5.0"])
    .execute();

// Controlling TRIAC (AC loads - ON/OFF only)
std::process::Command::new("megabas")
    .args(&["0", "triac", "1", "1"])  // ON
    .execute();
std::process::Command::new("megabas")
    .args(&["0", "triac", "1", "0"])  // OFF
    .execute();

// Reading P499 pressure transducer
let voltage = read_analog_input(channel);
let psi = ((voltage - 0.5) / 4.0) * 500.0;  // 0.5-4.5V = 0-500 PSI
```

## Installation Process

### Phase 1: System Preparation (5 min)
1. Check RPi5 hardware
2. Verify NVMe SSD presence
3. Generate device fingerprint
4. Create unique serial number

### Phase 2: Dependencies (10 min)
1. System packages (apt-get)
   - libgtk-3-dev (UI framework)
   - libwebkit2gtk-4.1-dev (optional web view)
   - i2c-tools, libudev-dev
   - redis-server (caching)
   - cloudflared (tunnel)
2. Rust toolchain (if missing)
3. Board drivers (Sequent Microsystems)

### Phase 3: Build Process (15 min)
1. Compile Rust code with optimizations
   - Target: aarch64-unknown-linux-gnu
   - CPU: cortex-a76
   - LTO enabled, single codegen unit
2. Strip debug symbols
3. Create single binary (~15-20MB)

### Phase 4: Configuration (5 min)
1. Generate security keys (JWT, encryption)
2. Setup Cloudflare tunnel
3. Configure systemd service
4. Initialize database
5. Setup monitoring

### Phase 5: Verification (2 min)
1. Test I2C communication
2. Scan for boards
3. Check network connectivity
4. Verify service status

## Performance Optimizations

### SSD Optimizations
- SQLite: WAL mode, 1GB mmap, 4KB pages
- Redis: LRU cache, 1GB max memory
- Logs: Async writes, rotation
- noatime mount option

### CPU Optimizations
- Release build with -O3
- Link-time optimization (LTO)
- ARM NEON SIMD where applicable
- Target Cortex-A76 specifically

### Memory Management
- Pre-allocated buffers for I/O
- Connection pooling
- Lazy loading of resources
- 100MB SQLite cache

## Security Features

1. **Authentication**
   - JWT tokens (24hr expiry)
   - Bcrypt password hashing
   - Session management

2. **Encryption**
   - AES-256-GCM for sensitive data
   - TLS for external connections
   - Encrypted configuration

3. **Access Control**
   - Role-based permissions
   - API rate limiting
   - Audit logging

4. **Network Security**
   - Cloudflare tunnel (no open ports)
   - Local-only Redis/database
   - Firewall rules

## Monitoring & Diagnostics

### System Metrics
- CPU usage and temperature
- Memory utilization
- Disk I/O statistics
- Network throughput

### Application Metrics
- Request latency
- Database query time
- I/O operation success rate
- Sensor reading frequency

### Health Checks
- Service status (systemd)
- Board connectivity
- Database integrity
- Network connectivity

## Troubleshooting Guide

### Common Issues

1. **I2C Communication Failure**
   ```bash
   # Check I2C is enabled
   sudo raspi-config nonint do_i2c 0
   # Scan I2C bus
   i2cdetect -y 1
   ```

2. **Database Lock**
   ```bash
   # Stop service
   sudo systemctl stop nexus
   # Remove lock
   rm /var/lib/nexus/nexus.db-wal
   # Restart
   sudo systemctl start nexus
   ```

3. **Serial Port Access**
   ```bash
   # Add user to dialout group
   sudo usermod -a -G dialout $USER
   # List ports
   ls -la /dev/ttyUSB*
   ```

4. **Build Failures**
   ```bash
   # Clear cargo cache
   cargo clean
   # Update dependencies
   cargo update
   # Rebuild
   cargo build --release
   ```

## Directory Structure

```
/opt/nexus/                 # Installation directory
├── nexus-controller        # Main executable
├── config/                 # Configuration files
├── firmware/              # Board firmware
└── public/                # Assets (logo, icons)

/etc/nexus/                # System configuration
├── .env                   # Environment variables
├── serial                 # Device serial number
└── cloudflared.yml       # Tunnel configuration

/var/lib/nexus/           # Data directory
├── nexus.db              # SQLite database
└── cache/                # Application cache

/var/log/nexus/           # Log files
└── nexus.log             # Application logs

/var/backups/nexus/       # Backup directory
└── *.backup              # Database backups
```

## Development Notes

### Version 3.0 Changes - REAL HARDWARE ONLY
- **REMOVED**: All `rand::random()` calls for simulated data
- **REMOVED**: All mock/fake sensor readings
- **ADDED**: Direct hardware command execution
- **ADDED**: Real P499 pressure transducer support
- **FIXED**: Board capabilities match actual hardware specs
- **FIXED**: MegaBAS has 4 triacs (NOT relays)
- **FIXED**: 16univin is INPUTS ONLY
- **FIXED**: 16uout is OUTPUTS ONLY

### Building for Development
```bash
cargo build
RUST_LOG=debug ./target/debug/nexus-controller
```

### Building for Production  
```bash
cargo build --release --target aarch64-unknown-linux-gnu
strip target/aarch64-unknown-linux-gnu/release/nexus-controller
```

### Testing Hardware
```bash
# Test board communication
megabas 0 board
16univin 0 board

# Test sensor reading
megabas 0 ain 1-8

# Test output control
megabas 0 aout 1 5.0
megabas 0 triac 1 1  # ON
megabas 0 triac 1 0  # OFF
```

## License

Commercial License - Copyright (c) 2025 Automata Controls
Developed by Andrew Jewell Sr.

This software is proprietary and confidential. Unauthorized copying, modification, or distribution is strictly prohibited.