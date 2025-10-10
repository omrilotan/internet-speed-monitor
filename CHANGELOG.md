# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### 🐛 Bug Fixes
- **CRITICAL: Fixed Cron Test Execution** - Resolved bug where cron-scheduled tests were never actually executed despite proper scheduling calculation
- **Improved Cron Scheduling Logic** - Fixed `shouldRunTestNow` method that was preventing scheduled tests from running
- **Enhanced Cron Debug Logging** - Added detailed logging to help diagnose cron scheduling issues

## [1.3.2]

### 🐛 Bug Fixes
- **Fixed Schedule Corruption Issues** - Resolved critical bug where cron scheduling could become corrupted over time, causing tests to stop running
- **Improved Cron Expression Validation** - Added comprehensive validation for cron expressions with proper error handling and fallback mechanisms
- **Enhanced Schedule Object Integrity** - Implemented defensive copying to prevent schedule object corruption during IPC communication
- **Fixed Logging Parameter Issues** - Corrected logging function calls that were silently dropping important debugging information

### 🛠 Improvements
- **Strengthened Error Handling** - Replaced silent failures with clear error messages for invalid or corrupted schedules
- **Enhanced Schedule Validation** - Added multi-layer validation in renderer, main process, and speed monitor to catch invalid schedules early
- **Improved Debug Logging** - Consolidated and cleaned up debug logs for better readability while maintaining essential debugging information
- **Better Reliability** - Prevents cron-parser from silently accepting empty expressions and defaulting to unexpected behavior

### ✨ New Features
- **Enhanced Update Check Interface** - Added "Check for Updates" button to the About dialog (Help → About Internet Speed Monitor) for easy manual update checking
- **Manual Update Checking** - Users can now force update checks that bypass the 24-hour throttling with proper user feedback dialogs
- **Improved Update User Experience** - Added confirmation dialogs and direct links to GitHub releases when updates are available

### 📊 Technical Details
- Fixed `log()` function parameter handling that was causing missing values in debug output
- Added strict validation for cron expressions (must be strings with exactly 5 parts)
- Implemented JSON deep copying to prevent object reference corruption
- Enhanced error messages to provide actionable information for troubleshooting
- Improved schedule state management to prevent corruption during page reloads or IPC calls
- Enhanced About dialog with integrated update checking functionality and user-friendly error handling

## [1.3.1]

### 🛠 Improvements
- **Modernized Update Checking** - Replaced legacy `https` module with modern `fetch` API for better reliability and performance
- **Enhanced Update Notifications** - Improved timing of update notifications to ensure they display properly on app startup
- **Code Modernization** - Updated version checking code to use current JavaScript standards and async/await patterns

## [1.3.0]

### 🎉 New Features
- **Network Interface Detection** - Speed tests now detect and display the network connection type (WiFi, Ethernet, etc.) for each test
- **ISP Information** - Automatically lookup and display Internet Service Provider information for each speed test
- **Enhanced Test Results Table** - Recent Tests table now includes Network Interface and ISP columns for better network diagnostics
- **Improved Data Export** - CSV exports now include Network Interface and ISP data for comprehensive analysis

### 🛠 Improvements
- **Table Ordering** - Fixed historical data loading to correctly display newest tests at the top of the Recent Tests table
- **Data Storage** - Enhanced speed test data structure to include network interface and ISP information
- **Cross-platform Network Detection** - Added support for network interface detection on macOS, Windows, and Linux

### 📊 Technical Details
- Network interface detection uses OS-specific commands (route, networksetup, netsh, ip) for accurate results
- ISP lookup utilizes public APIs (ipify.org, ip-api.com) with proper timeout handling
- Maintains backward compatibility with existing speed test data

## [1.2.3]

### 🚑 Patch Release
- **Release** - Bumped package.json and docs to 1.2.3 and created annotated git tag `v1.2.3`.
- **Docs** - Updated website links and cache-bust tokens to reference v1.2.3.

### 🛠 Fixes & Stability
- **Cron scheduling reliability** - Ensured cron next-run calculations are performed in the main process and exposed via a secure `getNextTestTime` IPC in `preload.js`.
- **IPC contract** - `get-next-test-time` in `main.js` now returns a numeric timestamp (ms since epoch) and logs parsing success/failure; renderer converts the value to a Date.
- **Renderer resilience** - Updated `src/renderer/renderer.js` to call the main IPC asynchronously, handle unexpected responses, and fall back safely when necessary.
- **Cron parser compatibility** - Replaced incompatible `parser.parseExpression` usage with `CronExpressionParser.parse(...)` in `main.js` and `src/speedMonitor.js`.
- **Cron behavior** - Cron mode no longer runs an immediate test on start; it waits for the first scheduled occurrence.
- **Logging & diagnostics** - Added console logs to main to make cron parse failures visible at runtime; added a small node test during development to confirm the cron-parser API.

### 📝 Notes
- If you still see "next test in 5 minutes" after upgrading, fully restart the Electron app to ensure the updated main process is running (old main process instances will continue to return the fallback).

## [1.2.2]

### 🐛 Bug Fixes
- **Cron Scheduling** - Fixed cron next-run calculation by moving cron parsing to main process and using the correct cron-parser API
- **Cron Behavior** - Cron mode no longer runs an immediate test at startup; the first test runs at the next scheduled occurrence
- **Status Display** - Ensure status shows "Sleeping" when monitoring is active but waiting for next scheduled test; "Running" is shown only during an active test

