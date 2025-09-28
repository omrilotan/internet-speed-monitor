# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.1.7] - 2025-09-28

### 🐛 Critical Bug Fix
- **Data Loading Issue Fixed** - Resolved critical issue where app was displaying oldest data instead of most recent speed test results
- **Chart Display Correction** - Fixed chart to show the latest 20 speed tests instead of the oldest 20 entries
- **Data Ordering Enhancement** - Improved data storage to maintain proper chronological order in JSON file
- **Real-time Updates** - Ensured new speed test results are properly added to data arrays and displayed immediately

### 🔧 Technical Improvements
- **Backend Data Sorting** - Added automatic sorting during data initialization and save operations
- **Frontend Data Processing** - Corrected slice operations to display newest data first in both chart and table
- **Data Persistence** - Enhanced data storage consistency to prevent ordering issues

## [1.1.6] - 2025-09-28

### 🎨 UI Modernization
- **Icon-Only Controls** - Modernized interface with clean icon-only buttons (▶️ Start/Stop, ⏺️ Test)
- **Reorganized Hamburger Menu** - Streamlined dropdown menu order: Export → Clear All → Clear Until
- **Enhanced Menu Integration** - Moved debug functions to application menu under Help for cleaner interface
- **Improved Button Styling** - Larger 44px icons with zero padding for better touch targets
- **Cleaner Layout** - Removed footer links and simplified control area for focused user experience

### 🔧 Technical Improvements
- **Menu System Overhaul** - Integrated debug functions into native application menu with proper IPC handling
- **UI Code Cleanup** - Removed deprecated DOM element references and improved JavaScript error handling
- **Enhanced Accessibility** - Better hover effects and visual feedback for interactive elements
- **Data Management** - Confirmed robust data persistence and loading functionality

## [1.1.5] - 2025-09-28

### ✨ User Experience Enhancements
- **Smart Interval Stepping** - Added intelligent increment/decrement for interval input using arrow keys or mouse wheel (1, 5, 10, 15, 30, 60 minutes)
- **Round to Nearest Feature** - New checkbox to automatically round test times to clean intervals (e.g., top of hour, half hour, quarter hour)
- **Automatic Value Adjustment** - Input values automatically adjust to optimal intervals when rounding is enabled
- **Improved Layout** - Moved rounding checkbox to separate row for cleaner interface organization

### 🎯 Smart Scheduling
- **Intelligent Rounding Logic** - Different rounding rules based on interval: 60min→hour, 30min→half hour, 15min→quarter hour, etc.
- **Input Value Synchronization** - The interval input automatically updates to the nearest optimal value when rounding is toggled
- **Enhanced User Feedback** - Added tooltips and visual hints for smart stepping and rounding features

### 🔧 Technical Improvements
- **Enhanced Input Handling** - Custom event handlers for sophisticated stepping behavior
- **Smart Time Calculations** - Advanced logic for calculating rounded next test times based on current time and interval
- **Improved User Interface** - Better visual hierarchy and spacing in control sections

## [1.1.4] - 2025-09-25

### 🎨 UI/UX Improvements
- **Update Notification Banner** - Redesigned update notifications to appear as a full-width banner below the header instead of inline
- **Responsive Design Fix** - Resolved header overflow issues on smaller screens where download buttons were getting cut off
- **Header Layout Optimization** - Improved header layout with better spacing and element arrangement
- **Mobile-Friendly Interface** - Enhanced responsive design for better mobile and small screen experience

### 🔧 Technical Enhancements
- **CSS Architecture** - Refactored notification styles for better maintainability and responsive behavior
- **Layout System** - Improved flexbox implementation for more reliable cross-device compatibility
- **Performance Optimization** - Streamlined CSS with cleaner responsive breakpoints

## [1.1.3] - 2025-09-24

### ✨ User Experience Improvements
- **24-Hour Time Format** - Updated "Next Test" and "Last Test" displays to use 24-hour format for consistency and clarity
- **Historical Data on Startup** - "Last Test" now automatically loads and displays the most recent test data when the app starts
- **Native Application Help Menu** - Added proper application Help menu with "About" dialog, GitHub repository link, and documentation link
- **Footer Links** - Added footer with clickable links to GitHub repository and project website

### 🌐 Website Enhancements
- **Clickable Logo Navigation** - Logo and icon in documentation pages (Quick Start, User Manual, FAQ) now link back to homepage
- **Logo Hover Effects** - Added subtle hover effects to improve user interaction feedback
- **Copyright Removal** - Removed all copyright mentions from website and app interface per user request

### 🔧 Technical Improvements
- **External Link Handling** - Enhanced IPC communication to handle external link opening from renderer process
- **Menu System** - Implemented native Electron menu system with proper macOS integration
- **Navigation Enhancement** - Improved website navigation consistency across all documentation pages

## [1.1.2] - 2025-09-24

### 📚 Documentation Updates
- **User Experience** - Improved README with better organization and troubleshooting guidance

### 🔧 Technical Changes
- **Debug Configuration** - Enhanced build pipeline with better debugging capabilities
- **Asset Management** - Maintained unified icon asset structure

## [1.1.1] - 2025-01-22

### 🔧 Bug Fixes
- **macOS Security Fix** - Enhanced macOS security configuration with proper entitlements and code signing settings
- **Icon Path Fix** - Fixed app icon access by creating proper asset symlinks to ensure icons display correctly across all platforms
- **Gatekeeper Compatibility** - Improved macOS Gatekeeper compatibility with additional security configurations

### 📦 Distribution
- **macOS Installation** - Added comprehensive macOS installation instructions with multiple security workaround methods
- **Asset Management** - Unified icon asset management between app and documentation
- **Build System** - Enhanced GitHub Actions workflow with improved macOS build configurations

### 📚 Documentation
- **Security Guide** - Detailed macOS security dialog workarounds including right-click method, System Preferences, and Terminal commands
- **Installation** - Comprehensive installation instructions explaining why security warnings occur and how to resolve them

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
