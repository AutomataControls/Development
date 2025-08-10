#!/bin/bash

# Create Professional GUI Installer for Nexus Controller
# With commercial license, logo, progress bars, status windows

set -e

VERSION="3.0.0"
INSTALLER_NAME="nexus-installer-gui-v${VERSION}.run"

echo "Building Professional GUI Installer..."

# Create installer directory
rm -rf /tmp/nexus-installer
mkdir -p /tmp/nexus-installer/{src,assets}

# Copy source files
cp -r src /tmp/nexus-installer/
cp Cargo.toml /tmp/nexus-installer/
cp build.rs /tmp/nexus-installer/
cp -r installer /tmp/nexus-installer/

# Create the GUI installer in Python with tkinter
cat > /tmp/nexus-installer/gui_installer.py << 'INSTALLER_GUI'
#!/usr/bin/env python3

import tkinter as tk
from tkinter import ttk, messagebox, scrolledtext
import subprocess
import threading
import os
import sys
import time

class NexusInstaller:
    def __init__(self, root):
        self.root = root
        self.root.title("Automata Nexus Controller v3.0.0 - Professional Installer")
        self.root.geometry("800x600")
        self.root.resizable(False, False)
        
        # Professional light theme colors (matching the app)
        self.bg_color = "#ffffff"  # White background
        self.fg_color = "#0f172a"  # Dark text
        self.accent_color = "#14b8a6"  # Teal
        self.button_bg = "#14b8a6"
        self.button_hover = "#0d9488"
        
        self.root.configure(bg=self.bg_color)
        
        self.current_step = 0
        self.total_steps = 10
        self.installing = False
        
        self.create_license_screen()
    
    def create_license_screen(self):
        """Commercial License Acknowledgement Screen"""
        # Clear window
        for widget in self.root.winfo_children():
            widget.destroy()
        
        # Logo and Title
        title_frame = tk.Frame(self.root, bg=self.bg_color)
        title_frame.pack(pady=20)
        
        # Logo placeholder (would be actual logo image)
        logo_label = tk.Label(title_frame, text="🏢", font=("Arial", 48), 
                              bg=self.bg_color, fg=self.accent_color)
        logo_label.pack()
        
        title_label = tk.Label(title_frame, 
                              text="AUTOMATA NEXUS CONTROLLER",
                              font=("Arial", 24, "bold"),
                              bg=self.bg_color, fg=self.fg_color)
        title_label.pack()
        
        subtitle_label = tk.Label(title_frame,
                                 text="Professional Building Automation System",
                                 font=("Arial", 12),
                                 bg=self.bg_color, fg=self.accent_color)
        subtitle_label.pack()
        
        # License Frame
        license_frame = tk.Frame(self.root, bg="#f8fafc", relief=tk.RAISED, bd=2)
        license_frame.pack(padx=40, pady=20, fill=tk.BOTH, expand=True)
        
        license_title = tk.Label(license_frame,
                                text="COMMERCIAL LICENSE AGREEMENT",
                                font=("Arial", 14, "bold"),
                                bg="#f8fafc", fg=self.fg_color)
        license_title.pack(pady=10)
        
        # License text
        license_text = scrolledtext.ScrolledText(license_frame, 
                                                 wrap=tk.WORD,
                                                 width=80, height=15,
                                                 bg="#ffffff", fg=self.fg_color,
                                                 font=("Courier", 10))
        license_text.pack(padx=20, pady=10)
        
        license_content = """COMMERCIAL SOFTWARE LICENSE AGREEMENT
Automata Nexus Controller v3.0.0

Copyright (c) 2025 Automata Controls
Developed by Andrew Jewell Sr.

IMPORTANT - READ CAREFULLY:

This is proprietary commercial software. By installing this software, you 
acknowledge and agree to the following terms:

1. LICENSE GRANT: Automata Controls grants you a non-exclusive, 
   non-transferable license to use this software on authorized hardware.

2. RESTRICTIONS: You may not:
   - Reverse engineer, decompile, or disassemble the software
   - Redistribute or resell the software
   - Remove or alter any proprietary notices

3. HARDWARE CONTROL: This software directly controls physical hardware
   including HVAC equipment, sensors, and actuators. Improper use may
   result in equipment damage or safety hazards.

4. NO WARRANTY: This software is provided "AS IS" without warranty of
   any kind, either express or implied.

5. LIMITATION OF LIABILITY: In no event shall Automata Controls be liable
   for any damages arising from the use of this software.

6. SUPPORT: Commercial support is available at support@automatacontrols.com

By clicking "I Accept", you agree to be bound by these terms."""
        
        license_text.insert(tk.END, license_content)
        license_text.config(state=tk.DISABLED)
        
        # Accept checkbox
        self.accept_var = tk.BooleanVar()
        accept_check = tk.Checkbutton(self.root,
                                      text="I accept the terms of the Commercial License Agreement",
                                      variable=self.accept_var,
                                      bg=self.bg_color, fg=self.fg_color,
                                      selectcolor=self.bg_color,
                                      activebackground=self.bg_color,
                                      font=("Arial", 11))
        accept_check.pack(pady=10)
        
        # Buttons
        button_frame = tk.Frame(self.root, bg=self.bg_color)
        button_frame.pack(pady=20)
        
        self.install_btn = tk.Button(button_frame,
                                     text="INSTALL",
                                     command=self.start_installation,
                                     bg=self.button_bg, fg="white",
                                     font=("Arial", 12, "bold"),
                                     padx=30, pady=10,
                                     state=tk.DISABLED)
        self.install_btn.pack(side=tk.LEFT, padx=10)
        
        cancel_btn = tk.Button(button_frame,
                              text="CANCEL",
                              command=self.cancel_installation,
                              bg="#64748b", fg="white",
                              font=("Arial", 12),
                              padx=30, pady=10)
        cancel_btn.pack(side=tk.LEFT, padx=10)
        
        # Enable install button when license accepted
        def check_accept():
            if self.accept_var.get():
                self.install_btn.config(state=tk.NORMAL)
            else:
                self.install_btn.config(state=tk.DISABLED)
        
        accept_check.config(command=check_accept)
    
    def create_installation_screen(self):
        """Main installation screen with progress and status"""
        # Clear window
        for widget in self.root.winfo_children():
            widget.destroy()
        
        # Header with logo
        header_frame = tk.Frame(self.root, bg=self.bg_color, height=100)
        header_frame.pack(fill=tk.X, pady=10)
        header_frame.pack_propagate(False)
        
        # Logo and title in header
        logo_label = tk.Label(header_frame, text="🏢", font=("Arial", 32),
                             bg=self.bg_color, fg=self.accent_color)
        logo_label.pack(side=tk.LEFT, padx=20)
        
        title_frame = tk.Frame(header_frame, bg=self.bg_color)
        title_frame.pack(side=tk.LEFT, padx=10)
        
        tk.Label(title_frame, text="AUTOMATA NEXUS CONTROLLER",
                font=("Arial", 18, "bold"),
                bg=self.bg_color, fg=self.fg_color).pack(anchor="w")
        tk.Label(title_frame, text="Installing version 3.0.0",
                font=("Arial", 11),
                bg=self.bg_color, fg=self.accent_color).pack(anchor="w")
        
        # Main progress section
        progress_frame = tk.Frame(self.root, bg="#1e293b", relief=tk.RAISED, bd=2)
        progress_frame.pack(padx=20, pady=10, fill=tk.X)
        
        # Current operation label
        self.operation_label = tk.Label(progress_frame,
                                       text="Preparing installation...",
                                       font=("Arial", 12),
                                       bg="#f8fafc", fg=self.fg_color)
        self.operation_label.pack(pady=10)
        
        # Main progress bar
        self.main_progress = ttk.Progressbar(progress_frame,
                                            length=700,
                                            mode='determinate',
                                            maximum=100)
        self.main_progress.pack(pady=10)
        
        # Progress percentage
        self.progress_label = tk.Label(progress_frame,
                                     text="0%",
                                     font=("Arial", 10),
                                     bg="#1e293b", fg=self.accent_color)
        self.progress_label.pack()
        
        # Step progress
        self.step_label = tk.Label(progress_frame,
                                  text=f"Step 0 of {self.total_steps}",
                                  font=("Arial", 10),
                                  bg="#f8fafc", fg=self.fg_color)
        self.step_label.pack(pady=5)
        
        # Status window
        status_frame = tk.Frame(self.root, bg="#1e293b", relief=tk.RAISED, bd=2)
        status_frame.pack(padx=20, pady=10, fill=tk.BOTH, expand=True)
        
        tk.Label(status_frame, text="Installation Log:",
                font=("Arial", 11, "bold"),
                bg="#1e293b", fg=self.fg_color).pack(anchor="w", padx=10, pady=5)
        
        # Status text area
        self.status_text = scrolledtext.ScrolledText(status_frame,
                                                    wrap=tk.WORD,
                                                    width=90, height=12,
                                                    bg="#ffffff", fg="#64748b",
                                                    font=("Courier", 9))
        self.status_text.pack(padx=10, pady=5, fill=tk.BOTH, expand=True)
        
        # Feature list on the side
        feature_frame = tk.Frame(self.root, bg=self.bg_color)
        feature_frame.pack(pady=10)
        
        tk.Label(feature_frame, text="Installing Components:",
                font=("Arial", 11, "bold"),
                bg=self.bg_color, fg=self.fg_color).pack()
        
        features = [
            "✓ Rust/egui Native Application",
            "✓ MegaBAS Board Drivers", 
            "✓ 16-Universal Input Drivers",
            "✓ 16-Output Board Drivers",
            "✓ P499 Pressure Transducer Support",
            "✓ WTVB01-485 Vibration Sensors",
            "✓ Modbus/BACnet Protocols",
            "✓ SQLite Database",
            "✓ Systemd Service"
        ]
        
        for feature in features:
            tk.Label(feature_frame, text=feature,
                    font=("Arial", 9),
                    bg=self.bg_color, fg="#64748b").pack(anchor="w")
    
    def start_installation(self):
        """Start the installation process"""
        if not self.accept_var.get():
            messagebox.showerror("License Agreement", 
                               "You must accept the license agreement to continue.")
            return
        
        self.create_installation_screen()
        self.installing = True
        
        # Start installation in background thread
        install_thread = threading.Thread(target=self.run_installation)
        install_thread.daemon = True
        install_thread.start()
    
    def run_installation(self):
        """Run the actual installation steps"""
        steps = [
            ("Checking system requirements...", self.check_requirements),
            ("Installing system dependencies...", self.install_dependencies),
            ("Installing Rust toolchain...", self.install_rust),
            ("Enabling I2C interface...", self.enable_i2c),
            ("Installing board drivers...", self.install_drivers),
            ("Building Nexus application...", self.build_application),
            ("Setting up database...", self.setup_database),
            ("Creating systemd service...", self.create_service),
            ("Starting Nexus service...", self.start_service),
            ("Installation complete!", self.finish_installation)
        ]
        
        for i, (description, function) in enumerate(steps):
            self.current_step = i + 1
            self.update_ui(description, self.current_step)
            
            try:
                function()
            except Exception as e:
                self.log_status(f"ERROR: {str(e)}", error=True)
                messagebox.showerror("Installation Failed", 
                                   f"Installation failed at step {self.current_step}:\n{str(e)}")
                self.root.quit()
                return
            
            time.sleep(0.5)  # Brief pause between steps
    
    def update_ui(self, operation, step):
        """Update the UI with current progress"""
        progress = (step / self.total_steps) * 100
        
        self.root.after(0, lambda: self.operation_label.config(text=operation))
        self.root.after(0, lambda: self.main_progress.config(value=progress))
        self.root.after(0, lambda: self.progress_label.config(text=f"{int(progress)}%"))
        self.root.after(0, lambda: self.step_label.config(text=f"Step {step} of {self.total_steps}"))
        
        self.log_status(f"\n>>> {operation}")
    
    def log_status(self, message, error=False):
        """Log status message to the status window"""
        def update():
            self.status_text.config(state=tk.NORMAL)
            if error:
                self.status_text.insert(tk.END, message + "\n", "error")
                self.status_text.tag_config("error", foreground="#ef4444")
            else:
                self.status_text.insert(tk.END, message + "\n")
            self.status_text.see(tk.END)
            self.status_text.config(state=tk.DISABLED)
        
        self.root.after(0, update)
    
    def run_command(self, command, description=""):
        """Run a shell command and log output"""
        self.log_status(f"Running: {description or command}")
        result = subprocess.run(command, shell=True, capture_output=True, text=True)
        
        if result.stdout:
            self.log_status(result.stdout)
        
        if result.returncode != 0:
            if result.stderr:
                self.log_status(result.stderr, error=True)
            raise Exception(f"Command failed: {command}")
        
        return result
    
    def check_requirements(self):
        """Check system requirements"""
        self.log_status("Checking architecture...")
        result = self.run_command("uname -m", "Check architecture")
        
        if "aarch64" not in result.stdout:
            raise Exception("This installer requires ARM64/aarch64 architecture")
        
        self.log_status("✓ System requirements met")
    
    def install_dependencies(self):
        """Install system dependencies"""
        deps = [
            "build-essential", "pkg-config", "libssl-dev",
            "libsqlite3-dev", "sqlite3", "libgtk-3-dev",
            "libgl1-mesa-dev", "libegl1-mesa-dev", "i2c-tools",
            "python3-smbus", "python3-serial", "python3-pymodbus",
            "git", "make", "gcc", "curl"
        ]
        
        self.log_status(f"Installing {len(deps)} packages...")
        self.run_command("sudo apt-get update", "Update package list")
        self.run_command(f"sudo apt-get install -y {' '.join(deps)}", 
                        "Install dependencies")
        self.log_status("✓ Dependencies installed")
    
    def install_rust(self):
        """Install Rust if not present"""
        self.log_status("Checking for Rust...")
        result = subprocess.run("which cargo", shell=True, capture_output=True)
        
        if result.returncode != 0:
            self.log_status("Installing Rust toolchain...")
            self.run_command(
                "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y",
                "Install Rust"
            )
            self.run_command("source $HOME/.cargo/env", "Configure Rust environment")
        else:
            self.log_status("✓ Rust already installed")
    
    def enable_i2c(self):
        """Enable I2C interface"""
        self.log_status("Enabling I2C interface...")
        self.run_command("sudo raspi-config nonint do_i2c 0", "Enable I2C")
        self.log_status("✓ I2C enabled")
    
    def install_drivers(self):
        """Install board drivers"""
        repos = [
            ("MegaBAS", "https://github.com/SequentMicrosystems/megabas-rpi.git"),
            ("16-Relay", "https://github.com/SequentMicrosystems/16relind-rpi.git"),
            ("8-Relay", "https://github.com/SequentMicrosystems/8relind-rpi.git"),
            ("16-UnivIn", "https://github.com/SequentMicrosystems/16univin-rpi.git"),
            ("16-UnivOut", "https://github.com/SequentMicrosystems/16uout-rpi.git")
        ]
        
        for name, url in repos:
            self.log_status(f"Installing {name} driver...")
            repo_name = url.split('/')[-1].replace('.git', '')
            
            self.run_command(f"sudo mkdir -p /opt/nexus/firmware", "Create firmware dir")
            
            if not os.path.exists(f"/opt/nexus/firmware/{repo_name}"):
                self.run_command(f"sudo git clone {url} /opt/nexus/firmware/{repo_name}",
                               f"Clone {name}")
            
            self.run_command(f"cd /opt/nexus/firmware/{repo_name} && sudo make && sudo make install",
                           f"Build {name}")
        
        self.log_status("✓ All board drivers installed")
    
    def build_application(self):
        """Build the Nexus application"""
        self.log_status("Building Nexus Controller (this may take several minutes)...")
        
        # Copy source files
        self.run_command("sudo mkdir -p /opt/nexus", "Create app directory")
        self.run_command("sudo cp -r * /opt/nexus/", "Copy source files")
        
        # Build
        self.run_command(
            "cd /opt/nexus && cargo build --release",
            "Build application"
        )
        
        self.run_command(
            "sudo cp /opt/nexus/target/release/nexus-controller /usr/local/bin/",
            "Install binary"
        )
        
        self.log_status("✓ Application built and installed")
    
    def setup_database(self):
        """Setup SQLite database"""
        self.log_status("Setting up database...")
        
        self.run_command("sudo mkdir -p /var/lib/nexus", "Create data directory")
        
        sql_commands = """
        CREATE TABLE IF NOT EXISTS sensor_data (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
            board_type TEXT,
            channel INTEGER,
            value REAL
        );
        CREATE TABLE IF NOT EXISTS board_config (
            board_id TEXT PRIMARY KEY,
            config TEXT
        );
        """
        
        self.run_command(
            f"sudo sqlite3 /var/lib/nexus/nexus.db '{sql_commands}'",
            "Initialize database"
        )
        
        self.run_command("sudo chown -R pi:pi /var/lib/nexus", "Set permissions")
        self.log_status("✓ Database configured")
    
    def create_service(self):
        """Create systemd service"""
        self.log_status("Creating systemd service...")
        
        service_content = """[Unit]
Description=Automata Nexus Controller
After=network.target

[Service]
Type=simple
User=pi
WorkingDirectory=/opt/nexus
ExecStart=/usr/local/bin/nexus-controller
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
"""
        
        with open("/tmp/nexus.service", "w") as f:
            f.write(service_content)
        
        self.run_command("sudo cp /tmp/nexus.service /etc/systemd/system/", "Install service")
        self.run_command("sudo systemctl daemon-reload", "Reload systemd")
        self.run_command("sudo systemctl enable nexus.service", "Enable service")
        
        self.log_status("✓ Service configured")
    
    def start_service(self):
        """Start the Nexus service"""
        self.log_status("Starting Nexus Controller...")
        self.run_command("sudo systemctl start nexus.service", "Start service")
        time.sleep(2)
        
        result = self.run_command("sudo systemctl is-active nexus.service", "Check service")
        
        if "active" in result.stdout:
            self.log_status("✓ Nexus Controller is running!")
        else:
            raise Exception("Failed to start service")
    
    def finish_installation(self):
        """Complete the installation"""
        self.log_status("\n" + "="*50)
        self.log_status("✅ INSTALLATION COMPLETE!")
        self.log_status("="*50)
        
        # Get IP address
        result = subprocess.run("hostname -I | awk '{print $1}'", 
                              shell=True, capture_output=True, text=True)
        ip = result.stdout.strip()
        
        self.log_status(f"\nAccess the controller at: http://{ip}:8080")
        self.log_status("\nTest commands:")
        self.log_status("  megabas 0 board     - Check MegaBAS")
        self.log_status("  systemctl status nexus - Check service")
        
        # Show completion dialog
        messagebox.showinfo("Installation Complete",
                          f"Automata Nexus Controller v3.0.0 has been successfully installed!\n\n"
                          f"Access at: http://{ip}:8080\n\n"
                          f"The service is running and will start automatically on boot.")
        
        # Add close button
        close_btn = tk.Button(self.root,
                            text="CLOSE",
                            command=self.root.quit,
                            bg=self.button_bg, fg="white",
                            font=("Arial", 12, "bold"),
                            padx=30, pady=10)
        close_btn.pack(pady=20)
    
    def cancel_installation(self):
        """Cancel the installation"""
        if self.installing:
            if messagebox.askyesno("Cancel Installation",
                                  "Installation is in progress. Are you sure you want to cancel?"):
                self.root.quit()
        else:
            self.root.quit()