### 🔧 Technical Fixes
- **Security/IPC** - Exposed a safe `getNextTestTime` IPC endpoint in the preload script for accurate next-run calculations from the renderer
- **API Compatibility** - Updated code to use `CronExpressionParser.parse(...)` to match the installed cron-parser API and avoid runtime errors


### 🐛 Bug Fixes
- **UI Improvements** - Fixed hamburger dropdown visibility issue in header by removing overflow hidden
- **Status Text** - Changed "Starting monitoring..." to cleaner "Running" status for better UX
- **Layout Fixes** - Added proper spacing between status indicator and hamburger menu (20px margin)
- **Dropdown Positioning** - Increased z-index to 1000 for better dropdown visibility above other elements
- **HTML Structure** - Removed duplicate "schedule-info" next test display, keeping only the stat-card version
- **Element IDs** - Fixed duplicate next-test-time IDs by using unique next-test-time-display ID

### 🎨 Interface Enhancements
- **Header Layout** - Moved hamburger menu to end of header for better accessibility and standard UI conventions
- **Code Cleanup** - Removed unused nextTestInfo and nextTestTimeDisplay references
- **CSS Optimization** - Updated CSS selectors to match new unique element IDs

## [1.2.0]

### 🚀 Notable Features
- **Advanced Cron Scheduling** - Complete cron-style scheduling system with UNIX cron expression support
- **Dual Scheduling Modes** - Choose between Simple Interval (1-60 minutes) or flexible Cron Expression scheduling
- **Smart Preset Buttons** - Quick-access buttons for common schedules: 5m, 15m, 30m, 1h, 1d, Work hours, and Custom
- **Real-time Cron Validation** - Live validation and description of cron expressions with helpful error messages
- **Enhanced Custom Scheduling** - Support for complex schedules like "Every 30 minutes during work hours" or "Daily at specific times"

### ✨ User Experience Improvements
- **Streamlined Interface** - Removed "Schedule:" label for cleaner, more compact design
- **Responsive Cron Controls** - Optimized layout that adapts to different screen sizes without overflow
- **Interactive Cron Help** - Built-in link to crontab.guru for creating custom schedules with helpful guidance
- **Better Preset Organization** - Curated preset selection (removed 10m and 2h, added 1d) for optimal user workflow
- **Chart Context Information** - Added "(last 20 data points)" subtitle to Speed History chart for clarity

### 🎯 Enhanced Scheduling Capabilities
- **Work Hours Preset** - "Every hour, 9am-5pm, Mon-Fri" for business environment monitoring
- **Daily Monitoring** - "1d" preset for once-daily speed checks at midnight
- **Flexible Intervals** - Support for any cron-compatible schedule including minutes, hours, days, weeks, months
- **Custom Expressions** - Full UNIX cron syntax support with real-time validation and descriptions

### 🔧 Technical Improvements
- **Cron Parser Integration** - Added cron-parser library for robust schedule parsing and validation
- **Node-cron Implementation** - Integrated node-cron for reliable cron-based task scheduling
- **Improved Space Utilization** - CSS optimizations for better use of horizontal space in cron controls
- **Enhanced Error Handling** - Better error messages and fallback handling for invalid cron expressions
- **Code Cleanup** - Removed legacy round-to-nearest logic and unused scheduling code

### 📚 Documentation Updates
- **Feature Documentation** - Updated README and docs to reflect new cron scheduling capabilities
- **User Guide Enhancement** - Added explanation of both scheduling modes and preset options
- **Technical Reference** - Documented cron expression support and validation features

## [1.1.7]

### 🐛 Critical Bug Fix
- **Data Loading Issue Fixed** - Resolved critical issue where app was displaying oldest data instead of most recent speed test results
- **Chart Display Correction** - Fixed chart to show the latest 20 speed tests instead of the oldest 20 entries
- **Data Ordering Enhancement** - Improved data storage to maintain proper chronological order in JSON file
- **Real-time Updates** - Ensured new speed test results are properly added to data arrays and displayed immediately

### 🔧 Technical Improvements
- **Backend Data Sorting** - Added automatic sorting during data initialization and save operations
- **Frontend Data Processing** - Corrected slice operations to display newest data first in both chart and table
- **Data Persistence** - Enhanced data storage consistency to prevent ordering issues

## [1.1.6]

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

## [1.1.5]

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

## [1.1.4]

### 🎨 UI/UX Improvements
- **Update Notification Banner** - Redesigned update notifications to appear as a full-width banner below the header instead of inline
- **Responsive Design Fix** - Resolved header overflow issues on smaller screens where download buttons were getting cut off
- **Header Layout Optimization** - Improved header layout with better spacing and element arrangement
- **Mobile-Friendly Interface** - Enhanced responsive design for better mobile and small screen experience

### 🔧 Technical Enhancements
- **CSS Architecture** - Refactored notification styles for better maintainability and responsive behavior
- **Layout System** - Improved flexbox implementation for more reliable cross-device compatibility
- **Performance Optimization** - Streamlined CSS with cleaner responsive breakpoints

## [1.1.3]

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

## [1.1.2]

### 📚 Documentation Updates
- **User Experience** - Improved README with better organization and troubleshooting guidance

### 🔧 Technical Changes
- **Debug Configuration** - Enhanced build pipeline with better debugging capabilities
- **Asset Management** - Maintained unified icon asset structure

## [1.1.1]

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

## [1.1.0]

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
