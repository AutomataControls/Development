#!/bin/bash

################################################################################
# Automata Nexus Controller - Complete Native Rust Installation Script
# For Raspberry Pi 5 with NVMe SSD
# Version: 2.0.0 - Full Production Build
################################################################################

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Installation paths
INSTALL_BASE="/opt/automata-nexus"
SRC_DIR="$INSTALL_BASE/src"
BUILD_DIR="$INSTALL_BASE/build"
DATA_DIR="$INSTALL_BASE/data"
LOG_DIR="$INSTALL_BASE/logs"
CONFIG_DIR="$INSTALL_BASE/config"

# Log file
LOG_FILE="/tmp/nexus_install_$(date +%Y%m%d_%H%M%S).log"

# Function to print colored output
print_status() {
    echo -e "${GREEN}[✓]${NC} $1"
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" >> "$LOG_FILE"
}

print_error() {
    echo -e "${RED}[✗]${NC} $1"
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] ERROR: $1" >> "$LOG_FILE"
}

print_warning() {
    echo -e "${YELLOW}[!]${NC} $1"
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] WARNING: $1" >> "$LOG_FILE"
}

print_info() {
    echo -e "${BLUE}[i]${NC} $1"
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] INFO: $1" >> "$LOG_FILE"
}

# Check if running as root
check_root() {
    if [[ $EUID -ne 0 ]]; then
        print_error "This script must be run as root (use sudo)"
        exit 1
    fi
}

