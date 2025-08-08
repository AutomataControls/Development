#!/bin/bash

################################################################################
# Build All-In-One Installer for Nexus Controller
# Creates a single binary that installs everything
################################################################################

set -e

echo "========================================"
echo "Building Nexus Controller Installer"
echo "========================================"
echo ""

# Check if we have necessary tools
if ! command -v makeself &> /dev/null; then
    echo "Installing makeself..."
    sudo apt-get update
    sudo apt-get install -y makeself
fi

# Create build directory
BUILD_DIR="nexus-installer-package"
rm -rf "$BUILD_DIR"
mkdir -p "$BUILD_DIR"

echo "Copying files to package..."

# Copy all necessary files
cp -r src "$BUILD_DIR/"
cp -r installer "$BUILD_DIR/"
cp Cargo_complete.toml "$BUILD_DIR/Cargo.toml"
cp Makefile "$BUILD_DIR/"
cp README_COMPLETE.md "$BUILD_DIR/README.md"
cp INSTALLATION_GUIDE.md "$BUILD_DIR/"

# Create the main installer script
cat > "$BUILD_DIR/install.sh" << 'EOF'
#!/bin/bash

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}"
echo "============================================="
echo "  Automata Nexus Controller Installer"
echo "  Native Rust Edition for Raspberry Pi 5"
echo "============================================="
echo -e "${NC}"

# Check if running as root
if [[ $EUID -ne 0 ]]; then
   echo -e "${RED}This installer must be run as root (use sudo)${NC}"
   exit 1
fi

# Run the main installer
if [[ -f "installer/install_nexus_complete.sh" ]]; then
    bash installer/install_nexus_complete.sh
else
    echo -e "${RED}Installer script not found!${NC}"
    exit 1
fi

echo -e "${GREEN}Installation complete!${NC}"
EOF

chmod +x "$BUILD_DIR/install.sh"
chmod +x "$BUILD_DIR/installer/"*.sh

# Create self-extracting installer using makeself
echo "Creating self-extracting installer..."

makeself --gzip \
    --complevel 9 \
    --notemp \
    --target /tmp/nexus-install \
    "$BUILD_DIR" \
    "nexus-installer.run" \
    "Automata Nexus Controller Installer" \
    ./install.sh

# Make it executable
chmod +x nexus-installer.run

# Get file info
SIZE=$(du -h nexus-installer.run | cut -f1)
MD5=$(md5sum nexus-installer.run | cut -d' ' -f1)

echo ""
echo "========================================"
echo "✓ Installer created successfully!"
echo "========================================"
echo "File: nexus-installer.run"
echo "Size: $SIZE"
echo "MD5: $MD5"
echo ""
echo "To install on Raspberry Pi 5:"
echo "1. Copy to RPi: scp nexus-installer.run pi@<rpi-ip>:/home/pi/"
echo "2. Run installer: sudo ./nexus-installer.run"
echo ""
echo "The installer will:"
echo "- Check system requirements"
echo "- Install all dependencies"
echo "- Install Rust toolchain"
echo "- Build the application"
echo "- Configure services"
echo "- Run tests"
echo ""

# Cleanup
rm -rf "$BUILD_DIR"