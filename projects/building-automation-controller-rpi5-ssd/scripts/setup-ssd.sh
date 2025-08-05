#!/bin/bash
# Quick SSD setup script for RPi5

echo "=== Automata Nexus RPi5 SSD Setup ==="
echo

# Check if running as root
if [ "$EUID" -ne 0 ]; then 
    echo "Please run as root (use sudo)"
    exit 1
fi

# Check for NVMe device
if [ ! -e /dev/nvme0n1 ]; then
    echo "Error: No NVMe device found at /dev/nvme0n1"
    echo "Please ensure your NVMe SSD is properly connected to the PCIe HAT"
    exit 1
fi

echo "Found NVMe device:"
nvme list | grep nvme0

# Check if already partitioned
if [ -e /dev/nvme0n1p1 ]; then
    echo "Warning: /dev/nvme0n1 is already partitioned"
    read -p "Do you want to continue and format the existing partition? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
else
    # Create partition
    echo "Creating partition on /dev/nvme0n1..."
    parted -s /dev/nvme0n1 mklabel gpt
    parted -s /dev/nvme0n1 mkpart primary ext4 0% 100%
    sleep 2
fi

# Format partition
echo "Formatting /dev/nvme0n1p1 with ext4..."
mkfs.ext4 -F /dev/nvme0n1p1

# Create mount point
echo "Creating mount point at /mnt/ssd..."
mkdir -p /mnt/ssd

# Mount the SSD
echo "Mounting SSD..."
mount /dev/nvme0n1p1 /mnt/ssd

# Get UUID
UUID=$(blkid -s UUID -o value /dev/nvme0n1p1)
echo "SSD UUID: $UUID"

# Add to fstab
echo "Adding to /etc/fstab..."
if ! grep -q "$UUID" /etc/fstab; then
    echo "UUID=$UUID /mnt/ssd ext4 defaults,noatime,nodiratime 0 2" >> /etc/fstab
    echo "Added to fstab"
else
    echo "Already in fstab"
fi

# Create directory structure
echo "Creating directory structure..."
mkdir -p /mnt/ssd/automata-nexus/{app,data,logs,cache,backups}
mkdir -p /mnt/ssd/metrics

# Set permissions
echo "Setting permissions..."
chown -R Automata:Automata /mnt/ssd/automata-nexus 2>/dev/null || chown -R pi:pi /mnt/ssd/automata-nexus

# Enable TRIM support
echo "Enabling TRIM support..."
systemctl enable fstrim.timer

# Display SSD info
echo
echo "=== SSD Setup Complete ==="
echo "Mount point: /mnt/ssd"
echo "Space available: $(df -h /mnt/ssd | tail -1 | awk '{print $4}')"
echo
echo "SSD Health:"
nvme smart-log /dev/nvme0 | grep -E "temperature|percentage_used|available_spare"
echo
echo "You can now run the installer:"
echo "sudo python3 installer/install-automata-nexus-rpi5.py"