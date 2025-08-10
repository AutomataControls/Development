#!/bin/bash

# Create FINAL installer with ALL REAL HARDWARE implementations
# This packages the corrected source code for building on target

set -e

VERSION="3.0.0-REAL"
BUILD_DATE=$(date +%Y%m%d%H%M)
RELEASE_NAME="nexus-installer-v${VERSION}"

echo "Creating FINAL installer with REAL hardware control..."

# Create release directory
rm -rf release
mkdir -p release/nexus

# Copy all corrected source files
echo "Copying source files with REAL implementations..."
cp -r src release/nexus/
cp Cargo.toml release/nexus/
cp build.rs release/nexus/

# Copy installer and scripts
cp -r installer release/nexus/

# Create the main installer script
cat > release/nexus/install.sh << 'INSTALLER_SCRIPT'
#!/bin/bash

# AUTOMATA NEXUS CONTROLLER v3.0 - REAL HARDWARE CONTROL
# ALL SIMULATIONS REMOVED - DIRECT HARDWARE ACCESS ONLY

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${CYAN}═══════════════════════════════════════════════════════════════════${NC}"
echo -e "${CYAN}   AUTOMATA NEXUS v3.0 - REAL HARDWARE CONTROL EDITION${NC}"
echo -e "${CYAN}═══════════════════════════════════════════════════════════════════${NC}"
echo ""
echo -e "${GREEN}This version features:${NC}"
echo "  ✓ REAL sensor readings from MegaBAS/16univin boards"
echo "  ✓ REAL control outputs to triacs and analog outputs"
echo "  ✓ REAL P499 pressure transducer readings (0.5-4.5V)"
echo "  ✓ REAL Modbus/BACnet device communication"
echo "  ✓ REAL SQLite database operations"
echo "  ✓ REAL git firmware updates"
echo "  ✓ REAL system command execution"
echo ""
echo -e "${RED}NO SIMULATIONS - Everything connects to actual hardware!${NC}"
echo ""

# Check architecture
if [ "$(uname -m)" != "aarch64" ]; then
    echo -e "${RED}ERROR: Requires ARM64/aarch64 architecture${NC}"
    exit 1
fi

# Install dependencies
echo -e "${CYAN}Installing system dependencies...${NC}"
sudo apt-get update
sudo apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    libsqlite3-dev \
    sqlite3 \
    libgtk-3-dev \
    libgl1-mesa-dev \
    libegl1-mesa-dev \
    i2c-tools \
    python3-smbus \
    python3-serial \
    python3-pymodbus \
    git \
    make \
    gcc \
    curl

# Install Rust if not present
if ! command -v cargo &> /dev/null; then
    echo -e "${CYAN}Installing Rust...${NC}"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

# Enable I2C
echo -e "${CYAN}Enabling I2C interface...${NC}"
sudo raspi-config nonint do_i2c 0

# Install board drivers
echo -e "${CYAN}Installing Sequent Microsystems board drivers...${NC}"
REPOS=(
    "https://github.com/SequentMicrosystems/megabas-rpi.git"
    "https://github.com/SequentMicrosystems/16relind-rpi.git"
    "https://github.com/SequentMicrosystems/8relind-rpi.git"
    "https://github.com/SequentMicrosystems/16univin-rpi.git"
    "https://github.com/SequentMicrosystems/16uout-rpi.git"
)

sudo mkdir -p /opt/nexus/firmware
for repo in "${REPOS[@]}"; do
    name=$(basename $repo .git)
    echo "Installing $name..."
    if [ ! -d "/opt/nexus/firmware/$name" ]; then
        sudo git clone $repo /opt/nexus/firmware/$name
    fi
    cd /opt/nexus/firmware/$name
    sudo make && sudo make install
done

# Copy application files
echo -e "${CYAN}Installing Nexus application...${NC}"
sudo mkdir -p /opt/nexus
sudo cp -r * /opt/nexus/
cd /opt/nexus

# Build the application
echo -e "${CYAN}Building Nexus Controller (this may take a while)...${NC}"
export RUSTFLAGS='-C target-cpu=cortex-a76 -C opt-level=3'
cargo build --release

# Install binary
sudo cp target/release/nexus-controller /usr/local/bin/
sudo chmod +x /usr/local/bin/nexus-controller

# Create directories
sudo mkdir -p /var/lib/nexus
sudo mkdir -p /var/log/nexus
sudo mkdir -p /etc/nexus

# Setup database
echo -e "${CYAN}Setting up database...${NC}"
sudo sqlite3 /var/lib/nexus/nexus.db << 'SQL'
CREATE TABLE IF NOT EXISTS sensor_data (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
    board_type TEXT,
    board_stack INTEGER,
    channel INTEGER,
    value REAL,
    units TEXT
);

