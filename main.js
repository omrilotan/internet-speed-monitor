const { app, BrowserWindow, ipcMain, shell, Menu } = require('electron');
const path = require('path');
const fs = require('fs');
const https = require('https');

// Get current version from package.json
const packageJson = require('./package.json');
const CURRENT_VERSION = packageJson.version;

// Version check configuration
const VERSION_CHECK_URL = 'https://api.github.com/repos/omrilotan/internet-speed-monitor/releases/latest';
const VERSION_CHECK_FILE = path.join(app.getPath('userData'), 'last-version-check.json');

// Setup logging to file for debugging
const logFile = path.join(app.getPath('userData'), 'debug.log');

function logToFile(message) {
  const timestamp = new Date().toISOString();
  const logMessage = `${timestamp}: ${message}\n`;
  try {
    fs.appendFileSync(logFile, logMessage);
  } catch (error) {
    // Ignore file write errors
  }
}

function log(message) {
  console.log(message);
  logToFile(message);
}

function logError(message, error) {
  console.error(message, error);
  logToFile(`ERROR: ${message} - ${error?.message || error}`);
}

// Version checking functions
async function checkForUpdates() {
  try {
    if (!shouldCheckForUpdates()) {
      log('Skipping update check (checked within last 24 hours)');
      return null;
    }
    
    log('Checking for updates...');
    
    const latestVersion = await fetchLatestVersion();
    saveLastCheckDate(); // Save check date regardless of result
    
    if (latestVersion && isNewerVersion(latestVersion, CURRENT_VERSION)) {
      log(`New version available: ${latestVersion} (current: ${CURRENT_VERSION})`);
      
      // Create update info object
      const updateInfo = {
        latestVersion,
        currentVersion: CURRENT_VERSION,
        checkDate: new Date().toISOString()
      };
      
      return updateInfo;
    } else {
      log(`No updates available. Current version ${CURRENT_VERSION} is latest.`);
      return null;
    }
  } catch (error) {
    logError('Error checking for updates:', error);
    return null;
  }
}

function fetchLatestVersion() {
  return new Promise((resolve, reject) => {
    const options = {
      hostname: 'api.github.com',
      path: '/repos/omrilotan/internet-speed-monitor/releases/latest',
      headers: {
        'User-Agent': 'Internet-Speed-Monitor-App'
      }
    };

    const req = https.get(options, (res) => {
      let data = '';
      
      res.on('data', (chunk) => {
        data += chunk;
      });
      
      res.on('end', () => {
        try {
          const release = JSON.parse(data);
          const version = release.tag_name?.replace('v', ''); // Remove 'v' prefix if present
          resolve(version);
        } catch (error) {
          reject(error);
        }
      });
    });
    
    req.on('error', (error) => {
      reject(error);
    });
    
    req.setTimeout(10000, () => {
      req.destroy();
      reject(new Error('Request timeout'));
    });
  });
}

function isNewerVersion(latest, current) {
  const parseVersion = (version) => {
    return version.split('.').map(num => parseInt(num, 10));
  };
  
  const latestParts = parseVersion(latest);
  const currentParts = parseVersion(current);
  
  for (let i = 0; i < Math.max(latestParts.length, currentParts.length); i++) {
    const latestPart = latestParts[i] || 0;
    const currentPart = currentParts[i] || 0;
    
    if (latestPart > currentPart) return true;
    if (latestPart < currentPart) return false;
  }
  
  return false;
}

function shouldCheckForUpdates() {
  try {
    if (!fs.existsSync(VERSION_CHECK_FILE)) {
      return true; // First time, should check
    }
    
    const data = JSON.parse(fs.readFileSync(VERSION_CHECK_FILE, 'utf8'));
    const lastCheck = new Date(data.lastCheckDate);
    const now = new Date();
    const daysSinceLastCheck = (now - lastCheck) / (1000 * 60 * 60 * 24);
    
    return daysSinceLastCheck >= 1; // Check once per day
  } catch (error) {
    return true; // If error reading file, should check
  }
}

function saveLastCheckDate() {
  try {
    const data = {
      lastCheckDate: new Date().toISOString(),
      currentVersion: CURRENT_VERSION
    };
    fs.writeFileSync(VERSION_CHECK_FILE, JSON.stringify(data, null, 2));
  } catch (error) {
    logError('Error saving last check date:', error);
  }
}

log('=== MAIN.JS LOADED ===');
log('Electron version: ' + process.versions.electron);
log('Node version: ' + process.versions.node);
log('Process platform: ' + process.platform);
log('Process arch: ' + process.arch);
log('App is packaged: ' + app.isPackaged);
log('Log file location: ' + logFile);

