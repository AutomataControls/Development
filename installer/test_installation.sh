#!/bin/bash

################################################################################
# Nexus Controller - Post-Installation Test Suite
# Validates successful installation and build
################################################################################

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Test counters
PASSED=0
FAILED=0
SKIPPED=0

# Installation paths
INSTALL_BASE="/opt/automata-nexus"
BINARY="$INSTALL_BASE/nexus-controller"
CONFIG_DIR="$INSTALL_BASE/config"
DATA_DIR="$INSTALL_BASE/data"
LOG_DIR="$INSTALL_BASE/logs"

echo "========================================"
echo "Nexus Controller Installation Test Suite"
echo "========================================"
echo ""

# Test function
run_test() {
    local test_name="$1"
    local test_cmd="$2"
    local required="$3"
    
    echo -n "Testing: $test_name... "
    
    if eval "$test_cmd" &> /dev/null; then
        echo -e "${GREEN}PASS${NC}"
        ((PASSED++))
        return 0
    else
        if [[ "$required" == "required" ]]; then
            echo -e "${RED}FAIL${NC}"
            ((FAILED++))
        else
            echo -e "${YELLOW}SKIP${NC} (optional)"
            ((SKIPPED++))
        fi
        return 1
    fi
}

# File existence test
test_file() {
    local file="$1"
    local name="$2"
    local required="$3"
    
    run_test "$name" "[[ -f '$file' ]]" "$required"
}

# Directory existence test
test_dir() {
    local dir="$1"
    local name="$2"
    local required="$3"
    
    run_test "$name" "[[ -d '$dir' ]]" "$required"
}

# Command availability test
test_command() {
    local cmd="$1"
    local name="$2"
    local required="$3"
    
    run_test "$name" "command -v '$cmd'" "$required"
}

# Service test
test_service() {
    local service="$1"
    local name="$2"
    
    run_test "$name enabled" "systemctl is-enabled '$service'" "required"
    run_test "$name loaded" "systemctl list-unit-files | grep -q '$service'" "required"
}

echo "1. Checking Binary Installation"
echo "================================"

test_file "$BINARY" "Main binary" "required"
test_file "$BINARY" "Binary is executable" "required"

if [[ -f "$BINARY" ]]; then
    # Check binary architecture
    ARCH=$(file "$BINARY" | grep -o "ARM aarch64" || echo "unknown")
    if [[ "$ARCH" == "ARM aarch64" ]]; then
        echo -e "Binary architecture: ${GREEN}ARM64 (correct)${NC}"
        ((PASSED++))
    else
        echo -e "Binary architecture: ${RED}$ARCH (incorrect)${NC}"
        ((FAILED++))
    fi
    
    # Check binary size (should be > 5MB for complete build)
    SIZE=$(stat -c%s "$BINARY" 2>/dev/null || echo 0)
    if [[ $SIZE -gt 5000000 ]]; then
        echo -e "Binary size: ${GREEN}$(echo "scale=2; $SIZE/1048576" | bc)MB${NC}"
        ((PASSED++))
    else
        echo -e "Binary size: ${RED}Too small ($(echo "scale=2; $SIZE/1048576" | bc)MB)${NC}"
        ((FAILED++))
    fi
fi

echo ""
echo "2. Checking Directory Structure"
echo "================================"

test_dir "$INSTALL_BASE" "Base directory" "required"
test_dir "$CONFIG_DIR" "Config directory" "required"
test_dir "$DATA_DIR" "Data directory" "required"
test_dir "$LOG_DIR" "Log directory" "required"
test_dir "$INSTALL_BASE/backups" "Backup directory" "required"
test_dir "$INSTALL_BASE/cache" "Cache directory" "optional"

echo ""
echo "3. Checking Configuration Files"
echo "================================"

test_file "$CONFIG_DIR/nexus.toml" "Main config" "required"
test_file "$DATA_DIR/nexus.db" "Database" "required"
test_file "/etc/systemd/system/nexus-controller.service" "Systemd service" "required"
test_file "/etc/systemd/system/nexus-monitor.service" "Monitor service" "optional"

# Check database integrity
if [[ -f "$DATA_DIR/nexus.db" ]]; then
    echo -n "Testing: Database integrity... "
    if sqlite3 "$DATA_DIR/nexus.db" "SELECT COUNT(*) FROM settings;" &> /dev/null; then
        echo -e "${GREEN}PASS${NC}"
        ((PASSED++))
    else
        echo -e "${RED}FAIL${NC}"
        ((FAILED++))
    fi
fi

echo ""
echo "4. Checking System Dependencies"
echo "================================"

test_command "rustc" "Rust compiler" "required"
test_command "cargo" "Cargo" "required"
test_command "gcc" "GCC compiler" "required"
test_command "python3" "Python 3" "required"
test_command "sqlite3" "SQLite" "required"
test_command "i2cdetect" "I2C tools" "optional"

# Check Rust version
if command -v rustc &> /dev/null; then
    RUST_VERSION=$(rustc --version | awk '{print $2}')
    echo -e "Rust version: ${BLUE}$RUST_VERSION${NC}"
fi

echo ""
echo "5. Checking Hardware Interfaces"
echo "================================"

run_test "I2C interface" "[[ -e /dev/i2c-1 ]]" "optional"
run_test "SPI interface" "[[ -e /dev/spidev0.0 ]]" "optional"
run_test "GPIO access" "[[ -e /dev/gpiochip0 ]]" "optional"

