#!/bin/bash

# Build script for Automata Nexus Rust Controller
# This compiles everything needed for the complete system

set -e

echo "Building Automata Nexus AI Controller (Rust Edition)..."

# Check if running on ARM64 (Raspberry Pi 5)
if [[ $(uname -m) == "aarch64" ]]; then
    echo "Detected ARM64 architecture (Raspberry Pi 5)"
    export TARGET="aarch64-unknown-linux-gnu"
    export RUSTFLAGS="-C target-cpu=cortex-a76"
else
    echo "Building for development (x86_64)"
    export TARGET="x86_64-unknown-linux-gnu"
fi

# Install Rust if not present
if ! command -v cargo &> /dev/null; then
    echo "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

# Add ARM64 target if on Pi5
if [[ $(uname -m) == "aarch64" ]]; then
    rustup target add aarch64-unknown-linux-gnu
fi

# Install system dependencies
echo "Installing system dependencies..."
sudo apt-get update
sudo apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    libsqlite3-dev \
    libgtk-3-dev \
    libwebkit2gtk-4.0-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    libudev-dev \
    libusb-1.0-0-dev \
    python3-dev \
    python3-pip

# Install Python dependencies for board interface
echo "Installing Python dependencies..."
pip3 install --user pyserial modbus-tk pymodbus

# Create required directories
sudo mkdir -p /opt/nexus/{firmware,logic,config,data}
sudo mkdir -p /var/lib/nexus
sudo mkdir -p /var/log/nexus
sudo mkdir -p /etc/nexus

# Copy Python board interface
echo "Copying board interface..."
sudo cp ../building-automation-controller-rpi5-ssd/scripts/megabas_interface.py /opt/nexus/
sudo chmod +x /opt/nexus/megabas_interface.py

# Copy configuration files
echo "Copying configuration..."
sudo cp -r config/* /etc/nexus/ 2>/dev/null || true

# Build the Rust application
echo "Building Rust application..."
cargo build --release --target=$TARGET

# Create standalone binary
echo "Creating standalone executable..."
cp target/$TARGET/release/nexus-controller ./nexus-controller
strip ./nexus-controller

# Create systemd service
echo "Creating systemd service..."
sudo tee /etc/systemd/system/nexus.service > /dev/null <<EOF
[Unit]
Description=Automata Nexus AI Controller
After=network.target

[Service]
Type=simple
User=pi
WorkingDirectory=/opt/nexus
ExecStart=/opt/nexus/nexus-controller
Restart=always
RestartSec=10
Environment="RUST_LOG=info"

[Install]
WantedBy=multi-user.target
EOF

# Copy binary to installation directory
sudo cp ./nexus-controller /opt/nexus/
sudo chmod +x /opt/nexus/nexus-controller

# Set permissions
sudo chown -R pi:pi /opt/nexus
sudo chown -R pi:pi /var/lib/nexus
sudo chown -R pi:pi /var/log/nexus

# Initialize database
echo "Initializing database..."
sqlite3 /var/lib/nexus/nexus.db < migrations/001_initial.sql 2>/dev/null || true

# Enable and start service
echo "Enabling service..."
sudo systemctl daemon-reload
sudo systemctl enable nexus.service

echo ""
echo "Build complete!"
echo "Binary size: $(du -h ./nexus-controller | cut -f1)"
echo ""
echo "To start the service:"
echo "  sudo systemctl start nexus"
echo ""
echo "To view logs:"
echo "  sudo journalctl -u nexus -f"
echo ""
echo "Access the application at:"
echo "  http://localhost:3000"