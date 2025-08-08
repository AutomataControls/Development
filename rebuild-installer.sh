#\!/bin/bash

echo "Rebuilding Nexus Installer with corrected NVMe setup and PIN 2196..."

# Create temp build directory
BUILD_DIR="/tmp/nexus-rebuild-$$"
mkdir -p "$BUILD_DIR"

# Copy all required files
echo "Copying installer files..."
cp -r installer "$BUILD_DIR/"
cp -r installer-binary "$BUILD_DIR/" 2>/dev/null || true
cp -r src "$BUILD_DIR/" 2>/dev/null || true
cp Cargo.toml "$BUILD_DIR/" 2>/dev/null || true
cp Cargo_complete.toml "$BUILD_DIR/" 2>/dev/null || true

# Create main installer script
cat > "$BUILD_DIR/installer_main.sh" << 'INSTALLER'
#\!/bin/bash

echo "=========================================="
echo "Automata Nexus Controller - Installation"
echo "=========================================="
echo ""
echo "Checking for root permissions..."

if [[ $EUID -ne 0 ]]; then
    echo "This installer must be run as root (use sudo)"
    exit 1
fi

# Install GUI dependencies first
echo "Installing GUI dependencies..."
apt-get update >/dev/null 2>&1
apt-get install -y python3 python3-tk python3-pil python3-pil.imagetk >/dev/null 2>&1

# Launch GUI installer
if [[ -f "installer-binary/nexus-installer.py" ]]; then
    python3 installer-binary/nexus-installer.py
elif [[ -f "installer/install_nexus_complete.sh" ]]; then
    bash installer/install_nexus_complete.sh
else
    echo "Error: Installation files not found"
    exit 1
fi
INSTALLER

chmod +x "$BUILD_DIR/installer_main.sh"

# Create archive
echo "Creating installer archive..."
cd "$BUILD_DIR"
tar czf nexus-installer-data.tar.gz *
cd - >/dev/null

# Create self-extracting script
echo "Creating self-extracting installer..."
cat > nexus-installer-corrected.run << 'HEADER'
#\!/bin/bash
# Self-extracting installer for Automata Nexus Controller
# Created: $(date)

ARCHIVE_START=$(awk '/^__ARCHIVE_START__/ { print NR + 1; exit }' "$0")

echo "=========================================="
echo "Automata Nexus Controller - Professional Building Automation"
echo "Native Rust Implementation for Raspberry Pi 5"
echo "=========================================="
echo ""
echo "Extracting installation files..."

TEMP_DIR="/tmp/nexus-install-$$"
mkdir -p "$TEMP_DIR"

tail -n +$ARCHIVE_START "$0" | tar xz -C "$TEMP_DIR"

cd "$TEMP_DIR"
bash installer_main.sh
EXIT_CODE=$?

cd /
rm -rf "$TEMP_DIR"

exit $EXIT_CODE

__ARCHIVE_START__
HEADER

# Append the archive
cat "$BUILD_DIR/nexus-installer-data.tar.gz" >> nexus-installer-corrected.run

chmod +x nexus-installer-corrected.run

# Get size
SIZE=$(du -h nexus-installer-corrected.run | cut -f1)

# Clean up
rm -rf "$BUILD_DIR"

echo ""
echo "=========================================="
echo "✓ Installer rebuilt successfully\!"
echo "=========================================="
echo "File: nexus-installer-corrected.run"
echo "Size: $SIZE"
echo ""
echo "The installer now includes:"
echo "  • NVMe SSD setup and OS migration"
echo "  • Correct PIN: 2196"
echo "  • GUI installer with logo"
echo "  • Commercial license agreement"
echo "  • Cloudflare tunnel setup"
echo ""
echo "To install on Raspberry Pi 5:"
echo "  1. Copy to RPi: scp nexus-installer-corrected.run pi@<rpi-ip>:/home/pi/"
echo "  2. Make executable: chmod +x nexus-installer-corrected.run"
echo "  3. Run installer: sudo ./nexus-installer-corrected.run"
echo ""