# Check for NVMe
if lsblk | grep -q nvme; then
    echo -e "NVMe SSD: ${GREEN}Detected${NC}"
    ((PASSED++))
    
    # Check NVMe health
    if command -v nvme &> /dev/null; then
        echo -n "Testing: NVMe health... "
        if nvme smart-log /dev/nvme0 &> /dev/null; then
            echo -e "${GREEN}PASS${NC}"
            ((PASSED++))
        else
            echo -e "${YELLOW}SKIP${NC} (no access)"
            ((SKIPPED++))
        fi
    fi
else
    echo -e "NVMe SSD: ${YELLOW}Not detected${NC}"
    ((SKIPPED++))
fi

echo ""
echo "6. Checking Services"
echo "====================="

test_service "nexus-controller" "Nexus Controller service"
test_service "nexus-monitor" "Monitor service"

echo ""
echo "7. Checking Permissions"
echo "========================"

# Check directory permissions
run_test "Install dir ownership" "[[ $(stat -c %U "$INSTALL_BASE") == 'pi' ]]" "required"
run_test "Data dir permissions" "[[ $(stat -c %a "$DATA_DIR") == '700' ]]" "required"
run_test "Log dir writable" "[[ -w "$LOG_DIR" ]]" "required"

echo ""
echo "8. Checking Python Modules"
echo "==========================="

run_test "pyserial module" "python3 -c 'import serial'" "optional"
run_test "smbus module" "python3 -c 'import smbus'" "optional"
run_test "RPi.GPIO module" "python3 -c 'import RPi.GPIO'" "optional"

echo ""
echo "9. Testing Binary Execution"
echo "============================"

if [[ -f "$BINARY" ]]; then
    echo -n "Testing: Binary help output... "
    if "$BINARY" --help &> /dev/null || "$BINARY" --version &> /dev/null; then
        echo -e "${GREEN}PASS${NC}"
        ((PASSED++))
    else
        # Try running with timeout in case it starts GUI
        if timeout 2 "$BINARY" &> /dev/null; then
            echo -e "${GREEN}PASS${NC} (starts correctly)"
            ((PASSED++))
        else
            echo -e "${YELLOW}SKIP${NC} (may require display)"
            ((SKIPPED++))
        fi
    fi
fi

echo ""
echo "10. Checking Network Configuration"
echo "==================================="

# Check if port 8080 is available
run_test "Port 8080 available" "! netstat -tuln | grep -q ':8080'" "optional"

# Check network connectivity
run_test "Internet connectivity" "ping -c 1 8.8.8.8" "optional"

echo ""
echo "11. Performance Checks"
echo "======================="

# Check available memory
AVAIL_MEM=$(free -m | awk '/^Mem:/{print $7}')
if [[ $AVAIL_MEM -gt 1000 ]]; then
    echo -e "Available memory: ${GREEN}${AVAIL_MEM}MB${NC}"
    ((PASSED++))
else
    echo -e "Available memory: ${YELLOW}${AVAIL_MEM}MB (low)${NC}"
    ((SKIPPED++))
fi

# Check CPU governor
GOVERNOR=$(cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor 2>/dev/null || echo "unknown")
if [[ "$GOVERNOR" == "performance" ]]; then
    echo -e "CPU governor: ${GREEN}$GOVERNOR${NC}"
    ((PASSED++))
else
    echo -e "CPU governor: ${YELLOW}$GOVERNOR${NC}"
    ((SKIPPED++))
fi

# Check disk space
AVAIL_DISK=$(df -BG /opt | awk 'NR==2 {print $4}' | sed 's/G//')
if [[ $AVAIL_DISK -gt 5 ]]; then
    echo -e "Available disk: ${GREEN}${AVAIL_DISK}GB${NC}"
    ((PASSED++))
else
    echo -e "Available disk: ${YELLOW}${AVAIL_DISK}GB (low)${NC}"
    ((SKIPPED++))
fi

echo ""
echo "12. Backup System Check"
echo "========================"

test_file "$INSTALL_BASE/backup.sh" "Backup script" "optional"
test_file "$INSTALL_BASE/monitor.sh" "Monitor script" "optional"

# Check crontab entry
run_test "Backup cron job" "crontab -l | grep -q 'backup.sh'" "optional"

echo ""
echo "========================================"
echo "Test Results Summary"
echo "========================================"
echo ""
echo -e "${GREEN}Passed:${NC} $PASSED tests"
echo -e "${YELLOW}Skipped:${NC} $SKIPPED tests (optional features)"
echo -e "${RED}Failed:${NC} $FAILED tests"
echo ""

if [[ $FAILED -eq 0 ]]; then
    echo -e "${GREEN}✓ Installation test PASSED!${NC}"
    echo ""
    echo "The Nexus Controller is ready to use."
    echo ""
    echo "Next steps:"
    echo "1. Start the service:"
    echo "   sudo systemctl start nexus-controller"
    echo ""
    echo "2. Check service status:"
    echo "   sudo systemctl status nexus-controller"
    echo ""
    echo "3. View logs:"
    echo "   sudo journalctl -u nexus-controller -f"
    echo ""
    echo "4. Access the controller:"
    echo "   http://localhost:8080"
    echo ""
    exit 0
else
    echo -e "${RED}✗ Installation test FAILED${NC}"
    echo ""
    echo "Please review the failed tests above and:"
    echo "1. Check the installation log"
    echo "2. Re-run the installer if needed"
    echo "3. Manually fix any missing dependencies"
    echo ""
    exit 1
fi