# Resonance Logs - Desktop Application

[![GitHub Downloads](https://img.shields.io/github/downloads/resonance-logs/resonance-logs/total?style=for-the-badge&color=%23ff9800)](https://github.com/resonance-logs/resonance-logs/releases) [![Discord](https://img.shields.io/discord/1417447600608510015?color=%235865F2&label=Discord&style=for-the-badge)](https://discord.gg/aPPHe8Uq8Q)

[![GitHub Release](https://img.shields.io/github/v/release/resonance-logs/resonance-logs?style=flat-square)](https://github.com/resonance-logs/resonance-logs/releases) [![GitHub License](https://img.shields.io/github/license/resonance-logs/resonance-logs?style=flat-square)](https://github.com/resonance-logs/resonance-logs/blob/main/LICENSE) [![GitHub Issues](https://img.shields.io/github/issues/resonance-logs/resonance-logs?style=flat-square)](https://github.com/resonance-logs/resonance-logs/issues) [![GitHub Stars](https://img.shields.io/github/stars/resonance-logs/resonance-logs?style=flat-square)](https://github.com/resonance-logs/resonance-logs/stargazers) [![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey?style=flat-square)]()

> **Live combat logging for Blue Protocol: Star Resonance**  
> **Website:** [bpsr.app](https://bpsr.app/) • **Web Platform:** [resonance-website](https://github.com/resonance-logs/resonance-website)

---

## Overview

Resonance Logs is a friendly, cross-platform desktop application designed for **Blue Protocol: Star Resonance** players who want live, actionable feedback during gameplay. Monitor your performance in real-time, track encounters, and share your achievements with the community.

### Key Features

- **Real-time Performance** - Live DPS/HPS monitoring with instant feedback
- **Encounter Tracking** - Save and review past encounters to spot improvement trends  
- **Cloud Sync** - Upload encounters to the Resonance Website for rich analytics
- **Leaderboards** - Compare your performance against the community
- **Detailed Analytics** - Performance breakdowns and improvement suggestions
- **Cross-Platform** - Available for Windows, macOS, and Linux

---

## Quick Start

### Download & Install

1. **Visit [Releases Page](https://github.com/resonance-logs/resonance-logs/releases)**
2. **Download the latest version** for your operating system:
   - **Windows:** `.exe` installer or portable `.zip`
   - **macOS:** `.dmg` disk image  
   - **Linux:** `.AppImage` or `.deb` package

3. **Install and Launch** the application
4. **Connect to the game** and start tracking your encounters

### First Time Setup

1. **Launch Resonance Logs**
2. **Configure Game Connection** (automatic for most setups)
3. **Set up Cloud Sync** (optional) to upload to the website
4. **Start Playing** - encounters will be automatically tracked!

---

## How It Works

### Real-Time Monitoring
- **Live DPS/HPS Display** - See your damage and healing performance in real-time
- **Moment-to-Moment Insights** - Get instant feedback on your gameplay
- **Performance Alerts** - Notifications for significant events and improvements

### Encounter Management
- **Automatic Recording** - Encounters are captured automatically during gameplay
- **Detailed Breakdown** - View damage done, healing, and key performance metrics
- **Historical Data** - Track your progress over time with saved encounters

### Community Integration
- **Upload to Website** - Share encounters on [bpsr.app](https://bpsr.app/) for:
  - Rich analytics and visualizations
  - Leaderboard competition
  - Class and build comparisons
  - Community discussions

---

## Tech Stack

![Tech Stack](image.png)

**Core Technologies:**
[![Svelte](https://img.shields.io/badge/Svelte-4-FF3E00?logo=svelte&style=flat-square)](https://svelte.dev/) [![Tauri](https://img.shields.io/badge/Tauri-2-24C8D8?logo=tauri&style=flat-square)](https://tauri.app/) [![Rust](https://img.shields.io/badge/Rust-1.70+-000000?logo=rust&style=flat-square)](https://www.rust-lang.org/)

---

## Project Architecture

```
resonance-logs/
├── src/                    # Main application source
│   ├── core/              # Core logging and parsing logic
│   ├── ui/                # User interface components
│   ├── network/           # Website communication
│   └── database/          # Local data storage
├── assets/                # Application resources
├── build/                 # Build configurations
└── docs/                  # Documentation
```

---

## Integration with Resonance Website

Resonance Logs seamlessly integrates with the [Resonance Website](https://github.com/resonance-logs/resonance-website) to provide comprehensive combat log analysis:

### Website Features
- **Interactive Leaderboards** - Compare your performance globally
- **Detailed Encounter Pages** - Rich analytics with damage breakdowns
- **Historical Statistics** - Track your progress over time
- **Class Analytics** - Performance data by class and specialization
- **Community Features** - Share encounters and discuss strategies

### Upload Process
1. **Select Encounters** - Choose which encounters to share
2. **Configure Privacy** - Set public/private visibility
3. **Upload** - Sync to the website automatically
4. **Share** - Get a shareable link for your encounters

---

## Development

### Prerequisites
- **Node.js** 18+ (for development)
- **Electron** or equivalent framework
- **Git**

### Local Development

1. **Clone the repository:**
   ```bash
   git clone https://github.com/resonance-logs/resonance-logs.git
   cd resonance-logs
   ```

2. **Install dependencies:**
   ```bash
   npm install
   # or
   yarn install
   ```

3. **Start development server:**
   ```bash
   npm run dev
   # or
   yarn dev
   ```

4. **Build for production:**
   ```bash
   npm run build
   # or
   yarn build
   ```

### Contributing

We welcome contributions! Please see our [Contributing Guidelines](CONTRIBUTING.md) for details.

**Development Workflow:**
1. Fork the repository
2. Create a feature branch: `git checkout -b feature/amazing-feature`
3. Make your changes
4. Test thoroughly
5. Submit a pull request

---

## Configuration

### Settings & Preferences

**Performance Monitoring:**
- Update frequency settings
- DPS/HPS calculation preferences
- Alert thresholds and notifications

**Cloud Sync:**
- Website authentication
- Upload preferences and privacy settings
- Sync frequency options

**Game Connection:**
- Auto-detection settings
- Manual configuration for custom setups
- Connection troubleshooting tools

---

## Data & Privacy

### Local Data Storage
- **Encounters** are stored locally by default
- **Performance history** remains on your device
- **Full control** over what data to upload

### Cloud Upload (Optional)
- **Selective uploading** - choose which encounters to share
- **Privacy controls** - set encounter visibility
- **Secure transmission** - encrypted data transfer to website

---

## Community & Support

- **Discord:** [Join our community](https://discord.gg/aPPHe8Uq8Q)
- **Issues:** [GitHub Issues](https://github.com/resonance-logs/resonance-logs/issues)
- **Documentation:** [Wiki](https://github.com/resonance-logs/resonance-logs/wiki)
- **Website:** [bpsr.app](https://bpsr.app/)

---

## Credits & Acknowledgments

### Foundation
- **Built from:** [winjwinj/bpsr-logs](https://github.com/winjwinj/bpsr-logs)
- **Original inspiration** for combat log analysis

### Data Resources
- [PotRooms/StarResonanceData](https://github.com/PotRooms/StarResonanceData) - Blue Protocol game data
- [snoww/loa-logs](https://github.com/snoww/loa-logs) - Combat log analysis patterns
- [uwuowo.mathi.moe](http://uwuowo.mathi.moe/) - Additional game resources

### Community
- **Beta testers** who helped shape the application
- **Contributors** who improved the codebase
- **Blue Protocol community** for feedback and support

---

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## What's Next?

### Planned Features
- **Mobile companion app** for remote monitoring
- **Advanced analytics** with more detailed breakdowns
- **Discord bot integration** for automated sharing
- **Customizable themes** and interface options

### Version History
- **v1.0** - Initial release with core functionality
- **v1.1** - Enhanced UI and performance improvements
- **v1.2** - Cloud sync and website integration
- **v2.0** - Major redesign with advanced features (coming soon)

---

<div align="center">

**Track. Analyze. Improve. Share.**

**Made with love by the Resonance Logs team**

[Website](https://bpsr.app) • [Discord](https://discord.gg/aPPHe8Uq8Q) • [Web Platform](https://github.com/resonance-logs/resonance-website)

</div>