CREATE TABLE IF NOT EXISTS board_config (
    board_id TEXT PRIMARY KEY,
    board_type TEXT,
    stack_level INTEGER,
    enabled BOOLEAN,
    config TEXT
);

CREATE TABLE IF NOT EXISTS logic_files (
    id TEXT PRIMARY KEY,
    name TEXT,
    content TEXT,
    active BOOLEAN,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_sensor_timestamp ON sensor_data(timestamp);
PRAGMA journal_mode=WAL;
PRAGMA synchronous=NORMAL;
SQL

# Create systemd service
echo -e "${CYAN}Creating systemd service...${NC}"
sudo cat > /etc/systemd/system/nexus.service << 'SERVICE'
[Unit]
Description=Automata Nexus Controller - Real Hardware
After=network.target

[Service]
Type=simple
User=pi
WorkingDirectory=/opt/nexus
ExecStart=/usr/local/bin/nexus-controller
Restart=always
RestartSec=10
Environment="RUST_LOG=info"

[Install]
WantedBy=multi-user.target
SERVICE

# Set permissions
sudo chown -R pi:pi /opt/nexus
sudo chown -R pi:pi /var/lib/nexus
sudo chown -R pi:pi /var/log/nexus

# Start service
echo -e "${CYAN}Starting Nexus Controller...${NC}"
sudo systemctl daemon-reload
sudo systemctl enable nexus.service
sudo systemctl start nexus.service

echo -e "${GREEN}═══════════════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}✅ Installation Complete!${NC}"
echo -e "${GREEN}═══════════════════════════════════════════════════════════════════${NC}"
echo ""
echo "The Nexus Controller is now running with REAL hardware control!"
echo ""
echo "Access: http://$(hostname -I | awk '{print $1}'):8080"
echo ""
echo "Test commands:"
echo "  megabas 0 board         - Check MegaBAS board"
echo "  megabas 0 ain 1         - Read analog input 1"
echo "  megabas 0 aout 1 5.0    - Set analog output 1 to 5V"
echo "  megabas 0 triac 1 50    - Set triac 1 to 50%"
echo "  16univin 0 board        - Check 16univin board"
echo "  systemctl status nexus  - Check service status"
echo "  journalctl -u nexus -f  - View live logs"
echo ""
echo -e "${CYAN}All UI modules are now connected to REAL hardware!${NC}"
INSTALLER_SCRIPT

chmod +x release/nexus/install.sh

# Create self-extracting archive
echo "Creating self-extracting installer..."
cd release
tar czf nexus-payload.tar.gz nexus/

cat > ${RELEASE_NAME}.run << 'HEADER'
#!/bin/bash
# Automata Nexus Controller v3.0 - REAL Hardware Edition
echo "Extracting Nexus Controller..."
PAYLOAD_LINE=`awk '/^__PAYLOAD_BEGINS__/ { print NR + 1; exit 0; }' $0`
tail -n +$PAYLOAD_LINE $0 | tar xzf - -C /tmp
cd /tmp/nexus && ./install.sh
exit 0
__PAYLOAD_BEGINS__
HEADER

cat nexus-payload.tar.gz >> ${RELEASE_NAME}.run
chmod +x ${RELEASE_NAME}.run

# Calculate size and checksum
SIZE=$(du -h ${RELEASE_NAME}.run | cut -f1)
MD5=$(md5sum ${RELEASE_NAME}.run | cut -d' ' -f1)

# Clean up
rm -rf nexus nexus-payload.tar.gz

echo ""
echo "═══════════════════════════════════════════════════════════════════"
echo "✅ FINAL INSTALLER CREATED - REAL HARDWARE EDITION!"
echo "═══════════════════════════════════════════════════════════════════"
echo ""
echo "Installer: release/${RELEASE_NAME}.run"
echo "Version: ${VERSION}"
echo "Build: ${BUILD_DATE}"
echo "Size: ${SIZE}"
echo "MD5: ${MD5}"
echo ""
echo "This installer includes:"
echo "  • ALL UI modules with REAL hardware calls"
echo "  • NO simulated/mock/fake data"
echo "  • Direct board control via megabas/16univin commands"
echo "  • Real P499 pressure transducer support"
echo "  • Real Modbus/BACnet communication"
echo "  • Real SQLite database operations"
echo "  • Real git firmware updates"
echo ""
echo "To install:"
echo "  1. Copy to Pi5: scp release/${RELEASE_NAME}.run pi@raspberry:/home/pi/"
echo "  2. Make executable: chmod +x ${RELEASE_NAME}.run"
echo "  3. Run: sudo ./${RELEASE_NAME}.run"
echo ""
echo "⚠️  This version controls REAL equipment - no simulations!"