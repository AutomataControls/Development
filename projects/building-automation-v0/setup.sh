#!/bin/bash
# HVAC Diagnostic System Setup Script for Raspberry Pi

echo "🍓 Setting up HVAC Diagnostic System on Raspberry Pi..."

# Update system
echo "Updating system packages..."
sudo apt update && sudo apt upgrade -y

# Install Python dependencies
echo "Installing Python dependencies..."
sudo apt install -y python3-pip python3-dev

# Install megabas library
echo "Installing megabas library..."
sudo pip3 install SMmegabas

# Install Rust and Cargo (if not already installed)
if ! command -v cargo &> /dev/null; then
    echo "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source ~/.cargo/env
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

# Make Python script executable
chmod +x scripts/sensor_interface.py

# Create systemd service for auto-start (optional)
echo "Creating systemd service..."
sudo tee /etc/systemd/system/hvac-diagnostics.service > /dev/null <<EOF
[Unit]
Description=HVAC Diagnostic System
After=network.target

[Service]
Type=simple
User=pi
WorkingDirectory=/home/pi/hvac-diagnostic-system
ExecStart=/home/pi/.cargo/bin/cargo tauri dev
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
EOF

echo "✅ Setup complete!"
echo ""
echo "To start the application:"
echo "  npm install"
echo "  cargo tauri dev"
echo ""
echo "To enable auto-start on boot:"
echo "  sudo systemctl enable hvac-diagnostics"
echo "  sudo systemctl start hvac-diagnostics"
echo ""
echo "Hardware Connection Guide:"
echo "1. Connect P499 transducers to SM-I-002 HAT 0-10V inputs"
echo "2. Channel 1: Suction pressure transducer"
echo "3. Channel 2: Discharge pressure transducer"
echo "4. Channels 3-4: Additional pressure points as needed"
echo "5. Ensure 24VAC power supply is connected to HAT"
echo ""
echo "P499 Transducer Wiring:"
echo "- Red wire: +24VDC supply"
echo "- Black wire: Ground/Common"
echo "- White wire: 0-10V signal output (connect to HAT input)"
