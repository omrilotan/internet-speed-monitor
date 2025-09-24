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
- `start-monitoring` - Begins periodic speed tests
- `stop-monitoring` - Stops monitoring
- `test-once-now` - Performs single speed test without starting monitoring
- `speed-test-started` - Event sent when any test begins
- `get-monitoring-status` - Retrieves current monitoring state
- `get-speed-tests` - Retrieves stored speed data
- `get-historical-data` - Gets historical data for charts
- `clear-history` - Removes all stored data
- `export-csv` - Exports data to CSV format
- `get-debug-log` - Retrieves debug log contents
- `clear-debug-log` - Clears the debug log file

### Preload Script (`preload.js`)

The preload script provides a secure API bridge between the main and renderer processes:

```javascript
window.electronAPI = {
  // Speed monitoring
  startMonitoring: (interval) => ipcRenderer.invoke('start-monitoring', interval),
  stopMonitoring: () => ipcRenderer.invoke('stop-monitoring'),
  getMonitoringStatus: () => ipcRenderer.invoke('get-monitoring-status'),
  testOnceNow: () => ipcRenderer.invoke('test-once-now'),
  
  // Data management
  getSpeedTests: (limit) => ipcRenderer.invoke('get-speed-tests', limit),
  getHistoricalData: (limit) => ipcRenderer.invoke('get-historical-data', limit),
  clearHistory: () => ipcRenderer.invoke('clear-history'),
  exportCSV: () => ipcRenderer.invoke('export-csv'),
  
  // Debug
  getDebugLog: () => ipcRenderer.invoke('get-debug-log'),
  clearDebugLog: () => ipcRenderer.invoke('clear-debug-log'),
  
  // Events
  onSpeedTestResult: (callback) => ipcRenderer.on('speed-test-result', callback),
  onSpeedTestStarted: (callback) => ipcRenderer.on('speed-test-started', callback),
  onMonitoringStatus: (callback) => ipcRenderer.on('monitoring-status', callback)
}
```

### Speed Testing (`src/speedMonitor.js`)

#### Technology Stack
- **Library**: `fast-speedtest-api` (v0.3.2)
- **Service**: Netflix's Fast.com API
- **Metrics**: Download speed (real), Upload speed (estimated), Ping (simulated)

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

```javascript
{
  "timestamp": "2025-09-23T10:30:00.000Z",
  "downloadSpeed": 45.2,
  "uploadSpeed": 4.5,
  "ping": 23
}
```

#### Storage Location
- **Windows**: `%APPDATA%/internet-speed-monitor/`
- **macOS**: `~/Library/Application Support/internet-speed-monitor/`
- **Linux**: `~/.config/internet-speed-monitor/`

#### Data Management
- Records are stored in `speed-data.json`
- Automatic cleanup of records older than 30 days
- Export functionality to CSV format
- Thread-safe file operations

### Frontend (`src/renderer/`)

#### UI Framework
- **Base**: HTML5, CSS3, Vanilla JavaScript
- **Charts**: Chart.js v4.4.0 for data visualization
- **Styling**: Custom CSS with modern design patterns and responsive breakpoints
- **Layout**: CSS Grid and Flexbox for responsive design
- **Responsive**: Adapts to desktop, tablet, and mobile screen sizes

#### Key UI Features
- **Primary Controls**: Start/Stop monitoring and Test Once Now in main control row
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
1. Update version in `package.json`
2. Update `CHANGELOG.md`
3. Commit and push to main
4. Auto-tag workflow creates version tag
5. Release workflow builds and publishes binaries

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
- [ ] Status indicators show correct states (Stopped/Running/Sleeping)
- [ ] Next test countdown displays and updates correctly
- [ ] Test running indicator shows during active tests
- [ ] Test running indicator and next test display are mutually exclusive
- [ ] Median statistics calculate and display correctly
- [ ] Data persists between sessions
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