# Setup NVMe SSD and migrate OS
setup_nvme_ssd() {
    print_info "Setting up NVMe SSD and migrating OS..."
    
    # Check for NVMe device
    if ! lsblk | grep -q nvme; then
        print_error "No NVMe SSD detected! Please install an NVMe SSD and try again."
        exit 1
    fi
    
    NVME_DEVICE="/dev/$(lsblk | grep nvme | head -1 | awk '{print $1}')"
    print_status "Found NVMe device: $NVME_DEVICE"
    
    # Check if already running from NVMe
    ROOT_DEVICE=$(mount | grep " / " | cut -d' ' -f1)
    if [[ "$ROOT_DEVICE" == *"nvme"* ]]; then
        print_status "System already running from NVMe SSD"
        return 0
    fi
    
    print_warning "System currently running from SD card. Will migrate to NVMe SSD."
    
    # Partition the NVMe drive
    print_info "Partitioning NVMe SSD..."
    
    # Unmount if mounted
    umount ${NVME_DEVICE}* 2>/dev/null || true
    
    # Create GPT partition table
    parted -s ${NVME_DEVICE} mklabel gpt
    
    # Create boot partition (512MB)
    parted -s ${NVME_DEVICE} mkpart primary fat32 1MiB 513MiB
    parted -s ${NVME_DEVICE} set 1 boot on
    
    # Create root partition (rest of disk)
    parted -s ${NVME_DEVICE} mkpart primary ext4 513MiB 100%
    
    # Wait for partitions to appear
    sleep 2
    partprobe ${NVME_DEVICE}
    
    # Format partitions
    print_info "Formatting partitions..."
    mkfs.vfat -F 32 ${NVME_DEVICE}p1
    mkfs.ext4 -F -O ^has_journal,extent,huge_file,flex_bg,dir_index,64bit,extent,sparse_super2 ${NVME_DEVICE}p2
    
    # Mount partitions
    print_info "Mounting NVMe partitions..."
    mkdir -p /mnt/ssd
    mount ${NVME_DEVICE}p2 /mnt/ssd
    mkdir -p /mnt/ssd/boot/firmware
    mount ${NVME_DEVICE}p1 /mnt/ssd/boot/firmware
    
    # Clone OS to NVMe
    print_info "Cloning OS to NVMe SSD (this will take 10-15 minutes)..."
    
    # Use rsync to copy everything except certain directories
    rsync -axHAWXS --numeric-ids --info=progress2 \
        --exclude=/proc/* \
        --exclude=/sys/* \
        --exclude=/dev/* \
        --exclude=/tmp/* \
        --exclude=/run/* \
        --exclude=/mnt/* \
        --exclude=/media/* \
        --exclude=/lost+found \
        / /mnt/ssd/
    
    # Create excluded directories
    mkdir -p /mnt/ssd/{proc,sys,dev,tmp,run,mnt,media}
    
    # Update fstab for NVMe
    print_info "Updating fstab for NVMe boot..."
    BOOT_UUID=$(blkid -s UUID -o value ${NVME_DEVICE}p1)
    ROOT_UUID=$(blkid -s UUID -o value ${NVME_DEVICE}p2)
    
    cat > /mnt/ssd/etc/fstab << EOF
proc            /proc           proc    defaults          0       0
UUID=$ROOT_UUID /               ext4    defaults,noatime,discard  0       1
UUID=$BOOT_UUID /boot/firmware  vfat    defaults          0       2
tmpfs           /tmp            tmpfs   defaults,noatime,mode=1777  0       0
EOF
    
    # Update boot configuration
    print_info "Updating boot configuration..."
    
    # Update cmdline.txt for NVMe boot
    sed -i "s|root=[^ ]*|root=UUID=$ROOT_UUID|g" /mnt/ssd/boot/firmware/cmdline.txt
    
    # Update config.txt for PCIe Gen3
    if ! grep -q "dtparam=pciex1" /mnt/ssd/boot/firmware/config.txt; then
        cat >> /mnt/ssd/boot/firmware/config.txt << EOF

# Enable PCIe for NVMe SSD
dtparam=pciex1
dtparam=pciex1_gen=3
dtparam=nvme
EOF
    fi
    
    # Update bootloader for NVMe boot priority
    print_info "Configuring bootloader for NVMe boot..."
    
    # Install rpi-eeprom if not present
    apt-get install -y rpi-eeprom
    
    # Update bootloader
    rpi-eeprom-update -a
    
    # Set boot order to prefer NVMe
    cat > /tmp/bootconf.txt << EOF
[all]
BOOT_UART=0
WAKE_ON_GPIO=1
POWER_OFF_ON_HALT=0

# Boot from NVMe first, then SD card, then USB
BOOT_ORDER=0xf416

# Enable PCIe
PCIE_PROBE=1
EOF
    
    rpi-eeprom-config --apply /tmp/bootconf.txt
    
    print_status "NVMe SSD setup complete! System will boot from NVMe after reboot."
    print_warning "SD card can be removed after reboot (keep as backup if desired)."
    
    # Set global flag that we set up NVMe
    NVME_SETUP_DONE=true
}

# Check hardware
check_hardware() {
    print_info "Checking hardware requirements..."
    
    # Check for Raspberry Pi 5
    if grep -q "Raspberry Pi 5" /proc/cpuinfo; then
        print_status "Raspberry Pi 5 detected"
    else
        print_warning "Not running on Raspberry Pi 5 - continuing anyway"
    fi
    
    # Check for 64-bit OS
    if [[ $(uname -m) == "aarch64" ]]; then
        print_status "64-bit OS detected"
    else
        print_error "64-bit OS required. Please install 64-bit Raspberry Pi OS"
        exit 1
    fi
    
    # Setup NVMe SSD first
    setup_nvme_ssd
    
    # Check available memory
    TOTAL_MEM=$(free -m | awk '/^Mem:/{print $2}')
    if [[ $TOTAL_MEM -lt 4000 ]]; then
        print_warning "Less than 4GB RAM detected. Build may be slow."
    else
        print_status "Adequate RAM detected: ${TOTAL_MEM}MB"
    fi
}

# Enable hardware interfaces
enable_hardware() {
    print_info "Enabling hardware interfaces..."
    
    # Enable I2C
    raspi-config nonint do_i2c 0 || true
    
    # Enable SPI
    raspi-config nonint do_spi 0 || true
    
    # Add modules
    if ! grep -q "i2c-dev" /etc/modules; then
        echo "i2c-dev" >> /etc/modules
    fi
    if ! grep -q "i2c-bcm2835" /etc/modules; then
        echo "i2c-bcm2835" >> /etc/modules
    fi
    
    print_status "Hardware interfaces enabled"
}

# Optimize system for RPi5
optimize_system() {
    print_info "Optimizing system for Raspberry Pi 5..."
    
    # Enable PCIe Gen3 for NVMe
    if ! grep -q "dtparam=pciex1" /boot/firmware/config.txt; then
        cat >> /boot/firmware/config.txt << EOF

# Enable PCIe for NVMe SSD
dtparam=pciex1
dtparam=pciex1_gen=3
EOF
        print_status "PCIe Gen3 enabled for NVMe"
    fi
    
    # Set performance governor
    for cpu in /sys/devices/system/cpu/cpu[0-3]/cpufreq/scaling_governor; do
        echo performance > $cpu 2>/dev/null || true
    done
    
    # Optimize kernel parameters
    cat > /etc/sysctl.d/99-nexus-optimization.conf << EOF
# Nexus Controller Optimizations
vm.swappiness=10
vm.vfs_cache_pressure=50
vm.dirty_background_ratio=5
vm.dirty_ratio=10
net.core.rmem_max=134217728
net.core.wmem_max=134217728
fs.file-max=2097152
kernel.pid_max=4194304
EOF
    
    sysctl -p /etc/sysctl.d/99-nexus-optimization.conf > /dev/null 2>&1
    
    print_status "System optimizations applied"
}

# Update system packages
update_system() {
    print_info "Updating system packages..."
    
    # Add Rust repository
    if ! grep -q "debian.rust-lang.org" /etc/apt/sources.list.d/*.list 2>/dev/null; then
        curl -sSf https://packages.cloud.google.com/apt/doc/apt-key.gpg | apt-key add - || true
    fi
    
    apt-get update >> "$LOG_FILE" 2>&1
    
    # Don't upgrade everything to save time, just update package lists
    print_status "Package lists updated"
}

# Install system dependencies
install_dependencies() {
    print_info "Installing system dependencies..."
    
    # Essential packages for Rust compilation
    ESSENTIAL_PACKAGES=(
        build-essential
        gcc
        g++
        make
        cmake
        pkg-config
        libssl-dev
        libclang-dev
        llvm-dev
        clang
    )
    
    # GUI dependencies for egui
    GUI_PACKAGES=(
        libx11-dev
        libxcb1-dev
        libxcb-render0-dev
        libxcb-shape0-dev
        libxcb-xfixes0-dev
        libxkbcommon-dev
        libgl1-mesa-dev
        libglu1-mesa-dev
        libegl1-mesa-dev
        libwayland-dev
        libxrandr-dev
        libxi-dev
        libxxf86vm-dev
    )
    
    # Hardware communication packages
    HARDWARE_PACKAGES=(
        libudev-dev
        libusb-1.0-0-dev
        i2c-tools
        python3-smbus
        python3-dev
        python3-pip
    )
    
    # Database and networking
    DB_PACKAGES=(
        sqlite3
        libsqlite3-dev
        redis-server
        curl
        wget
        git
        jq
    )
    
    # Performance monitoring
    MONITORING_PACKAGES=(
        htop
        iotop
        sysstat
        nvme-cli
    )
    
    # Install all packages
    print_info "Installing essential packages..."
    for package in "${ESSENTIAL_PACKAGES[@]}"; do
        apt-get install -y "$package" >> "$LOG_FILE" 2>&1 || print_warning "Failed to install $package"
    done
    
    print_info "Installing GUI packages..."
    for package in "${GUI_PACKAGES[@]}"; do
        apt-get install -y "$package" >> "$LOG_FILE" 2>&1 || print_warning "Failed to install $package"
    done
    
    print_info "Installing hardware packages..."
    for package in "${HARDWARE_PACKAGES[@]}"; do
        apt-get install -y "$package" >> "$LOG_FILE" 2>&1 || print_warning "Failed to install $package"
    done
    
    print_info "Installing database packages..."
    for package in "${DB_PACKAGES[@]}"; do
        apt-get install -y "$package" >> "$LOG_FILE" 2>&1 || print_warning "Failed to install $package"
    done
    
    print_info "Installing monitoring packages..."
    for package in "${MONITORING_PACKAGES[@]}"; do
        apt-get install -y "$package" >> "$LOG_FILE" 2>&1 || print_warning "Failed to install $package"
    done
    
    print_status "System dependencies installed"
}

# Install Rust toolchain
install_rust() {
    print_info "Installing Rust toolchain..."
    
    # Check if Rust is already installed
    if command -v rustc &> /dev/null; then
        RUST_VERSION=$(rustc --version | awk '{print $2}')
        print_info "Rust $RUST_VERSION already installed"
        
        # Update to latest
        rustup update >> "$LOG_FILE" 2>&1 || true
    else
        # Install Rust
        print_info "Downloading Rust installer..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y >> "$LOG_FILE" 2>&1
        
        # Source cargo env
        source "$HOME/.cargo/env"
    fi
    
    # Add ARM64 target
    rustup target add aarch64-unknown-linux-gnu >> "$LOG_FILE" 2>&1
    
    # Install additional tools
    cargo install cargo-binutils >> "$LOG_FILE" 2>&1 || true
    cargo install cargo-edit >> "$LOG_FILE" 2>&1 || true
    
    print_status "Rust toolchain installed"
}

# Install Python libraries
install_python_libs() {
    print_info "Installing Python libraries..."
    
    # Upgrade pip
    python3 -m pip install --upgrade pip >> "$LOG_FILE" 2>&1
    
    # Install required Python packages
    PYTHON_PACKAGES=(
        pyserial
        pymodbus
        influxdb-client
        redis
        psutil
        RPi.GPIO
        smbus2
        adafruit-circuitpython-ads1x15
    )
    
    for package in "${PYTHON_PACKAGES[@]}"; do
        python3 -m pip install "$package" >> "$LOG_FILE" 2>&1 || print_warning "Failed to install $package"
    done
    
    print_status "Python libraries installed"
}

# Install Sequent Microsystems libraries
install_sequent_libs() {
    print_info "Installing Sequent Microsystems libraries..."
    
    # Check if firmware directory exists locally
    FIRMWARE_DIR="/home/Automata/firmware"
    
    if [[ -d "$FIRMWARE_DIR" ]]; then
        print_info "Using local firmware directory..."
        
        # Install from local firmware directory
        BOARD_DIRS=(
            "megabas-rpi"
            "8relind-rpi"
            "16relind-rpi"
            "16univin-rpi"
            "16uout-rpi"
        )
        
        for board_dir in "${BOARD_DIRS[@]}"; do
            BOARD_PATH="$FIRMWARE_DIR/$board_dir"
            
            if [[ -d "$BOARD_PATH" ]]; then
                print_info "Installing $board_dir from local firmware..."
                cd "$BOARD_PATH"
                
                # Compile C drivers if Makefile exists
                if [[ -f "Makefile" ]]; then
                    print_info "Compiling $board_dir drivers..."
                    make clean >> "$LOG_FILE" 2>&1
                    make >> "$LOG_FILE" 2>&1
                    make install >> "$LOG_FILE" 2>&1 || print_warning "Failed to install $board_dir drivers"
                fi
                
                # Install Python library
                if [[ -d "python" ]]; then
                    cd python
                    if [[ -f "setup.py" ]]; then
                        python3 setup.py install >> "$LOG_FILE" 2>&1 || print_warning "Failed to install $board_dir Python library"
                    fi
                    cd ..
                elif [[ -f "setup.py" ]]; then
                    python3 setup.py install >> "$LOG_FILE" 2>&1 || print_warning "Failed to install $board_dir Python library"
                fi
            fi
        done
        
    else
        print_info "Downloading firmware from GitHub..."
        
        TEMP_DIR="/tmp/sequent_libs"
        mkdir -p "$TEMP_DIR"
        cd "$TEMP_DIR"
        
        # List of Sequent repos
        SEQUENT_REPOS=(
            "megabas-rpi"
            "megaind-rpi"
            "16relind-rpi"
            "16univin-rpi"
            "8relind-rpi"
            "16uout-rpi"
        )
        
        for repo in "${SEQUENT_REPOS[@]}"; do
            print_info "Installing $repo..."
            
            # Clone repository
            git clone "https://github.com/SequentMicrosystems/${repo}.git" >> "$LOG_FILE" 2>&1 || continue
            
            cd "$repo"
            
            # Compile C drivers if Makefile exists
            if [[ -f "Makefile" ]]; then
                print_info "Compiling $repo drivers..."
                make clean >> "$LOG_FILE" 2>&1
                make >> "$LOG_FILE" 2>&1
                make install >> "$LOG_FILE" 2>&1 || print_warning "Failed to install $repo drivers"
            fi
            
            # Install Python library
            if [[ -d "python" ]]; then
                cd python
                if [[ -f "setup.py" ]]; then
                    python3 setup.py install >> "$LOG_FILE" 2>&1 || print_warning "Failed to install $repo Python library"
                fi
                cd ..
            elif [[ -f "setup.py" ]]; then
                python3 setup.py install >> "$LOG_FILE" 2>&1 || print_warning "Failed to install $repo Python library"
            fi
            
            cd ..
        done
        
        # Cleanup
        cd /
        rm -rf "$TEMP_DIR"
    fi
    
    # Copy firmware interface to install location
    if [[ -f "$SCRIPT_DIR/../src/firmware_interface.py" ]]; then
        cp "$SCRIPT_DIR/../src/firmware_interface.py" "$INSTALL_BASE/firmware_interface.py"
        chmod +x "$INSTALL_BASE/firmware_interface.py"
        print_status "Firmware interface installed"
    fi
    
    # Test firmware installation
    print_info "Testing firmware installation..."
    python3 -c "import megabas; print('MegaBAS OK')" >> "$LOG_FILE" 2>&1 || print_warning "MegaBAS library not working"
    python3 -c "import lib8relind; print('8-Relay OK')" >> "$LOG_FILE" 2>&1 || print_warning "8-Relay library not working"
    python3 -c "import SM16relind; print('16-Relay OK')" >> "$LOG_FILE" 2>&1 || print_warning "16-Relay library not working"
    python3 -c "import lib16univin; print('16-UnivIn OK')" >> "$LOG_FILE" 2>&1 || print_warning "16-UnivIn library not working"
    
    print_status "Sequent libraries installed"
}

# Create directory structure
create_directories() {
    print_info "Creating directory structure..."
    
    mkdir -p "$INSTALL_BASE"
    mkdir -p "$SRC_DIR"
    mkdir -p "$BUILD_DIR"
    mkdir -p "$DATA_DIR"
    mkdir -p "$LOG_DIR"
    mkdir -p "$CONFIG_DIR"
    mkdir -p "$INSTALL_BASE/backups"
    mkdir -p "$INSTALL_BASE/cache"
    
    print_status "Directory structure created"
}

# Copy source code
copy_source() {
    print_info "Copying source code..."
    
    # Find the source directory
    SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
    
    # Copy all Rust source files
    if [[ -d "$PROJECT_ROOT/src" ]]; then
        cp -r "$PROJECT_ROOT/src" "$SRC_DIR/"
        print_status "Source code copied"
    else
        print_error "Source directory not found at $PROJECT_ROOT/src"
        exit 1
    fi
    
    # Copy Cargo files
    if [[ -f "$PROJECT_ROOT/Cargo_complete.toml" ]]; then
        cp "$PROJECT_ROOT/Cargo_complete.toml" "$SRC_DIR/Cargo.toml"
    elif [[ -f "$PROJECT_ROOT/Cargo.toml" ]]; then
        cp "$PROJECT_ROOT/Cargo.toml" "$SRC_DIR/"
    fi
    
    # Copy any assets
    if [[ -d "$PROJECT_ROOT/assets" ]]; then
        cp -r "$PROJECT_ROOT/assets" "$SRC_DIR/"
    fi
}

# Configure Cargo for optimal build
configure_cargo() {
    print_info "Configuring Cargo for optimal build..."
    
    mkdir -p "$SRC_DIR/.cargo"
    
    cat > "$SRC_DIR/.cargo/config.toml" << 'EOF'
[build]
target = "aarch64-unknown-linux-gnu"
jobs = 4

[target.aarch64-unknown-linux-gnu]
linker = "aarch64-linux-gnu-gcc"
rustflags = [
    "-C", "target-cpu=cortex-a76",
    "-C", "opt-level=3",
    "-C", "lto=thin",
    "-C", "codegen-units=1",
    "-C", "embed-bitcode=yes"
]

[profile.release]
opt-level = 3
lto = "thin"
codegen-units = 1
strip = true
debug = false
panic = "abort"

[net]
retry = 5
git-fetch-with-cli = true
EOF
    
    print_status "Cargo configured for RPi5"
}

# Build the Rust application
build_application() {
    print_info "Building Nexus Controller application..."
    
    cd "$SRC_DIR"
    
    # Set environment variables for build
    export CARGO_HOME="$HOME/.cargo"
    export RUSTUP_HOME="$HOME/.rustup"
    export PATH="$CARGO_HOME/bin:$PATH"
    export RUST_BACKTRACE=1
    export CARGO_TARGET_DIR="$BUILD_DIR"
    
    # Clean any previous builds
    cargo clean >> "$LOG_FILE" 2>&1 || true
    
    # Update dependencies
    print_info "Updating dependencies..."
    cargo update >> "$LOG_FILE" 2>&1
    
    # Build in release mode
    print_info "Starting build (this will take 10-20 minutes)..."
    
    # Use nice to lower priority so system stays responsive
    nice -n 10 cargo build --release --target aarch64-unknown-linux-gnu 2>&1 | while read -r line; do
        if [[ $line == *"Compiling"* ]]; then
            # Extract package name being compiled
            package=$(echo "$line" | sed -n 's/.*Compiling \([^ ]*\).*/\1/p')
            echo -ne "\rCompiling: $package                    "
        elif [[ $line == *"error"* ]]; then
            echo ""
            print_error "$line"
        elif [[ $line == *"warning"* ]]; then
            echo "$line" >> "$LOG_FILE"
        fi
    done
    
    echo "" # New line after progress
    
    # Check if build succeeded
    if [[ -f "$BUILD_DIR/aarch64-unknown-linux-gnu/release/nexus-controller" ]]; then
        print_status "Application built successfully"
        
        # Copy binary to install location
        cp "$BUILD_DIR/aarch64-unknown-linux-gnu/release/nexus-controller" "$INSTALL_BASE/nexus-controller"
        chmod +x "$INSTALL_BASE/nexus-controller"
    else
        print_error "Build failed - binary not found"
        print_info "Check $LOG_FILE for details"
        exit 1
    fi
}

