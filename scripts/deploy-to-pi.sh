#!/bin/bash

# AutomataNexus Vibration Monitor - Raspberry Pi Deployment Script
# Deploys optimized WTVB01-485 sensor monitoring app to Raspberry Pi

set -e

# Configuration
PI_HOST="${1:-pi@raspberrypi.local}"
PI_DIR="/home/pi/vibration-monitor"
BUILD_TYPE="${2:-release}"

echo "========================================"
echo "AutomataNexus Vibration Monitor Deploy"
echo "Deploying to: $PI_HOST"
echo "Target directory: $PI_DIR"
echo "Build type: $BUILD_TYPE"
echo "========================================"

# Step 1: Build the optimized Tauri app
echo ""
echo "Step 1: Building Tauri application..."
cd src-tauri

# Use the optimized WTVB01 main file
cp src/main-wtvb01.rs src/main.rs

# Build for ARM (Raspberry Pi)
if [ "$BUILD_TYPE" == "release" ]; then
    echo "Building optimized release version..."
    cargo build --release --target armv7-unknown-linux-gnueabihf
    BINARY_PATH="target/armv7-unknown-linux-gnueabihf/release/automatanexus-vibration-monitor"
else
    echo "Building debug version..."
    cargo build --target armv7-unknown-linux-gnueabihf
    BINARY_PATH="target/armv7-unknown-linux-gnueabihf/debug/automatanexus-vibration-monitor"
fi

cd ..

# Step 2: Create deployment package
echo ""
echo "Step 2: Creating deployment package..."
rm -rf deploy-package
mkdir -p deploy-package

# Copy binary
cp "src-tauri/$BINARY_PATH" deploy-package/automatanexus-vibration-monitor

# Copy UI files
cp src/index.html deploy-package/
cp -r src/assets deploy-package/ 2>/dev/null || true

# Copy run script
cat > deploy-package/run-monitor.sh << 'EOF'
#!/bin/bash

echo "AutomataNexus Vibration Monitor"
echo "================================"

# Check USB permissions
if ! groups | grep -q dialout; then
    echo "WARNING: User not in dialout group!"
    echo "Run: sudo usermod -a -G dialout $USER"
    echo "Then logout and login again."
fi

# List available USB ports
echo ""
echo "Available USB ports:"
ls -la /dev/ttyUSB* 2>/dev/null || echo "No USB serial devices found"

# Check for WTVB01 sensors at different baud rates
echo ""
echo "Scanning for WTVB01-485 sensors..."
echo "Checking 115200 baud (optimized)..."
echo "Checking 230400 baud (maximum speed)..."
echo "Checking 9600 baud (factory default)..."

# Run the monitor
echo ""
echo "Starting vibration monitor..."
./automatanexus-vibration-monitor
EOF

chmod +x deploy-package/run-monitor.sh

# Create systemd service file
cat > deploy-package/vibration-monitor.service << EOF
[Unit]
Description=AutomataNexus Vibration Monitor
After=network.target

[Service]
Type=simple
User=pi
WorkingDirectory=$PI_DIR
ExecStart=$PI_DIR/automatanexus-vibration-monitor
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
EOF

# Step 3: Deploy to Raspberry Pi
echo ""
echo "Step 3: Deploying to Raspberry Pi..."

# Create directory on Pi
ssh $PI_HOST "mkdir -p $PI_DIR"

# Copy files
echo "Copying files..."
scp -r deploy-package/* $PI_HOST:$PI_DIR/

# Set permissions
echo "Setting permissions..."
ssh $PI_HOST "chmod +x $PI_DIR/automatanexus-vibration-monitor $PI_DIR/run-monitor.sh"

# Step 4: Configure on Pi
echo ""
echo "Step 4: Configuring on Raspberry Pi..."

ssh $PI_HOST << 'REMOTE_SCRIPT'
# Add user to dialout group if needed
if ! groups | grep -q dialout; then
    echo "Adding user to dialout group..."
    sudo usermod -a -G dialout $USER
    echo "NOTE: You need to logout and login for group change to take effect!"
fi

# Install systemd service (optional)
read -p "Install as system service? (y/n): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    sudo cp ~/vibration-monitor/vibration-monitor.service /etc/systemd/system/
    sudo systemctl daemon-reload
    sudo systemctl enable vibration-monitor
    echo "Service installed. Use 'sudo systemctl start vibration-monitor' to start."
fi

echo ""
echo "Deployment complete!"
echo ""
echo "WTVB01-485 Sensor Setup:"
echo "========================"
echo "1. Connect WTVB01-485 sensors via USB-RS485 adapters"
echo "2. Default Modbus address: 0x50"
echo "3. Optimized baud rate: 115200 (default)"
echo "4. Maximum baud rate: 230400 (24x faster than factory)"
echo "5. High-speed mode: 1000Hz sampling available"
echo ""
echo "To run manually:"
echo "  cd ~/vibration-monitor"
echo "  ./run-monitor.sh"
echo ""
echo "To run as service:"
echo "  sudo systemctl start vibration-monitor"
echo "  sudo systemctl status vibration-monitor"
echo ""
echo "Speed Optimization:"
echo "==================="
echo "The app will automatically try to optimize sensors to 230400 baud."
echo "Use the 'Optimize Speed' button in the UI to manually optimize."
echo "Use the 'Enable 1000Hz Mode' button for maximum sampling rate."
echo ""
echo "Burst Reading:"
echo "=============="
echo "Use 'Burst Read' button to read all 19 registers in one command."
echo "This is much faster than reading registers individually."

REMOTE_SCRIPT

echo ""
echo "========================================"
echo "Deployment Complete!"
echo "========================================"
echo ""
echo "Next steps:"
echo "1. SSH to Pi: ssh $PI_HOST"
echo "2. Run monitor: cd $PI_DIR && ./run-monitor.sh"
echo "3. Open browser to: http://$(echo $PI_HOST | cut -d@ -f2):1420"
echo ""
echo "Performance Tips:"
echo "- Use burst reading for fastest sensor updates"
echo "- Optimize to 230400 baud for 24x speed improvement"
echo "- Enable 1000Hz mode for critical vibration monitoring"