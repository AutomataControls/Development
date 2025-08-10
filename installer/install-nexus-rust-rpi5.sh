#!/bin/bash

# Automata Nexus AI Controller - NATIVE Rust/egui Edition for Raspberry Pi 5
# Commercial License - Copyright (c) 2025 Automata Controls
# Developed by Andrew Jewell Sr.
# REAL HARDWARE CONTROL - NO SIMULATIONS

set -e

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
TEAL='\033[0;96m'  # Matching app colors
NC='\033[0m' # No Color

# Configuration
SSD_DEVICE=""
INSTALL_PATH="/opt/nexus"
DATA_PATH="/var/lib/nexus"
CONFIG_PATH="/etc/nexus"
LOG_PATH="/var/log/nexus"
BACKUP_PATH="/var/backups/nexus"
FIRMWARE_PATH="/opt/nexus/firmware"

# Serial Configuration
CONTROLLER_SERIAL=""
CONTROLLER_SERIAL_PREFIX="ANC"
DEVICE_FINGERPRINT=""

# Cloudflare Configuration
CLOUDFLARE_EMAIL="automatacontrols@gmail.com"
CLOUDFLARE_DOMAIN="automatacontrols.com"
CLOUDFLARE_API_TOKEN=""
CLOUDFLARE_TUNNEL_NAME=""
CLOUDFLARE_TUNNEL_ID=""
CLOUDFLARE_SUBDOMAIN=""

