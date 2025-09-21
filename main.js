const { app, BrowserWindow, ipcMain } = require('electron');
const path = require('path');
const fs = require('fs');

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
      nodeIntegration: true,
      contextIsolation: false
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

// Send speed test results to renderer
function sendSpeedTestResult(result) {
  if (mainWindow && mainWindow.webContents) {
    mainWindow.webContents.send('speed-test-result', result);
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
    speedMonitor = new SpeedMonitor(dataStore, sendSpeedTestResult);
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
  
  log('Starting module initialization...');
  const initSuccess = await initializeModules();
  log('Initialization result: ' + initSuccess);
  
  if (!initSuccess) {
    logError('CRITICAL: Failed to initialize modules on app start');
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
    
    const csvData = dataStore.exportAsCSV();
    if (csvData) {
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
        return { success: true, filePath: result.filePath };
      } else {
        return { success: false, error: 'Export cancelled' };
      }
    } else {
      return { success: false, error: 'No data to export' };
    }
  } catch (error) {
    logError('Error exporting CSV:', error);
    return { success: false, error: error.message };
  }
});
