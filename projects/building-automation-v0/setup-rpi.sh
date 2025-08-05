#!/bin/bash
# Building Automation Control Center Setup Script for Raspberry Pi

echo "🍓 Setting up Building Automation Control Center on Raspberry Pi..."

# Update system
echo "Updating system packages..."
sudo apt update && sudo apt upgrade -y

# Install Python dependencies
echo "Installing Python dependencies..."
sudo apt install -y python3-pip python3-dev python3-smbus i2c-tools

# Install all SM board libraries
echo "Installing Sequent Microsystems libraries..."
sudo pip3 install SMmegabas
sudo pip3 install SM8relind
sudo pip3 install SM16relind
sudo pip3 install SM16uout
sudo pip3 install SM16univin

# Enable I2C
echo "Enabling I2C interface..."
sudo raspi-config nonint do_i2c 0

# Install Rust and Cargo (if not already installed)
if ! command -v cargo &> /dev/null; then
    echo "Installing Rust for ARM..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source ~/.cargo/env
    rustup target add armv7-unknown-linux-gnueabihf
fi

# Install Node.js (if not already installed)
if ! command -v node &> /dev/null; then
    echo "Installing Node.js..."
    curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
    sudo apt-get install -y nodejs
fi

# Install Tauri CLI
echo "Installing Tauri CLI..."
cargo install tauri-cli

# Test board connections
echo "Testing board connections..."
python3 -c "
import sys
try:
    import megabas
    print('✅ megabas library installed successfully')
    try:
        version = megabas.getVer(0)
        print(f'✅ SM-I-002 Building Automation HAT detected (v{version})')
    except:
        print('⚠️  SM-I-002 Building Automation HAT not detected on stack 0')
except ImportError:
    print('❌ megabas library not installed')

try:
    import lib8relind
    print('✅ lib8relind library installed successfully')
except ImportError:
    print('❌ lib8relind library not installed')

try:
    import SM16relind
    print('✅ SM16relind library installed successfully')
except ImportError:
    print('❌ SM16relind library not installed')

try:
    import SM16uout.SM16uout
    print('✅ SM16uout library installed successfully')
except ImportError:
    print('❌ SM16uout library not installed')

try:
    import lib16univin
    print('✅ lib16univin library installed successfully')
except ImportError:
    print('❌ lib16univin library not installed')
"

# Create systemd service for auto-start (optional)
echo "Creating systemd service..."
sudo tee /etc/systemd/system/building-automation.service > /dev/null <<EOF
[Unit]
Description=Building Automation Control Center
After=network.target

[Service]
Type=simple
User=pi
WorkingDirectory=/home/pi/building-automation-center
ExecStart=/home/pi/.cargo/bin/cargo tauri dev
Restart=always
RestartSec=10
Environment=DISPLAY=:0

[Install]
WantedBy=multi-user.target
EOF

echo "✅ Setup complete!"
echo ""
echo "🚀 To start the application:"
echo "  npm install"
echo "  cargo tauri dev"
echo ""
echo "🔧 To build for production:"
echo "  cargo tauri build --target armv7-unknown-linux-gnueabihf"
echo ""
echo "⚡ To enable auto-start on boot:"
echo "  sudo systemctl enable building-automation"
echo "  sudo systemctl start building-automation"
echo ""
echo "📋 Hardware Connection Guide:"
echo "1. Connect SM boards to Raspberry Pi I2C bus"
echo "2. Set stack levels using address jumpers (0-7)"
echo "3. Ensure 24VAC power supply for SM-I-002 HAT"
echo "4. Connect sensors/actuators to appropriate channels"
echo ""
echo "🔍 Supported Boards:"
echo "• SM-I-002 Building Automation HAT (8 inputs, 4 outputs, 4 triacs)"
echo "• SM8relind 8-Relay Board"
echo "• SM16relind 16-Relay Board"
echo "• SM16uout 16 Analog Output Board"
echo "• SM16univin 16 Universal Input Board"
