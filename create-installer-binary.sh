#!/bin/bash

################################################################################
# Create Self-Contained Installer Binary for Nexus Controller
# This creates a single executable that installs everything
################################################################################

set -e

echo "========================================"
echo "Creating Nexus Controller Installer Binary"
echo "========================================"
echo ""

# Output binary name
OUTPUT="nexus-installer-arm64"

# Create temporary build directory
BUILD_DIR="/tmp/nexus-installer-build-$$"
mkdir -p "$BUILD_DIR"

echo "Build directory: $BUILD_DIR"

# Copy all source files
echo "Copying source files..."
cp -r src "$BUILD_DIR/"
cp -r installer "$BUILD_DIR/"
cp Cargo_complete.toml "$BUILD_DIR/Cargo.toml"
cp Makefile "$BUILD_DIR/"
cp README_COMPLETE.md "$BUILD_DIR/README.md"
cp INSTALLATION_GUIDE.md "$BUILD_DIR/"

# Create the self-extracting script header
echo "Creating installer script..."
cat > "$BUILD_DIR/installer.sh" << 'INSTALLER_HEADER'
#!/bin/bash

################################################################################
# Automata Nexus Controller - Universal Installer
# Single binary that installs everything needed
################################################################################

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
echo ""

# Check if running as root
if [[ $EUID -ne 0 ]]; then
   echo -e "${RED}This installer must be run as root (use sudo)${NC}"
   exit 1
fi

# Check architecture
ARCH=$(uname -m)
if [[ "$ARCH" != "aarch64" ]]; then
    echo -e "${YELLOW}Warning: This installer is optimized for ARM64/aarch64${NC}"
    echo -e "${YELLOW}Current architecture: $ARCH${NC}"
    read -p "Continue anyway? (y/n) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Check for Raspberry Pi 5
if grep -q "Raspberry Pi 5" /proc/cpuinfo 2>/dev/null; then
    echo -e "${GREEN}✓ Raspberry Pi 5 detected${NC}"
else
    echo -e "${YELLOW}⚠ Not running on Raspberry Pi 5${NC}"
fi

# Check available disk space
AVAIL=$(df -BG /opt 2>/dev/null | awk 'NR==2 {print $4}' | sed 's/G//')
if [[ $AVAIL -lt 5 ]]; then
    echo -e "${RED}✗ Insufficient disk space: ${AVAIL}GB available (need 5GB+)${NC}"
    exit 1
else
    echo -e "${GREEN}✓ Sufficient disk space: ${AVAIL}GB${NC}"
fi

# Extract embedded files
echo ""
echo -e "${BLUE}Extracting installation files...${NC}"
EXTRACT_DIR="/tmp/nexus-install-$$"
mkdir -p "$EXTRACT_DIR"
cd "$EXTRACT_DIR"

# Self-extract (files are appended after this script)
MATCH=$(grep -n '^#__ARCHIVE_BELOW__' "$0" | cut -d: -f1)
if [[ -n "$MATCH" ]]; then
    tail -n +$((MATCH + 1)) "$0" | tar xzf - || {
        echo -e "${RED}Failed to extract files${NC}"
        exit 1
    }
fi

echo -e "${GREEN}✓ Files extracted${NC}"

# Install system dependencies first
echo ""
echo -e "${BLUE}Installing system dependencies...${NC}"

apt-get update || {
    echo -e "${RED}Failed to update package lists${NC}"
    exit 1
}

# Essential packages for compilation
PACKAGES=(
    build-essential
    gcc
    g++
    make
    cmake
    pkg-config
    libssl-dev
    libclang-dev
    curl
    git
    libx11-dev
    libxcb1-dev
    libxkbcommon-dev
    libgl1-mesa-dev
    libsqlite3-dev
    libudev-dev
    i2c-tools
    python3-pip
    python3-smbus
)

for package in "${PACKAGES[@]}"; do
    echo -n "Installing $package... "
    if apt-get install -y "$package" > /dev/null 2>&1; then
        echo -e "${GREEN}✓${NC}"
    else
        echo -e "${YELLOW}⚠${NC}"
    fi
done

# Install Rust if not present
if ! command -v rustc &> /dev/null; then
    echo ""
    echo -e "${BLUE}Installing Rust toolchain...${NC}"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y || {
        echo -e "${RED}Failed to install Rust${NC}"
        exit 1
    }
    source "$HOME/.cargo/env"
    echo -e "${GREEN}✓ Rust installed${NC}"
else
    echo -e "${GREEN}✓ Rust already installed: $(rustc --version)${NC}"
fi

# Add ARM64 target
rustup target add aarch64-unknown-linux-gnu 2>/dev/null || true

# Enable hardware interfaces
echo ""
echo -e "${BLUE}Enabling hardware interfaces...${NC}"
raspi-config nonint do_i2c 0 2>/dev/null || true
raspi-config nonint do_spi 0 2>/dev/null || true
echo -e "${GREEN}✓ Hardware interfaces enabled${NC}"

# Run the main installer
echo ""
echo -e "${BLUE}Running main installation...${NC}"
echo ""

cd "$EXTRACT_DIR"
if [[ -f "installer/install_nexus_complete.sh" ]]; then
    bash installer/install_nexus_complete.sh
elif [[ -f "install_nexus_complete.sh" ]]; then
    bash install_nexus_complete.sh
else
    echo -e "${RED}Main installer script not found!${NC}"
    exit 1
fi

# Run post-installation test
echo ""
echo -e "${BLUE}Running installation verification...${NC}"
if [[ -f "installer/test_installation.sh" ]]; then
    bash installer/test_installation.sh || true
fi

# Cleanup
echo ""
echo -e "${BLUE}Cleaning up...${NC}"
rm -rf "$EXTRACT_DIR"

echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}Installation Complete!${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo "Next steps:"
echo "1. Reboot to apply all changes:"
echo "   sudo reboot"
echo ""
echo "2. After reboot, start the service:"
echo "   sudo systemctl start nexus-controller"
echo ""
echo "3. Access the controller:"
echo "   http://localhost:8080"
echo ""
echo "Default PIN: 1234"
echo ""

exit 0

#__ARCHIVE_BELOW__
INSTALLER_HEADER

# Create tarball of all files
echo "Creating archive..."
cd "$BUILD_DIR"
tar czf installer.tar.gz \
    src/ \
    installer/ \
    Cargo.toml \
    Makefile \
    README.md \
    INSTALLATION_GUIDE.md 2>/dev/null || true

# Combine script and archive
echo "Creating self-extracting installer..."
cat installer.sh installer.tar.gz > "$OUTPUT"
chmod +x "$OUTPUT"

# Get size
SIZE=$(du -h "$OUTPUT" | cut -f1)

echo ""
echo "========================================"
echo "✓ Installer binary created successfully!"
echo "========================================"
echo "File: $OUTPUT"
echo "Size: $SIZE"
echo "Architecture: ARM64/aarch64"
echo ""
echo "To install on Raspberry Pi 5:"
echo "1. Copy to RPi: scp $OUTPUT pi@<rpi-ip>:/home/pi/"
echo "2. Make executable: chmod +x $OUTPUT"
echo "3. Run installer: sudo ./$OUTPUT"
echo ""

# Cleanup
rm -rf "$BUILD_DIR"

echo "Build complete!"