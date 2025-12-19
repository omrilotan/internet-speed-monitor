# Contributing to Internet Speed Monitor

Thank you for your interest in contributing to Internet Speed Monitor! This document provides technical details for developers who want to understand, modify, or extend the application.

## Project Structure

```
monitor/
├── main.js                     # Main Electron process
├── preload.js                  # Security bridge for IPC
├── package.json               # Project dependencies and scripts
├── .github/
│   ├── workflows/             # GitHub Actions CI/CD
│   │   ├── release.yml        # Automated releases
│   │   ├── auto-tag.yml       # Automatic tagging
│   │   └── auto-release.yml   # Manual releases
│   └── copilot-instructions.md # Project guidelines
├── src/
│   ├── speedMonitor.js         # Speed testing module
│   ├── dataStore.js           # Data persistence module
│   └── renderer/
│       ├── index.html         # Main UI
│       ├── styles.css         # UI styling
│       └── renderer.js        # Frontend logic
├── assets/                    # Application icons and images
├── CHANGELOG.md              # Version history
├── CONTRIBUTING.md           # This file
└── README.md                 # User documentation
```

## Technical Architecture

### Electron 38.1.2 with Modern Security

This application uses the latest Electron framework with enhanced security features:

- **Context Isolation**: Enabled (`contextIsolation: true`)
- **Node Integration**: Disabled in renderer (`nodeIntegration: false`) 
- **Preload Script**: Secure IPC bridge using `contextBridge`
- **Content Security Policy**: Strict CSP headers for security

### Main Process (`main.js`)

The main process handles:
- Application lifecycle management
- Window creation and management
- IPC (Inter-Process Communication) handlers
- Background speed monitoring tasks
- System integration

Key IPC handlers:
- **Speed Monitoring**
  - `start-monitoring` - Begins periodic speed tests with interval or cron expression
  - `stop-monitoring` - Stops active monitoring
  - `test-once-now` - Performs single speed test without starting monitoring
  - `get-monitoring-status` - Retrieves current monitoring state with next test time

- **Data Management**
  - `get-speed-tests` - Retrieves up to N most recent speed tests
  - `get-historical-data` - Gets historical data for chart visualization
  - `get-speed-tests-by-range` - Retrieves tests within a date range
  - `get-date-range-bounds` - Gets earliest and latest test dates
  - `clear-history` - Removes all stored speed test data
  - `clear-data-until-date` - Removes tests up to a specific date
  - `export-csv` - Exports all data to CSV format
  - `export-csv-date-range` - Exports data for a specific date range to CSV

- **Debug & Logging**
  - `get-debug-log` - Retrieves debug log file contents
  - `clear-debug-log` - Clears the debug log file

- **External & System**
  - `open-external` - Opens external URLs in default browser
  - `check-for-updates` - Checks GitHub releases for newer versions
  - `get-current-version` - Returns current application version
  - `get-next-test-time` - Calculates next execution time for interval/cron schedule

- **Settings & System Integration**
  - `load-settings` - Loads saved application settings
  - `save-settings` - Persists application settings to file
  - `set-launch-at-startup` - Configures app to launch at system startup
  - `get-launch-at-startup` - Retrieves launch-at-startup configuration status

### Preload Script (`preload.js`)

The preload script provides a secure API bridge between the main and renderer processes:

```javascript
window.electronAPI = {
  // Speed monitoring
  startMonitoring: (schedule) => ipcRenderer.invoke('start-monitoring', schedule),
  stopMonitoring: () => ipcRenderer.invoke('stop-monitoring'),
  getMonitoringStatus: () => ipcRenderer.invoke('get-monitoring-status'),
  testOnceNow: () => ipcRenderer.invoke('test-once-now'),
  
  // Data management
  getSpeedTests: (limit) => ipcRenderer.invoke('get-speed-tests', limit),
  getHistoricalData: (limit) => ipcRenderer.invoke('get-historical-data', limit),
  getSpeedTestsByDateRange: (startDate, endDate) => ipcRenderer.invoke('get-speed-tests-by-range', startDate, endDate),
  clearHistory: () => ipcRenderer.invoke('clear-history'),
  clearDataUntilDate: (cutoffDate) => ipcRenderer.invoke('clear-data-until-date', cutoffDate),
  exportCSV: () => ipcRenderer.invoke('export-csv'),
  exportCSVDateRange: (startDate, endDate) => ipcRenderer.invoke('export-csv-date-range', startDate, endDate),
  getDateRangeBounds: () => ipcRenderer.invoke('get-date-range-bounds'),
  
  // Debug
  getDebugLog: () => ipcRenderer.invoke('get-debug-log'),
  clearDebugLog: () => ipcRenderer.invoke('clear-debug-log'),
  
  // External & Utilities
  openExternal: (url) => ipcRenderer.invoke('open-external', url),
  
  // Version checking & updates
  checkForUpdates: () => ipcRenderer.invoke('check-for-updates'),
  getCurrentVersion: () => ipcRenderer.invoke('get-current-version'),
  getNextTestTime: (schedule) => ipcRenderer.invoke('get-next-test-time', schedule),
  
  // Settings management
  loadSettings: () => ipcRenderer.invoke('load-settings'),
  saveSettings: (settings) => ipcRenderer.invoke('save-settings', settings),
  setLaunchAtStartup: (enable) => ipcRenderer.invoke('set-launch-at-startup', enable),
  getLaunchAtStartup: () => ipcRenderer.invoke('get-launch-at-startup'),
  
  // Event listeners
  onSpeedTestResult: (callback) => ipcRenderer.on('speed-test-result', callback),
  onSpeedTestStarted: (callback) => ipcRenderer.on('speed-test-started', callback),
  onMonitoringStatus: (callback) => ipcRenderer.on('monitoring-status', callback),
  removeAllListeners: (channel) => ipcRenderer.removeAllListeners(channel)
}
```

### Speed Testing (`src/speedMonitor.js`)

#### Technology Stack
- **Library**: `fast-speedtest-api` (v0.3.2)
- **Service**: Netflix's Fast.com API
- **Metrics**: Download speed (real), Upload speed (estimated), Ping (simulated)
- **Scheduling**: Dual mode support with `cron-parser` (v5.4.0) and `node-cron` (v4.2.1)

#### Scheduling Modes
The app supports two scheduling approaches:

1. **Simple Interval Mode**: Fixed time intervals (e.g., every 15 minutes, 1 hour, 1 day)
2. **Cron Expression Mode**: Advanced scheduling using UNIX cron syntax (e.g., work hours, specific days/times)

Cron expressions enable precise control like "every 30 minutes during work hours on weekdays" or "daily at 9 AM and 5 PM".

#### Implementation Details
```javascript
class SpeedMonitor {
  constructor(dataStore) {
    this.dataStore = dataStore;
    this.fastSpeedtest = new FastSpeedtest({
      token: "YXNkZmFzZGxmbnNkYWZoYXNkZmhrYWxm", // Fast.com token
      verbose: false,
      timeout: 10000,
      https: true,
      urlCount: 5,
      bufferSize: 8,
      unit: FastSpeedtest.UNITS.Mbps
    });
  }
}
```

#### Speed Calculation Notes
- **Download Speed**: Actual measurement from Fast.com
- **Upload Speed**: Estimated as 10% of download speed (realistic for most connections)
- **Ping**: Simulated random value between 10-50ms (placeholder for future implementation)

### Data Storage (`src/dataStore.js`)

#### Storage Format
Data is stored in JSON format in the user's application data directory:

```json
{
  "speedTests": [
    {
      "id": 1702809600000,
      "timestamp": "2025-12-17T12:00:00Z",
      "download": 150.5,
      "upload": 45.2,
      "ping": 25,
      "server": "Server Name",
      "isp": "ISP Name",
      "networkInterface": "WiFi",
      "created_at": "2025-12-17T12:00:00.000Z"
    }
  ]
}
```

#### Storage Location
- **Windows**: `%APPDATA%/Internet Speed Monitor/speed_tests.json`
- **macOS**: `~/Library/Application Support/Internet Speed Monitor/speed_tests.json`
- **Linux**: `~/.config/Internet Speed Monitor/speed_tests.json`

#### Data Management
- Records are stored in `speed_tests.json` (JSON format)
- Data is sorted by creation date in descending order (newest first)
- All test data is retained (no automatic cleanup)
- Export functionality to CSV format
- Records include timestamp, speeds, ping, server info, ISP info, and network interface details

### Frontend (`src/renderer/`)

#### UI Framework
- **Base**: HTML5, CSS3, Vanilla JavaScript
- **Charts**: Chart.js v4.4.0 for data visualization
- **Styling**: Custom CSS with modern design patterns and responsive breakpoints
- **Layout**: CSS Grid and Flexbox for responsive design
- **Responsive**: Adapts to desktop, tablet, and mobile screen sizes

