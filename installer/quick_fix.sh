#!/bin/bash

################################################################################
# Nexus Controller - Quick Fix Script
# Automatically fixes common installation issues
################################################################################

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo "========================================"
echo "Nexus Controller Quick Fix Tool"
echo "========================================"
echo ""

# Check if running as root
if [[ $EUID -ne 0 ]]; then
   echo -e "${RED}This script must be run as root (use sudo)${NC}"
   exit 1
fi

FIXED=0
FAILED=0

fix_issue() {
    local description="$1"
    local fix_command="$2"
    
    echo -e "${BLUE}Fixing:${NC} $description"
    if eval "$fix_command" &> /dev/null; then
        echo -e "  ${GREEN}✓ Fixed${NC}"
        ((FIXED++))
    else
        echo -e "  ${RED}✗ Failed${NC}"
        ((FAILED++))
    fi
}

echo "1. Checking and fixing system updates..."
echo "========================================"

fix_issue "Update package lists" "apt-get update"
fix_issue "Fix broken packages" "apt-get --fix-broken install -y"

echo ""
echo "2. Installing missing dependencies..."
echo "====================================="

# Essential packages that are often missing
MISSING_PACKAGES=(
    "build-essential"
    "pkg-config"
    "libssl-dev"
    "libclang-dev"
    "libx11-dev"
    "libxcb1-dev"
    "libxkbcommon-dev"
    "libgl1-mesa-dev"
    "libsqlite3-dev"
    "libudev-dev"
    "i2c-tools"
    "python3-pip"
    "git"
    "curl"
    "cmake"
)

for package in "${MISSING_PACKAGES[@]}"; do
    if ! dpkg -l | grep -q "^ii.*$package"; then
        fix_issue "Install $package" "apt-get install -y $package"
    fi
done

echo ""
echo "3. Fixing Rust installation..."
echo "=============================="

if ! command -v rustc &> /dev/null; then
    echo "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
    echo -e "  ${GREEN}✓ Rust installed${NC}"
    ((FIXED++))
fi

if command -v rustup &> /dev/null; then
    fix_issue "Update Rust" "rustup update"
    fix_issue "Add ARM64 target" "rustup target add aarch64-unknown-linux-gnu"
fi

echo ""
echo "4. Fixing hardware interfaces..."
echo "================================"

fix_issue "Enable I2C" "raspi-config nonint do_i2c 0"
fix_issue "Enable SPI" "raspi-config nonint do_spi 0"
fix_issue "Add i2c-dev module" "echo 'i2c-dev' >> /etc/modules"
fix_issue "Add i2c-bcm2835 module" "echo 'i2c-bcm2835' >> /etc/modules"

echo ""
echo "5. Fixing Python modules..."
echo "==========================="

fix_issue "Upgrade pip" "python3 -m pip install --upgrade pip"
fix_issue "Install pyserial" "python3 -m pip install pyserial"
fix_issue "Install smbus2" "python3 -m pip install smbus2"
fix_issue "Install RPi.GPIO" "python3 -m pip install RPi.GPIO"

echo ""
echo "6. Fixing permissions..."
echo "========================"

INSTALL_BASE="/opt/automata-nexus"
if [[ -d "$INSTALL_BASE" ]]; then
    fix_issue "Set ownership" "chown -R pi:pi $INSTALL_BASE"
    fix_issue "Set execute permissions" "chmod +x $INSTALL_BASE/*.sh 2>/dev/null || true"
    fix_issue "Set data permissions" "chmod 700 $INSTALL_BASE/data 2>/dev/null || true"
    fix_issue "Set log permissions" "chmod 777 $INSTALL_BASE/logs 2>/dev/null || true"
fi

echo ""
echo "7. Fixing systemd services..."
echo "============================="

if [[ -f "/etc/systemd/system/nexus-controller.service" ]]; then
    fix_issue "Reload systemd" "systemctl daemon-reload"
    fix_issue "Enable service" "systemctl enable nexus-controller"
fi

echo ""
echo "8. Optimizing system..."
echo "======================"

# Enable performance governor
for cpu in /sys/devices/system/cpu/cpu[0-3]/cpufreq/scaling_governor; do
    echo performance > $cpu 2>/dev/null || true
done
echo -e "  ${GREEN}✓ Performance governor set${NC}"
((FIXED++))

# PCIe Gen3 for NVMe
if ! grep -q "dtparam=pciex1" /boot/firmware/config.txt 2>/dev/null; then
    if [[ -f /boot/firmware/config.txt ]]; then
        cat >> /boot/firmware/config.txt << EOF

# Enable PCIe for NVMe SSD
dtparam=pciex1
dtparam=pciex1_gen=3
EOF
        echo -e "  ${GREEN}✓ PCIe Gen3 enabled${NC}"
        ((FIXED++))
    fi
fi

echo ""
echo "9. Creating missing directories..."
echo "=================================="

DIRECTORIES=(
    "$INSTALL_BASE"
    "$INSTALL_BASE/data"
    "$INSTALL_BASE/logs"
    "$INSTALL_BASE/config"
    "$INSTALL_BASE/backups"
    "$INSTALL_BASE/cache"
)

for dir in "${DIRECTORIES[@]}"; do
    if [[ ! -d "$dir" ]]; then
        fix_issue "Create $dir" "mkdir -p $dir"
    fi
done

echo ""
echo "10. Checking build requirements..."
echo "==================================="

# Check available disk space
AVAIL=$(df -BG /opt 2>/dev/null | awk 'NR==2 {print $4}' | sed 's/G//')
if [[ $AVAIL -lt 5 ]]; then
    echo -e "  ${YELLOW}⚠ Warning: Low disk space (${AVAIL}GB available)${NC}"
    echo "  Consider freeing up space or using external storage"
else
    echo -e "  ${GREEN}✓ Sufficient disk space (${AVAIL}GB)${NC}"
fi

# Check available memory
MEM=$(free -m | awk '/^Mem:/{print $2}')
if [[ $MEM -lt 4000 ]]; then
    echo -e "  ${YELLOW}⚠ Warning: Low memory (${MEM}MB)${NC}"
    echo "  Build may be slow. Consider adding swap:"
    echo "  sudo fallocate -l 4G /swapfile"
    echo "  sudo chmod 600 /swapfile"
    echo "  sudo mkswap /swapfile"
    echo "  sudo swapon /swapfile"
else
    echo -e "  ${GREEN}✓ Sufficient memory (${MEM}MB)${NC}"
fi

echo ""
echo "========================================"
echo "Quick Fix Summary"
echo "========================================"
echo ""
echo -e "${GREEN}Fixed:${NC} $FIXED issues"
echo -e "${RED}Failed:${NC} $FAILED issues"
echo ""

if [[ $FAILED -eq 0 ]]; then
    echo -e "${GREEN}✓ All fixable issues resolved!${NC}"
    echo ""
    echo "Next steps:"
    echo "1. Run the installation test:"
    echo "   sudo bash test_installation.sh"
    echo ""
    echo "2. If tests pass, start the service:"
    echo "   sudo systemctl start nexus-controller"
    echo ""
    
    if grep -q "dtparam=pciex1" /boot/firmware/config.txt 2>/dev/null; then
        echo -e "${YELLOW}Note: A reboot is required for PCIe changes${NC}"
        echo "   sudo reboot"
    fi
else
    echo -e "${YELLOW}⚠ Some issues could not be automatically fixed${NC}"
    echo ""
    echo "Please check the failed items above and:"
    echo "1. Manually install missing packages"
    echo "2. Check system logs for errors"
    echo "3. Re-run the main installer if needed"
fi

echo ""