def main():
    # Check if running with GUI capability
    if os.environ.get('DISPLAY') or os.environ.get('WAYLAND_DISPLAY'):
        root = tk.Tk()
        app = NexusInstaller(root)
        root.mainloop()
    else:
        print("ERROR: No display detected. This installer requires a GUI environment.")
        print("Please run this installer from the Raspberry Pi desktop or via VNC/X11 forwarding.")
        sys.exit(1)

if __name__ == "__main__":
    main()
INSTALLER_GUI

chmod +x /tmp/nexus-installer/gui_installer.py

# Create wrapper script
cat > /tmp/nexus-installer/install.sh << 'WRAPPER'
#!/bin/bash

echo "Starting Automata Nexus Professional GUI Installer..."

# Check for Python3 and tkinter
if ! command -v python3 &> /dev/null; then
    echo "Installing Python3..."
    sudo apt-get update
    sudo apt-get install -y python3 python3-tk
fi

# Make sure tkinter is installed
python3 -c "import tkinter" 2>/dev/null || {
    echo "Installing Python tkinter..."
    sudo apt-get install -y python3-tk
}

# Run the GUI installer
python3 gui_installer.py

WRAPPER

chmod +x /tmp/nexus-installer/install.sh

# Create self-extracting installer
cd /tmp
tar czf nexus-installer.tar.gz nexus-installer/