#### Key UI Features
- **Primary Controls**: Start/Stop monitoring and Test Once Now in main control row
- **Dual Scheduling Modes**: 
  - Simple Interval with preset buttons (5m, 15m, 30m, 1h, 1d)
  - Cron Expression with validation, work hours preset, and custom patterns
- **Schedule Validation**: Real-time validation with error messages and next run preview
- **Utility Controls**: Debug, clear, and export functions in secondary row
- **Status Management**: Real-time status indicators with color coding:
  - Green: Running test or active monitoring
  - Orange: Sleeping (monitoring active, between tests)
  - Gray: Stopped
- **Test Progress**: Running indicator with emoji and countdown to next test
- **Median Statistics**: Dedicated section showing median performance across all data
- **Mutual Exclusivity**: Smart display logic prevents conflicting UI elements

#### State Management
```javascript
// Status states with proper transitions
const STATUS_STATES = {
  STOPPED: 'stopped',
  INITIALIZING: 'initializing', 
  RUNNING: 'running',
  SLEEPING: 'sleeping'
};

// Status transition flow
// Stopped → Initializing → Running (during test) → Sleeping (between tests) → Running (next test)
```

#### Chart Implementation
```javascript
const chart = new Chart(ctx, {
  type: 'line',
  data: {
    labels: timestamps,
    datasets: [{
      label: 'Download Speed (Mbps)',
      data: downloadSpeeds,
      borderColor: '#007bff',
      tension: 0.4
    }]
  },
  options: {
    responsive: true,
    plugins: {
      legend: { position: 'top' },
      title: { display: true, text: 'Internet Speed History' }
    }
  }
});
```

## Development Setup

### Prerequisites
- **Node.js**: v22.19.0 (latest LTS recommended)
- **npm**: Latest version
- **Git**: For version control

### Installation
```bash
# Clone the repository
git clone https://github.com/omrilotan/internet-speed-monitor.git
cd internet-speed-monitor

# Install dependencies
npm install

# Run in development mode
npm start

# Or run with debugging enabled
npm run dev
```

### Available Scripts

#### Development
- `npm start` - Run the application in development mode
- `npm run dev` - Run with debug mode enabled (shows detailed logs)

#### Building
- `npm run build` - Build for current platform
- `npm run build:mac` - Build macOS installer (.dmg)
- `npm run build:win` - Build Windows installer (.exe)
- `npm run build:linux` - Build Linux packages (.AppImage, .deb)
- `npm run build:all` - Build for all platforms
- `npm run pack` - Create unpacked build (for testing)

#### Release Management
- `npm run release` - Build and publish to GitHub (requires GH_TOKEN)
- `npm run version` - Update changelog automatically

## Build Configuration

### Electron Builder Configuration
The build process is configured in `package.json`:

```json
"build": {
  "appId": "com.omrilotan.internet-speed-monitor",
  "productName": "Internet Speed Monitor",
  "directories": {
    "output": "dist"
  },
  "files": [
    "main.js",
    "preload.js",
    "src/**/*",
    "assets/**/*",
    "node_modules/**/*",
    "package.json"
  ],
  "mac": {
    "category": "public.app-category.utilities",
    "target": [
      { "target": "dmg", "arch": ["x64", "arm64"] },
      { "target": "zip", "arch": ["x64", "arm64"] }
    ]
  },
  "win": {
    "target": [
      { "target": "nsis", "arch": ["x64"] },
      { "target": "portable", "arch": ["x64"] }
    ]
  },
  "linux": {
    "target": [
      { "target": "AppImage", "arch": ["x64"] },
      { "target": "deb", "arch": ["x64"] }
    ],
    "maintainer": "Omri Lotan <omri@omrilotan.com>"
  }
}
```

## CI/CD Pipeline

### GitHub Actions Workflows

#### 1. Release Workflow (`.github/workflows/release.yml`)
- **Trigger**: Tag push (`v*`)
- **Purpose**: Build and create GitHub releases
- **Platforms**: macOS, Windows, Linux
- **Artifacts**: DMG, EXE, AppImage, DEB files

#### 2. Auto-Tag Workflow (`.github/workflows/auto-tag.yml`)
- **Trigger**: Push to main branch
- **Purpose**: Automatically create tags for new versions
- **Logic**: Checks if `package.json` version has corresponding tag

#### 3. Manual Release Workflow (`.github/workflows/auto-release.yml`)
- **Trigger**: Manual dispatch
- **Purpose**: Create releases without version changes

