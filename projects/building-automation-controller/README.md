# Building Automation Controller

A comprehensive Tauri-based control system for Sequent Microsystems building automation hardware on Raspberry Pi.

## Features

- **MegaBAS HAT Control**
  - 8x 0-10V analog inputs with thermistor support
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
- Channels 1-8: 0-10V inputs
- Channels 1-4: 0-10V outputs
- Triacs 1-4: AC control
- Contacts 1-4: Dry contact inputs

### Expansion Boards (Stack Levels 1-7)
Configure via the UI based on your hardware setup.