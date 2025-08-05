#!/usr/bin/env python3
"""
Automata Nexus Installation Media Creator
Creates SD cards and USB drives with the installer and all components
Copyright (c) 2024 Automata Nexus. All rights reserved.
"""

import os
import sys
import tkinter as tk
from tkinter import ttk, filedialog, messagebox, scrolledtext
import shutil
import zipfile
import json
import platform
from pathlib import Path
import threading
import time

class InstallerCreator:
    def __init__(self):
        self.root = tk.Tk()
        self.root.title("Automata Nexus - Installation Media Creator")
        self.root.geometry("900x700")
        
        # Center window
        self.root.update_idletasks()
        x = (self.root.winfo_screenwidth() // 2) - (450)
        y = (self.root.winfo_screenheight() // 2) - (350)
        self.root.geometry(f"900x700+{x}+{y}")
        
        # Variables
        self.sd_card_path = tk.StringVar()
        self.include_components = {
            "base_system": tk.BooleanVar(value=True),
            "python_libs": tk.BooleanVar(value=True),
            "node_modules": tk.BooleanVar(value=True),
            "drivers": tk.BooleanVar(value=True),
            "demo_logic": tk.BooleanVar(value=True),
        }
        
        self.setup_ui()
        
    def setup_ui(self):
        # Header with dark theme to match installer
        header_frame = tk.Frame(self.root, bg="#1a1a1a", height=120)
        header_frame.pack(fill=tk.X)
        header_frame.pack_propagate(False)
        
        # Logo and title - matching the actual installer
        logo_frame = tk.Frame(header_frame, bg="#1a1a1a")
        logo_frame.pack(expand=True)
        
        # Try to load actual logo
        try:
            from PIL import Image, ImageTk
            logo_path = os.path.join(os.path.dirname(__file__), "public", "automata-nexus-logo.png")
            if os.path.exists(logo_path):
                img = Image.open(logo_path)
                img = img.resize((48, 48), Image.Resampling.LANCZOS)
                self.logo = ImageTk.PhotoImage(img)
                tk.Label(logo_frame, image=self.logo, bg="#1a1a1a").pack(side=tk.LEFT, padx=10)
            else:
                tk.Label(logo_frame, text="🏭", font=("Arial", 48), 
                        bg="#1a1a1a", fg="white").pack(side=tk.LEFT, padx=10)
        except:
            tk.Label(logo_frame, text="🏭", font=("Arial", 48), 
                    bg="#1a1a1a", fg="white").pack(side=tk.LEFT, padx=10)
        
        title_frame = tk.Frame(logo_frame, bg="#1a1a1a")
        title_frame.pack(side=tk.LEFT)
        
        tk.Label(title_frame, text="Automata Nexus Control Center", 
                font=("Arial", 24, "bold"), bg="#1a1a1a", fg="white").pack(anchor=tk.W)
        tk.Label(title_frame, text="Installation Media Creator", 
                font=("Arial", 14), bg="#1a1a1a", fg="#00ff00").pack(anchor=tk.W)
        
        # Main content
        main_frame = tk.Frame(self.root, bg="#f3f4f6")
        main_frame.pack(fill=tk.BOTH, expand=True, padx=20, pady=20)
        
        # Target selection
        target_frame = tk.LabelFrame(main_frame, text="Target Media", 
                                   font=("Arial", 12, "bold"), bg="#f3f4f6")
        target_frame.pack(fill=tk.X, pady=(0, 10))
        
        tk.Label(target_frame, text="Select SD Card or USB Drive:", 
                bg="#f3f4f6").grid(row=0, column=0, padx=10, pady=10, sticky=tk.W)
        
        path_frame = tk.Frame(target_frame, bg="#f3f4f6")
        path_frame.grid(row=1, column=0, columnspan=2, padx=10, pady=(0, 10), sticky=tk.EW)
        
        self.path_entry = tk.Entry(path_frame, textvariable=self.sd_card_path, width=50)
        self.path_entry.pack(side=tk.LEFT, fill=tk.X, expand=True)
        
        tk.Button(path_frame, text="Browse", command=self.browse_target,
                 bg="#3b82f6", fg="white", padx=15).pack(side=tk.LEFT, padx=(5, 0))
        
        # Detected drives
        if platform.system() == "Windows":
            self.detect_drives_windows(target_frame)
        
        # Components selection
        comp_frame = tk.LabelFrame(main_frame, text="Installation Components", 
                                  font=("Arial", 12, "bold"), bg="#f3f4f6")
        comp_frame.pack(fill=tk.X, pady=10)
        
        components = [
            ("base_system", "Base System & Application", "Core control center application and runtime"),
            ("python_libs", "Python Libraries", "Sequent Microsystems libraries and dependencies"),
            ("node_modules", "Pre-built Node Modules", "Pre-compiled Node.js modules for faster installation"),
            ("drivers", "Hardware Drivers", "Pre-compiled Sequent board drivers"),
            ("demo_logic", "Demo Logic Files", "Example control logic for various equipment types"),
        ]
        
        for i, (key, label, desc) in enumerate(components):
            comp_container = tk.Frame(comp_frame, bg="#f3f4f6")
            comp_container.grid(row=i, column=0, sticky=tk.EW, padx=10, pady=5)
            
            tk.Checkbutton(comp_container, text=label, variable=self.include_components[key],
                          font=("Arial", 11, "bold"), bg="#f3f4f6").pack(anchor=tk.W)
            tk.Label(comp_container, text=desc, font=("Arial", 9), 
                    fg="#6b7280", bg="#f3f4f6").pack(anchor=tk.W, padx=(25, 0))
        
        # Options
        options_frame = tk.LabelFrame(main_frame, text="Installation Options", 
                                    font=("Arial", 12, "bold"), bg="#f3f4f6")
        options_frame.pack(fill=tk.X, pady=10)
        
        self.auto_start = tk.BooleanVar(value=True)
        tk.Checkbutton(options_frame, text="Enable auto-start on boot", 
                      variable=self.auto_start, bg="#f3f4f6").pack(anchor=tk.W, padx=10, pady=5)
        
        self.create_backup = tk.BooleanVar(value=True)
        tk.Checkbutton(options_frame, text="Create configuration backup on upgrade", 
                      variable=self.create_backup, bg="#f3f4f6").pack(anchor=tk.W, padx=10, pady=5)
        
        # Progress
        progress_frame = tk.LabelFrame(main_frame, text="Progress", 
                                     font=("Arial", 12, "bold"), bg="#f3f4f6")
        progress_frame.pack(fill=tk.BOTH, expand=True, pady=10)
        
        self.progress_label = tk.Label(progress_frame, text="Ready to create installation media", 
                                     bg="#f3f4f6")
        self.progress_label.pack(pady=5)
        
        self.progress_bar = ttk.Progressbar(progress_frame, length=400, mode='determinate')
        self.progress_bar.pack(pady=5)
        
        self.log_text = scrolledtext.ScrolledText(progress_frame, height=8, width=70)
        self.log_text.pack(padx=10, pady=5, fill=tk.BOTH, expand=True)
        
        # Buttons
        button_frame = tk.Frame(main_frame, bg="#f3f4f6")
        button_frame.pack(fill=tk.X, pady=10)
        
        tk.Button(button_frame, text="Create Installation Media", 
                 command=self.create_media, bg="#00aa00", fg="white", 
                 font=("Arial", 12, "bold"), padx=20, pady=10).pack(side=tk.RIGHT)
        
        tk.Button(button_frame, text="Exit", command=self.root.quit,
                 bg="#6b7280", fg="white", padx=20, pady=10).pack(side=tk.RIGHT, padx=(0, 10))
    
    def detect_drives_windows(self, parent):
        """Detect removable drives on Windows"""
        drives_frame = tk.Frame(parent, bg="#f3f4f6")
        drives_frame.grid(row=2, column=0, columnspan=2, padx=10, pady=10, sticky=tk.EW)
        
        tk.Label(drives_frame, text="Detected Removable Drives:", 
                font=("Arial", 10, "bold"), bg="#f3f4f6").pack(anchor=tk.W)
        
        import win32api
        drives = win32api.GetLogicalDriveStrings()
        drives = drives.split('\000')[:-1]
        
        removable_drives = []
        for drive in drives:
            drive_type = win32api.GetDriveType(drive)
            if drive_type == 2:  # Removable drive
                removable_drives.append(drive)
        
        if removable_drives:
            for drive in removable_drives:
                tk.Button(drives_frame, text=f"Use {drive}", 
                         command=lambda d=drive: self.sd_card_path.set(d),
                         bg="#dbeafe", padx=10).pack(side=tk.LEFT, padx=5)
        else:
            tk.Label(drives_frame, text="No removable drives detected", 
                    fg="#6b7280", bg="#f3f4f6").pack()
    
    def browse_target(self):
        """Browse for target directory"""
        path = filedialog.askdirectory(title="Select SD Card or USB Drive")
        if path:
            self.sd_card_path.set(path)
    
    def create_media(self):
        """Create installation media"""
        if not self.sd_card_path.get():
            messagebox.showerror("Error", "Please select a target drive")
            return
        
        # Confirm
        if not messagebox.askyesno("Confirm", 
                                  f"Create installation media on:\n{self.sd_card_path.get()}\n\n"
                                  "This will copy the installer files to the selected drive."):
            return
        
        # Start creation in thread
        thread = threading.Thread(target=self.create_media_thread)
        thread.start()
    
    def create_media_thread(self):
        """Create installation media in background thread"""
        try:
            target_path = Path(self.sd_card_path.get())
            
            self.log("Creating Automata Nexus installation media...")
            self.log(f"Target: {target_path}")
            
            # Create directory structure
            self.update_progress("Creating directory structure...", 10)
            installer_dir = target_path / "automata-nexus-installer"
            installer_dir.mkdir(exist_ok=True)
            
            # Copy installer scripts
            self.update_progress("Copying installer scripts...", 20)
            shutil.copy("installer/install-automata-nexus.py", installer_dir)
            shutil.copy("installer/uninstall-automata-nexus.py", installer_dir)
            
            # Make scripts executable
            os.chmod(installer_dir / "install-automata-nexus.py", 0o755)
            os.chmod(installer_dir / "uninstall-automata-nexus.py", 0o755)
            
            # Create components directory
            components_dir = installer_dir / "components"
            components_dir.mkdir(exist_ok=True)
            
            # Package application
            if self.include_components["base_system"].get():
                self.update_progress("Packaging application...", 30)
                self.package_application(components_dir)
            
            # Copy Python libraries
            if self.include_components["python_libs"].get():
                self.update_progress("Copying Python libraries...", 40)
                self.copy_python_libs(components_dir)
            
            # Copy pre-built node modules
            if self.include_components["node_modules"].get():
                self.update_progress("Copying Node modules...", 50)
                self.copy_node_modules(components_dir)
            
            # Copy drivers
            if self.include_components["drivers"].get():
                self.update_progress("Copying drivers...", 60)
                self.copy_drivers(components_dir)
            
            # Copy demo logic
            if self.include_components["demo_logic"].get():
                self.update_progress("Copying demo logic files...", 70)
                self.copy_demo_logic(components_dir)
            
            # Create manifest
            self.update_progress("Creating manifest...", 80)
            self.create_manifest(installer_dir)
            
            # Create auto-run script
            self.update_progress("Creating auto-run script...", 90)
            self.create_autorun(installer_dir)
            
            # Create README
            self.create_readme(target_path)
            
            self.update_progress("Installation media created successfully!", 100)
            self.log("\n" + "="*50)
            self.log("Installation media is ready!")
            self.log("="*50)
            self.log("\nTo install on Raspberry Pi:")
            self.log("1. Insert this media into your Raspberry Pi")
            self.log("2. Open terminal and navigate to the drive")
            self.log("3. Run: sudo python3 automata-nexus-installer/install-automata-nexus.py")
            
            messagebox.showinfo("Success", "Installation media created successfully!")
            
        except Exception as e:
            self.log(f"ERROR: {str(e)}")
            messagebox.showerror("Error", f"Failed to create installation media:\n{str(e)}")
    
    def package_application(self, dest_dir):
        """Package the application"""
        self.log("Packaging application files...")
        
        app_archive = dest_dir / "app.tar.gz"
        # In production, create tar.gz of built application
        # For now, create a marker file
        app_archive.write_text("Application archive placeholder")
    
    def copy_python_libs(self, dest_dir):
        """Copy Python library wheels"""
        self.log("Copying Python libraries...")
        
        libs_dir = dest_dir / "python_libs"
        libs_dir.mkdir(exist_ok=True)
        
        # In production, download wheels for:
        # megabas, SM16relind, SM16univin, SM16uout, SM8relind
    
    def copy_node_modules(self, dest_dir):
        """Copy pre-built node modules"""
        self.log("Copying Node modules...")
        
        # In production, copy pre-built node_modules for ARM64
    
    def copy_drivers(self, dest_dir):
        """Copy hardware drivers"""
        self.log("Copying hardware drivers...")
        
        drivers_dir = dest_dir / "drivers"
        drivers_dir.mkdir(exist_ok=True)
    
    def copy_demo_logic(self, dest_dir):
        """Copy demo logic files"""
        self.log("Copying demo logic files...")
        
        logic_dir = dest_dir / "demo_logic"
        logic_dir.mkdir(exist_ok=True)
        
        # Create example air handler logic
        ahu_logic = '''// Air Handler Control Logic
module.exports = {
    name: "Air Handler Control",
    equipment_type: "AHU",
    execute: function(inputs, outputs) {
        const { metrics, settings } = inputs;
        
        // Supply air temperature control
        const supplyTemp = metrics.SupplyTemp || 55;
        const supplySetpoint = settings.supply_temp_setpoint || 55;
        
        if (supplyTemp > supplySetpoint + 2) {
            outputs.cooling_valve_position = Math.min(100, outputs.cooling_valve_position + 5);
            outputs.heating_valve_position = Math.max(0, outputs.heating_valve_position - 5);
        } else if (supplyTemp < supplySetpoint - 2) {
            outputs.heating_valve_position = Math.min(100, outputs.heating_valve_position + 5);
            outputs.cooling_valve_position = Math.max(0, outputs.cooling_valve_position - 5);
        }
        
        return outputs;
    }
};'''
        
        (logic_dir / "air_handler.js").write_text(ahu_logic)
    
    def create_manifest(self, installer_dir):
        """Create installation manifest"""
        manifest = {
            "version": "1.0.0",
            "created": time.strftime("%Y-%m-%d %H:%M:%S"),
            "components": {
                key: var.get() 
                for key, var in self.include_components.items()
            },
            "options": {
                "auto_start": self.auto_start.get(),
                "create_backup": self.create_backup.get()
            }
        }
        
        with open(installer_dir / "manifest.json", "w") as f:
            json.dump(manifest, f, indent=2)
    
    def create_autorun(self, installer_dir):
        """Create auto-run script"""
        autorun_script = '''#!/bin/bash
echo "Automata Nexus Automation Control Center - Installer"
echo "===================================================="
echo ""
echo "This will install the control center on your Raspberry Pi."
echo ""
cd "$(dirname "$0")"
sudo python3 install-automata-nexus.py
'''
        
        autorun_path = installer_dir / "install.sh"
        autorun_path.write_text(autorun_script)
        os.chmod(autorun_path, 0o755)
    
    def create_readme(self, target_path):
        """Create README file"""
        readme = '''# Automata Nexus Automation Control Center

## Installation Instructions

1. Boot your Raspberry Pi with Raspberry Pi OS Bullseye (64-bit recommended)

2. Open a terminal and navigate to this drive:
   ```
   cd /media/pi/YOUR_DRIVE_NAME
   ```

3. Run the installer:
   ```
   sudo python3 automata-nexus-installer/install-automata-nexus.py
   ```

4. Follow the on-screen instructions

## System Requirements

- Raspberry Pi 3B+, 4, or newer
- Raspberry Pi OS Bullseye (32-bit or 64-bit)
- 2GB RAM minimum, 4GB recommended
- 8GB SD card minimum
- Internet connection for initial setup

## Support

For support, visit: https://automatanexus.com/support
Email: support@automatanexus.com

Copyright (c) 2024 Automata Nexus. All rights reserved.
'''
        
        (target_path / "README.txt").write_text(readme)
    
    def update_progress(self, message, percent):
        """Update progress bar and status"""
        self.root.after(0, lambda: [
            self.progress_label.config(text=message),
            self.progress_bar.config(value=percent)
        ])
    
    def log(self, message):
        """Add message to log"""
        self.root.after(0, lambda: [
            self.log_text.insert(tk.END, message + "\n"),
            self.log_text.see(tk.END)
        ])
    
    def run(self):
        """Run the installer creator"""
        self.root.mainloop()

if __name__ == "__main__":
    app = InstallerCreator()
    app.run()