### Release Process

This project uses automated GitHub releases for streamlined version management.

#### Method 1: Automated Release (Recommended)
1. Go to GitHub repository → **Actions** tab
2. Select **"Auto Release"** workflow
3. Click **"Run workflow"**
4. Choose version type: `patch`, `minor`, or `major`
5. Click **"Run workflow"**

**This automatically:**
- Bumps the version in `package.json`
- Updates `CHANGELOG.md`
- Creates a git tag
- Builds for all platforms (macOS, Windows, Linux)
- Creates a GitHub release with all installers

#### Method 2: Tag-Based Release
```bash
# Create and push a version tag
git tag v1.0.1
git push origin v1.0.1
```

#### Method 3: Manual Release
```bash
# Update version manually
npm version patch  # or minor, major
git push
git push --tags
```

#### What Gets Built
- **macOS**: `.dmg` installers (Intel + Apple Silicon) + `.zip` files
- **Windows**: `.exe` NSIS installer + portable `.exe`
- **Linux**: `.AppImage` + `.deb` packages

All builds include update metadata files (`latest-mac.yml`, `latest.yml`, `latest-linux.yml`) for electron-updater integration.

## Publishing & Distribution

### GitHub Releases (Current Method)

The project uses GitHub Releases for distribution. Releases are fully automated via GitHub Actions workflows.

**Release Checklist:**
- [ ] Update version in `package.json`
- [ ] Update `CHANGELOG.md` with changes
- [ ] Test the application thoroughly
- [ ] Commit and push changes to main
- [ ] Trigger release workflow (manual or tag-based)
- [ ] Verify builds complete successfully
- [ ] Test downloaded installers
- [ ] Announce the release

### Alternative Distribution Methods

#### Mac App Store
**Requirements:**
- Apple Developer Account ($99/year)
- Code signing certificates
- App Store guidelines compliance

**Configuration:**
```json
"mas": {
  "category": "public.app-category.utilities",
  "hardenedRuntime": true,
  "entitlements": "entitlements.mas.plist",
  "entitlementsInherit": "entitlements.mas.inherit.plist"
}
```

#### Microsoft Store
**Requirements:**
- Microsoft Developer Account
- Windows Store certification

**Configuration:**
```json
"appx": {
  "applicationId": "InternetSpeedMonitor",
  "backgroundColor": "transparent",
  "showNameOnTiles": false
}
```

#### Snap Store (Linux)
**Configuration:**
```json
"snap": {
  "summary": "Monitor your internet speed",
  "description": "An Electron app that monitors internet connectivity speed at set intervals"
}
```

### Code Signing

#### macOS Code Signing
1. Obtain Apple Developer certificates
2. Add to `package.json`:
```json
"mac": {
  "identity": "Developer ID Application: Your Name"
}
```

#### Windows Code Signing
1. Obtain code signing certificate (.p12 or .pfx)
2. Add to `package.json`:
```json
"win": {
  "certificateFile": "path/to/certificate.p12",
  "certificatePassword": "password"
}
```

**Note:** Store certificates and passwords securely. Use GitHub Secrets for CI/CD.

### Auto-Update Support

To enable auto-updates in the application:

1. Install electron-updater:
```bash
npm install electron-updater
```

2. Add to `main.js`:
```javascript
const { autoUpdater } = require('electron-updater');

app.whenReady().then(() => {
  autoUpdater.checkForUpdatesAndNotify();
});
```

The automated builds already include the necessary metadata files for auto-updates.

### Build File Sizes (Approximate)
- macOS DMG: ~150-200MB
- Windows NSIS: ~120-150MB
- Linux AppImage: ~130-170MB

## Adding Features

### Speed Testing Enhancements
To add new speed test providers:

1. Create a new module in `src/`
2. Implement the same interface as `SpeedMonitor`
3. Update the configuration to allow provider selection

### UI Extensions
To add new visualizations:

1. Extend `src/renderer/renderer.js`
2. Add new Chart.js configurations
3. Update the HTML structure in `src/renderer/index.html`
4. Style with CSS in `src/renderer/styles.css`

### Data Export Formats
To add new export formats:

1. Extend `src/dataStore.js`
2. Add new IPC handlers in `main.js`
3. Update the renderer to call new export methods

## Security Considerations

### Electron Security Best Practices
- ✅ Context isolation enabled
- ✅ Node integration disabled in renderer
- ✅ Preload script for secure IPC
- ✅ Content Security Policy headers
- ✅ No remote content loading
- ✅ Sandboxed renderer processes

