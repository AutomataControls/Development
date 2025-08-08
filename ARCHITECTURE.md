# Automata Nexus AI Controller - System Architecture

## Overview

The Automata Nexus AI Controller is a **100% native Rust application** designed specifically for Raspberry Pi 5 running from NVMe SSD. It provides professional building automation control with real-time I/O management, vibration monitoring, BMS protocol integration, and weather services.

## Key Design Principles

1. **Native Performance** - Pure Rust with no web technologies or JavaScript runtime
2. **Hardware Optimized** - Built for ARM64 Cortex-A76 (RPi5) with SSD optimization
3. **Real-time Capable** - Direct hardware access via Linux embedded HAL
4. **Memory Safe** - Rust's ownership system prevents memory leaks
5. **Single Binary** - Everything compiles to one executable (~15-20MB)

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
│  │ Megabas  │  8-Relay   │  WTVB01   │  BMS Devices    │  │
│  │  Board   │   Board    │  Sensors  │  (BACnet/Modbus)│  │
│  └──────────┴────────────┴───────────┴─────────────────┘  │
└──────────────────────────────────────────────────────────────┘
```

## Component Details

### 1. User Interface Layer (egui)
- **Technology**: egui - immediate mode native GUI
- **Rendering**: OpenGL ES 2.0 (hardware accelerated on Pi5)
- **Color Scheme**: Teal/Cyan (#14b8a6 primary)
- **Components**:
  - Login Screen with JWT authentication
  - Dashboard with real-time metrics
  - I/O Control Panel for board management
  - Vibration Monitor with ISO 10816-3 compliance
  - BMS Protocol manager
  - Weather display
  - Settings panel

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

#### Board Communication (`boards.rs`)
- I2C communication via rppal crate
- Support for:
  - Megabas I/O Board (0x48)
  - 8-Relay boards (0x38-0x3F)
  - 8-Input boards (0x20-0x27)
  - Building Automation (virtual)
- Real-time state synchronization
- Background polling at 1Hz

#### Vibration Sensors (`sensors.rs`)
- WTVB01-485 Modbus RTU protocol
- RS485 over USB serial
- 3-axis vibration + velocity monitoring
- Temperature sensing
- ISO 10816-3 severity zones
- Calibration support (zero-point, sensitivity)

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

## Data Flow

### 1. Startup Sequence
```
1. Load configuration from /etc/nexus/.env
2. Generate/load device serial number
3. Initialize SQLite database
4. Start background services (tokio tasks)
5. Launch egui UI window
6. Display login screen
```

### 2. Real-time Data Flow
```
Hardware → HAL Driver → Background Task → State Update → UI Refresh
   ↑                                           ↓
   └──────────── User Command ←────────────────┘
```

### 3. Network Communication
```
Local Device → Cloudflare Tunnel → Internet
                    ↓
            nexuscontroller-{serial}.automatacontrols.com
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

### Cross-Compilation (from x86_64)
```bash
# Install cross-compilation tools
cargo install cross
# Build for Pi5
cross build --release --target aarch64-unknown-linux-gnu
```

## License

Commercial License - Copyright (c) 2025 Automata Controls
Developed by Andrew Jewell Sr.

This software is proprietary and confidential. Unauthorized copying, modification, or distribution is strictly prohibited.