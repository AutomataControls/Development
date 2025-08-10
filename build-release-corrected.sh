#!/bin/bash

# Build Release Binary with All Board Integration Corrections
# This creates the final installer with corrected board specifications

set -e

CYAN='\033[0;36m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${CYAN}═══════════════════════════════════════════════════════════════════${NC}"
echo -e "${CYAN}    Building Automata Nexus Release - WITH CORRECTIONS${NC}"
echo -e "${CYAN}═══════════════════════════════════════════════════════════════════${NC}"
echo

# Version info
VERSION="2.1.0"
BUILD_DATE=$(date +"%Y-%m-%d")
BUILD_NUM=$(date +"%Y%m%d%H%M")

echo -e "${CYAN}Version: ${VERSION}${NC}"
echo -e "${CYAN}Build: ${BUILD_NUM}${NC}"
echo -e "${CYAN}Date: ${BUILD_DATE}${NC}"
echo

# Create build directory
BUILD_DIR="/tmp/nexus-release-${BUILD_NUM}"
mkdir -p "$BUILD_DIR"

echo -e "${CYAN}[1/7] Copying source files...${NC}"
# Copy all source files with corrections
cp -r src "$BUILD_DIR/"
cp -r migrations "$BUILD_DIR/"
cp -r installer "$BUILD_DIR/"
cp Cargo.toml "$BUILD_DIR/"
cp build.rs "$BUILD_DIR/"

# Copy test scripts
cp test_ui_firmware_integration.py "$BUILD_DIR/"
cp test_firmware.py "$BUILD_DIR/" 2>/dev/null || true

# Create version file
cat > "$BUILD_DIR/version.txt" << EOF
Automata Nexus AI Controller
Version: ${VERSION}
Build: ${BUILD_NUM}
Date: ${BUILD_DATE}

IMPORTANT CORRECTIONS IN THIS RELEASE:
- MegaBAS: 4 triacs, 4 analog outputs, 8 configurable inputs (NO RELAYS)
- 16univin: 16 universal INPUTS ONLY
- 16uout: 16 analog OUTPUTS ONLY
- 8relind/16relind: Separate relay boards
- Fixed input channel ranges for all boards
- P499 pressure transducers properly supported
EOF

echo -e "${CYAN}[2/7] Creating documentation...${NC}"
# Create README with corrections
cat > "$BUILD_DIR/README_BOARDS.md" << 'EOF'
# Automata Nexus - Board Specifications (CORRECTED)

## IMPORTANT: Actual Board Capabilities

Based on thorough review of firmware source code, these are the CORRECT specifications:

### MegaBAS (Building Automation System)
- **4 Triacs** - AC control (NOT relays!)
- **4 Analog Outputs** - 0-10V
- **8 Configurable Inputs** - Can be:
  - 0-10V analog input
  - 1K thermistor OR dry contact
  - 10K thermistor
- **NO RELAYS** on MegaBAS!

### 16univin (Universal Input Board)
- **16 Universal INPUTS ONLY**
- Each input can be configured as:
  - 0-10V analog
  - 1K thermistor
  - 10K thermistor
  - Dry contact
- **NO OUTPUTS** on this board

### 16uout (Universal Output Board)
- **16 Analog OUTPUTS ONLY**
- All outputs are 0-10V
- **NO INPUTS** on this board

### 8relind (8-Relay Board)
- **8 Relay Outputs ONLY**
- Industrial relay board
- **SEPARATE** from MegaBAS

### 16relind (16-Relay Board)
- **16 Relay Outputs ONLY**
- Industrial relay board
- **SEPARATE** from MegaBAS

## Common Mistakes (Now Fixed)
1. ❌ MegaBAS has relays - **WRONG!** It has triacs
2. ❌ 16univin has outputs - **WRONG!** It's input only
3. ❌ 16uout has inputs - **WRONG!** It's output only
4. ✅ All boards now correctly represented in UI
5. ✅ Input channel ranges dynamically adjust based on board type
EOF

echo -e "${CYAN}[3/7] Updating installer with corrections...${NC}"
# Already updated in installer/install-nexus-rust-rpi5.sh

echo -e "${CYAN}[4/7] Creating main installer script...${NC}"
cat > "$BUILD_DIR/install.sh" << 'MAIN_INSTALLER'
#!/bin/bash

# Automata Nexus Installer - Release with Corrections
set -e