### Data Privacy
- All data stored locally
- No data transmitted to external servers (except speed tests)
- No analytics or tracking
- User controls all data export/deletion

## Testing

### Manual Testing Checklist
- [ ] App starts without errors
- [ ] Speed monitoring starts/stops correctly
- [ ] "Test Once Now" button works without starting monitoring
- [ ] Simple Interval mode with preset buttons (5m, 15m, 30m, 1h, 1d) works correctly
- [ ] Cron Expression mode validates expressions in real-time
- [ ] Invalid cron expressions show appropriate error messages
- [ ] Valid cron expressions show next scheduled run time
- [ ] Work hours preset loads correct cron expression
- [ ] Schedule mode switching preserves user input where appropriate
- [ ] Status indicators show correct states (Stopped/Running/Sleeping)
- [ ] Next test countdown displays and updates correctly for both scheduling modes
- [ ] Test running indicator shows during active tests
- [ ] Test running indicator and next test display are mutually exclusive
- [ ] Median statistics calculate and display correctly
- [ ] Data persists between sessions in SQLite database
- [ ] Charts update in real-time with new data
- [ ] Export functionality works (CSV format)
- [ ] Clear history functionality works and resets all UI elements
- [ ] Debug log shows/clears correctly
- [ ] UI responsive to window resizing (desktop/tablet/mobile)
- [ ] All buttons and controls work as expected

### Debug Mode
Enable debug logging by running:
```bash
npm run dev
```

This enables:
- Detailed console logging
- Developer tools auto-open
- Error stack traces
- Performance monitoring

## Troubleshooting

### Common Development Issues

#### 1. Build Failures
```bash
# Clear node_modules and reinstall
rm -rf node_modules package-lock.json
npm install

# Check Node.js version
node --version  # Should be v22+
```

#### 2. Speed Test Failures
- Verify internet connectivity
- Check if Fast.com is accessible
- Examine network firewall settings

#### 3. IPC Communication Issues
- Verify preload script is loading
- Check context isolation settings
- Ensure all IPC channels are properly defined

#### 4. Build Distribution Issues
- Verify all assets are included in `files` array
- Check code signing certificates (macOS/Windows)
- Ensure proper permissions for target platforms

### Debugging Tools
- **Electron DevTools**: Built-in Chromium developer tools
- **Main Process Debugging**: Use VS Code debugger
- **Network Monitoring**: Built-in network panel
- **Performance Profiling**: Chrome performance tools

## Dependencies

### Production Dependencies
- `electron`: ^38.1.2 (Latest stable)
- `fast-speedtest-api`: ^0.3.2 (Speed testing)
- `chart.js`: ^4.4.0 (Data visualization)
- `cron-parser`: ^5.4.0 (Cron expression parsing and validation)
- `node-cron`: ^4.2.1 (Cron job scheduling)

### Development Dependencies
- `electron-builder`: ^26.0.12 (Building/packaging)
- `auto-changelog`: ^2.4.0 (Changelog generation)

### Security Auditing
```bash
npm audit
npm audit fix
```

## Version Management

### Semantic Versioning
This project follows [Semantic Versioning](https://semver.org/):
- **MAJOR**: Breaking changes
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes (backward compatible)

### Changelog
All changes are documented in `CHANGELOG.md` following [Keep a Changelog](https://keepachangelog.com/) format.

## Contributing Guidelines

1. **Fork** the repository
2. **Create** a feature branch (`git checkout -b feature/amazing-feature`)
3. **Commit** your changes (`git commit -m 'Add amazing feature'`)
4. **Push** to the branch (`git push origin feature/amazing-feature`)
5. **Open** a Pull Request

### Code Style
- Use semicolons
- 2-space indentation
- Descriptive variable names
- JSDoc comments for functions
- Error handling for all async operations

### Pull Request Requirements
- [ ] Code follows project style guidelines
- [ ] All tests pass
- [ ] Feature is documented
- [ ] No breaking changes (unless major version)
- [ ] CHANGELOG.md updated

## Support

For technical questions or contributions:
1. Check existing [Issues](https://github.com/omrilotan/internet-speed-monitor/issues)
2. Create a new issue with:
   - Clear description
   - Steps to reproduce
   - Expected vs actual behavior
   - Environment details (OS, Node.js version, etc.)

## License

This project is released into the public domain under the UNLICENSE - see [UNLICENSE](UNLICENSE) file for details.