# Setup database
setup_database() {
    print_info "Setting up database..."
    
    # Create database initialization script
    cat > "$INSTALL_BASE/init_db.sql" << 'EOF'
-- Nexus Controller Database Schema

-- Settings table
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Board configurations
CREATE TABLE IF NOT EXISTS board_configs (
    board_id TEXT PRIMARY KEY,
    board_type TEXT NOT NULL,
    config TEXT NOT NULL,
    enabled BOOLEAN DEFAULT 1,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Channel data
CREATE TABLE IF NOT EXISTS channel_data (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    board_id TEXT NOT NULL,
    channel_type TEXT NOT NULL,
    channel_index INTEGER NOT NULL,
    value REAL NOT NULL,
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (board_id) REFERENCES board_configs(board_id)
);

-- Metrics
CREATE TABLE IF NOT EXISTS metrics (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    metric_name TEXT NOT NULL,
    metric_value REAL NOT NULL,
    tags TEXT,
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Alarms
CREATE TABLE IF NOT EXISTS alarms (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    alarm_type TEXT NOT NULL,
    severity TEXT NOT NULL,
    message TEXT NOT NULL,
    acknowledged BOOLEAN DEFAULT 0,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    acknowledged_at DATETIME
);

-- Audit log
CREATE TABLE IF NOT EXISTS audit_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user TEXT NOT NULL,
    action TEXT NOT NULL,
    details TEXT,
    ip_address TEXT,
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes
CREATE INDEX IF NOT EXISTS idx_channel_data_timestamp ON channel_data(timestamp);
CREATE INDEX IF NOT EXISTS idx_metrics_timestamp ON metrics(timestamp);
CREATE INDEX IF NOT EXISTS idx_alarms_severity ON alarms(severity);
CREATE INDEX IF NOT EXISTS idx_audit_log_timestamp ON audit_log(timestamp);

-- Insert default settings
INSERT OR IGNORE INTO settings (key, value) VALUES
    ('system_name', 'Nexus Controller'),
    ('version', '2.0.0'),
    ('location', 'Main Building'),
    ('timezone', 'America/New_York');
EOF
    
    # Initialize database
    sqlite3 "$DATA_DIR/nexus.db" < "$INSTALL_BASE/init_db.sql"
    
    print_status "Database initialized"
}

# Create configuration files
create_configs() {
    print_info "Creating configuration files..."
    
    # Main configuration
    cat > "$CONFIG_DIR/nexus.toml" << 'EOF'
# Nexus Controller Configuration

[system]
name = "Nexus Controller"
version = "2.0.0"
location = "Main Building"

[server]
host = "0.0.0.0"
port = 8080
workers = 4

[database]
path = "/opt/automata-nexus/data/nexus.db"
pool_size = 10
timeout = 30

[logging]
level = "info"
file = "/opt/automata-nexus/logs/nexus.log"
max_size = "100MB"
max_backups = 10

[hardware]
i2c_bus = 1
spi_bus = 0
gpio_chip = "/dev/gpiochip0"

[boards]
megabas = { enabled = true, address = 0 }
megaind = { enabled = false, address = 1 }

[security]
auth_enabled = true
session_timeout = 3600
pin_required = true
default_pin = "2196"
EOF
    
    # Create systemd service
    cat > "/etc/systemd/system/nexus-controller.service" << EOF
[Unit]
Description=Automata Nexus Controller
After=network.target

[Service]
Type=simple
User=pi
Group=pi
WorkingDirectory=$INSTALL_BASE
Environment="RUST_LOG=info"
Environment="DATABASE_PATH=$DATA_DIR/nexus.db"
ExecStart=$INSTALL_BASE/nexus-controller
Restart=always
RestartSec=10

# Performance
CPUSchedulingPolicy=fifo
CPUSchedulingPriority=50
IOSchedulingClass=realtime
IOSchedulingPriority=0

# Logging
StandardOutput=append:$LOG_DIR/nexus.log
StandardError=append:$LOG_DIR/nexus-error.log

[Install]
WantedBy=multi-user.target
EOF
    
    systemctl daemon-reload
    
    print_status "Configuration files created"
}

# Setup monitoring
setup_monitoring() {
    print_info "Setting up monitoring..."
    
    # Create monitoring script
    cat > "$INSTALL_BASE/monitor.sh" << 'EOF'
#!/bin/bash

# Nexus Controller Monitor

while true; do
    # Check if service is running
    if systemctl is-active nexus-controller > /dev/null; then
        echo "$(date): Service running"
    else
        echo "$(date): Service stopped - restarting"
        systemctl restart nexus-controller
    fi
    
    # Log system metrics
    echo "$(date): CPU: $(top -bn1 | grep "Cpu(s)" | awk '{print $2}')%"
    echo "$(date): Memory: $(free -m | awk '/^Mem:/{printf "%.1f%%", $3/$2*100}')"
    
    # Check NVMe health
    if command -v nvme &> /dev/null; then
        nvme smart-log /dev/nvme0 2>/dev/null | grep -E "temperature|available_spare"
    fi
    
    sleep 60
done
EOF
    
    chmod +x "$INSTALL_BASE/monitor.sh"
    
    # Create monitor service
    cat > "/etc/systemd/system/nexus-monitor.service" << EOF
[Unit]
Description=Nexus Controller Monitor
After=nexus-controller.service

[Service]
Type=simple
ExecStart=$INSTALL_BASE/monitor.sh
Restart=always
StandardOutput=append:$LOG_DIR/monitor.log

[Install]
WantedBy=multi-user.target
EOF
    
    systemctl daemon-reload
    systemctl enable nexus-monitor
    
    print_status "Monitoring configured"
}

# Setup backup
setup_backup() {
    print_info "Setting up automated backups..."
    
    # Create backup script
    cat > "$INSTALL_BASE/backup.sh" << 'EOF'
#!/bin/bash

BACKUP_DIR="/opt/automata-nexus/backups"
DATA_DIR="/opt/automata-nexus/data"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="$BACKUP_DIR/backup_$TIMESTAMP.tar.gz"

# Create backup
tar -czf "$BACKUP_FILE" -C "$DATA_DIR" .

# Keep only last 7 days of backups
find "$BACKUP_DIR" -name "backup_*.tar.gz" -mtime +7 -delete

echo "$(date): Backup completed - $BACKUP_FILE"
EOF
    
    chmod +x "$INSTALL_BASE/backup.sh"
    
    # Add to crontab
    (crontab -l 2>/dev/null; echo "0 2 * * * $INSTALL_BASE/backup.sh >> $LOG_DIR/backup.log 2>&1") | crontab -
    
    print_status "Backup system configured"
}

# Final setup
final_setup() {
    print_info "Performing final setup..."
    
    # Set permissions
    chown -R pi:pi "$INSTALL_BASE"
    chmod -R 755 "$INSTALL_BASE"
    chmod -R 777 "$LOG_DIR"
    chmod -R 700 "$DATA_DIR"
    
    # Enable services
    systemctl enable nexus-controller
    
    print_status "Final setup completed"
}

# Main installation function
main() {
    echo "=================================="
    echo "Automata Nexus Controller Installer"
    echo "Native Rust Implementation v2.0.0"
    echo "=================================="
    echo ""
    
    print_info "Installation log: $LOG_FILE"
    echo ""
    
    # Run installation steps
    check_root
    check_hardware
    enable_hardware
    optimize_system
    update_system
    install_dependencies
    install_rust
    install_python_libs
    install_sequent_libs
    create_directories
    copy_source
    configure_cargo
    build_application
    setup_database
    create_configs
    setup_monitoring
    setup_backup
    final_setup
    
    echo ""
    echo "=================================="
    print_status "Installation completed successfully!"
    echo "=================================="
    echo ""
    echo "Next steps:"
    echo "1. Reboot to apply hardware changes:"
    echo "   sudo reboot"
    echo ""
    echo "2. After reboot, start the service:"
    echo "   sudo systemctl start nexus-controller"
    echo ""
    echo "3. Check service status:"
    echo "   sudo systemctl status nexus-controller"
    echo ""
    echo "4. View logs:"
    echo "   sudo journalctl -u nexus-controller -f"
    echo ""
    echo "5. Access the controller:"
    echo "   http://localhost:8080"
    echo "   http://$(hostname -I | cut -d' ' -f1):8080"
    echo ""
    echo "Default PIN: 2196"
    echo ""
    echo "Installation log saved to: $LOG_FILE"
    echo ""
}

# Run main function
main "$@"