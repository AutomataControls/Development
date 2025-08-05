#!/bin/bash

# Automata Nexus Web Server Startup Script
# Starts the control center as a web server accessible on the network

echo "Starting Automata Nexus Control Center Web Server..."
echo "This will be accessible from any device on your network"

# Get the Pi's IP address
PI_IP=$(hostname -I | awk '{print $1}')
echo "Pi IP Address: $PI_IP"

# Build the application for production
echo "Building application..."
npm run build

# Start Next.js in production mode on all interfaces
echo "Starting web server..."
echo "Access from any device at: http://$PI_IP:3000"
echo "Press Ctrl+C to stop the server"

# Export environment variables for network access
export HOSTNAME=0.0.0.0
export PORT=3000

# Start the Next.js server
npm start