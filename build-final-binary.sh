#!/bin/bash

# Build FINAL Rust/egui Nexus Controller with ALL REAL HARDWARE
# NO SIMULATIONS - EVERYTHING LIVE

set -e

# Colors
GREEN='\033[0;32m'
CYAN='\033[0;36m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${CYAN}═══════════════════════════════════════════════════════════════════${NC}"
echo -e "${CYAN}    Building FINAL Nexus Controller - REAL HARDWARE ONLY${NC}"
echo -e "${CYAN}═══════════════════════════════════════════════════════════════════${NC}"

VERSION="3.0.0"
BUILD_DATE=$(date +%Y%m%d%H%M)
RELEASE_NAME="nexus-controller-v${VERSION}-REAL"

# Check for Rust
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}ERROR: Rust/Cargo not installed!${NC}"
    echo "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

# Create build directory
mkdir -p build
cd build

echo -e "${CYAN}[1/8] Preparing source files...${NC}"
# Copy all source files
cp -r ../src .
cp ../Cargo.toml .
cp ../build.rs .

# Update Cargo.toml to remove rand dependency since we're using REAL hardware
echo -e "${CYAN}[2/8] Removing simulation dependencies...${NC}"
sed -i '/rand = /d' Cargo.toml

echo -e "${CYAN}[3/8] Installing target for ARM64...${NC}"
rustup target add aarch64-unknown-linux-gnu || true

echo -e "${CYAN}[4/8] Building release binary for Raspberry Pi 5...${NC}"
# Build with optimizations for Pi5's Cortex-A76
export RUSTFLAGS='-C target-cpu=cortex-a76 -C opt-level=3 -C lto=fat'
cargo build --release --target aarch64-unknown-linux-gnu

echo -e "${CYAN}[5/8] Creating installer package...${NC}"
mkdir -p ../release
cp target/aarch64-unknown-linux-gnu/release/nexus-controller ../release/

# Create the full installer
cat > ../release/install-nexus-real.sh << 'INSTALLER'
#!/bin/bash

# AUTOMATA NEXUS CONTROLLER v3.0 - REAL HARDWARE EDITION
# NO SIMULATIONS - DIRECT HARDWARE CONTROL

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${CYAN}════════════════════════════════════════════════════════════${NC}"
echo -e "${CYAN}   AUTOMATA NEXUS CONTROLLER - REAL HARDWARE EDITION${NC}"
echo -e "${CYAN}════════════════════════════════════════════════════════════${NC}"
echo ""
echo -e "${GREEN}This version controls REAL hardware:${NC}"
echo "  ✓ MegaBAS boards (4 triacs, 4 analog out, 8 inputs)"
echo "  ✓ 16univin boards (16 universal INPUTS)"
echo "  ✓ 16uout boards (16 analog OUTPUTS)"
echo "  ✓ 8relind/16relind boards (relay outputs)"
echo "  ✓ P499 pressure transducers (0.5-4.5V = 0-500 PSI)"
echo "  ✓ WTVB01-485 vibration sensors via Modbus RTU"
echo ""
echo -e "${RED}WARNING: This controls LIVE equipment!${NC}"
read -p "Continue installation? (y/n): " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    exit 1
fi

# Check if running on Raspberry Pi
if [ ! -f /proc/device-tree/model ]; then
    echo -e "${RED}ERROR: Not running on Raspberry Pi!${NC}"
    exit 1
fi

# Check architecture
ARCH=$(uname -m)
if [ "$ARCH" != "aarch64" ]; then
    echo -e "${RED}ERROR: Requires 64-bit OS (found: $ARCH)${NC}"
    exit 1
fi

echo -e "${CYAN}Installing system dependencies...${NC}"
sudo apt-get update
sudo apt-get install -y \
    i2c-tools \
    python3-smbus \
    python3-serial \
    python3-pymodbus \
    sqlite3 \
    libsqlite3-dev \
    libssl-dev \
    libgtk-3-dev \
    libgl1-mesa-dev \
    libegl1-mesa-dev

echo -e "${CYAN}Enabling I2C interface...${NC}"
sudo raspi-config nonint do_i2c 0

echo -e "${CYAN}Installing Sequent Microsystems board drivers...${NC}"
REPOS=(
    "https://github.com/SequentMicrosystems/megabas-rpi.git"
    "https://github.com/SequentMicrosystems/16relind-rpi.git"
    "https://github.com/SequentMicrosystems/8relind-rpi.git"
    "https://github.com/SequentMicrosystems/16univin-rpi.git"
    "https://github.com/SequentMicrosystems/16uout-rpi.git"
)

for repo in "${REPOS[@]}"; do
    name=$(basename $repo .git)
    echo "Installing $name..."
    if [ ! -d "/opt/nexus/firmware/$name" ]; then
        sudo git clone $repo /opt/nexus/firmware/$name
    fi
    cd /opt/nexus/firmware/$name
    sudo make && sudo make install
done

echo -e "${CYAN}Creating Nexus directories...${NC}"
sudo mkdir -p /opt/nexus
sudo mkdir -p /var/lib/nexus
sudo mkdir -p /var/log/nexus
sudo mkdir -p /etc/nexus

echo -e "${CYAN}Installing Nexus Controller...${NC}"
sudo cp nexus-controller /usr/local/bin/
sudo chmod +x /usr/local/bin/nexus-controller

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

CREATE TABLE IF NOT EXISTS configuration (
    key TEXT PRIMARY KEY,
    value TEXT,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_sensor_timestamp ON sensor_data(timestamp);
PRAGMA journal_mode=WAL;
PRAGMA synchronous=NORMAL;
SQL

sudo chown -R pi:pi /opt/nexus
sudo chown -R pi:pi /var/lib/nexus
sudo chown -R pi:pi /var/log/nexus

echo -e "${CYAN}Starting Nexus Controller...${NC}"
sudo systemctl daemon-reload
sudo systemctl enable nexus.service
sudo systemctl start nexus.service

echo -e "${GREEN}════════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}✅ Installation Complete!${NC}"
echo -e "${GREEN}════════════════════════════════════════════════════════════${NC}"
echo ""
echo "Access the controller at: http://$(hostname -I | awk '{print $1}'):8080"
echo ""
echo "Test commands:"
echo "  megabas 0 board     - Check MegaBAS board"
echo "  16univin 0 board    - Check 16univin board"
echo "  sudo systemctl status nexus - Check service status"
echo "  journalctl -u nexus -f     - View logs"
echo ""
echo -e "${CYAN}REAL HARDWARE IS NOW ACTIVE!${NC}"
INSTALLER

chmod +x ../release/install-nexus-real.sh

echo -e "${CYAN}[6/8] Creating self-extracting archive...${NC}"
cd ../release
tar czf nexus-payload.tar.gz nexus-controller install-nexus-real.sh

echo -e "${CYAN}[7/8] Creating final installer...${NC}"
cat > ${RELEASE_NAME}.run << 'HEADER'
#!/bin/bash
# Self-extracting installer for Nexus Controller v3.0
PAYLOAD_LINE=`awk '/^__PAYLOAD_BEGINS__/ { print NR + 1; exit 0; }' $0`
tail -n +$PAYLOAD_LINE $0 | tar xzf - -C /tmp
cd /tmp && ./install-nexus-real.sh
exit 0
__PAYLOAD_BEGINS__
HEADER

cat nexus-payload.tar.gz >> ${RELEASE_NAME}.run
chmod +x ${RELEASE_NAME}.run

echo -e "${CYAN}[8/8] Calculating checksums...${NC}"
MD5=$(md5sum ${RELEASE_NAME}.run | cut -d' ' -f1)
SIZE=$(du -h ${RELEASE_NAME}.run | cut -f1)

# Clean up
rm nexus-payload.tar.gz
cd ..
rm -rf build

echo -e "${GREEN}═══════════════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}✅ BUILD COMPLETE - REAL HARDWARE EDITION!${NC}"
echo -e "${GREEN}═══════════════════════════════════════════════════════════════════${NC}"
echo ""
echo -e "Installer: ${GREEN}release/${RELEASE_NAME}.run${NC}"
echo -e "Version: ${GREEN}${VERSION}${NC}"
echo -e "Build: ${GREEN}${BUILD_DATE}${NC}"
echo -e "Size: ${GREEN}${SIZE}${NC}"
echo -e "MD5: ${GREEN}${MD5}${NC}"
echo ""
echo -e "${CYAN}This build includes:${NC}"
echo "  • ALL UI modules using REAL hardware calls"
echo "  • NO simulated data - everything reads from actual boards"
echo "  • Direct control of triacs, analog outputs, relays"
echo "  • Real-time pressure readings from P499 transducers"
echo "  • SQLite database with NVMe optimizations"
echo "  • Systemd service for automatic startup"
echo ""
echo -e "${CYAN}To install on Raspberry Pi 5:${NC}"
echo "  1. Copy: scp release/${RELEASE_NAME}.run pi@<ip>:/home/pi/"
echo "  2. Run: chmod +x ${RELEASE_NAME}.run"
echo "  3. Install: sudo ./${RELEASE_NAME}.run"
echo ""
echo -e "${RED}⚠️  WARNING: This version controls REAL hardware!${NC}"
echo -e "${RED}    Ensure all equipment is safe before running.${NC}"