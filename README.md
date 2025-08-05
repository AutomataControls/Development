# AutomataNexus R&D Development Repository

Central repository for all research and development projects by AutomataControls.

## 🚀 Repository Structure

```
Development/
├── projects/               # Individual R&D projects
│   ├── building-automation-controller/  # Professional BAS for Raspberry Pi
│   ├── wtvb01-vibration-monitor/      # Raspberry Pi vibration monitoring system
│   ├── node-red-flows/                # Node-RED automation flows
│   └── [future-projects]/             # Additional projects
├── libraries/              # Shared libraries and components
├── documentation/          # Cross-project documentation
└── tools/                  # Development tools and utilities
```

## 📁 Projects

### 1. [Automata Nexus Control Center](./projects/building-automation-controller/)
Enterprise-grade building automation system competing with Niagara, Automated Logic, and Johnson Controls.
- **Tech Stack:** Rust, Tauri, Next.js 14, TypeScript, SQLite
- **Features:** Sequent Microsystems I/O, BMS integration, JavaScript logic engine, 0-10V scaling
- **Hardware:** Raspberry Pi 4/5, MegaBAS HAT, 16 Universal Input, 16 Analog Output, 8/16 Relay
- **Status:** ✅ Active Development

### 2. [WTVB01 Vibration Monitor](./projects/wtvb01-vibration-monitor/)
Professional industrial vibration monitoring system for Raspberry Pi using WIT-Motion WTVB01-485 sensors.
- **Tech Stack:** Rust, Tauri, HTML5
- **Features:** ISO 10816-3 compliance, 230400 baud optimization, burst reading
- **Status:** ✅ Active Development

### 3. [Node-RED Flows](./projects/node-red-flows/)
Collection of industrial automation flows and custom nodes.
- **Tech Stack:** Node-RED, JavaScript
- **Features:** BMS integration, Modbus communication, data logging
- **Status:** 🔄 In Progress

### 4. Future Projects
- BMS Integration Platform
- Industrial IoT Gateway
- Predictive Maintenance System
- Energy Monitoring Dashboard

## 🛠️ Development Setup

### Prerequisites
- Git
- Node.js 16+
- Rust 1.70+
- Python 3.8+

### Clone Repository
```bash
git clone https://github.com/AutomataControls/Development.git
cd Development
```

### Project-Specific Setup
Each project has its own README with detailed setup instructions. Navigate to the project folder for specifics.

## 📊 Project Status

| Project | Status | Language | Platform | Last Updated |
|---------|--------|----------|----------|--------------|
| Automata Nexus Control Center | ✅ Active | Rust/TypeScript | Raspberry Pi | 2025-08 |
| WTVB01 Monitor | ✅ Active | Rust/HTML | Raspberry Pi | 2025-01 |
| Node-RED Flows | 🔄 In Progress | JavaScript | Cross-platform | 2025-01 |

## 🤝 Contributing

This is a private R&D repository. For collaboration inquiries, contact:
- Email: AutomataControls@gmail.com

## 📝 Documentation

- [Project Guidelines](./documentation/guidelines.md)
- [API Documentation](./documentation/api/)
- [Hardware Specifications](./documentation/hardware/)

## 🔧 Tools

Utility scripts and development tools are available in the `/tools` directory:
- Deployment scripts
- Testing utilities
- Build automation

## 📜 License

© 2025 AutomataControls. All rights reserved.

Private repository - Commercial license required for any use.

## 🏆 Author

**AutomataControls**
- GitHub: [@AutomataControls](https://github.com/AutomataControls)
- Email: AutomataControls@gmail.com

---

*This repository contains proprietary research and development projects. Unauthorized use, reproduction, or distribution is strictly prohibited.*
