# Automata Nexus AI - Rust/Tauri Controller

Professional Building Automation Control System for Raspberry Pi 5

## Features

- **Full Rust/Tauri Implementation** - Native performance optimized for ARM64
- **Raspberry Pi 5 Optimized** - Built specifically for RPi5 with Bookworm OS
- **SSD Boot Support** - Complete SSD installation and boot configuration
- **Real-time I/O Control** - Support for Megabas, 8-relay, 8-inputs boards
- **Vibration Monitoring** - WTVB01-485 sensor integration with ISO 10816-3
- **BMS Integration** - BACnet, Modbus TCP/RTU, KNX protocols
- **Weather Integration** - Real-time weather data display
- **Building Automation** - 4 Triacs + 4 Analog outputs for HVAC control
- **Secure Authentication** - JWT-based auth with role management
- **Database Storage** - SQLite with automatic backups
- **Web Interface** - Modern responsive UI with Tauri

## System Requirements

- Raspberry Pi 5 (4GB+ RAM recommended)
- Raspberry Pi OS Bookworm 64-bit
- NVMe or USB SSD (minimum 128GB)
- Internet connection for initial setup

## Quick Installation

```bash
# Download installer
wget https://github.com/automata-nexus/controller/releases/latest/download/install-nexus-rpi5-ssd.sh

# Make executable
chmod +x install-nexus-rpi5-ssd.sh

# Run installer (as root)
sudo ./install-nexus-rpi5-ssd.sh
```

## Manual Build

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Add ARM64 target
rustup target add aarch64-unknown-linux-gnu

# Install dependencies
sudo apt-get update
sudo apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    libsqlite3-dev \
    libwebkit2gtk-4.1-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    i2c-tools
```

### Build

```bash
# Clone repository
git clone https://github.com/automata-nexus/controller.git
cd controller

# Build for ARM64
cargo build --release --target aarch64-unknown-linux-gnu

# Install
sudo cp target/aarch64-unknown-linux-gnu/release/nexus-controller /usr/local/bin/
```

## Configuration

Configuration files are stored in `/etc/nexus/`:

- `config.json` - Main configuration
- `boards.json` - Board configurations
- `sensors.json` - Sensor configurations
- `protocols.json` - Protocol settings

## Hardware Support

### I/O Boards
- Sequent Microsystems Megabas
- Sequent Microsystems 8-relay
- Sequent Microsystems 8-inputs
- Sequent Microsystems RTD
- Sequent Microsystems 4-20mA

### Vibration Sensors
- WTVB01-485 (Modbus RTU)
- ISO 10816-3 compliance

### Protocols
- BACnet/IP
- Modbus TCP
- Modbus RTU
- KNX (planned)

## API Endpoints

The application provides Tauri commands for all functionality:

### Authentication
- `login(username, password)`
- `logout()`
- `get_current_user(token)`
- `update_password(token, old_password, new_password)`

### Boards
- `scan_boards()`
- `get_board_state(board_id)`
- `set_relay(board_id, relay_num, value)`
- `set_dimmer(board_id, triac_num, value)`
- `set_analog_output(board_id, channel, value)`

### Sensors
- `scan_vibration_sensors()`
- `add_vibration_sensor(port, name, location, modbus_address)`
- `calibrate_sensor(port, calibration)`
- `get_sensor_readings(port)`

### Protocols
- `scan_bacnet_devices()`
- `scan_modbus_devices()`
- `read_modbus_registers(device, slave_id, start, count)`

### System
- `get_system_info()`
- `get_logs(level, module, limit)`
- `run_diagnostics()`
- `backup_config()`
- `restore_config(backup_file)`

## Web Interface

Access the web interface at:
```
http://<raspberry-pi-ip>
```

Default credentials:
- Username: `admin`
- Password: `Nexus`

## Service Management

```bash
# Start service
sudo systemctl start nexus

# Stop service
sudo systemctl stop nexus

# Restart service
sudo systemctl restart nexus

# View logs
sudo journalctl -u nexus -f
```

## Backup & Restore

```bash
# Create backup
nexus-controller --backup

# Restore from backup
nexus-controller --restore /path/to/backup.json
```

## Development

### Project Structure
```
Rust-SSD-Nexus-Controller/
├── src/
│   ├── main.rs          # Application entry point
│   ├── api.rs           # System API endpoints
│   ├── auth.rs          # Authentication module
│   ├── boards.rs        # Board communication
│   ├── sensors.rs       # Vibration sensors
│   ├── protocols.rs     # BMS protocols
│   ├── weather.rs       # Weather service
│   ├── database.rs      # Database operations
│   ├── state.rs         # Application state
│   ├── config.rs        # Configuration management
│   └── utils.rs         # Utility functions
├── migrations/          # Database migrations
├── installer/           # Installation scripts
├── firmware/           # Board firmware files
├── Cargo.toml          # Rust dependencies
└── tauri.conf.json     # Tauri configuration
```

### Building for Development

```bash
# Development build with hot reload
cargo tauri dev

# Production build
cargo tauri build
```

## Troubleshooting

### I2C Issues
```bash
# Enable I2C
sudo raspi-config nonint do_i2c 0

# Test I2C
i2cdetect -y 1
```

### Permission Issues
```bash
# Add user to required groups
sudo usermod -a -G i2c,spi,gpio,dialout $USER
```

### Database Issues
```bash
# Reset database
sudo rm /var/lib/nexus/nexus.db
sudo nexus-controller --init-db
```

## Security

- All passwords are bcrypt hashed
- JWT tokens expire after 24 hours
- HTTPS support via nginx reverse proxy
- Systemd security hardening enabled

## License

Copyright (c) 2025 Automata Controls
Developed by Andrew Jewell Sr.

## Support

For support, please contact:
- Email: support@automata.ai
- GitHub: https://github.com/automata-nexus/controller/issues

## Credits

- Tauri Framework
- Sequent Microsystems
- Rust Community
- Raspberry Pi Foundation