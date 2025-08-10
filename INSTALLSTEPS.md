# Automata Nexus Controller - Complete Installation Steps

## Binary Execution: `nexus-installer-complete.run`

### Phase 1: Terminal Pre-Installation

1. **Root Permission Check**
   - Verifies installer is running with sudo/root privileges
   - Exits with error if not root

2. **GUI Dependencies Installation**
   - Updates apt package lists
   - Installs python3
   - Installs python3-tk (Tkinter for GUI)
   - Installs python3-pil (Python Imaging Library)
   - Installs python3-pil.imagetk (Image support for Tkinter)
   - Installs python3-pip (Python package manager)

3. **GUI Launch Preparation**
   - Confirms all GUI dependencies are installed
   - Displays "GUI dependencies ready" message
   - Launches Python GUI installer

### Phase 2: GUI Installer Launch

4. **GUI Window Initialization**
   - Creates 900x700 pixel window
   - Sets title: "Automata Nexus Controller - Professional Building Automation"
   - Makes window non-resizable

5. **Header Display**
   - Shows AUTOMATA NEXUS logo/branding
   - Displays subtitle: "Professional Building Automation Control System"
   - Uses corporate colors (#14b8a6 teal, #0f172a dark background)

6. **Commercial License Agreement**
   - Displays full commercial software license agreement
   - Shows copyright notice
   - Lists all restrictions:
     - No redistribution
     - No reverse engineering
     - No removal of proprietary notices
     - Single device license
   - Shows warranty disclaimer
   - Shows limitation of liability
   - Requires explicit acceptance
   - Provides Decline and Accept buttons
   - Exits if declined

### Phase 3: NVMe SSD Setup and OS Migration

7. **NVMe SSD Detection**
   - Scans for NVMe devices (lsblk | grep nvme)
   - Identifies NVMe device (typically /dev/nvme0n1)
   - Checks SSD capacity and health
   - Runs nvme smart-log /dev/nvme0

8. **SSD Partitioning**
   - Creates GPT partition table on NVMe
   - Creates boot partition (512MB, FAT32)
   - Creates root partition (remaining space, ext4)
   - Aligns partitions for optimal SSD performance

9. **Filesystem Creation**
   - Formats boot partition as FAT32
   - Formats root partition as ext4 with optimizations:
     - stride=2,stripe-width=1024
     - discard option for TRIM support
     - noatime for reduced writes

10. **OS Cloning from SD Card to SSD**
    - Mounts NVMe partitions to /mnt/ssd
    - Uses rsync to copy entire OS from SD card:
      - rsync -axHAWXS --numeric-ids --info=progress2
      - Copies / (root filesystem)
      - Copies /boot
      - Preserves all permissions and attributes
    - Updates /mnt/ssd/etc/fstab for NVMe UUIDs
    - Updates /mnt/ssd/boot/cmdline.txt for NVMe boot

11. **Bootloader Configuration**
    - Installs rpi-eeprom-update
    - Updates bootloader to latest version
    - Configures BOOT_ORDER to prioritize NVMe:
      - BOOT_ORDER=0xf416 (NVMe, then SD, then USB)
    - Sets PCIE_PROBE=1 for NVMe detection
    - Runs rpi-eeprom-config --edit

12. **Boot Configuration for NVMe**
    - Updates /boot/firmware/config.txt:
      - dtparam=pciex1
      - dtparam=pciex1_gen=3
      - dtparam=nvme
    - Sets root=PARTUUID for NVMe root partition
    - Configures initramfs for NVMe boot

13. **System Requirements Check**
   - Checks system architecture (uname -m)
   - Verifies ARM64/aarch64 architecture
   - Checks /proc/cpuinfo for Raspberry Pi 5
   - Verifies BCM2712 processor if RPi5 not detected

14. **Directory Structure Creation on SSD**
   - Creates /mnt/ssd/opt/automata-nexus
   - Creates /mnt/ssd/opt/automata-nexus/data
   - Creates /mnt/ssd/opt/automata-nexus/logs
   - Creates /mnt/ssd/opt/automata-nexus/config
   - Creates /mnt/ssd/opt/automata-nexus/backups
   - Creates /mnt/ssd/opt/automata-nexus/cache
   - Creates /mnt/ssd/opt/automata-nexus/src
   - Creates /mnt/ssd/opt/automata-nexus/build

### Phase 4: Hardware Configuration

15. **I2C Interface Enable**
   - Runs raspi-config nonint do_i2c 0
   - Enables I2C hardware interface
   - Adds i2c-dev to /etc/modules
   - Adds i2c-bcm2835 to /etc/modules

16. **SPI Interface Enable**
    - Runs raspi-config nonint do_spi 0
    - Enables SPI hardware interface

11. **GPIO Configuration**
    - Configures GPIO chip access
    - Sets permissions for /dev/gpiochip0

12. **PCIe Configuration for NVMe**
    - Adds dtparam=pciex1 to /boot/firmware/config.txt
    - Adds dtparam=pciex1_gen=3 for PCIe Gen3 speed
    - Enables NVMe SSD support

### Phase 5: System Dependencies Installation

13. **Package Repository Update**
    - Runs apt-get update
    - Refreshes package lists

14. **Build Tools Installation**
    - Installs build-essential
    - Installs gcc
    - Installs g++
    - Installs make
    - Installs cmake
    - Installs pkg-config

15. **Development Libraries Installation**
    - Installs libssl-dev (SSL/TLS support)
    - Installs libclang-dev (Clang development)
    - Installs llvm-dev (LLVM libraries)
    - Installs clang (C language family frontend)

16. **GUI Libraries Installation (for egui)**
    - Installs libx11-dev (X11 development)
    - Installs libxcb1-dev (XCB development)
    - Installs libxcb-render0-dev
    - Installs libxcb-shape0-dev
    - Installs libxcb-xfixes0-dev
    - Installs libxkbcommon-dev (keyboard handling)
    - Installs libgl1-mesa-dev (OpenGL)
    - Installs libglu1-mesa-dev
    - Installs libegl1-mesa-dev
    - Installs libwayland-dev (Wayland support)
    - Installs libxrandr-dev
    - Installs libxi-dev
    - Installs libxxf86vm-dev

17. **Hardware Communication Libraries**
    - Installs libudev-dev (device management)
    - Installs libusb-1.0-0-dev (USB support)
    - Installs i2c-tools
    - Installs python3-smbus
    - Installs python3-dev

18. **Database and Networking**
    - Installs sqlite3
    - Installs libsqlite3-dev
    - Installs redis-server
    - Installs curl
    - Installs wget
    - Installs git
    - Installs jq (JSON processor)

19. **Web Server Installation**
    - Installs nginx
    - Configures for reverse proxy

20. **Performance Monitoring Tools**
    - Installs htop
    - Installs iotop
    - Installs sysstat
    - Installs nvme-cli (NVMe management)

### Phase 6: Rust Toolchain Installation

21. **Rust Installation Check**
    - Checks if rustc exists
    - If exists, runs rustup update
    - If not, downloads rustup installer

22. **Rustup Installation**
    - Downloads from https://sh.rustup.rs
    - Runs installer with -y flag
    - Installs to /root/.cargo

23. **Rust Configuration**
    - Sources /root/.cargo/env
    - Adds to PATH

24. **ARM64 Target Addition**
    - Runs rustup target add aarch64-unknown-linux-gnu
    - Enables cross-compilation for RPi5

25. **Cargo Tools Installation**
    - Installs cargo-binutils
    - Installs cargo-edit

### Phase 7: Python Dependencies Installation

26. **Pip Upgrade**
    - Upgrades pip to latest version

27. **Python Libraries Installation**
    - Installs pyserial (serial communication)
    - Installs pymodbus (Modbus protocol)
    - Installs influxdb-client
    - Installs redis (Redis client)
    - Installs psutil (system monitoring)
    - Installs RPi.GPIO (GPIO control)
    - Installs smbus2 (I2C communication)
    - Installs adafruit-circuitpython-ads1x15

28. **Sequent Microsystems Libraries**
    - Clones megabas-rpi repository
    - Installs megabas library
    - Clones megaind-rpi repository
    - Installs megaind library
    - Clones 16relind-rpi repository
    - Installs 16relind library
    - Clones 16univin-rpi repository
    - Installs 16univin library
    - Clones 8relind-rpi repository
    - Installs 8relind library

### Phase 8: Application Building

29. **Source Code Extraction**
    - Extracts embedded source code
    - Copies src/ directory
    - Copies Cargo.toml
    - Copies Makefile

30. **Cargo Configuration**
    - Creates .cargo/config.toml
    - Sets target to aarch64-unknown-linux-gnu
    - Sets optimization flags:
      - target-cpu=cortex-a76
      - opt-level=3
      - lto=thin
      - codegen-units=1

31. **Dependency Resolution**
    - Runs cargo update
    - Downloads all crate dependencies

32. **Rust Compilation**
    - Runs cargo build --release
    - Compiles all 13 UI modules
    - Links with system libraries
    - Strips debug symbols
    - Optimizes for size and speed

33. **Binary Installation**
    - Copies binary to /opt/automata-nexus/nexus-controller
    - Sets executable permissions

### Phase 9: Database Setup

34. **SQLite Database Creation**
    - Creates /opt/automata-nexus/data/nexus.db

35. **Schema Creation**
    - Creates settings table
    - Creates board_configs table
    - Creates channel_data table
    - Creates metrics table
    - Creates alarms table
    - Creates audit_log table

36. **Index Creation**
    - Creates index on channel_data(timestamp)
    - Creates index on metrics(timestamp)
    - Creates index on alarms(severity)
    - Creates index on audit_log(timestamp)

37. **Default Data Insertion**
    - Inserts default system settings
    - Sets system_name to "Nexus Controller"
    - Sets version to "2.0.0"
    - Sets timezone

38. **Database Optimization**
    - Sets PRAGMA cache_size = 100000
    - Sets PRAGMA mmap_size = 1073741824
    - Sets PRAGMA journal_mode = WAL
    - Sets PRAGMA synchronous = NORMAL
    - Runs VACUUM

### Phase 10: Cloudflare Tunnel Setup

39. **Cloudflared Installation**
    - Downloads cloudflared-linux-arm64.deb
    - Downloads from GitHub releases
    - Installs with dpkg -i

40. **Tunnel Configuration Preparation**
    - Creates /opt/automata-nexus/config/cloudflare
    - Sets up for tunnel authentication
    - Prepares for tunnel creation

41. **Cloudflare Service Setup**
    - Creates cloudflared service file
    - Configures auto-start
    - Sets up for remote access

### Phase 11: Remote Access Configuration

42. **SSH Configuration**
    - Ensures SSH is enabled
    - Configures SSH security settings

43. **VPN Support Setup**
    - Installs WireGuard if selected
    - Configures VPN keys
    - Sets up VPN routing

44. **Firewall Configuration**
    - Installs ufw if not present
    - Allows port 22 (SSH)
    - Allows port 8080 (Web UI)
    - Allows port 443 (HTTPS)
    - Enables firewall

45. **Digital Ocean VPN Client** (if selected)
    - Runs configure-do-server.sh
    - Sets up DO VPN connection

### Phase 12: Service Installation

46. **Main Service Creation**
    - Creates /etc/systemd/system/nexus-controller.service
    - Sets User=pi
    - Sets WorkingDirectory=/opt/automata-nexus
    - Sets environment variables
    - Configures restart policy

47. **Monitor Service Creation**
    - Creates /etc/systemd/system/nexus-monitor.service
    - Sets up health monitoring
    - Configures auto-restart

48. **Service Configuration**
    - Sets CPUSchedulingPolicy=fifo
    - Sets CPUSchedulingPriority=50
    - Sets IOSchedulingClass=realtime
    - Sets IOSchedulingPriority=0

49. **Service Enablement**
    - Runs systemctl daemon-reload
    - Runs systemctl enable nexus-controller
    - Runs systemctl enable nexus-monitor

### Phase 13: Performance Optimization

50. **CPU Governor Setting**
    - Sets all CPUs to performance mode
    - Writes to /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor

51. **I/O Scheduler Configuration**
    - Sets NVMe to mq-deadline scheduler
    - Optimizes for SSD performance

52. **Kernel Parameters**
    - Creates /etc/sysctl.d/99-automata-nexus.conf
    - Sets vm.swappiness=10
    - Sets vm.vfs_cache_pressure=50
    - Sets vm.dirty_background_ratio=5
    - Sets vm.dirty_ratio=10
    - Sets net.core.rmem_max=134217728
    - Sets net.core.wmem_max=134217728
    - Sets fs.file-max=2097152
    - Sets kernel.pid_max=4194304

53. **Memory Optimization**
    - Enables huge pages
    - Sets vm.nr_hugepages=64

54. **File Descriptor Limits**
    - Creates /etc/security/limits.d/99-automata-nexus.conf
    - Sets soft nofile 65536
    - Sets hard nofile 65536
    - Sets soft nproc 4096
    - Sets hard nproc 4096

### Phase 14: Backup Configuration

55. **Backup Script Creation**
    - Creates /opt/automata-nexus/backup.sh
    - Configures daily backups
    - Sets 7-day retention

56. **Crontab Configuration**
    - Adds backup job to crontab
    - Schedules for 2 AM daily

### Phase 15: Configuration Files

57. **Main Configuration**
    - Creates /opt/automata-nexus/config/nexus.toml
    - Sets server host and port
    - Configures database path
    - Sets logging levels

58. **Hardware Configuration**
    - Configures I2C bus settings
    - Configures SPI bus settings
    - Sets GPIO chip path

59. **Board Configuration**
    - Configures MegaBAS settings
    - Configures expansion board settings
    - Sets board addresses

60. **Security Configuration**
    - Enables authentication
    - Sets session timeout
    - Configures default PIN (2196)

### Phase 16: Installation Testing

61. **Binary Verification**
    - Checks nexus-controller exists
    - Verifies executable permissions
    - Tests binary execution

62. **Database Verification**
    - Tests database connection
    - Verifies tables exist
    - Checks data integrity

63. **Service Verification**
    - Checks services are installed
    - Verifies service files syntax
    - Tests service startup

64. **Hardware Interface Tests**
    - Tests I2C detection
    - Tests SPI availability
    - Tests GPIO access

65. **Network Connectivity**
    - Tests localhost connection
    - Verifies port 8080 is available
    - Tests internet connectivity

### Phase 17: Final Steps

66. **Permission Setting**
    - chown -R pi:pi /opt/automata-nexus
    - chmod 755 /opt/automata-nexus
    - chmod 700 /opt/automata-nexus/data
    - chmod 777 /opt/automata-nexus/logs

67. **Log File Creation**
    - Creates initial log files
    - Sets log rotation

68. **Cache Directory Setup**
    - Creates npm cache directory
    - Creates cargo cache directory
    - Creates pip cache directory

69. **Installation Log**
    - Saves complete installation log
    - Stores in /tmp/nexus_install_*.log

70. **Completion Dialog**
    - Shows success message
    - Displays next steps:
      - Reboot command (system will boot from NVMe)
      - Service start command
      - Access URL (http://localhost:8080)
      - Default PIN (2196)
      - Support email

### Phase 18: Post-Installation

71. **Reboot Required Flag**
    - Sets flag if kernel modules changed
    - Sets flag if PCIe settings changed
    - Notifies user system will boot from NVMe after reboot

72. **SD Card Removal Instructions**
    - Displays message that SD card can be removed after reboot
    - System will boot entirely from NVMe SSD
    - SD card no longer needed (can be kept as backup)

73. **Cleanup**
    - Removes temporary installation files
    - Cleans package cache
    - Removes build artifacts

## Total Components Installed

- **System Packages**: 40+
- **Python Libraries**: 15+
- **Rust Crates**: 50+
- **Configuration Files**: 10+
- **Service Files**: 2
- **Database Tables**: 6
- **UI Modules**: 13

## Files Created

- `/opt/automata-nexus/nexus-controller` (main binary)
- `/opt/automata-nexus/data/nexus.db` (database)
- `/opt/automata-nexus/config/nexus.toml` (configuration)
- `/etc/systemd/system/nexus-controller.service`
- `/etc/systemd/system/nexus-monitor.service`
- `/etc/sysctl.d/99-automata-nexus.conf`
- `/etc/security/limits.d/99-automata-nexus.conf`
- `/opt/automata-nexus/backup.sh`
- `/opt/automata-nexus/monitor.sh`

## Services Enabled

- `nexus-controller.service` - Main application
- `nexus-monitor.service` - Health monitoring
- `cloudflared.service` - Remote access tunnel (optional)
- `redis.service` - Cache server
- `nginx.service` - Web server (optional)

## Ports Configured

- **22** - SSH (if enabled)
- **8080** - Web UI (HTTP)
- **443** - HTTPS (if configured)
- **6379** - Redis (localhost only)

## Hardware Interfaces Enabled

- I2C (bus 1)
- SPI (bus 0)
- GPIO (gpiochip0)
- PCIe Gen3 (for NVMe)

## Performance Settings Applied

- CPU Governor: Performance
- I/O Scheduler: mq-deadline
- Swappiness: 10
- File descriptors: 65536
- Huge pages: Enabled

## End Result

A fully functional, optimized, and secured Automata Nexus Controller installation ready for professional building automation control on Raspberry Pi 5.