let mainWindow;
let speedMonitor;
let dataStore;
let isInitialized = false;

function createWindow() {
  console.log('=== CREATING WINDOW ===');
  mainWindow = new BrowserWindow({
    width: 1200,
    height: 800,
    webPreferences: {
      nodeIntegration: false,
      contextIsolation: true,
      preload: path.join(__dirname, 'preload.js')
    }
  });

  console.log('Loading renderer file...');
  mainWindow.loadFile('src/renderer/index.html');

  // Open DevTools in development
  if (!app.isPackaged) {
    console.log('Opening DevTools for development');
    mainWindow.webContents.openDevTools();
  } else {
    console.log('Production mode - DevTools not opened automatically');
  }

  mainWindow.on('closed', () => {
    console.log('Window closed');
    mainWindow = null;
  });
}

// Create application menu
function createMenu() {
  const template = [
    ...(process.platform === 'darwin' ? [{
      label: app.getName(),
      submenu: [
        { role: 'about' },
        { type: 'separator' },
        { role: 'services' },
        { type: 'separator' },
        { role: 'hide' },
        { role: 'hideothers' },
        { role: 'unhide' },
        { type: 'separator' },
        { role: 'quit' }
      ]
    }] : []),
    {
      label: 'File',
      submenu: [
        process.platform === 'darwin' ? { role: 'close' } : { role: 'quit' }
      ]
    },
    {
      label: 'Edit',
      submenu: [
        { role: 'undo' },
        { role: 'redo' },
        { type: 'separator' },
        { role: 'cut' },
        { role: 'copy' },
        { role: 'paste' },
        ...(process.platform === 'darwin' ? [
          { role: 'pasteandmatchstyle' },
          { role: 'delete' },
          { role: 'selectall' },
          { type: 'separator' },
          {
            label: 'Speech',
            submenu: [
              { role: 'startspeaking' },
              { role: 'stopspeaking' }
            ]
          }
        ] : [
          { role: 'delete' },
          { type: 'separator' },
          { role: 'selectall' }
        ])
      ]
    },
    {
      label: 'View',
      submenu: [
        { role: 'reload' },
        { role: 'forceReload' },
        { role: 'toggleDevTools' },
        { type: 'separator' },
        { role: 'resetZoom' },
        { role: 'zoomIn' },
        { role: 'zoomOut' },
        { type: 'separator' },
        { role: 'togglefullscreen' }
      ]
    },
    {
      label: 'Window',
      submenu: [
        { role: 'minimize' },
        { role: 'close' },
        ...(process.platform === 'darwin' ? [
          { type: 'separator' },
          { role: 'front' },
          { type: 'separator' },
          { role: 'window' }
        ] : [])
      ]
    },
    {
      role: 'help',
      submenu: [
        {
          label: 'About Internet Speed Monitor',
          click: () => {
            const { dialog } = require('electron');
            dialog.showMessageBox(mainWindow, {
              type: 'info',
              title: 'About Internet Speed Monitor',
              message: 'Internet Speed Monitor v1.1.4',
              detail: `A simple tool to monitor your internet connection speed at regular intervals.

Features:
• Automated speed testing at set intervals
• Real-time speed monitoring display
• Historical data visualization with charts
• CSV export functionality
• Debug logging for troubleshooting

Licensed under Unlicense`,
              buttons: ['OK']
            });
          }
        },
        {
          label: 'GitHub Repository',
          click: async () => {
            await shell.openExternal('https://github.com/omrilotan/internet-speed-monitor');
          }
        },
        {
          label: 'Documentation',
          click: async () => {
            await shell.openExternal('https://omrilotan.github.io/internet-speed-monitor/');
          }
        }
      ]
    }
  ];

  const menu = Menu.buildFromTemplate(template);
  Menu.setApplicationMenu(menu);
}

// Send speed test results to renderer
function sendSpeedTestResult(result) {
  if (mainWindow && mainWindow.webContents) {
    mainWindow.webContents.send('speed-test-result', result);
  }
}

// Send speed test started notification to renderer
function sendSpeedTestStarted() {
  if (mainWindow && mainWindow.webContents) {
    mainWindow.webContents.send('speed-test-started');
  }
}

