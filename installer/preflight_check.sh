#!/bin/bash

################################################################################
# Nexus Controller - Pre-flight Dependency Check
# Verifies all requirements before attempting compilation
################################################################################

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Counters
ERRORS=0
WARNINGS=0
SUCCESS=0

echo "=================================="
echo "Nexus Controller Pre-flight Check"
echo "=================================="
echo ""

# Function to check command
check_command() {
    local cmd=$1
    local name=$2
    local required=$3
    
    if command -v "$cmd" &> /dev/null; then
        version=$($cmd --version 2>&1 | head -1)
        echo -e "${GREEN}[✓]${NC} $name found: $version"
        ((SUCCESS++))
        return 0
    else
        if [[ "$required" == "required" ]]; then
            echo -e "${RED}[✗]${NC} $name not found (REQUIRED)"
            ((ERRORS++))
        else
            echo -e "${YELLOW}[!]${NC} $name not found (optional)"
            ((WARNINGS++))
        fi
        return 1
    fi
}

# Function to check package
check_package() {
    local pkg=$1
    local name=$2
    local required=$3
    
    if dpkg -l | grep -q "^ii.*$pkg"; then
        echo -e "${GREEN}[✓]${NC} $name installed"
        ((SUCCESS++))
        return 0
    else
        if [[ "$required" == "required" ]]; then
            echo -e "${RED}[✗]${NC} $name not installed (REQUIRED)"
            echo "     Install with: sudo apt-get install $pkg"
            ((ERRORS++))
        else
            echo -e "${YELLOW}[!]${NC} $name not installed (optional)"
            echo "     Install with: sudo apt-get install $pkg"
            ((WARNINGS++))
        fi
        return 1
    fi
}

# Function to check Rust crate
check_crate() {
    local crate=$1
    if cargo search "$crate" --limit 1 &> /dev/null; then
        echo -e "${GREEN}[✓]${NC} Crate '$crate' available"
        ((SUCCESS++))
        return 0
    else
        echo -e "${YELLOW}[!]${NC} Cannot verify crate '$crate'"
        ((WARNINGS++))
        return 1
    fi
}

echo "1. Checking System Requirements"
echo "================================"

# Check OS
if [[ $(uname -m) == "aarch64" ]]; then
    echo -e "${GREEN}[✓]${NC} 64-bit ARM architecture"
    ((SUCCESS++))
else
    echo -e "${RED}[✗]${NC} Not 64-bit ARM (found: $(uname -m))"
    ((ERRORS++))
fi

# Check memory
TOTAL_MEM=$(free -m | awk '/^Mem:/{print $2}')
if [[ $TOTAL_MEM -gt 3500 ]]; then
    echo -e "${GREEN}[✓]${NC} Sufficient RAM: ${TOTAL_MEM}MB"
    ((SUCCESS++))
else
    echo -e "${YELLOW}[!]${NC} Low RAM: ${TOTAL_MEM}MB (4GB+ recommended)"
    ((WARNINGS++))
fi

# Check disk space
AVAILABLE=$(df -BG /opt 2>/dev/null | awk 'NR==2 {print $4}' | sed 's/G//')
if [[ $AVAILABLE -gt 5 ]]; then
    echo -e "${GREEN}[✓]${NC} Sufficient disk space: ${AVAILABLE}GB"
    ((SUCCESS++))
else
    echo -e "${RED}[✗]${NC} Low disk space: ${AVAILABLE}GB (need 5GB+)"
    ((ERRORS++))
fi

echo ""
echo "2. Checking Build Tools"
echo "======================="

check_command "gcc" "GCC Compiler" "required"
check_command "g++" "G++ Compiler" "required"
check_command "make" "Make" "required"
check_command "cmake" "CMake" "optional"
check_command "pkg-config" "pkg-config" "required"
check_command "git" "Git" "required"

echo ""
echo "3. Checking Rust Toolchain"
echo "=========================="

if check_command "rustc" "Rust Compiler" "required"; then
    # Check Rust version
    RUST_VERSION=$(rustc --version | awk '{print $2}')
    REQUIRED_VERSION="1.75.0"
    if [[ "$(printf '%s\n' "$REQUIRED_VERSION" "$RUST_VERSION" | sort -V | head -n1)" == "$REQUIRED_VERSION" ]]; then
        echo -e "${GREEN}[✓]${NC} Rust version $RUST_VERSION meets minimum $REQUIRED_VERSION"
        ((SUCCESS++))
    else
        echo -e "${RED}[✗]${NC} Rust version $RUST_VERSION is below minimum $REQUIRED_VERSION"
        ((ERRORS++))
    fi
fi

check_command "cargo" "Cargo" "required"
check_command "rustup" "Rustup" "required"

# Check for ARM target
if rustup target list | grep -q "aarch64-unknown-linux-gnu (installed)"; then
    echo -e "${GREEN}[✓]${NC} ARM64 target installed"
    ((SUCCESS++))
else
    echo -e "${YELLOW}[!]${NC} ARM64 target not installed"
    echo "     Install with: rustup target add aarch64-unknown-linux-gnu"
    ((WARNINGS++))
fi

echo ""
echo "4. Checking System Libraries"
echo "============================"

# Essential libraries
check_package "libssl-dev" "OpenSSL development" "required"
check_package "libclang-dev" "Clang development" "required"
check_package "libudev-dev" "udev development" "required"

