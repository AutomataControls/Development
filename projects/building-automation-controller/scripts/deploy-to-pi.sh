#!/bin/bash

# Building Automation Controller - Raspberry Pi Deployment Script

set -e

# Configuration
PI_HOST="${1:-pi@raspberrypi.local}"
PI_DIR="/home/Automata/building-automation"
BUILD_TYPE="${2:-release}"

echo "========================================"
echo "Building Automation Controller Deploy"
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
    BINARY_PATH="target/armv7-unknown-linux-gnueabihf/release/building-automation-controller"
else
    echo "Building debug version..."
    cargo build --target armv7-unknown-linux-gnueabihf
    BINARY_PATH="target/armv7-unknown-linux-gnueabihf/debug/building-automation-controller"
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

# Copy Python interface
cp scripts/megabas_interface.py deploy-package/

# Create run script
cat > deploy-package/run-controller.sh << 'EOF'
#!/bin/bash

echo "Building Automation Controller"
echo "============================="

# Check for MegaBAS HAT
echo ""
echo "Checking for MegaBAS HAT..."
if python3 megabas_interface.py scan | grep -q "megabas"; then
    echo "✓ MegaBAS HAT detected"
else
    echo "✗ MegaBAS HAT not detected"
    echo "  - Check HAT is properly seated"
    echo "  - Verify I2C is enabled"
fi

# Check I2C permissions
if ! groups | grep -q i2c; then
    echo ""
    echo "WARNING: User not in i2c group!"
    echo "Run: sudo usermod -a -G i2c $USER"
    echo "Then logout and login again."
fi

# Start controller
echo ""
echo "Starting controller..."
export WEBKIT_DISABLE_COMPOSITING_MODE=1
./building-automation-controller
EOF

chmod +x deploy-package/run-controller.sh

# Create systemd service
cat > deploy-package/building-automation.service << EOF
[Unit]
Description=Building Automation Controller
After=network.target

[Service]
Type=simple
User=Automata
WorkingDirectory=$PI_DIR
ExecStart=$PI_DIR/building-automation-controller
Restart=always
RestartSec=10
Environment="RUST_LOG=info"
Environment="WEBKIT_DISABLE_COMPOSITING_MODE=1"

# I2C device access
SupplementaryGroups=i2c
PrivateDevices=no

[Install]
WantedBy=multi-user.target
EOF

# Create setup script
cat > deploy-package/setup-pi.sh << 'EOF'
#!/bin/bash

echo "Setting up Building Automation Controller..."

# Enable I2C
echo "Enabling I2C interface..."
sudo raspi-config nonint do_i2c 0

# Install required packages
echo "Installing dependencies..."
sudo apt-get update
sudo apt-get install -y \
    python3-smbus \
    i2c-tools \
    python3-pip \
    python3-dev \
    nodejs \
    npm

# Install Python libraries
echo "Installing Python libraries..."
pip3 install megabas
pip3 install SM16relind
pip3 install SM16univin  
pip3 install SM16uout
pip3 install SM8relind

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
ssh $PI_HOST "chmod +x $PI_DIR/building-automation-controller $PI_DIR/*.sh"

# Step 4: Configure on Pi
echo ""
echo "Step 4: Configuring on Raspberry Pi..."

ssh $PI_HOST << 'REMOTE_SCRIPT'
cd /home/Automata/building-automation

# Run setup if needed
if ! groups | grep -q i2c; then
    echo "Running initial setup..."
    ./setup-pi.sh
fi

echo ""
echo "Deployment complete!"
echo ""
echo "Hardware Setup:"
echo "=============="
echo "MegaBAS HAT (Stack 0):"
echo "- Analog Inputs 1-8: Connect 0-10V sensors"
echo "- Analog Outputs 1-4: Connect 0-10V actuators"
echo "- Triacs 1-4: Connect AC loads"
echo "- Dry Contacts 1-4: Connect switches/contacts"
echo ""
echo "Expansion Boards (Stack 1-7):"
echo "- 16-relay boards for binary outputs"
echo "- 16-universal input boards for additional sensors"
echo "- 16-analog output boards for additional actuators"
echo ""
echo "Default I/O Mapping for Control Logic:"
echo "- Supply Temp: Analog Input 1"
echo "- Return Temp: Analog Input 2"
echo "- Outdoor Temp: Analog Input 3"
echo "- Mixed Air Temp: Analog Input 4"
echo "- Static Pressure: Analog Input 5"
echo "- Heating Valve: Analog Output 1"
echo "- Cooling Valve: Analog Output 2"
echo "- Fan VFD: Analog Output 3"
echo "- Outdoor Damper: Triac 1"
echo ""
echo "To run manually:"
echo "  cd /home/Automata/building-automation"
echo "  ./run-controller.sh"
echo ""
echo "To install as service:"
echo "  sudo cp building-automation.service /etc/systemd/system/"
echo "  sudo systemctl daemon-reload"
echo "  sudo systemctl enable building-automation"
echo "  sudo systemctl start building-automation"
echo ""
echo "Access the controller at:"
echo "http://$(hostname -I | awk '{print $1}'):1420"

REMOTE_SCRIPT

echo ""
echo "========================================"
echo "Deployment Complete!"
echo "========================================"