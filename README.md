# WTVB01-485 Vibration Monitor for Raspberry Pi

Professional Industrial Vibration Monitoring System using WIT-Motion WTVB01-485 sensors with Tauri framework optimized for Raspberry Pi.

## 🚀 Features

- **High-Performance Sensor Communication**
  - Burst reading mode (all 19 registers in one command)
  - Optimized 230400 baud rate (24x faster than factory default)
  - 1000Hz high-speed sampling mode
  - Auto-baud detection (9600 to 230400)

- **ISO 10816-3 Compliance**
  - Real-time vibration analysis
  - Zone classification (A/B/C/D)
  - Alert level monitoring
  - FFT frequency analysis

- **Beautiful Glassmorphism UI**
  - Smooth tab animations
  - Real-time sensor displays
  - Responsive design optimized for Pi
  - WebKit-compatible styling

- **Raspberry Pi Optimized**
  - ARM binary compilation
  - Direct USB-RS485 support
  - Low resource usage
  - Systemd service integration

## 📋 Requirements

- Raspberry Pi 3/4/5
- WIT-Motion WTVB01-485 sensors
- USB-to-RS485 adapters
- Rust 1.70+ 
- Node.js 16+

## 🔧 Quick Start

### 1. Clone the repository
```bash
git clone https://github.com/yourusername/Development.git
cd Development/vibration-monitor-rpi
```

### 2. Install on Raspberry Pi
```bash
# Add user to dialout group for USB access
sudo usermod -a -G dialout $USER
# Logout and login again

# Build the application
cd src-tauri
cargo build --release

# Run the monitor
./target/release/wtvb01-vibration-monitor
```

### 3. Access the UI
Open browser to: `http://raspberrypi.local:1420`

## 🎯 Performance Optimization

### Burst Reading Mode
Reads all 19 sensor registers in a single Modbus command (<50ms):
```rust
sensor_manager.read_sensor_burst("/dev/ttyUSB0")
```

### Speed Optimization
Configure sensors for maximum 230400 baud rate:
```rust
sensor_manager.optimize_for_speed("/dev/ttyUSB0")
```

### High-Speed Mode (1000Hz)
Enable 1ms sampling interval:
```rust
sensor_manager.enable_high_speed_mode("/dev/ttyUSB0", 0x50)
```

## 📊 Sensor Specifications

- **Model**: WIT-Motion WTVB01-485
- **Protocol**: Modbus RTU over RS485
- **Default Address**: 0x50
- **Baud Rates**: 4800, 9600, 19200, 38400, 57600, 115200, 230400
- **Measurement Range**:
  - Acceleration: ±16g
  - Velocity: 0-1000 mm/s
  - Temperature: -40°C to +85°C
  - Frequency: 0-1000 Hz

## 📁 Project Structure

```
vibration-monitor-rpi/
├── src-tauri/              # Rust backend
│   ├── src/
│   │   ├── main.rs         # Tauri application entry
│   │   └── wtvb01_sensor.rs # WTVB01-485 driver
│   ├── Cargo.toml          # Rust dependencies
│   └── tauri.conf.json     # Tauri configuration
├── src/                    # Frontend UI
│   ├── index.html          # Main UI with glassmorphism
│   └── assets/             # Images and icons
├── scripts/                # Deployment and utilities
│   ├── deploy-to-pi.sh     # Automated deployment
│   ├── test-speed.sh       # Sensor speed testing
│   └── install-service.sh  # Systemd service setup
├── docs/                   # Documentation
│   └── WTVB01-Manual.pdf   # Sensor manual
└── README.md
```

## 🛠️ Development

### Building for Raspberry Pi (from development machine)
```bash
# Install ARM target
rustup target add armv7-unknown-linux-gnueabihf

# Cross-compile
cargo build --release --target armv7-unknown-linux-gnueabihf

# Deploy to Pi
./scripts/deploy-to-pi.sh pi@raspberrypi.local
```

### Running Tests
```bash
# Test sensor communication speeds
./scripts/test-speed.sh

# Run unit tests
cargo test
```

## 📈 Performance Benchmarks

| Baud Rate | Read Time | Improvement |
|-----------|-----------|-------------|
| 9600 (factory) | 120ms | Baseline |
| 115200 (optimized) | 10ms | 12x faster |
| 230400 (maximum) | 5ms | 24x faster |
| 230400 + Burst | <3ms | 40x faster |

## 🔐 License

© 2025 AutomataNexus AI & AutomataControls. All rights reserved.

Commercial license required for production use. Contact: DevOps@automatacontrols.com

## 🤝 Support

- **Email**: support@automatacontrols.com
- **Documentation**: [automatanexus.com/docs](https://automatanexus.com)
- **Issues**: [GitHub Issues](https://github.com/yourusername/Development/issues)

## 🏆 Credits

Developed by AutomataNexus AI for industrial vibration monitoring applications.