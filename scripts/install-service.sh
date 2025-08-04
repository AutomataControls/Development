#!/bin/bash

# WTVB01 Vibration Monitor - Systemd Service Installer
# Installs the monitor as a system service on Raspberry Pi

set -e

SERVICE_NAME="wtvb01-monitor"
SERVICE_FILE="/etc/systemd/system/${SERVICE_NAME}.service"
INSTALL_DIR="/opt/vibration-monitor"
USER="pi"

echo "======================================"
echo "WTVB01 Vibration Monitor Service Setup"
echo "======================================"

# Check if running as root
if [ "$EUID" -ne 0 ]; then 
    echo "Please run with sudo: sudo ./install-service.sh"
    exit 1
fi

# Create installation directory
echo "Creating installation directory..."
mkdir -p $INSTALL_DIR

# Copy application files
echo "Copying application files..."
cp -r ../src-tauri/target/release/wtvb01-vibration-monitor $INSTALL_DIR/
cp -r ../src $INSTALL_DIR/

# Create systemd service file
echo "Creating systemd service..."
cat > $SERVICE_FILE << EOF
[Unit]
Description=WTVB01-485 Vibration Monitor
After=network.target
StartLimitIntervalSec=0

[Service]
Type=simple
Restart=always
RestartSec=10
User=$USER
WorkingDirectory=$INSTALL_DIR
ExecStart=$INSTALL_DIR/wtvb01-vibration-monitor
StandardOutput=append:/var/log/${SERVICE_NAME}.log
StandardError=append:/var/log/${SERVICE_NAME}.error.log

# USB device permissions
SupplementaryGroups=dialout
PrivateDevices=no

# Performance tuning
Nice=-10
CPUSchedulingPolicy=fifo
CPUSchedulingPriority=50

[Install]
WantedBy=multi-user.target
EOF

# Set permissions
echo "Setting permissions..."
chown -R $USER:$USER $INSTALL_DIR
chmod +x $INSTALL_DIR/wtvb01-vibration-monitor

# Add user to dialout group
echo "Checking USB permissions..."
usermod -a -G dialout $USER 2>/dev/null || true

# Reload systemd
echo "Configuring systemd..."
systemctl daemon-reload
systemctl enable ${SERVICE_NAME}.service

echo ""
echo "======================================"
echo "Service installation complete!"
echo "======================================"
echo ""
echo "Service commands:"
echo "  Start:   sudo systemctl start ${SERVICE_NAME}"
echo "  Stop:    sudo systemctl stop ${SERVICE_NAME}"
echo "  Status:  sudo systemctl status ${SERVICE_NAME}"
echo "  Logs:    sudo journalctl -u ${SERVICE_NAME} -f"
echo ""
echo "The monitor will start automatically on boot."
echo "Access the UI at: http://$(hostname -I | awk '{print $1}'):1420"