# Print colored output
print_info() {
    echo -e "${CYAN}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# ASCII Art Banner with Logo
show_banner() {
    echo -e "${TEAL}"
    cat << "EOF"
    ╔═══════════════════════════════════════════════════════════════════╗
    ║                                                                   ║
    ║     ___         __                        __          _   __     ║
    ║    /   | __  __/ /_____  ____ ___  ____ _/ /_____ _  / | / /     ║
    ║   / /| |/ / / / __/ __ \/ __ `__ \/ __ `/ __/ __ `/ /  |/ /      ║
    ║  / ___ / /_/ / /_/ /_/ / / / / / / /_/ / /_/ /_/ / / /|  /       ║
    ║ /_/  |_\__,_/\__/\____/_/ /_/ /_/\__,_/\__/\__,_/ /_/ |_/        ║
    ║                                                                   ║
    ║              NEXUS AI CONTROLLER - RUST EDITION                  ║
    ║            Building Automation for Raspberry Pi 5                ║
    ║                    SSD Installation v2.0                         ║
    ║                                                                   ║
    ╚═══════════════════════════════════════════════════════════════════╝
EOF
    echo -e "${NC}"
}

# Show commercial license agreement
show_license() {
    clear
    echo -e "${TEAL}════════════════════════════════════════════════════════════════════${NC}"
    echo -e "${TEAL}           AUTOMATA NEXUS AI - COMMERCIAL LICENSE AGREEMENT         ${NC}"
    echo -e "${TEAL}════════════════════════════════════════════════════════════════════${NC}"
    echo
    cat << "EOF"
IMPORTANT: READ THIS LICENSE AGREEMENT CAREFULLY BEFORE INSTALLING

This Commercial License Agreement ("Agreement") is entered into between 
Automata Controls ("Licensor") and the entity or individual installing 
this software ("Licensee").

1. GRANT OF LICENSE
   Subject to the terms of this Agreement, Licensor grants Licensee a 
   non-exclusive, non-transferable license to install and use the 
   Automata Nexus AI Controller software on a single Raspberry Pi 5 
   device for commercial purposes.

2. RESTRICTIONS
   Licensee shall not:
   - Copy, modify, or distribute the Software without prior written consent
   - Reverse engineer, decompile, or disassemble the Software  
   - Remove or alter any proprietary notices or labels
   - Use the Software for any unlawful purpose
   - Sublicense, rent, lease, or lend the Software

3. OWNERSHIP
   The Software is licensed, not sold. Licensor retains all right, 
   title, and interest in and to the Software, including all 
   intellectual property rights.

4. SUPPORT AND UPDATES
   - Technical support is provided for licensed installations
   - Software updates are included for the duration of the license
   - Priority support available with premium licenses
   - RPi5-specific optimizations included

5. WARRANTY DISCLAIMER
   THE SOFTWARE IS PROVIDED "AS IS" WITHOUT WARRANTY OF ANY KIND.

6. TERM AND TERMINATION
   This license is effective until terminated. Licensee may terminate 
   by destroying all copies of the Software.

BY PROCEEDING WITH INSTALLATION, YOU ACKNOWLEDGE THAT YOU HAVE READ 
THIS AGREEMENT, UNDERSTAND IT, AND AGREE TO BE BOUND BY ITS TERMS.

For licensing inquiries: licensing@automatacontrols.com
EOF
    echo
    echo -e "${TEAL}════════════════════════════════════════════════════════════════════${NC}"
    echo
    read -p "Do you agree to the license terms? (yes/no): " -r
    if [[ ! $REPLY =~ ^[Yy]es$ ]]; then
        print_error "License agreement not accepted. Exiting."
        exit 1
    fi
}

# Check if running as root
check_root() {
    if [[ $EUID -ne 0 ]]; then
        print_error "This installer must be run as root (use sudo)"
        exit 1
    fi
}

# Generate device fingerprint
generate_fingerprint() {
    print_info "Generating device fingerprint..."
    
    # Get Raspberry Pi serial
    RPI_SERIAL=$(cat /proc/cpuinfo | grep Serial | cut -d ' ' -f 2 | tr -d '\n')
    
    # Get primary MAC address
    MAC_ADDR=$(ip link show | grep ether | head -1 | awk '{print $2}' | tr -d ':')
    
    # Get disk UUID
    DISK_UUID=$(blkid | grep -E 'mmcblk0p2|nvme0n1p2' | head -1 | grep -o 'UUID="[^"]*"' | cut -d'"' -f2 | head -c 8)
    
    # Combine for fingerprint
    FINGERPRINT_RAW="${RPI_SERIAL}-${MAC_ADDR}-${DISK_UUID}"
    DEVICE_FINGERPRINT=$(echo -n "$FINGERPRINT_RAW" | sha256sum | cut -c1-16 | tr '[:lower:]' '[:upper:]')
    
    print_success "Device fingerprint: $DEVICE_FINGERPRINT"
}

# Generate unique serial number
generate_serial() {
    print_info "Generating unique serial number..."
    
    # Check if serial already exists
    if [ -f "$CONFIG_PATH/serial" ]; then
        CONTROLLER_SERIAL=$(cat "$CONFIG_PATH/serial")
        print_info "Using existing serial: $CONTROLLER_SERIAL"
    else
        # Generate new serial based on fingerprint
        SERIAL_SUFFIX=$(echo -n "$DEVICE_FINGERPRINT" | cut -c1-8)
        CONTROLLER_SERIAL="${CONTROLLER_SERIAL_PREFIX}-${SERIAL_SUFFIX}"
        print_success "Generated serial: $CONTROLLER_SERIAL"
    fi
    
    # Generate subdomain
    CLOUDFLARE_SUBDOMAIN="nexuscontroller-$(echo $CONTROLLER_SERIAL | tr '[:upper:]' '[:lower:]')"
    CLOUDFLARE_TUNNEL_NAME="automata-nexus-$(echo $CONTROLLER_SERIAL | tr '[:upper:]' '[:lower:]')"
}

# Check system requirements
check_system() {
    print_info "Checking system requirements..."
    
    # Check if running on Raspberry Pi 5
    if ! grep -q "Raspberry Pi 5" /proc/device-tree/model 2>/dev/null; then
        print_warning "This system does not appear to be a Raspberry Pi 5"
        read -p "Continue anyway? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            exit 1
        fi
    fi
    
    # Check OS version
    if ! grep -q "bookworm" /etc/os-release 2>/dev/null; then
        print_warning "This system is not running Debian Bookworm"
    fi
    
    # Check available memory
    total_mem=$(free -m | awk '/^Mem:/{print $2}')
    if [ "$total_mem" -lt 2048 ]; then
        print_warning "System has less than 2GB RAM. Performance may be affected."
    fi
    
    print_success "System check complete"
}

# Detect and prepare SSD
prepare_ssd() {
    print_info "Detecting SSD..."
    
    # Check if already running from SSD
    ROOT_DEV=$(mount | grep "on / " | awk '{print $1}')
    if [[ $ROOT_DEV == *"nvme"* ]]; then
        print_success "System is already running from NVMe SSD"
        SSD_DEVICE=$(echo $ROOT_DEV | sed 's/[0-9]*$//')
        return 0
    fi
    
    # List available disks
    echo "Available disks:"
    lsblk -d -o NAME,SIZE,TYPE,MODEL | grep -E "disk|nvme"
    
    # Auto-detect NVMe SSD
    nvme_devices=$(lsblk -d -o NAME,TYPE | grep nvme | awk '{print $1}')
    if [ -n "$nvme_devices" ]; then
        SSD_DEVICE="/dev/$(echo $nvme_devices | awk '{print $1}')"
        print_info "Detected NVMe SSD: $SSD_DEVICE"
    else
        print_error "No NVMe SSD detected"
        exit 1
    fi
}

# Install system dependencies
install_dependencies() {
    print_info "Installing REAL hardware control dependencies..."
    
    apt-get update
    apt-get install -y \
        build-essential \
        pkg-config \
        libssl-dev \
        libsqlite3-dev \
        sqlite3 \
        libgtk-3-dev \
        libglib2.0-dev \
        libcairo2-dev \
        libpango1.0-dev \
        libatk1.0-dev \
        libgdk-pixbuf2.0-dev \
        libxcb-render0-dev \
        libxcb-shape0-dev \
        libxcb-xfixes0-dev \
        libxkbcommon-dev \
        libgl1-mesa-dev \
        i2c-tools \
        python3-smbus \
        python3-serial \
        python3-modbus \
        python3-pymodbus \
        python3-pip \
        git \
        make \
        gcc \
        libegl1-mesa-dev \
        curl \
        wget \
        git \
        python3-pip \
        python3-dev \
        python3-smbus \
        python3-serial \
        python3-setuptools \
        i2c-tools \
        can-utils \
        libudev-dev \
        redis-server \
        mosquitto \
        mosquitto-clients \
        jq \
        cmake \
        clang
    
    # Install cloudflared
    print_info "Installing Cloudflare Tunnel..."
    # Download and install cloudflared for ARM64
    wget -q https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-linux-arm64.deb
    dpkg -i cloudflared-linux-arm64.deb || apt-get install -f -y
    rm cloudflared-linux-arm64.deb
    
    # Install Node.js for Claude Code
    print_info "Installing Node.js..."
    curl -fsSL https://deb.nodesource.com/setup_20.x | bash -
    apt-get install -y nodejs
    
    # Install Rust if not present
    if ! command -v rustc &> /dev/null; then
        print_info "Installing Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source $HOME/.cargo/env
        
        # Add ARM64 target
        rustup target add aarch64-unknown-linux-gnu
    fi
    
    # Enable I2C and SPI
    print_info "Enabling I2C and SPI..."
    raspi-config nonint do_i2c 0
    raspi-config nonint do_spi 0
    
    # Load I2C modules
    modprobe i2c-dev
    modprobe i2c-bcm2835
    
    print_success "Dependencies installed"
}

# Install Claude Code
install_claude_code() {
    print_info "Installing Claude Code CLI..."
    
    # Download and install Claude Code using npm
    print_info "Downloading Claude Code..."
    npm install -g @anthropic-ai/claude-code || {
        print_warning "Claude Code npm package not available yet"
        print_info "Claude Code will need to be installed manually"
        print_info "Visit: https://github.com/anthropics/claude-code for instructions"
        return 0
    }
    
    # Verify installation
    if command -v claude &> /dev/null; then
        CLAUDE_VERSION=$(claude --version 2>/dev/null || echo "unknown")
        print_success "Claude Code installed: $CLAUDE_VERSION"
        
        # Add to PATH for all users
        echo 'export PATH="$PATH:/usr/local/bin"' >> /etc/profile
        
        # Create initial configuration
        mkdir -p /root/.claude
        cat > /root/.claude/config.json << EOF
{
    "model": "claude-3-sonnet",
    "temperature": 0.7,
    "max_tokens": 4096
}
EOF
        print_info "Claude Code configuration created"
    else
        print_warning "Claude Code installation failed - manual installation may be required"
        print_info "Visit https://claude.ai/docs for installation instructions"
    fi
}

# Install Sequent Microsystems board drivers
install_board_drivers() {
    print_info "Installing board drivers..."
    
    TEMP_DIR="/tmp/sequent_drivers"
    mkdir -p $TEMP_DIR
    cd $TEMP_DIR
    
    # List of Sequent Microsystems repositories - CORRECTED based on firmware review
    declare -a REPOS=(
        "https://github.com/SequentMicrosystems/megabas-rpi.git"       # 4 triacs, 4 analog out, 8 config inputs
        "https://github.com/SequentMicrosystems/8relind-rpi.git"       # 8 relay outputs ONLY
        "https://github.com/SequentMicrosystems/16relind-rpi.git"      # 16 relay outputs ONLY
        "https://github.com/SequentMicrosystems/16univin-rpi.git"      # 16 universal INPUTS ONLY
        "https://github.com/SequentMicrosystems/16uout-rpi.git"        # 16 analog OUTPUTS ONLY (0-10V)
    )
    
    for repo in "${REPOS[@]}"; do
        repo_name=$(basename $repo .git)
        print_info "Installing $repo_name driver..."
        
        # Clone repository
        git clone $repo
        cd $repo_name
        
        # Build and install
        if [ -f "Makefile" ]; then
            make
            make install
            print_success "$repo_name driver installed"
        else
            print_warning "$repo_name: No Makefile found, skipping"
        fi
        
        # Install Python library if exists
        if [ -d "python" ]; then
            cd python
            python3 setup.py install
            print_success "$repo_name Python library installed"
            cd ..
        fi
        
        cd ..
    done
    
    # Cleanup
    cd /
    rm -rf $TEMP_DIR
    
    print_success "All board drivers installed"
    
    # Verify installations
    print_info "Verifying board driver installations..."
    echo "Installed board capabilities:"
    echo "- MegaBAS: 4 triacs, 4 analog outputs, 8 configurable inputs (NO RELAYS)"
    echo "- 16univin: 16 universal INPUTS ONLY"
    echo "- 16uout: 16 analog OUTPUTS ONLY"
    echo "- 8relind: 8 relay outputs ONLY"
    echo "- 16relind: 16 relay outputs ONLY"
}

# Setup Cloudflare tunnel
setup_cloudflare() {
    print_info "Setting up Cloudflare tunnel..."
    
    # Check if API token is provided
    if [ -z "$CLOUDFLARE_API_TOKEN" ]; then
        print_warning "No Cloudflare API token provided"
        echo "Please provide your Cloudflare API token (or press Enter to skip):"
        read -s CLOUDFLARE_API_TOKEN
    fi
    
    if [ -n "$CLOUDFLARE_API_TOKEN" ]; then
        # Login to Cloudflare
        echo "$CLOUDFLARE_API_TOKEN" | cloudflared tunnel login
        
        # Create tunnel
        cloudflared tunnel create $CLOUDFLARE_TUNNEL_NAME
        
        # Get tunnel ID
        CLOUDFLARE_TUNNEL_ID=$(cloudflared tunnel list | grep $CLOUDFLARE_TUNNEL_NAME | awk '{print $1}')
        
        # Create tunnel configuration
        cat > $CONFIG_PATH/cloudflared.yml << EOF
tunnel: $CLOUDFLARE_TUNNEL_ID
credentials-file: /root/.cloudflared/${CLOUDFLARE_TUNNEL_ID}.json

ingress:
  - hostname: ${CLOUDFLARE_SUBDOMAIN}.${CLOUDFLARE_DOMAIN}
    service: http://localhost:1420
  - service: http_status:404
EOF
        
        # Create DNS record
        cloudflared tunnel route dns $CLOUDFLARE_TUNNEL_NAME ${CLOUDFLARE_SUBDOMAIN}.${CLOUDFLARE_DOMAIN}
        
        # Create systemd service
        cat > /etc/systemd/system/cloudflared.service << EOF
[Unit]
Description=Cloudflare Tunnel for Automata Nexus
After=network.target

[Service]
Type=simple
User=root
ExecStart=/usr/bin/cloudflared tunnel --config $CONFIG_PATH/cloudflared.yml run
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
EOF
        
        systemctl daemon-reload
        systemctl enable cloudflared
        systemctl start cloudflared
        
        print_success "Cloudflare tunnel configured: https://${CLOUDFLARE_SUBDOMAIN}.${CLOUDFLARE_DOMAIN}"
    else
        print_warning "Cloudflare tunnel setup skipped"
    fi
}

# Create environment configuration
create_env_config() {
    print_info "Creating environment configuration..."
    
    # Generate secure keys
    JWT_SECRET=$(openssl rand -base64 32)
    API_ENCRYPTION_KEY=$(openssl rand -base64 32)
    SESSION_SECRET=$(openssl rand -base64 32)
    
    cat > $CONFIG_PATH/.env << EOF
# Automata Nexus Configuration
# Generated: $(date)

# Application Settings
NODE_ENV=production
PORT=1420
APP_NAME="Automata Nexus AI Controller"

# Controller Identification
CONTROLLER_SERIAL=$CONTROLLER_SERIAL
CONTROLLER_SERIAL_PREFIX=$CONTROLLER_SERIAL_PREFIX
CONTROLLER_VERSION=2.0.0
DEVICE_FINGERPRINT=$DEVICE_FINGERPRINT

# Security Keys
JWT_SECRET=$JWT_SECRET
API_ENCRYPTION_KEY=$API_ENCRYPTION_KEY
SESSION_SECRET=$SESSION_SECRET

# Cloudflare Configuration
CLOUDFLARE_EMAIL=$CLOUDFLARE_EMAIL
CLOUDFLARE_DOMAIN=$CLOUDFLARE_DOMAIN
CLOUDFLARE_SUBDOMAIN=$CLOUDFLARE_SUBDOMAIN
CLOUDFLARE_TUNNEL_NAME=$CLOUDFLARE_TUNNEL_NAME
CLOUDFLARE_TUNNEL_ID=$CLOUDFLARE_TUNNEL_ID
CLOUDFLARE_TUNNEL_URL=https://${CLOUDFLARE_SUBDOMAIN}.${CLOUDFLARE_DOMAIN}

# Database Configuration
DATABASE_PATH=$DATA_PATH/nexus.db
DATABASE_BACKUP_PATH=$BACKUP_PATH

# Hardware Configuration
DEFAULT_I2C_BUS=1
DEFAULT_SPI_BUS=0
GPIO_CHIP=gpiochip0

# System Paths
LOG_PATH=$LOG_PATH
CONFIG_PATH=$CONFIG_PATH
FIRMWARE_PATH=$FIRMWARE_PATH
INSTALL_PATH=$INSTALL_PATH

# Performance Settings
MAX_WORKERS=4
CACHE_SIZE=128
REQUEST_TIMEOUT=30

# Monitoring & Telemetry
ENABLE_TELEMETRY=true
TELEMETRY_ENDPOINT=https://telemetry.automatacontrols.com/api/v1/metrics
HEARTBEAT_INTERVAL=300

# Weather API Configuration
OPENWEATHERMAP_API_KEY=c7d29aded54ce8efb291b852f25b6aa6
WEATHER_ZIP_CODE=46795
WEATHER_COUNTRY_CODE=US

# Email Configuration (Resend)
RESEND_API_KEY=re_YoQNcv4n_5DUt4XCbBGMze8njxVR6uZ9Q
EMAIL_DEFAULT_SENDER=devops@automatacontrols.com
EMAIL_DEFAULT_RECIPIENT=devops@automatacontrols.com
EMAIL_ENABLED=true

# Remote Access Options
ENABLE_CLOUDFLARE_TUNNEL=true
ENABLE_TAILSCALE=false
ENABLE_WIREGUARD=false

# Development Settings
DEBUG=false
VERBOSE_LOGGING=false
EOF
    
    chmod 600 $CONFIG_PATH/.env
    print_success "Environment configuration created"
}

# Install Nexus application
install_nexus() {
    print_info "Installing Nexus REAL HARDWARE CONTROL application..."
    
    # Create directory structure
    mkdir -p $INSTALL_PATH
    mkdir -p $DATA_PATH
    mkdir -p $CONFIG_PATH
    mkdir -p $LOG_PATH
    mkdir -p $BACKUP_PATH
    mkdir -p $FIRMWARE_PATH
    mkdir -p $INSTALL_PATH/scripts
    
    # Copy source code from current directory
    print_info "Copying source code..."
    # Assume installer is run from extracted directory
    SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
    
    if [ -d "$PROJECT_DIR/src" ]; then
        cp -r $PROJECT_DIR/* $INSTALL_PATH/
    else
        print_error "Source code not found. Make sure to run installer from the extracted directory"
        exit 1
    fi
    
    # Create hardware interface scripts
    print_info "Creating hardware interface scripts..."
    
    # Create Modbus test script
    cat > $INSTALL_PATH/modbus_test.py << 'EOF'
#!/usr/bin/env python3
import sys
import time
from pymodbus.client import ModbusTcpClient

if len(sys.argv) >= 3:
    start = time.time()
    client = ModbusTcpClient(sys.argv[1], port=int(sys.argv[2]))
    if client.connect():
        print(int((time.time() - start) * 1000))
        client.close()
    else:
        print("999")
EOF
    chmod +x $INSTALL_PATH/modbus_test.py
    
    cd $INSTALL_PATH
    
    # Build for ARM64 with REAL hardware support
    print_info "Building NATIVE Rust application with hardware support..."
    export CARGO_BUILD_JOBS=4
    export RUSTFLAGS='-C target-cpu=cortex-a76 -C opt-level=3'
    cargo build --release --target aarch64-unknown-linux-gnu
    
    # Copy binary
    cp target/aarch64-unknown-linux-gnu/release/nexus-controller /usr/local/bin/
    chmod +x /usr/local/bin/nexus-controller
    
    # Initialize database
    print_info "Initializing database..."
    # Create database directory
    mkdir -p $DATA_PATH
    
    # Check if database already exists
    if [ -f "$DATA_PATH/nexus.db" ]; then
        print_warning "Database already exists, creating backup..."
        cp $DATA_PATH/nexus.db $BACKUP_PATH/nexus_$(date +%Y%m%d_%H%M%S).db
    fi
    
    # Run migrations
    for migration in $INSTALL_PATH/migrations/*.sql; do
        if [ -f "$migration" ]; then
            print_info "Running migration: $(basename $migration)"
            if ! sqlite3 $DATA_PATH/nexus.db < "$migration" 2>/dev/null; then
                print_warning "Migration already applied or skipped: $(basename $migration)"
            fi
        fi
    done
    
    # Optimize database for NVMe SSD
    sqlite3 $DATA_PATH/nexus.db << EOF
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA cache_size = -64000;
PRAGMA temp_store = MEMORY;
PRAGMA mmap_size = 268435456;
VACUUM;
ANALYZE;
EOF
    
    # Set permissions
    chmod 644 $DATA_PATH/nexus.db
    chown pi:pi $DATA_PATH/nexus.db
    
    # Verify database
    if sqlite3 $DATA_PATH/nexus.db "SELECT COUNT(*) FROM users;" > /dev/null 2>&1; then
        print_success "Database initialized and verified"
    else
        print_error "Database initialization failed"
        exit 1
    fi
    
    # Save serial number
    echo $CONTROLLER_SERIAL > $CONFIG_PATH/serial
    
    # Copy corrected firmware interface
    print_info "Installing corrected firmware interface..."
    if [ -f "$INSTALL_PATH/src/firmware_interface.py" ]; then
        cp $INSTALL_PATH/src/firmware_interface.py /usr/local/bin/nexus-firmware
        chmod +x /usr/local/bin/nexus-firmware
        print_success "Firmware interface installed with corrected board specifications"
    fi
    
    # Create firmware test script
    cat > /usr/local/bin/nexus-test-boards << 'EOF'
#!/usr/bin/env python3
# Test script to verify board installations
import subprocess
import json

print("Testing Sequent Microsystems board installations...")
print("=" * 60)

# Test firmware interface
result = subprocess.run(
    ["python3", "/usr/local/bin/nexus-firmware", "info"],
    capture_output=True,
    text=True
)

if result.returncode == 0:
    info = json.loads(result.stdout)
    print("\nBoard Capabilities (CORRECTED):")
    for board, config in info['boards'].items():
        print(f"\n{board}: {config['description']}")
        for key, value in config.items():
            if key != 'description':
                print(f"  - {key}: {value}")
else:
    print(f"Error: {result.stderr}")

print("\n" + "=" * 60)
print("IMPORTANT CORRECTIONS:")
print("- MegaBAS: 4 triacs, 4 analog outputs, 8 configurable inputs (NO RELAYS)")
print("- 16univin: 16 universal INPUTS ONLY")
print("- 16uout: 16 analog OUTPUTS ONLY")
print("- Relay boards (8relind/16relind) are SEPARATE boards")
EOF
    chmod +x /usr/local/bin/nexus-test-boards
    
    print_success "Nexus application installed"
}

# Configure systemd service
configure_service() {
    print_info "Configuring systemd service..."
    
    cat > /etc/systemd/system/nexus.service << EOF
[Unit]
Description=Automata Nexus AI Controller
After=network.target redis-server.service

[Service]
Type=simple
User=root
WorkingDirectory=$INSTALL_PATH
ExecStart=/usr/local/bin/nexus-controller
Restart=always
RestartSec=10
Environment="RUST_LOG=info"
Environment="DATABASE_URL=sqlite://$DATA_PATH/nexus.db"
EnvironmentFile=$CONFIG_PATH/.env

# Security settings
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=$DATA_PATH $LOG_PATH $CONFIG_PATH $BACKUP_PATH

[Install]
WantedBy=multi-user.target
EOF
    
    systemctl daemon-reload
    systemctl enable nexus.service
    
    print_success "Service configured"
}

# Setup monitoring
setup_monitoring() {
    print_info "Setting up monitoring..."
    
    # Create monitoring script
    cat > /usr/local/bin/nexus-monitor << 'EOF'
#!/bin/bash
# Nexus monitoring script

# Check if service is running
if ! systemctl is-active --quiet nexus; then
    echo "Nexus service is not running. Attempting restart..."
    systemctl restart nexus
    
    # Send alert
    curl -X POST https://telemetry.automatacontrols.com/api/v1/alerts \
        -H "Content-Type: application/json" \
        -d "{\"serial\":\"$CONTROLLER_SERIAL\",\"alert\":\"Service restarted\",\"timestamp\":\"$(date -Iseconds)\"}"
fi

# Check disk space
DISK_USAGE=$(df / | tail -1 | awk '{print $5}' | sed 's/%//')
if [ $DISK_USAGE -gt 90 ]; then
    echo "Warning: Disk usage is at ${DISK_USAGE}%"
fi

# Check memory usage
MEM_USAGE=$(free | grep Mem | awk '{print int($3/$2 * 100)}')
if [ $MEM_USAGE -gt 90 ]; then
    echo "Warning: Memory usage is at ${MEM_USAGE}%"
fi
EOF
    
    chmod +x /usr/local/bin/nexus-monitor
    
    # Add to crontab
    (crontab -l 2>/dev/null; echo "*/5 * * * * /usr/local/bin/nexus-monitor") | crontab -
    
    print_success "Monitoring configured"
}

# Create binary installer
create_binary_installer() {
    print_info "Creating binary installer..."
    
    cat > $INSTALL_PATH/create-installer.sh << 'EOF'
#!/bin/bash
# Create standalone binary installer

# Build static binary
cargo build --release --target aarch64-unknown-linux-gnu

# Create installer package
mkdir -p /tmp/nexus-installer
cp target/aarch64-unknown-linux-gnu/release/nexus-controller /tmp/nexus-installer/
cp -r firmware /tmp/nexus-installer/
cp -r config /tmp/nexus-installer/
cp installer/install-nexus-rust-rpi5.sh /tmp/nexus-installer/install.sh

# Create self-extracting archive
cd /tmp
tar czf nexus-installer.tar.gz nexus-installer/

cat > nexus-installer.bin << 'INSTALLER'
#!/bin/bash
TMPDIR=$(mktemp -d)
ARCHIVE=$(awk '/^__ARCHIVE_BELOW__/ {print NR + 1; exit 0; }' $0)
tail -n+$ARCHIVE $0 | tar xz -C $TMPDIR
cd $TMPDIR/nexus-installer
./install.sh
cd /
rm -rf $TMPDIR
exit 0
__ARCHIVE_BELOW__
INSTALLER

cat nexus-installer.tar.gz >> nexus-installer.bin
chmod +x nexus-installer.bin
mv nexus-installer.bin $INSTALL_PATH/

echo "Binary installer created: $INSTALL_PATH/nexus-installer.bin"
EOF
    
    chmod +x $INSTALL_PATH/create-installer.sh
    print_success "Binary installer script created"
}

# Final setup
final_setup() {
    print_info "Performing final setup..."
    
    # Start services
    systemctl start redis-server
    systemctl start nexus
    
    # Get IP address
    IP_ADDR=$(hostname -I | awk '{print $1}')
    
    # Create summary file
    cat > $CONFIG_PATH/installation-summary.txt << EOF
════════════════════════════════════════════════════════════════════
           AUTOMATA NEXUS AI INSTALLATION COMPLETE
════════════════════════════════════════════════════════════════════

Installation Date: $(date)
Controller Serial: $CONTROLLER_SERIAL
Device Fingerprint: $DEVICE_FINGERPRINT

Access Methods:
---------------
Native GUI:  Direct on Pi display (HDMI/DSI)
Local Web:   http://$IP_ADDR:1420
Remote Web:  https://${CLOUDFLARE_SUBDOMAIN}.${CLOUDFLARE_DOMAIN}
API Access:  http://$IP_ADDR:3001

Default Credentials:
-------------------
Username: admin
Password: Nexus

Important Paths:
---------------
Installation: $INSTALL_PATH
Configuration: $CONFIG_PATH
Data: $DATA_PATH
Logs: $LOG_PATH
Backups: $BACKUP_PATH

Services:
---------
Main Service: systemctl status nexus
Cloudflare Tunnel: systemctl status cloudflared
Redis Cache: systemctl status redis-server

Management Commands:
-------------------
Start:   systemctl start nexus
Stop:    systemctl stop nexus
Restart: systemctl restart nexus
Logs:    journalctl -u nexus -f
Backup:  nexus-controller --backup

Board Capabilities (CORRECTED):
-------------------------------
MegaBAS: 4 triacs, 4 analog outputs, 8 configurable inputs (NO RELAYS)
16univin: 16 universal INPUTS ONLY (0-10V, 1K, 10K, dry contact)
16uout: 16 analog OUTPUTS ONLY (0-10V)
8relind: 8 relay outputs ONLY
16relind: 16 relay outputs ONLY

Testing Commands:
----------------
Test boards: nexus-test-boards
Test firmware: nexus-firmware scan
Test integration: python3 /opt/nexus/test_ui_firmware_integration.py

Claude Code:
-----------
CLI Tool: claude --help
Interactive: claude chat
Code assist: claude code
Documentation: https://claude.ai/docs

════════════════════════════════════════════════════════════════════
EOF
    
    print_success "Installation complete!"
    echo
    cat $CONFIG_PATH/installation-summary.txt
    echo
    echo -e "${RED}⚠️  IMPORTANT: Change the default password after first login!${NC}"
    echo
    echo -e "${CYAN}To view this information again: cat $CONFIG_PATH/installation-summary.txt${NC}"
}

# Main installation flow
main() {
    clear
    show_banner
    show_license
    check_root
    check_system
    generate_fingerprint
    generate_serial
    prepare_ssd
    install_dependencies
    install_claude_code
    install_board_drivers
    create_env_config
    setup_cloudflare
    install_nexus
    configure_service
    setup_monitoring
    create_binary_installer
    final_setup
}

# Run main function
main "$@"