async function initializeModules() {
  try {
    log('Starting module initialization...');
    log('Current working directory: ' + process.cwd());
    log('__dirname: ' + __dirname);
    log('app.isPackaged: ' + app.isPackaged);
    log('process.resourcesPath: ' + process.resourcesPath);
    
    // Initialize data store with error handling
    let DataStore, SpeedMonitor;
    
    try {
      log('Attempting to load DataStore...');
      DataStore = require('./src/dataStore');
      log('DataStore loaded successfully, type: ' + typeof DataStore);
    } catch (dsError) {
      logError('Failed to load DataStore:', dsError);
      throw new Error(`Cannot load DataStore: ${dsError.message}`);
    }
    
    try {
      log('Attempting to load SpeedMonitor...');
      SpeedMonitor = require('./src/speedMonitor');
      log('SpeedMonitor loaded successfully, type: ' + typeof SpeedMonitor);
    } catch (smError) {
      logError('Failed to load SpeedMonitor:', smError);
      throw new Error(`Cannot load SpeedMonitor: ${smError.message}`);
    }
    
    log('Creating DataStore instance...');
    dataStore = new DataStore();
    log('DataStore instance created');
    
    log('Initializing DataStore...');
    await dataStore.initialize();
    log('DataStore initialized');

    log('Creating SpeedMonitor instance...');
    speedMonitor = new SpeedMonitor(dataStore, sendSpeedTestResult, sendSpeedTestStarted);
    log('SpeedMonitor instance created');
    
    isInitialized = true;
    log('Modules initialized successfully');
    return true;
  } catch (error) {
    logError('Failed to initialize modules:', error);
    logError('Error stack:', error);
    logError('Error name: ' + error.name);
    logError('Error message: ' + error.message);
    if (error.code) logError('Error code: ' + error.code);
    isInitialized = false;
    return false;
  }
}

app.whenReady().then(async () => {
  log('=== APP READY ===');
  log('Creating window...');
  createWindow();
  
  // Create application menu with Help section
  createMenu();
  
  log('Starting module initialization...');
  const initSuccess = await initializeModules();
  log('Initialization result: ' + initSuccess);
  
  if (!initSuccess) {
    logError('CRITICAL: Failed to initialize modules on app start');
  }
  
  // Check for updates on app start (respects daily frequency limit)
  try {
    log('Checking for updates...');
    const updateInfo = await checkForUpdates();
    if (updateInfo && updateInfo.latestVersion) {
      log(`Update available: v${updateInfo.latestVersion}`);
      // Send update info to renderer if window is ready
      if (mainWindow) {
        mainWindow.webContents.send('update-available', updateInfo);
      }
    } else {
      log('No updates available');
    }
  } catch (error) {
    logError('Failed to check for updates:', error.message);
  }
  
  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) {
      createWindow();
    }
  });
});

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    if (speedMonitor) {
      speedMonitor.stop();
    }
    app.quit();
  }
});

// IPC handlers - now with proper initialization checks
ipcMain.handle('start-monitoring', async (event, interval) => {
  log('=== START MONITORING REQUESTED ===');
  log('start-monitoring called, isInitialized: ' + isInitialized);
  log('speedMonitor exists: ' + !!speedMonitor);
  log('dataStore exists: ' + !!dataStore);
  
  if (!isInitialized) {
    log('Modules not initialized, attempting to initialize...');
    const success = await initializeModules();
    log('Re-initialization result: ' + success);
    if (!success) {
      logError('CRITICAL: Re-initialization failed');
      return { success: false, error: 'Failed to initialize modules - check console for details' };
    }
  }
  
  if (!speedMonitor) {
    logError('CRITICAL: speedMonitor is null after initialization');
    return { success: false, error: 'Speed monitor is null after initialization' };
  }
  
  if (!isInitialized) {
    logError('CRITICAL: isInitialized is false after initialization attempt');
    return { success: false, error: 'Initialization flag not set properly' };
  }
  
  try {
    console.log('Starting speed monitor with interval:', interval);
    speedMonitor.start(interval);
    return { success: true };
  } catch (error) {
    console.error('Error starting speed monitor:', error);
    return { success: false, error: 'Error starting speed monitor: ' + error.message };
  }
});

ipcMain.handle('stop-monitoring', () => {
  if (speedMonitor && isInitialized) {
    speedMonitor.stop();
    return { success: true };
  }
  return { success: false, error: 'Speed monitor not initialized' };
});

ipcMain.handle('get-historical-data', async (event, limit) => {
  if (!isInitialized) {
    await initializeModules();
  }
  
  if (dataStore && isInitialized) {
    return await dataStore.getSpeedTests(limit);
  }
  return [];
});

ipcMain.handle('get-speed-tests', async (event, limit) => {
  if (!isInitialized) {
    await initializeModules();
  }
  
  if (dataStore && isInitialized) {
    return await dataStore.getSpeedTests(limit);
  }
  return [];
});

