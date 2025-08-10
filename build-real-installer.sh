#!/bin/bash

# Build ACTUAL installer with source code included

set -e

VERSION="3.0.0"
INSTALLER_NAME="nexus-installer-v${VERSION}.run"

echo "Building actual installer with source code..."

# Create temporary build directory
rm -rf /tmp/nexus-build
mkdir -p /tmp/nexus-build

# Copy ALL source files
echo "Copying source files..."
cp -r src /tmp/nexus-build/
cp Cargo.toml /tmp/nexus-build/
cp build.rs /tmp/nexus-build/
cp -r installer /tmp/nexus-build/

# Create the installer script
cat > /tmp/nexus-build/install.sh << 'INSTALLER'
#!/bin/bash
set -e

echo "════════════════════════════════════════════════════════════"
echo "   AUTOMATA NEXUS CONTROLLER v3.0.0 Installation"
echo "════════════════════════════════════════════════════════════"

# Install dependencies
echo "Installing dependencies..."
sudo apt-get update
sudo apt-get install -y \
    build-essential pkg-config libssl-dev libsqlite3-dev sqlite3 \
    libgtk-3-dev libgl1-mesa-dev libegl1-mesa-dev \
    i2c-tools python3-smbus python3-serial python3-pymodbus \
    git make gcc curl

# Install Rust
if ! command -v cargo &> /dev/null; then
    echo "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

# Enable I2C
sudo raspi-config nonint do_i2c 0

# Install board drivers
echo "Installing board drivers..."
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
    if [ ! -d "/opt/nexus/firmware/$name" ]; then
        sudo git clone $repo /opt/nexus/firmware/$name
        cd /opt/nexus/firmware/$name
        sudo make && sudo make install
    fi
done

# Build and install Nexus
echo "Building Nexus Controller..."
sudo mkdir -p /opt/nexus
sudo cp -r * /opt/nexus/
cd /opt/nexus
cargo build --release
sudo cp target/release/nexus-controller /usr/local/bin/

# Setup database
sudo mkdir -p /var/lib/nexus
sudo sqlite3 /var/lib/nexus/nexus.db << SQL
CREATE TABLE IF NOT EXISTS sensor_data (
    id INTEGER PRIMARY KEY,
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
    board_type TEXT,
    channel INTEGER,
    value REAL
);
CREATE TABLE IF NOT EXISTS board_config (
    board_id TEXT PRIMARY KEY,
    config TEXT
);
SQL

# Create service
sudo tee /etc/systemd/system/nexus.service > /dev/null << SERVICE
[Unit]
Description=Automata Nexus Controller
After=network.target

[Service]
Type=simple
User=pi
ExecStart=/usr/local/bin/nexus-controller
Restart=always

[Install]
WantedBy=multi-user.target
SERVICE

# Start service
sudo systemctl daemon-reload
sudo systemctl enable nexus.service
sudo systemctl start nexus.service

echo "✅ Installation complete! Access at http://$(hostname -I | awk '{print $1}'):8080"
INSTALLER

chmod +x /tmp/nexus-build/install.sh

# Create tar archive
cd /tmp/nexus-build
tar czf ../nexus-payload.tar.gz *

# Create self-extracting installer
cd /tmp
cat > $INSTALLER_NAME << 'HEADER'
#!/bin/bash
TMPDIR=$(mktemp -d)
ARCHIVE=$(awk '/^__ARCHIVE__/ {print NR + 1; exit 0;}' "$0")
tail -n +$ARCHIVE "$0" | tar xz -C $TMPDIR
cd $TMPDIR && bash install.sh
rm -rf $TMPDIR
exit 0
__ARCHIVE__
HEADER

cat nexus-payload.tar.gz >> $INSTALLER_NAME
chmod +x $INSTALLER_NAME

# Move to release directory
mv $INSTALLER_NAME /home/Automata/Development/projects/Rust-SSD-Nexus-Controller/release/

# Cleanup
rm -rf /tmp/nexus-build /tmp/nexus-payload.tar.gz

echo "Installer created: release/$INSTALLER_NAME"
ls -lah /home/Automata/Development/projects/Rust-SSD-Nexus-Controller/release/$INSTALLER_NAME