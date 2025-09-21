# Internet Speed Monitor

An Electron application that monitors internet connectivity speed at set intervals and stores the data for analysis and visualization.

## Features

- **Periodic Speed Testing**: Automatically runs internet speed tests at configurable intervals (1-60 minutes)
- **Real-time Monitoring**: View current download speed, upload speed, and ping values
- **Data Persistence**: Stores all speed test results locally using JSON files
- **Historical Data**: View recent test results in a table format
- **Speed Charts**: Visualize speed trends over time with interactive charts
- **User-friendly Interface**: Clean, modern UI built with HTML, CSS, and Chart.js

## Project Structure

```
monitor/
├── main.js                     # Main Electron process
├── package.json               # Project dependencies and scripts
├── .github/
│   └── copilot-instructions.md # Project guidelines
├── src/
│   ├── speedMonitor.js         # Speed testing module
│   ├── dataStore.js           # Data persistence module
│   └── renderer/
│       ├── index.html         # Main UI
│       ├── styles.css         # UI styling
│       └── renderer.js        # Frontend logic
└── README.md                  # This file
```

## Installation & Setup

### Option 1: Download Pre-built App (Recommended)
1. Go to the [Releases page](https://github.com/omrilotan/internet-speed-monitor/releases)
2. Download the appropriate installer for your platform:
   - **macOS**: Download the `.dmg` file
   - **Windows**: Download the `.exe` installer
   - **Linux**: Download the `.AppImage` file

### Option 2: Build from Source
1. **Prerequisites**:
   - Node.js (v16 or higher)
   - npm package manager

2. **Install Dependencies**:
   ```bash
   npm install
   ```

3. **Run the Application**:
   ```bash
   npm start
   ```

4. **Build for Distribution**:
   ```bash
   # For your current platform
   npm run build
   
   # For specific platforms
   npm run build:mac     # macOS
   npm run build:win     # Windows
   npm run build:linux   # Linux
   npm run build:all     # All platforms
   ```

## Usage

1. **Starting Monitoring**:
   - Set your desired test interval (1-60 minutes)
   - Click "Start Monitoring" to begin automatic speed tests
   - The app will run an initial test immediately

2. **Viewing Results**:
   - Current speeds are displayed in the stat cards at the top
   - Historical data is shown in the chart and table below
   - The status indicator shows when monitoring is active

3. **Stopping Monitoring**:
   - Click "Stop Monitoring" to halt automatic testing
   - Data is automatically saved and persists between sessions

## Technical Details

### Speed Testing
- Uses the `fast-speedtest-api` library for download speed tests
- Upload speeds are estimated (10% of download speed)
- Ping values are simulated for demonstration purposes
- Tests are performed using Netflix's Fast.com service

### Data Storage
- Speed test results are stored in JSON format
- Data files are saved in the user's application data directory
- No external database dependencies required
- Automatic cleanup of old records (configurable)

### Architecture
- **Main Process**: Handles app lifecycle, IPC communication, and background tasks
- **Renderer Process**: Manages the UI and user interactions
- **Speed Monitor**: Handles periodic testing and result processing
- **Data Store**: Manages data persistence and retrieval

## Development

### Available Scripts
- `npm start` - Run the application in development mode
- `npm run dev` - Run with debug mode enabled
- `npm run build` - Build the application for distribution

### Building for Distribution
```bash
npm run build
```

### Adding Features
- Extend the `speedMonitor.js` module for additional test types
- Modify `dataStore.js` for different storage options
- Update the UI in `src/renderer/` for new visualizations

## Configuration

The application can be configured by modifying the following:

- **Test Interval**: Set via the UI (1-60 minutes)
- **Data Retention**: Modify `dataStore.js` to change how long data is kept
- **Speed Test Settings**: Adjust parameters in `speedMonitor.js`

## Troubleshooting

### Common Issues

1. **Speed Tests Failing**:
   - Check internet connectivity
   - Verify firewall settings allow the app to access the internet
   - Try restarting the application

2. **Data Not Persisting**:
   - Ensure the app has write permissions to the user data directory
   - Check available disk space

3. **UI Not Updating**:
   - Restart the application
   - Check the developer console for JavaScript errors (Ctrl/Cmd + Shift + I)

### Debug Mode
Run with debug mode to see detailed logs:
```bash
npm run dev
```

## Future Enhancements

- Real upload speed testing (requires additional service integration)
- Actual ping measurements using system tools
- Export data to CSV/JSON formats
- Network quality analysis and recommendations
- Multiple server testing for better accuracy
- Mobile app companion

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Test thoroughly
5. Submit a pull request

## License

MIT License - see LICENSE file for details

## Support

For issues and questions:
1. Check the troubleshooting section above
2. Enable debug mode to gather more information
3. Open an issue with detailed reproduction steps
