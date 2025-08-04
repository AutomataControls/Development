#!/bin/bash

# Refrigerant Diagnostic Monitor - Raspberry Pi Deployment Script
# Deploys app with P499 transducer support via Sequent Microsystems HAT

set -e

# Configuration
PI_HOST="${1:-pi@raspberrypi.local}"
PI_DIR="/home/pi/refrigerant-monitor"
BUILD_TYPE="${2:-release}"

echo "========================================"
echo "Refrigerant Diagnostic Monitor Deploy"
echo "Deploying to: $PI_HOST"
echo "Target directory: $PI_DIR"
echo "Build type: $BUILD_TYPE"
echo "========================================"

# Step 1: Build the Tauri app
echo ""
echo "Step 1: Building Tauri application..."
cd ../src-tauri

# Build for ARM
if [ "$BUILD_TYPE" == "release" ]; then
    echo "Building optimized release version..."
    cargo build --release --target armv7-unknown-linux-gnueabihf
    BINARY_PATH="target/armv7-unknown-linux-gnueabihf/release/refrigerant-diagnostic-monitor"
else
    echo "Building debug version..."
    cargo build --target armv7-unknown-linux-gnueabihf
    BINARY_PATH="target/armv7-unknown-linux-gnueabihf/debug/refrigerant-diagnostic-monitor"
fi

cd ..

# Step 2: Create deployment package
echo ""
echo "Step 2: Creating deployment package..."
rm -rf deploy-package
mkdir -p deploy-package

# Copy binary
cp "src-tauri/$BINARY_PATH" deploy-package/

# Copy UI files
cp -r src deploy-package/

# Copy Python HAT interface
cp scripts/sm_4_20ma.py deploy-package/

# Create run script
cat > deploy-package/run-monitor.sh << 'EOF'
#!/bin/bash

echo "Refrigerant Diagnostic Monitor"
echo "=============================="

# Check for Sequent Microsystems HAT
echo ""
echo "Checking for 0-10V/4-20mA HAT..."
if python3 sm_4_20ma.py test 0; then
    echo "✓ HAT detected at stack level 0"
else
    echo "✗ HAT not detected - check connections"
    echo "  - Ensure HAT is properly seated on GPIO pins"
    echo "  - Check I2C is enabled: sudo raspi-config"
    echo "  - Verify with: i2cdetect -y 1"
fi

# List available channels
echo ""
echo "Scanning analog inputs..."
python3 sm_4_20ma.py scan 0 voltage || true

# Check I2C permissions
if ! groups | grep -q i2c; then
    echo ""
    echo "WARNING: User not in i2c group!"
    echo "Run: sudo usermod -a -G i2c $USER"
    echo "Then logout and login again."
fi

# Run the monitor
echo ""
echo "Starting diagnostic monitor..."
./refrigerant-diagnostic-monitor
EOF

chmod +x deploy-package/run-monitor.sh

# Create systemd service
cat > deploy-package/refrigerant-monitor.service << EOF
[Unit]
Description=Refrigerant Diagnostic Monitor
After=network.target

[Service]
Type=simple
User=pi
WorkingDirectory=$PI_DIR
ExecStart=$PI_DIR/refrigerant-diagnostic-monitor
Restart=always
RestartSec=10
Environment="RUST_LOG=info"

# I2C device access
SupplementaryGroups=i2c
PrivateDevices=no

[Install]
WantedBy=multi-user.target
EOF

# Create setup script
cat > deploy-package/setup-pi.sh << 'EOF'
#!/bin/bash

echo "Setting up Refrigerant Monitor on Raspberry Pi..."

# Enable I2C
echo "Enabling I2C interface..."
sudo raspi-config nonint do_i2c 0

# Install required packages
echo "Installing dependencies..."
sudo apt-get update
sudo apt-get install -y python3-smbus i2c-tools python3-pip

# Add user to i2c group
echo "Adding user to i2c group..."
sudo usermod -a -G i2c $USER

# Test I2C
echo ""
echo "Testing I2C bus..."
i2cdetect -y 1

echo ""
echo "Setup complete!"
echo "Please logout and login again for group changes to take effect."
EOF

chmod +x deploy-package/setup-pi.sh

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
ssh $PI_HOST "chmod +x $PI_DIR/refrigerant-diagnostic-monitor $PI_DIR/*.sh"

# Step 4: Configure on Pi
echo ""
echo "Step 4: Configuring on Raspberry Pi..."

ssh $PI_HOST << 'REMOTE_SCRIPT'
cd ~/refrigerant-monitor

# Run setup if needed
if ! groups | grep -q i2c; then
    echo "Running initial setup..."
    ./setup-pi.sh
fi

# Install Python dependencies
pip3 install smbus

echo ""
echo "Deployment complete!"
echo ""
echo "P499 Transducer Setup:"
echo "====================="
echo "1. Connect P499 transducers to Sequent Microsystems HAT"
echo "2. 0-10V models: Connect to voltage inputs (channels 0-7)"
echo "3. 4-20mA models: Connect to current inputs (channels 0-7)"
echo "4. Ensure proper grounding and shielding"
echo ""
echo "Supported P499 Models:"
echo "- P499VCP-xxx: 0-10V output"
echo "- P499ACP-xxx: 4-20mA output"
echo "- P499RCP-xxx: 0.5-4.5V ratiometric (use 0-10V input)"
echo ""
echo "To run manually:"
echo "  cd ~/refrigerant-monitor"
echo "  ./run-monitor.sh"
echo ""
echo "To install as service:"
echo "  sudo cp refrigerant-monitor.service /etc/systemd/system/"
echo "  sudo systemctl daemon-reload"
echo "  sudo systemctl enable refrigerant-monitor"
echo "  sudo systemctl start refrigerant-monitor"

REMOTE_SCRIPT

echo ""
echo "========================================"
echo "Deployment Complete!"
echo "========================================"
echo ""
echo "Access the monitor at:"
echo "http://$(echo $PI_HOST | cut -d@ -f2):1420"
echo ""
echo "Features:"
echo "- 100+ refrigerant database"
echo "- Real-time P499 transducer readings"
echo "- AI-based fault detection"
echo "- P-T charts and calculators"
echo "- ASHRAE 207-2021 compliant diagnostics"