ipcMain.handle('get-monitoring-status', async () => {
  if (!isInitialized) {
    await initializeModules();
  }
  
  if (speedMonitor && isInitialized) {
    return speedMonitor.getStatus();
  }
  return { isRunning: false };
});

// Debug IPC handler to get log file contents
ipcMain.handle('get-debug-log', async () => {
  try {
    if (fs.existsSync(logFile)) {
      const logContent = fs.readFileSync(logFile, 'utf8');
      return { success: true, content: logContent };
    } else {
      return { success: false, error: 'Log file does not exist' };
    }
  } catch (error) {
    return { success: false, error: error.message };
  }
});

// Clear debug log IPC handler
ipcMain.handle('clear-debug-log', async () => {
  try {
    if (fs.existsSync(logFile)) {
      fs.writeFileSync(logFile, '');
      log('Debug log cleared by user');
      return { success: true };
    } else {
      return { success: true }; // Consider it successful if file doesn't exist
    }
  } catch (error) {
    log('Error clearing debug log: ' + error.message);
    return { success: false, error: error.message };
  }
});

// Clear history IPC handler
ipcMain.handle('clear-history', async () => {
  try {
    if (!isInitialized || !dataStore) {
      const initSuccess = await initializeModules();
      if (!initSuccess) {
        return { success: false, error: 'Failed to initialize data store' };
      }
    }
    
    const success = await dataStore.clearAllData();
    return { success, error: success ? null : 'Failed to clear data' };
  } catch (error) {
    logError('Error clearing history:', error);
    return { success: false, error: error.message };
  }
});

// Export CSV IPC handler
ipcMain.handle('export-csv', async () => {
  try {
    if (!isInitialized || !dataStore) {
      const initSuccess = await initializeModules();
      if (!initSuccess) {
        return { success: false, error: 'Failed to initialize data store' };
      }
    }
    
    log('Getting data for CSV export...');
    // Get all speed tests (no limit)
    const speedTests = await dataStore.getSpeedTests(999999);
    log('Speed tests found: ' + speedTests.length);
    
    const csvData = dataStore.exportAsCSV();
    if (csvData && csvData.length > 0) {
      // Use Electron's dialog to save the file
      const { dialog } = require('electron');
      const result = await dialog.showSaveDialog(mainWindow, {
        title: 'Export Speed Test Data',
        defaultPath: `speed-test-data-${new Date().toISOString().split('T')[0]}.csv`,
        filters: [
          { name: 'CSV Files', extensions: ['csv'] },
          { name: 'All Files', extensions: ['*'] }
        ]
      });
      
      if (!result.canceled && result.filePath) {
        fs.writeFileSync(result.filePath, csvData);
        if (speedTests.length === 0) {
          return { success: true, filePath: result.filePath, message: 'Empty CSV file exported (no speed tests recorded yet)' };
        } else {
          return { success: true, filePath: result.filePath, message: `Exported ${speedTests.length} speed test records` };
        }
      } else {
        return { success: false, error: 'Export cancelled' };
      }
    } else {
      return { success: false, error: 'Failed to generate CSV data' };
    }
  } catch (error) {
    logError('Error exporting CSV:', error);
    return { success: false, error: error.message };
  }
});

// Test once now handler
ipcMain.handle('test-once-now', async () => {
  log('Manual test requested via IPC');
  try {
    if (!speedMonitor) {
      // If monitoring isn't started, create a temporary speed monitor instance
      const sendResultCallback = (result) => {
        if (mainWindow && !mainWindow.isDestroyed()) {
          mainWindow.webContents.send('speed-test-result', result);
        }
      };
      
      const tempSpeedMonitor = new SpeedMonitor(dataStore, sendResultCallback);
      await tempSpeedMonitor.runSpeedTest();
      return { success: true, message: 'Manual test completed' };
    } else {
      // Use existing monitor instance
      await speedMonitor.runSpeedTest();
      return { success: true, message: 'Manual test completed' };
    }
  } catch (error) {
    logError('Error running manual test:', error);
    return { success: false, error: error.message };
  }
});

// Handle opening external links
ipcMain.handle('open-external', async (event, url) => {
  try {
    await shell.openExternal(url);
    return { success: true };
  } catch (error) {
    logError('Error opening external link:', error);
    return { success: false, error: error.message };
  }
});

// Handle version checking
ipcMain.handle('check-for-updates', async () => {
  try {
    const updateInfo = await checkForUpdates();
    saveLastCheckDate();
    return updateInfo;
  } catch (error) {
    logError('Error in check-for-updates handler:', error);
    return null;
  }
});

ipcMain.handle('get-current-version', async () => {
  return CURRENT_VERSION;
});