cat > $INSTALLER_NAME << 'HEADER'
#!/bin/bash
# Automata Nexus Controller v3.0.0 - Professional GUI Installer
echo "Extracting installer files..."
TMPDIR=$(mktemp -d)
ARCHIVE=$(awk '/^__ARCHIVE__/ {print NR + 1; exit 0;}' "$0")
tail -n +$ARCHIVE "$0" | tar xz -C $TMPDIR
cd $TMPDIR/nexus-installer && bash install.sh
rm -rf $TMPDIR
exit 0
__ARCHIVE__
HEADER

cat nexus-installer.tar.gz >> $INSTALLER_NAME
chmod +x $INSTALLER_NAME

# Move to release directory
mv $INSTALLER_NAME /home/Automata/Development/projects/Rust-SSD-Nexus-Controller/release/

# Cleanup
rm -rf /tmp/nexus-installer /tmp/nexus-installer.tar.gz

SIZE=$(du -h /home/Automata/Development/projects/Rust-SSD-Nexus-Controller/release/$INSTALLER_NAME | cut -f1)

echo ""
echo "══════════════════════════════════════════════════════════════════"
echo "✅ PROFESSIONAL GUI INSTALLER CREATED!"
echo "══════════════════════════════════════════════════════════════════"
echo ""
echo "Installer: release/$INSTALLER_NAME"
echo "Size: $SIZE"
echo ""
echo "Features:"
echo "  ✓ Professional GUI interface with Automata logo"
echo "  ✓ Commercial License acknowledgement screen"
echo "  ✓ Install/Cancel buttons"
echo "  ✓ Real-time progress bars"
echo "  ✓ Status window showing installation progress"
echo "  ✓ Component checklist"
echo "  ✓ Light professional theme with teal accents"
echo "  ✓ Automatic service startup"
echo ""
echo "The installer will:"
echo "  1. Show license agreement with accept checkbox"
echo "  2. Enable Install button only after accepting"
echo "  3. Display professional installation screen with logo"
echo "  4. Show progress bars and status messages"
echo "  5. Install all dependencies and drivers"
echo "  6. Build and start the service"
echo "  7. Show completion dialog with access URL"