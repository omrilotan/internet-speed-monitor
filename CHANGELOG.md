# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.1]

### Major Updates
- **Upgraded to Electron 38.1.2** (from 27.0.0) - Latest stable version with Node.js 22.19.0
- **Upgraded electron-builder to 26.0.12** (from 24.6.4) - Latest build tools

### Security Improvements
- **Enhanced security architecture** with modern Electron standards
- **Enabled contextIsolation: true** - Isolates renderer context for better security
- **Disabled nodeIntegration: false** - Prevents direct Node.js access in renderer
- **Added secure preload script** (`preload.js`) - Safe IPC bridge using contextBridge

### Technical Enhancements
- **Refactored IPC communication** - Updated renderer to use secure electronAPI
- **Added missing IPC handlers** - Fixed get-speed-tests handler for better reliability
- **Updated build configuration** - Includes preload script in packaged builds
- **Maintained full backward compatibility** - All existing features preserved

### Functionality Verified
- ✅ Speed monitoring with configurable intervals
- ✅ Data persistence and historical data loading  
- ✅ CSV export functionality
- ✅ Clear history feature
- ✅ Real-time charts and visualization
- ✅ Cross-platform builds and packaging

### Benefits
- **Latest Node.js runtime** (22.19.0 vs 18.17.1) - Better performance and security
- **Future-proof architecture** - Follows modern Electron best practices  
- **Enhanced security** - Isolated renderer context prevents security vulnerabilities
- **Maintained compatibility** - All features working seamlessly after upgrade

## [1.0.0]

### Added
- Initial release of Internet Speed Monitor
- Periodic internet speed testing at configurable intervals (1-60 minutes)
- Real-time monitoring display with download/upload speeds and ping
- Historical data storage using JSON files
- Interactive charts for speed visualization using Chart.js
- Clean, modern user interface with status indicators
- Cross-platform support (macOS, Windows, Linux)
- Automated builds using electron-builder

### Technical Details
- Built with Electron framework
- Uses fast-speedtest-api for speed testing
- JSON-based data persistence (no external database required)
- Chart.js for data visualization
- Responsive UI design

### Features
- Start/stop monitoring controls
- Configurable test intervals
- Speed history charts
- Recent test results table
- Persistent data storage
- Cross-platform compatibility
