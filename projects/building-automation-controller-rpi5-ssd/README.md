# Automata Nexus Building Automation Controller

<div align="center">

![Version](https://img.shields.io/badge/version-1.0.0-blue.svg)
![License](https://img.shields.io/badge/license-Commercial-red.svg)
![Platform](https://img.shields.io/badge/platform-Raspberry%20Pi-green.svg)
![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)
![Node](https://img.shields.io/badge/node-18%2B-green.svg)
![Tauri](https://img.shields.io/badge/Tauri-1.5-blue.svg)

![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)
![ASHRAE](https://img.shields.io/badge/ASHRAE-207--2021-blue.svg)
![ISO](https://img.shields.io/badge/ISO-10816--3-green.svg)
![BACnet Ready](https://img.shields.io/badge/BACnet-Ready-orange.svg)
![Modbus](https://img.shields.io/badge/Modbus-RTU%2FTCP-blue.svg)

**Professional Building Automation System for Raspberry Pi**

[Features](#features) • [Hardware](#hardware-requirements) • [Installation](#installation) • [Documentation](#documentation) • [License](#license)

</div>

---

## 🏢 Overview

A comprehensive Tauri-based control system competing with industry leaders like Niagara, Automated Logic, Delta Controls, and Johnson Controls. Built specifically for Sequent Microsystems building automation hardware on Raspberry Pi.

## ✨ Features

### 🔧 Hardware Control
- **MegaBAS HAT Control**
  - 8x Universal inputs (software configurable):
    - 0-10V analog input (default)
    - 1K thermistor or dry contact
    - 10K Type 2 thermistor with Steinhart-Hart equation
  - 4x 0-10V analog outputs
  - 4x Triac outputs
  - 4x Dry contact inputs with counters
  - Onboard sensors (power supply, temperature)
  - RTC and watchdog timer

- **Expansion Board Support**
  - 16-channel relay boards
  - 16-channel universal input boards
  - 16-channel analog output boards
  - 8-channel relay boards

### 📊 Advanced Monitoring
- **Vibration Monitoring (Optional)**
  - WIT-Motion WTVB01-485 sensors via RS485/Modbus RTU
  - ISO 10816-3 compliant vibration analysis
  - Real-time equipment health monitoring
  - Automatic alert thresholds

- **HVAC Refrigerant Diagnostics (Optional)**
  - P499 pressure transducer support (0-10V models)
  - 100+ refrigerant database (R-410A, R-454B, R-32, etc.)
  - Superheat/subcooling calculations
  - Real-time fault detection and diagnostics
  - ASHRAE 207-2021 compliant

### 🌐 Integration & Connectivity
- **Data Integration**
  - BMS metric sender with InfluxDB line protocol
  - Processing validation proxy for enhanced data validation
  - Configurable equipment types and zones
  - Real-time monitoring and control

### 💻 Software Features
- **Logic Engine**
  - JavaScript-based control logic
  - BMS command integration with local fallback
  - Real-time execution with state management
  
- **Database & Analytics**
  - SQLite metrics storage with 7-day retention
  - Real-time trend visualization
  - Historical data analysis
  
- **User Interface**
  - Modern web-based UI (Tauri + Next.js)
  - Real-time data updates
  - Maintenance mode with 2-hour timer
  - Professional glassmorphism design

## 📋 Requirements

### Hardware
- Raspberry Pi 3/4/5 (8GB recommended)
- Sequent Microsystems MegaBAS HAT
- Optional expansion boards (16-relay, 16-input, etc.)
- Optional P499 pressure transducers
- Optional WTVB01-485 vibration sensors

### Software
- Raspberry Pi OS Bullseye (32-bit or 64-bit)
- Python 3.x
- Node.js 18+
- Rust 1.75+

## 🚀 Installation

### Quick Install (Recommended)
```bash
# Clone the repository
git clone https://github.com/AutomataControls/Development.git
cd Development/projects/building-automation-controller

# Run the graphical installer
sudo python3 installer/install-automata-nexus.py
```

### Manual Installation
```bash
# Deploy to Raspberry Pi
./scripts/deploy-to-pi.sh pi@raspberrypi.local
```

The installer will:
- ✓ Install all system dependencies
- ✓ Configure I2C and serial interfaces
- ✓ Install Sequent Microsystems drivers
- ✓ Build and install the application
- ✓ Set up systemd service
- ✓ Configure proper permissions

## 🖥️ Usage

Access the web interface at `http://raspberrypi.local:1420`

### Main Features:

1. **I/O Control Tab**: Direct control of all inputs/outputs
2. **Expansion Boards Tab**: Configure and control expansion boards
3. **Data Export Tab**: Configure BMS and processing endpoints
4. **Monitoring Tab**: Real-time system status and logs
5. **Protocol Manager**: BACnet and Modbus integration
6. **Vibration Monitoring**: ISO 10816-3 compliant monitoring
7. **Refrigerant Diagnostics**: ASHRAE 207-2021 compliant analysis

## Hardware Configuration

### MegaBAS HAT (Stack Level 0)
- Channels 1-8: Universal inputs (configurable as 0-10V, 1K thermistor, or 10K Type 2 thermistor)
- Channels 1-4: 0-10V outputs
- Triacs 1-4: AC control
- Contacts 1-4: Dry contact inputs

#### Configuring Input Types
For MegaBAS HAT v5.0+, input types are software configurable:
- Default: 0-10V analog input
- Option 1: 1K thermistor or dry contact
- Option 2: 10K Type 2 thermistor (with automatic temperature conversion)

Input types can be configured through the UI or via the API.

### Expansion Boards (Stack Levels 1-7)
Configure via the UI based on your hardware setup.