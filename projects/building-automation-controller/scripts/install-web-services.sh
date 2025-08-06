#!/bin/bash

echo "Installing Automata Nexus Web Services..."

# Stop current dev server if running
echo "Stopping any running dev servers..."
pkill -f "npm run dev"
pkill -f "node backend-server.js"

# Install new dependencies
echo "Installing dependencies..."
cd /opt/automata-nexus/app
npm install express cors sqlite3 sqlite concurrently

# Copy backend server
echo "Copying backend server..."
cp /home/Automata/Development/projects/building-automation-controller/backend-server.js /opt/automata-nexus/app/

# Build production version
echo "Building production version..."
npm run build

# Copy service files
echo "Installing systemd services..."
sudo cp /home/Automata/Development/projects/building-automation-controller/services/automata-nexus-backend.service /etc/systemd/system/
sudo cp /home/Automata/Development/projects/building-automation-controller/services/automata-nexus-frontend.service /etc/systemd/system/

# Reload systemd
sudo systemctl daemon-reload

# Stop the old Tauri service
echo "Stopping old Tauri service..."
sudo systemctl stop automata-nexus-control-center
sudo systemctl disable automata-nexus-control-center

# Enable new services
echo "Enabling new services..."
sudo systemctl enable automata-nexus-backend
sudo systemctl enable automata-nexus-frontend

# Start services
echo "Starting services..."
sudo systemctl start automata-nexus-backend
sleep 2
sudo systemctl start automata-nexus-frontend

# Check status
echo ""
echo "Service Status:"
sudo systemctl status automata-nexus-backend --no-pager
echo ""
sudo systemctl status automata-nexus-frontend --no-pager

echo ""
echo "Installation complete!"
echo "Access the web interface at: http://$(hostname -I | awk '{print $1}'):1420"
echo ""
echo "To check logs:"
echo "  sudo journalctl -u automata-nexus-backend -f"
echo "  sudo journalctl -u automata-nexus-frontend -f"