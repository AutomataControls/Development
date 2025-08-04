# Installation Guide - WTVB01 Vibration Monitor for Raspberry Pi

## Prerequisites

### Hardware Requirements
- Raspberry Pi 3, 4, or 5
- At least 4GB SD card
- WIT-Motion WTVB01-485 sensors (1-3 units)
- USB-to-RS485 adapters (one per sensor)
- 5V power supply for Raspberry Pi

### Software Requirements
- Raspberry Pi OS (Bullseye or Bookworm)
- Internet connection for initial setup

## Step 1: Prepare Raspberry Pi

```bash
# Update system
sudo apt update && sudo apt upgrade -y

# Install required packages
sudo apt install -y curl git build-essential pkg-config libssl-dev

# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Add ARM target for cross-compilation (optional, for building on PC)
rustup target add armv7-unknown-linux-gnueabihf
```

## Step 2: USB Permissions

```bash
# Add user to dialout group for USB serial access
sudo usermod -a -G dialout $USER

# IMPORTANT: Logout and login again for group changes to take effect
# Or reboot: sudo reboot
```

## Step 3: Clone and Build

```bash
# Clone the repository
git clone https://github.com/yourusername/Development.git
cd Development/vibration-monitor-rpi

# Build the application (takes 5-10 minutes on Pi)
cd src-tauri
cargo build --release

# Verify build
./target/release/wtvb01-vibration-monitor --version
```

## Step 4: Connect Sensors

1. Connect WTVB01-485 sensors to USB-RS485 adapters
2. Plug USB adapters into Raspberry Pi USB ports
3. Verify detection:
```bash
ls -la /dev/ttyUSB*
# Should show: /dev/ttyUSB0, /dev/ttyUSB1, etc.
```

## Step 5: Test Sensors

```bash
# Test sensor communication at different speeds
cd ~/Development/vibration-monitor-rpi
./scripts/test-speed.sh

# Expected output:
# Testing port: /dev/ttyUSB0
# ✓ Sensor responded at 115200 baud
# Completed 10 reads in 95ms
# Average: 9ms per read
```

## Step 6: Run the Application

### Manual Start
```bash
cd ~/Development/vibration-monitor-rpi/src-tauri
./target/release/wtvb01-vibration-monitor

# Application starts on port 1420
# Open browser to: http://raspberrypi.local:1420
```

### Install as Service (Recommended)
```bash
cd ~/Development/vibration-monitor-rpi/scripts
sudo ./install-service.sh

# Start the service
sudo systemctl start wtvb01-monitor

# Check status
sudo systemctl status wtvb01-monitor

# View logs
sudo journalctl -u wtvb01-monitor -f
```

## Step 7: Access the UI

Open a web browser on any device on your network:
- Local: `http://localhost:1420`
- Network: `http://raspberrypi.local:1420`
- By IP: `http://[PI-IP-ADDRESS]:1420`

## Step 8: Configure Sensors

1. Click "Configuration" tab
2. Click "Scan USB Ports" 
3. For each detected sensor:
   - Click "Configure This Sensor"
   - Enter equipment details
   - Save configuration

## Step 9: Optimize Performance

### Speed Optimization (Recommended)
1. Click "Optimize Speed (230400)" button
2. This sets all sensors to maximum baud rate
3. 24x faster than factory default!

### Enable Burst Reading
1. Use "Burst Read" button for each sensor
2. Reads all 19 registers in one command
3. Reduces read time to <3ms

### High-Speed Mode (Optional)
1. Click "Enable 1000Hz Mode"
2. Enables 1ms sampling interval
3. Required for critical vibration monitoring

## Troubleshooting

### No USB Ports Detected
```bash
# Check USB devices
lsusb
# Check permissions
groups  # Should include 'dialout'
# Check kernel messages
dmesg | grep ttyUSB
```

### Permission Denied
```bash
# Add to dialout group
sudo usermod -a -G dialout $USER
# Logout and login again
```

### Sensor Not Responding
```bash
# Test different baud rates
./scripts/test-speed.sh
# Check wiring and power
# Verify RS485 adapter LED indicators
```

### High CPU Usage
```bash
# Reduce sampling rate in UI
# Use burst reading mode
# Check system resources
htop
```

## Performance Tips

1. **Use Burst Reading**: Reads all data in one command
2. **Optimize Baud Rate**: Set to 230400 for best speed
3. **Limit Sensor Count**: 3 sensors maximum for smooth operation
4. **Use Wired Network**: Better than WiFi for remote access
5. **Disable Unused Features**: Turn off FFT if not needed

## Support

- Documentation: https://automatanexus.com/docs
- Email: support@automatacontrols.com
- Issues: https://github.com/yourusername/Development/issues