#!/bin/bash

# Automata Nexus AI - Binary Builder for Raspberry Pi 5
# Creates a standalone binary installer

set -e

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${CYAN}Building Automata Nexus AI Binary Installer...${NC}"

# Build static binary with optimizations
echo -e "${CYAN}Building optimized ARM64 binary...${NC}"
export CARGO_BUILD_JOBS=4
export RUSTFLAGS='-C target-cpu=cortex-a76 -C opt-level=3 -C lto=true -C codegen-units=1'

cargo build --release --target aarch64-unknown-linux-gnu

# Create installer package directory
echo -e "${CYAN}Creating installer package...${NC}"
INSTALLER_DIR="/tmp/nexus-installer"
rm -rf $INSTALLER_DIR
mkdir -p $INSTALLER_DIR

# Copy necessary files
cp target/aarch64-unknown-linux-gnu/release/nexus-controller $INSTALLER_DIR/
cp -r firmware $INSTALLER_DIR/ 2>/dev/null || true
cp -r config $INSTALLER_DIR/ 2>/dev/null || true
cp -r public $INSTALLER_DIR/ 2>/dev/null || true
cp installer/install-nexus-rust-rpi5.sh $INSTALLER_DIR/install.sh
chmod +x $INSTALLER_DIR/install.sh

# Copy migrations
cp -r migrations $INSTALLER_DIR/

# Create version file
echo "2.0.0" > $INSTALLER_DIR/VERSION

# Create self-extracting installer
echo -e "${CYAN}Creating self-extracting installer...${NC}"
cd /tmp
tar czf nexus-installer.tar.gz nexus-installer/

# Create the self-extracting script
cat > nexus-installer.bin << 'INSTALLER_HEADER'
#!/bin/bash

# Automata Nexus AI - Self-Extracting Installer
# Copyright (c) 2025 Automata Controls

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
TEAL='\033[0;96m'
NC='\033[0m'

# Show banner
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
    ║              NEXUS AI CONTROLLER - BINARY INSTALLER              ║
    ║                         Version 2.0.0                            ║
    ║                                                                   ║
    ╚═══════════════════════════════════════════════════════════════════╝
EOF
echo -e "${NC}"

# Check if running as root
if [[ $EUID -ne 0 ]]; then
   echo -e "${RED}This installer must be run as root (use sudo)${NC}"
   exit 1
fi

# Extract archive
echo -e "${CYAN}Extracting installation files...${NC}"
TMPDIR=$(mktemp -d)
ARCHIVE=$(awk '/^__ARCHIVE_BELOW__/ {print NR + 1; exit 0; }' $0)
tail -n+$ARCHIVE $0 | tar xz -C $TMPDIR

# Run installer
cd $TMPDIR/nexus-installer
./install.sh

# Cleanup
cd /
rm -rf $TMPDIR

echo -e "${GREEN}Installation complete!${NC}"
exit 0

__ARCHIVE_BELOW__
INSTALLER_HEADER

# Append the archive
cat nexus-installer.tar.gz >> nexus-installer.bin
chmod +x nexus-installer.bin

# Move to project directory
mv nexus-installer.bin $PWD/

# Calculate size and checksum
SIZE=$(du -h nexus-installer.bin | cut -f1)
SHA256=$(sha256sum nexus-installer.bin | cut -d' ' -f1)

echo
echo -e "${GREEN}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}Binary installer created successfully!${NC}"
echo -e "${GREEN}═══════════════════════════════════════════════════════════════${NC}"
echo -e "File: ${CYAN}nexus-installer.bin${NC}"
echo -e "Size: ${CYAN}$SIZE${NC}"
echo -e "SHA256: ${CYAN}$SHA256${NC}"
echo
echo -e "To install on a Raspberry Pi 5:"
echo -e "  1. Copy nexus-installer.bin to the Pi"
echo -e "  2. chmod +x nexus-installer.bin"
echo -e "  3. sudo ./nexus-installer.bin"
echo -e "${GREEN}═══════════════════════════════════════════════════════════════${NC}"

# Cleanup
rm -rf /tmp/nexus-installer
rm -f /tmp/nexus-installer.tar.gz