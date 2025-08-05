# Building Automation Controller

A comprehensive Tauri-based control system for Sequent Microsystems building automation hardware on Raspberry Pi.

## Features

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

- **Data Integration**
  - BMS metric sender with InfluxDB line protocol
  - Processing validation proxy for enhanced data validation
  - Configurable equipment types and zones
  - Real-time monitoring and control

## Requirements

- Raspberry Pi running Bullseye 32-bit OS
- Sequent Microsystems MegaBAS HAT
- Python 3.x with megabas library
- Optional expansion boards

## Installation

1. Deploy to Raspberry Pi:
   ```bash
   ./scripts/deploy-to-pi.sh pi@raspberrypi.local
   ```

2. The deployment script will:
   - Install required Python libraries (megabas, SM16relind, SM16univin, SM16uout, SM8relind)
   - Configure I2C interface
   - Set up the application

## Usage

Access the web interface at `http://raspberrypi.local:1420`

### Main Features:

1. **I/O Control Tab**: Direct control of all inputs/outputs
2. **Expansion Boards Tab**: Configure and control expansion boards
3. **Data Export Tab**: Configure BMS and processing endpoints
4. **Monitoring Tab**: Real-time system status and logs

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