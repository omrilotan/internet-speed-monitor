# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.1.0] - 2025-09-24

### ✨ New Features
- **Test Once Now Button** - Added purple "Test Once Now" button for on-demand speed testing without starting monitoring, now prominently placed in the main control row
- **Median Speed Display** - New median statistics section showing median download, upload, and ping values across all historical data
- **Next Test Countdown** - Real-time countdown timer showing when the next scheduled test will run during active monitoring
- **Enhanced UI Layout** - Reorganized interface with "Last Test" moved to first position and improved button arrangement
- **Test Running Indicator** - Added visual indicator with emoji and status text when speed tests are actively running
- **Sleeping Status** - New "Sleeping" status with orange indicator shows when monitoring is active but waiting between scheduled tests
- **Clear Debug Log Button** - Added dedicated button to clear the debug log file for better log management

### 🎨 UI/UX Improvements
- **Responsive Design** - Added responsive breakpoints for better mobile and tablet experience
- **Button Layout Optimization** - Reorganized button layout with primary controls (Start/Stop/Test Now) in top row and utilities (Debug, Clear, Export) in bottom row
- **Flexible Controls** - Top row controls now wrap gracefully on narrow screens
- **Visual Hierarchy** - Improved spacing and alignment for better user experience
- **Median Stats Integration** - Dedicated section for median calculations with consistent styling
- **Mutual Exclusivity** - Test running indicator and next test display now show one OR the other, never both simultaneously
- **Status Color Coding** - Enhanced status indicators with proper color coding (green for running, orange for sleeping, gray for stopped)

### 🔧 Technical Enhancements
- **IPC Communication** - Added `test-once-now` and `speed-test-started` IPC handlers for enhanced test coordination
- **Median Calculation Algorithm** - Efficient median calculation with data limit management (keeps last 100 tests for performance)
- **Timer Management** - Comprehensive next test timer with automatic cleanup and display updates
- **Enhanced Status Logic** - Improved status synchronization between monitoring state and UI display with proper state machine
- **Data Store Architecture** - Better separation of manual vs scheduled test execution
- **Mutual Exclusivity Logic** - Smart display logic prevents conflicting UI elements from showing simultaneously
- **Debug Log Management** - Backend support for clearing debug logs with proper file handling and logging

### 🐛 Bug Fixes
- **Clear History Functionality** - Fixed clear history button to properly reset all UI elements including current stats, median stats, chart, and data arrays
- **Status Display Synchronization** - Resolved issue where status remained "running" after stopping manual tests
- **DOM Element References** - Fixed incorrect element ID references in UI reset functions
- **Data Persistence** - Corrected `clearAllData()` method to properly clear `this.data.speedTests` array
- **UI State Management** - Enhanced `loadHistoricalData()` to handle empty data scenarios correctly
- **Display Race Conditions** - Fixed timing issues where test running indicator and next test display could appear simultaneously
- **Timer Interference** - Resolved conflicts between automatic timer updates and manual display control

### 🔄 Code Quality
- **Enhanced Error Handling** - Better error management in data clearing, UI reset operations, and debug log management
- **Improved Function Organization** - Cleaner separation between current stats and median stats updates
- **Consistent Element Access** - Standardized DOM element access patterns throughout renderer
- **Memory Management** - Optimized data array management to prevent memory leaks in long-running sessions
- **State Machine Architecture** - Implemented proper state transitions for monitoring status (Stopped → Initializing → Running → Sleeping)

### 📱 Responsive Features
- **Mobile-First Design** - Controls adapt to screen width with proper wrapping and centering
- **Tablet Optimization** - Medium screen breakpoints for optimal tablet viewing
- **Button Scaling** - Smart button sizing with minimum widths to prevent text truncation
- **Layout Flexibility** - Flex-based layout system for consistent cross-device experience

## [1.0.2]

### Bug Fixes
- **Fixed GitHub Actions workflow permissions** - Added `contents: write` permission to allow automatic release creation
- **Resolved 403 Forbidden error** when electron-builder attempts to publish releases to GitHub
- **Enhanced workflow reliability** with proper permission scopes for issues and pull-requests

### Infrastructure
- **Improved CI/CD pipeline** - GitHub Actions now successfully creates releases with downloadable binaries
- **Cross-platform builds** working correctly for macOS (DMG/ZIP), Windows (EXE/NSIS), and Linux (AppImage/DEB)

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
