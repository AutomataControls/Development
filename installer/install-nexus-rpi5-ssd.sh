#!/bin/bash

# Automata Nexus AI Controller - Raspberry Pi 5 SSD Installer
# Copyright (c) 2025 Automata Controls
# Developed by Andrew Jewell Sr.

set -e

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
SSD_DEVICE=""
INSTALL_PATH="/opt/nexus"
DATA_PATH="/var/lib/nexus"
CONFIG_PATH="/etc/nexus"
LOG_PATH="/var/log/nexus"
BACKUP_PATH="/var/backups/nexus"

# Print colored output
print_info() {
    echo -e "${CYAN}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# ASCII Art Banner
show_banner() {
    echo -e "${CYAN}"
    cat << "EOF"
    ___         __                        __          _   __                      
   /   | __  __/ /_____  ____ ___  ____ _/ /_____ _  / | / /__  _  ____  _______
  / /| |/ / / / __/ __ \/ __ `__ \/ __ `/ __/ __ `/ /  |/ / _ \| |/_/ / / / ___/
 / ___ / /_/ / /_/ /_/ / / / / / / /_/ / /_/ /_/ / / /|  /  __/>  </ /_/ (__  ) 
/_/  |_\__,_/\__/\____/_/ /_/ /_/\__,_/\__/\__,_/ /_/ |_/\___/_/|_|\__,_/____/  
                                                                                 
            Building Automation Controller for Raspberry Pi 5
                        SSD Installation Script v1.0
EOF
    echo -e "${NC}"
}

# Check if running as root
check_root() {
    if [[ $EUID -ne 0 ]]; then
        print_error "This script must be run as root"
        exit 1
    fi
}

# Check system requirements
check_system() {
    print_info "Checking system requirements..."
    
    # Check if running on Raspberry Pi 5
    if ! grep -q "Raspberry Pi 5" /proc/device-tree/model 2>/dev/null; then
        print_warning "This system does not appear to be a Raspberry Pi 5"
        read -p "Continue anyway? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            exit 1
        fi
    fi
    
    # Check OS version
    if ! grep -q "bookworm" /etc/os-release 2>/dev/null; then
        print_warning "This system is not running Debian Bookworm"
    fi
    
    # Check available memory
    total_mem=$(free -m | awk '/^Mem:/{print $2}')
    if [ "$total_mem" -lt 2048 ]; then
        print_warning "System has less than 2GB RAM. Performance may be affected."
    fi
    
    print_success "System check complete"
}

# Detect and prepare SSD
prepare_ssd() {
    print_info "Detecting SSD..."
    
    # List available disks
    echo "Available disks:"
    lsblk -d -o NAME,SIZE,TYPE,MODEL | grep -E "disk|nvme"
    
    # Auto-detect NVMe SSD
    nvme_devices=$(lsblk -d -o NAME,TYPE | grep nvme | awk '{print $1}')
    if [ -n "$nvme_devices" ]; then
        SSD_DEVICE="/dev/$(echo $nvme_devices | awk '{print $1}')"
        print_info "Detected NVMe SSD: $SSD_DEVICE"
    else
        # Look for USB SSDs
        usb_devices=$(lsblk -d -o NAME,TRAN | grep usb | awk '{print $1}')
        if [ -n "$usb_devices" ]; then
            SSD_DEVICE="/dev/$(echo $usb_devices | awk '{print $1}')"
            print_info "Detected USB SSD: $SSD_DEVICE"
        fi
    fi
    
    if [ -z "$SSD_DEVICE" ]; then
        print_error "No SSD detected. Please connect an SSD and try again."
        exit 1
    fi
    
    print_warning "Selected device: $SSD_DEVICE"
    print_warning "ALL DATA ON THIS DEVICE WILL BE ERASED!"
    read -p "Continue with installation on $SSD_DEVICE? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
    
    # Partition the SSD
    print_info "Partitioning SSD..."
    
    # Unmount any existing partitions
    umount ${SSD_DEVICE}* 2>/dev/null || true
    
    # Create partition table
    parted -s $SSD_DEVICE mklabel gpt
    
    # Create partitions
    # 1. Boot partition (512MB)
    parted -s $SSD_DEVICE mkpart primary fat32 1MiB 513MiB
    parted -s $SSD_DEVICE set 1 boot on
    
    # 2. Root partition (50GB)
    parted -s $SSD_DEVICE mkpart primary ext4 513MiB 50GiB
    
    # 3. Data partition (remaining space)
    parted -s $SSD_DEVICE mkpart primary ext4 50GiB 100%
    
    # Wait for partitions to be created
    sleep 2
    partprobe $SSD_DEVICE
    
    # Format partitions
    print_info "Formatting partitions..."
    
    # Determine partition naming (nvme0n1p1 vs sda1)
    if [[ $SSD_DEVICE == *"nvme"* ]]; then
        BOOT_PART="${SSD_DEVICE}p1"
        ROOT_PART="${SSD_DEVICE}p2"
        DATA_PART="${SSD_DEVICE}p3"
    else
        BOOT_PART="${SSD_DEVICE}1"
        ROOT_PART="${SSD_DEVICE}2"
        DATA_PART="${SSD_DEVICE}3"
    fi
    
    mkfs.vfat -F 32 -n BOOT $BOOT_PART
    mkfs.ext4 -L nexus-root $ROOT_PART
    mkfs.ext4 -L nexus-data $DATA_PART
    
    print_success "SSD prepared successfully"
}

# Install system dependencies
install_dependencies() {
    print_info "Installing system dependencies..."
    
    apt-get update
    apt-get install -y \
        build-essential \
        pkg-config \
        libssl-dev \
        libsqlite3-dev \
        libwebkit2gtk-4.1-dev \
        libgtk-3-dev \
        libayatana-appindicator3-dev \
        librsvg2-dev \
        curl \
        wget \
        git \
        python3-pip \
        python3-dev \
        i2c-tools \
        can-utils \
        nginx \
        supervisor \
        redis-server \
        mosquitto \
        mosquitto-clients \
        nodejs \
        npm
    
    # Install Rust
    if ! command -v rustc &> /dev/null; then
        print_info "Installing Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source $HOME/.cargo/env
        
        # Add ARM64 target
        rustup target add aarch64-unknown-linux-gnu
    fi
    
    # Enable I2C and SPI
    print_info "Enabling I2C and SPI..."
    raspi-config nonint do_i2c 0
    raspi-config nonint do_spi 0
    
    # Add user to necessary groups
    usermod -a -G i2c,spi,gpio,dialout nexus 2>/dev/null || true
    
    print_success "Dependencies installed"
}

# Mount SSD partitions
mount_ssd() {
    print_info "Mounting SSD partitions..."
    
    # Create mount points
    mkdir -p /mnt/nexus-root
    mkdir -p /mnt/nexus-data
    
    # Mount partitions
    mount $ROOT_PART /mnt/nexus-root
    mount $DATA_PART /mnt/nexus-data
    
    print_success "SSD mounted"
}

# Install Nexus application
install_nexus() {
    print_info "Installing Nexus application..."
    
    # Create directory structure
    mkdir -p /mnt/nexus-root$INSTALL_PATH
    mkdir -p /mnt/nexus-data/lib/nexus
    mkdir -p /mnt/nexus-data/etc/nexus
    mkdir -p /mnt/nexus-data/log/nexus
    mkdir -p /mnt/nexus-data/backups/nexus
    
    # Build the application
    print_info "Building Nexus application..."
    cd /tmp
    
    # Clone or copy the source code
    if [ -d "/home/Automata/Development/projects/Rust-SSD-Nexus-Controller" ]; then
        cp -r /home/Automata/Development/projects/Rust-SSD-Nexus-Controller nexus-build
    else
        git clone https://github.com/automata-nexus/controller.git nexus-build
    fi
    
    cd nexus-build
    
    # Build for ARM64
    cargo build --release --target aarch64-unknown-linux-gnu
    
    # Install binary
    cp target/aarch64-unknown-linux-gnu/release/nexus-controller /mnt/nexus-root$INSTALL_PATH/
    chmod +x /mnt/nexus-root$INSTALL_PATH/nexus-controller
    
    # Copy resources
    cp -r icons /mnt/nexus-root$INSTALL_PATH/
    cp -r config /mnt/nexus-data/etc/nexus/
    cp -r firmware /mnt/nexus-root$INSTALL_PATH/
    
    # Create symbolic links
    ln -sf /mnt/nexus-data/lib/nexus $DATA_PATH
    ln -sf /mnt/nexus-data/etc/nexus $CONFIG_PATH
    ln -sf /mnt/nexus-data/log/nexus $LOG_PATH
    ln -sf /mnt/nexus-data/backups/nexus $BACKUP_PATH
    
    print_success "Nexus application installed"
}

# Configure systemd service
configure_service() {
    print_info "Configuring systemd service..."
    
    cat > /etc/systemd/system/nexus.service << EOF
[Unit]
Description=Automata Nexus AI Controller
After=network.target

[Service]
Type=simple
User=nexus
Group=nexus
WorkingDirectory=$INSTALL_PATH
ExecStart=$INSTALL_PATH/nexus-controller
Restart=always
RestartSec=10
Environment="RUST_LOG=info"
Environment="DATABASE_URL=sqlite://$DATA_PATH/nexus.db"

# Security settings
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=$DATA_PATH $LOG_PATH $CONFIG_PATH $BACKUP_PATH

[Install]
WantedBy=multi-user.target
EOF
    
    # Create nexus user
    useradd -r -s /bin/false nexus 2>/dev/null || true
    
    # Set permissions
    chown -R nexus:nexus /mnt/nexus-root$INSTALL_PATH
    chown -R nexus:nexus /mnt/nexus-data/lib/nexus
    chown -R nexus:nexus /mnt/nexus-data/etc/nexus
    chown -R nexus:nexus /mnt/nexus-data/log/nexus
    chown -R nexus:nexus /mnt/nexus-data/backups/nexus
    
    # Enable service
    systemctl daemon-reload
    systemctl enable nexus.service
    
    print_success "Service configured"
}

# Configure nginx reverse proxy
configure_nginx() {
    print_info "Configuring nginx..."
    
    cat > /etc/nginx/sites-available/nexus << EOF
server {
    listen 80;
    listen [::]:80;
    server_name _;
    
    location / {
        proxy_pass http://localhost:1420;
        proxy_http_version 1.1;
        proxy_set_header Upgrade \$http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host \$host;
        proxy_cache_bypass \$http_upgrade;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
    }
    
    location /ws {
        proxy_pass http://localhost:1420/ws;
        proxy_http_version 1.1;
        proxy_set_header Upgrade \$http_upgrade;
        proxy_set_header Connection "upgrade";
    }
}
EOF
    
    ln -sf /etc/nginx/sites-available/nexus /etc/nginx/sites-enabled/
    rm -f /etc/nginx/sites-enabled/default
    
    nginx -t && systemctl restart nginx
    
    print_success "Nginx configured"
}

# Configure boot from SSD
configure_boot() {
    print_info "Configuring boot from SSD..."
    
    # Update /boot/firmware/cmdline.txt to boot from SSD
    cp /boot/firmware/cmdline.txt /boot/firmware/cmdline.txt.bak
    
    # Get UUID of root partition
    ROOT_UUID=$(blkid -s UUID -o value $ROOT_PART)
    
    # Update boot configuration
    sed -i "s|root=[^ ]*|root=UUID=$ROOT_UUID|" /boot/firmware/cmdline.txt
    
    # Update fstab
    cat > /mnt/nexus-root/etc/fstab << EOF
# /etc/fstab: static file system information
UUID=$ROOT_UUID / ext4 defaults,noatime 0 1
UUID=$(blkid -s UUID -o value $BOOT_PART) /boot/firmware vfat defaults 0 2
UUID=$(blkid -s UUID -o value $DATA_PART) /data ext4 defaults,noatime 0 2
EOF
    
    print_success "Boot configured"
}

# Install board drivers
install_drivers() {
    print_info "Installing board drivers..."
    
    # Install Sequent Microsystems drivers
    cd /tmp
    
    # Megabas driver
    git clone https://github.com/SequentMicrosystems/megabas-rpi.git
    cd megabas-rpi
    make install
    cd ..
    
    # 8-relay driver
    git clone https://github.com/SequentMicrosystems/8relind-rpi.git
    cd 8relind-rpi
    make install
    cd ..
    
    # RTD driver
    git clone https://github.com/SequentMicrosystems/rtd-rpi.git
    cd rtd-rpi
    make install
    cd ..
    
    print_success "Drivers installed"
}

# Final setup
final_setup() {
    print_info "Performing final setup..."
    
    # Initialize database
    sudo -u nexus $INSTALL_PATH/nexus-controller --init-db
    
    # Set default admin password
    sudo -u nexus $INSTALL_PATH/nexus-controller --reset-admin-password
    
    # Start service
    systemctl start nexus.service
    
    # Get IP address
    IP_ADDR=$(hostname -I | awk '{print $1}')
    
    print_success "Installation complete!"
    echo
    echo -e "${GREEN}========================================${NC}"
    echo -e "${GREEN}     Automata Nexus AI Installed!      ${NC}"
    echo -e "${GREEN}========================================${NC}"
    echo
    echo -e "${CYAN}Access the web interface at:${NC}"
    echo -e "${YELLOW}http://$IP_ADDR${NC}"
    echo
    echo -e "${CYAN}Default credentials:${NC}"
    echo -e "Username: ${YELLOW}admin${NC}"
    echo -e "Password: ${YELLOW}Nexus${NC}"
    echo
    echo -e "${RED}Please change the default password after first login!${NC}"
    echo
    echo -e "${CYAN}To reboot from SSD, run:${NC}"
    echo -e "${YELLOW}sudo reboot${NC}"
}

# Main installation flow
main() {
    show_banner
    check_root
    check_system
    prepare_ssd
    mount_ssd
    install_dependencies
    install_nexus
    configure_service
    configure_nginx
    configure_boot
    install_drivers
    final_setup
}

# Run main function
main "$@"