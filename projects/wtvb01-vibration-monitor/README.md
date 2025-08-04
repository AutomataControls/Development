# WTVB01-485 Vibration Monitor for Raspberry Pi

Professional industrial vibration monitoring system using WIT-Motion WTVB01-485 sensors.

## ✨ Features

- **High-Performance Communication**: 230400 baud (24x faster than factory)
- **Burst Reading**: All 19 registers in one command (<3ms)
- **1000Hz Sampling**: High-speed mode for critical monitoring
- **ISO 10816-3 Compliant**: Professional vibration analysis
- **Beautiful UI**: Glassmorphism design optimized for Pi

## 🚀 Quick Start

```bash
# On Raspberry Pi
cd src-tauri
cargo build --release
./target/release/wtvb01-vibration-monitor
```

Access UI at: `http://raspberrypi.local:1420`

## 📁 Project Structure

```
wtvb01-vibration-monitor/
├── src-tauri/          # Rust backend
│   └── src/
│       ├── main.rs     # Tauri application
│       └── wtvb01_sensor.rs # WTVB01 driver
├── src/                # Frontend UI
│   └── index.html      # Glassmorphism interface
├── scripts/            # Utility scripts
│   ├── deploy-to-pi.sh
│   └── test-speed.sh
└── README.md
```

## 🔧 Installation

See [INSTALL.md](./INSTALL.md) for detailed setup instructions.

## 📊 Performance

| Mode | Baud Rate | Read Time | Improvement |
|------|-----------|-----------|-------------|
| Factory | 9600 | 120ms | Baseline |
| Optimized | 115200 | 10ms | 12x faster |
| Maximum | 230400 | 5ms | 24x faster |
| Burst | 230400 | <3ms | 40x faster |

## 🎯 Key Commands

- **Burst Read**: Read all 19 registers at once
- **Optimize Speed**: Set sensors to 230400 baud
- **Enable 1000Hz**: Activate high-speed sampling

## 📝 License

© 2025 AutomataControls. Commercial license required.