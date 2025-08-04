# Multi-Refrigerant Diagnostic Monitor

Professional HVAC/Refrigeration diagnostic system supporting 100+ refrigerants with real-time fault detection using P499 pressure transducers and Sequent Microsystems HAT.

## 🌟 Features

- **100 Refrigerant Database**: Complete P-T relationships and diagnostic parameters
- **Real-time Monitoring**: Continuous pressure/temperature readings via P499 transducers
- **AI Fault Detection**: Pattern recognition for common HVAC/refrigeration faults
- **Multi-System Support**: R-410A, R-454B, R-32, R-22, R-134a, R-290, R-600a, R-717, R-744, and 90+ more
- **ASHRAE Compliant**: Follows ASHRAE 207-2021 and NIST FDD guidelines
- **Beautiful UI**: Glassmorphism design optimized for Raspberry Pi

## 🔧 Hardware Requirements

- Raspberry Pi 3/4/5
- Sequent Microsystems 0-10V/4-20mA HAT
- P499 Series Pressure Transducers (0-10V or 4-20mA models)
- Temperature sensors (K-type thermocouples)
- USB-RS485 adapter (optional for additional sensors)

## 📊 Supported Refrigerants

### Primary HVAC & Residential (25)
- R-410A, R-454B, R-32, R-22, R-134a
- A2L Next-Gen: R-452B, R-466A, R-468A
- Natural: R-290, R-600a, R-717, R-744
- HFO: R-1234yf, R-1234ze(E), R-513A

### Commercial & Industrial (25)
- R-404A, R-507A, R-448A, R-449A
- Transport: R-442A, R-444A, R-444B
- Ultra-low temp: R-23, R-508A, R-508B

### Automotive & Specialty (25)
- R-1234yf (automotive standard)
- R-152a, R-445A
- Natural variants: R-600, R-601, R-1270

### Emerging & Legacy (25)
- Next-gen A2L: R-455A, R-456A, R-457A
- Inorganic: R-702 to R-744A
- Legacy CFC/HCFC: R-11, R-12, R-13, R-114

## 🚀 Quick Start

```bash
cd src-tauri
cargo build --release
./target/release/refrigerant-diagnostic-monitor
```

## 📈 Diagnostic Capabilities

- **Superheat/Subcooling Calculation**
- **Fault Pattern Recognition**
- **Pressure-Temperature Validation**
- **Efficiency Analysis**
- **Predictive Maintenance Alerts**

## 🔌 P499 Transducer Integration

Supports all P499 models:
- 0.5-4.5V ratiometric
- 0-10V analog
- 4-20mA current loop

Pressure ranges: -10 to 750 PSI

## 📝 License

© 2025 AutomataControls. Commercial license required.