# GUI libraries for egui
echo ""
echo "GUI Libraries (for egui):"
check_package "libx11-dev" "X11 development" "required"
check_package "libxcb1-dev" "XCB development" "required"
check_package "libxkbcommon-dev" "XKB common" "required"
check_package "libgl1-mesa-dev" "OpenGL development" "required"
check_package "libwayland-dev" "Wayland development" "optional"

# Database libraries
echo ""
echo "Database Libraries:"
check_package "libsqlite3-dev" "SQLite development" "required"
check_package "redis-server" "Redis server" "optional"

# Hardware libraries
echo ""
echo "Hardware Libraries:"
check_package "i2c-tools" "I2C tools" "required"
check_package "python3-smbus" "Python SMBus" "required"

echo ""
echo "5. Checking Python Dependencies"
echo "================================"

if check_command "python3" "Python 3" "required"; then
    # Check Python version
    PYTHON_VERSION=$(python3 --version | awk '{print $2}')
    echo "     Python version: $PYTHON_VERSION"
fi

check_command "pip3" "pip3" "required"

# Check Python modules
echo ""
echo "Python Modules:"
python3 -c "import serial" 2>/dev/null && echo -e "${GREEN}[✓]${NC} pyserial installed" && ((SUCCESS++)) || \
    (echo -e "${YELLOW}[!]${NC} pyserial not installed" && ((WARNINGS++)))

python3 -c "import smbus" 2>/dev/null && echo -e "${GREEN}[✓]${NC} smbus installed" && ((SUCCESS++)) || \
    (echo -e "${YELLOW}[!]${NC} smbus not installed" && ((WARNINGS++)))

python3 -c "import RPi.GPIO" 2>/dev/null && echo -e "${GREEN}[✓]${NC} RPi.GPIO installed" && ((SUCCESS++)) || \
    (echo -e "${YELLOW}[!]${NC} RPi.GPIO not installed" && ((WARNINGS++)))

echo ""
echo "6. Checking Hardware Interfaces"
echo "================================"

# Check I2C
if [[ -e /dev/i2c-1 ]]; then
    echo -e "${GREEN}[✓]${NC} I2C interface enabled"
    ((SUCCESS++))
else
    echo -e "${YELLOW}[!]${NC} I2C interface not enabled"
    echo "     Enable with: sudo raspi-config nonint do_i2c 0"
    ((WARNINGS++))
fi

# Check SPI
if [[ -e /dev/spidev0.0 ]]; then
    echo -e "${GREEN}[✓]${NC} SPI interface enabled"
    ((SUCCESS++))
else
    echo -e "${YELLOW}[!]${NC} SPI interface not enabled"
    echo "     Enable with: sudo raspi-config nonint do_spi 0"
    ((WARNINGS++))
fi

# Check for NVMe
if lsblk | grep -q nvme; then
    echo -e "${GREEN}[✓]${NC} NVMe SSD detected"
    ((SUCCESS++))
else
    echo -e "${YELLOW}[!]${NC} No NVMe SSD detected"
    ((WARNINGS++))
fi

echo ""
echo "7. Checking Cargo Dependencies"
echo "==============================="

# Check if Cargo.toml exists
if [[ -f "../Cargo.toml" ]] || [[ -f "../Cargo_complete.toml" ]]; then
    echo -e "${GREEN}[✓]${NC} Cargo.toml found"
    ((SUCCESS++))
    
    # Try to check dependencies
    echo "Verifying key crates availability:"
    check_crate "egui"
    check_crate "eframe"
    check_crate "tokio"
    check_crate "serde"
    check_crate "sqlx"
    check_crate "serialport"
else
    echo -e "${RED}[✗]${NC} Cargo.toml not found"
    ((ERRORS++))
fi

echo ""
echo "8. Checking Network Connectivity"
echo "================================="

# Check internet connection
if ping -c 1 crates.io &> /dev/null; then
    echo -e "${GREEN}[✓]${NC} Internet connection OK (crates.io reachable)"
    ((SUCCESS++))
else
    echo -e "${RED}[✗]${NC} Cannot reach crates.io"
    ((ERRORS++))
fi

if ping -c 1 github.com &> /dev/null; then
    echo -e "${GREEN}[✓]${NC} GitHub reachable"
    ((SUCCESS++))
else
    echo -e "${YELLOW}[!]${NC} Cannot reach GitHub"
    ((WARNINGS++))
fi

echo ""
echo "=================================="
echo "Pre-flight Check Complete"
echo "=================================="
echo ""
echo -e "${GREEN}Passed:${NC} $SUCCESS checks"
echo -e "${YELLOW}Warnings:${NC} $WARNINGS (optional items)"
echo -e "${RED}Errors:${NC} $ERRORS (must fix before build)"
echo ""

if [[ $ERRORS -eq 0 ]]; then
    echo -e "${GREEN}✓ System is ready for Nexus Controller installation!${NC}"
    echo ""
    echo "Run the installer with:"
    echo "  sudo bash install_nexus_complete.sh"
    exit 0
else
    echo -e "${RED}✗ Please fix the errors above before proceeding${NC}"
    echo ""
    echo "Quick fix commands:"
    echo "  sudo apt-get update"
    echo "  sudo apt-get install build-essential pkg-config libssl-dev"
    echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi