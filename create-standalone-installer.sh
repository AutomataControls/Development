#!/bin/bash

# Create a completely standalone self-extracting installer for Automata Nexus
# This creates a single binary that users can download and run
# No need to clone the repo!

set -e

CYAN='\033[0;36m'
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${CYAN}Creating standalone Automata Nexus installer...${NC}"

# Create temp directory for packaging
TEMP_DIR=$(mktemp -d)
INSTALLER_NAME="nexus-installer-rpi5"
OUTPUT_DIR="/home/Automata/Development/projects/Rust-SSD-Nexus-Controller"

echo -e "${CYAN}Packaging source code...${NC}"

# Create the package structure
mkdir -p $TEMP_DIR/$INSTALLER_NAME/src
mkdir -p $TEMP_DIR/$INSTALLER_NAME/migrations
mkdir -p $TEMP_DIR/$INSTALLER_NAME/installer

# Copy all source files
cp -r src/* $TEMP_DIR/$INSTALLER_NAME/src/
cp -r migrations/* $TEMP_DIR/$INSTALLER_NAME/migrations/
cp -r installer/* $TEMP_DIR/$INSTALLER_NAME/installer/
cp Cargo.toml $TEMP_DIR/$INSTALLER_NAME/
cp build.rs $TEMP_DIR/$INSTALLER_NAME/
cp README.md $TEMP_DIR/$INSTALLER_NAME/
cp ARCHITECTURE.md $TEMP_DIR/$INSTALLER_NAME/

# Create the main installer script that will be executed
cat > $TEMP_DIR/$INSTALLER_NAME/run-installer.sh << 'EOF'
#!/bin/bash
# Automata Nexus Standalone Installer
# This script is run after extraction

set -e

CYAN='\033[0;36m'
GREEN='\033[0;32m'
TEAL='\033[0;96m'
NC='\033[0m'

echo -e "${TEAL}"
cat << "BANNER"
    ╔═══════════════════════════════════════════════════════════════════╗
    ║                                                                   ║
    ║     ___         __                        __          _   __     ║
    ║    /   | __  __/ /_____  ____ ___  ____ _/ /_____ _  / | / /     ║
    ║   / /| |/ / / / __/ __ \/ __ `__ \/ __ `/ __/ __ `/ /  |/ /      ║
    ║  / ___ / /_/ / /_/ /_/ / / / / / / /_/ / /_/ /_/ / / /|  /       ║
    ║ /_/  |_\__,_/\__/\____/_/ /_/ /_/\__,_/\__/\__,_/ /_/ |_/        ║
    ║                                                                   ║
    ║          NEXUS AI CONTROLLER - STANDALONE INSTALLER              ║
    ║               Building Automation for Raspberry Pi 5             ║
    ║                         Version 2.0                              ║
    ║                                                                   ║
    ╚═══════════════════════════════════════════════════════════════════╝
BANNER
echo -e "${NC}"

echo -e "${CYAN}Welcome to Automata Nexus Standalone Installer${NC}"
echo "This installer will set up everything automatically on your Raspberry Pi 5"
echo

# Check if running as root
if [[ $EUID -ne 0 ]]; then
   echo -e "${RED}This installer must be run as root (use sudo)${NC}"
   exit 1
fi

# Copy source to installation location
echo -e "${CYAN}Extracting source code...${NC}"
INSTALL_PATH="/opt/nexus"
mkdir -p $INSTALL_PATH
cp -r * $INSTALL_PATH/

# Run the main installer
cd $INSTALL_PATH
chmod +x installer/install-nexus-rust-rpi5.sh
./installer/install-nexus-rust-rpi5.sh

echo -e "${GREEN}Installation complete!${NC}"
EOF

chmod +x $TEMP_DIR/$INSTALLER_NAME/run-installer.sh

# Create the self-extracting script header
cat > $TEMP_DIR/installer-header.sh << 'HEADER'
#!/bin/bash
# Automata Nexus AI Controller - Self-Extracting Installer
# Copyright (c) 2025 Automata Controls
# Single-file installer - no repo cloning needed!

TEMP_EXTRACT=$(mktemp -d)
ARCHIVE_START=$(awk '/^__ARCHIVE_START__/ {print NR + 1; exit 0; }' "$0")

echo "Extracting Automata Nexus installer..."
tail -n +$ARCHIVE_START "$0" | tar xz -C $TEMP_EXTRACT

cd $TEMP_EXTRACT/nexus-installer-rpi5
./run-installer.sh

# Cleanup
cd /
rm -rf $TEMP_EXTRACT

exit 0
__ARCHIVE_START__
HEADER

# Create the archive
echo -e "${CYAN}Creating archive...${NC}"
cd $TEMP_DIR
tar czf nexus-installer.tar.gz $INSTALLER_NAME/

# Combine header and archive
cat installer-header.sh nexus-installer.tar.gz > $OUTPUT_DIR/$INSTALLER_NAME.bin

# Make it executable
chmod +x $OUTPUT_DIR/$INSTALLER_NAME.bin

# Get file size
FILE_SIZE=$(du -h $OUTPUT_DIR/$INSTALLER_NAME.bin | cut -f1)

# Cleanup
rm -rf $TEMP_DIR

echo -e "${GREEN}✅ Standalone installer created successfully!${NC}"
echo
echo -e "${CYAN}═══════════════════════════════════════════════════════════════════${NC}"
echo -e "Installer: ${GREEN}$OUTPUT_DIR/$INSTALLER_NAME.bin${NC}"
echo -e "Size: ${GREEN}$FILE_SIZE${NC}"
echo
echo "To install on a fresh Raspberry Pi 5:"
echo "  1. Copy this single file to the Pi"
echo "  2. Run: sudo ./$INSTALLER_NAME.bin"
echo
echo "No need to clone the repository!"
echo -e "${CYAN}═══════════════════════════════════════════════════════════════════${NC}"