CYAN='\033[0;36m'
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${CYAN}═══════════════════════════════════════════════════════════════════${NC}"
echo -e "${CYAN}         Automata Nexus AI Controller Installation${NC}"
echo -e "${CYAN}              Version 2.1.0 - With Board Corrections${NC}"
echo -e "${CYAN}═══════════════════════════════════════════════════════════════════${NC}"
echo

# Check root
if [[ $EUID -ne 0 ]]; then
   echo -e "${RED}This installer must be run as root (use sudo)${NC}"
   exit 1
fi

# Show corrections
echo -e "${GREEN}This release includes CORRECTED board specifications:${NC}"
echo "• MegaBAS: 4 triacs (NOT relays), 4 analog out, 8 config inputs"
echo "• 16univin: 16 INPUTS ONLY"
echo "• 16uout: 16 OUTPUTS ONLY"
echo "• Relay boards are SEPARATE (8relind/16relind)"
echo

read -p "Continue with installation? (yes/no): " -r
if [[ ! $REPLY =~ ^[Yy]es$ ]]; then
    exit 1
fi

# Run the actual installer
if [ -f "installer/install-nexus-rust-rpi5.sh" ]; then
    bash installer/install-nexus-rust-rpi5.sh
else
    echo -e "${RED}Error: Installer script not found${NC}"
    exit 1
fi
MAIN_INSTALLER

chmod +x "$BUILD_DIR/install.sh"

echo -e "${CYAN}[5/7] Creating self-extracting archive...${NC}"
cd "$BUILD_DIR"
tar czf nexus-installer.tar.gz *
cd - >/dev/null

echo -e "${CYAN}[6/7] Building final installer...${NC}"
cat > nexus-installer-v${VERSION}.run << 'HEADER'
#!/bin/bash
# Automata Nexus Self-Extracting Installer
# Version 2.1.0 - With Board Corrections

TEMP_DIR=$(mktemp -d)
ARCHIVE_START=$(awk '/^__ARCHIVE_START__/ { print NR + 1; exit }' "$0")

echo "Extracting Automata Nexus installer v2.1.0..."
tail -n +$ARCHIVE_START "$0" | tar xz -C "$TEMP_DIR"

cd "$TEMP_DIR"
bash install.sh
EXIT_CODE=$?

cd /
rm -rf "$TEMP_DIR"
exit $EXIT_CODE

__ARCHIVE_START__
HEADER

# Append archive
cat "$BUILD_DIR/nexus-installer.tar.gz" >> nexus-installer-v${VERSION}.run
chmod +x nexus-installer-v${VERSION}.run

# Get file info
FILE_SIZE=$(du -h nexus-installer-v${VERSION}.run | cut -f1)
MD5_HASH=$(md5sum nexus-installer-v${VERSION}.run | cut -d' ' -f1)

echo -e "${CYAN}[7/7] Cleaning up...${NC}"
rm -rf "$BUILD_DIR"

echo
echo -e "${GREEN}═══════════════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}✅ Build Complete!${NC}"
echo -e "${GREEN}═══════════════════════════════════════════════════════════════════${NC}"
echo
echo -e "Installer: ${GREEN}nexus-installer-v${VERSION}.run${NC}"
echo -e "Version: ${GREEN}${VERSION}${NC}"
echo -e "Build: ${GREEN}${BUILD_NUM}${NC}"
echo -e "Size: ${GREEN}${FILE_SIZE}${NC}"
echo -e "MD5: ${GREEN}${MD5_HASH}${NC}"
echo
echo -e "${CYAN}Key Corrections in This Release:${NC}"
echo "  • MegaBAS correctly shows 4 triacs (not relays)"
echo "  • 16univin properly configured as INPUT ONLY"
echo "  • 16uout properly configured as OUTPUT ONLY"
echo "  • Dynamic input channel ranges based on board type"
echo "  • All 5 board drivers included in installer"
echo
echo -e "${CYAN}Installation Instructions:${NC}"
echo "  1. Copy to RPi5: scp nexus-installer-v${VERSION}.run pi@<ip>:/home/pi/"
echo "  2. Make executable: chmod +x nexus-installer-v${VERSION}.run"
echo "  3. Run: sudo ./nexus-installer-v${VERSION}.run"
echo
echo -e "${GREEN}Board Test Commands (after installation):${NC}"
echo "  • nexus-test-boards        - Verify board installations"
echo "  • nexus-firmware scan      - Scan for connected boards"
echo "  • nexus-firmware info      - Show board capabilities"
echo