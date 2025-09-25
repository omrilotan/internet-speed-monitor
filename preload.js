const { contextBridge, ipcRenderer } = require('electron');

// Expose protected methods that allow the renderer process to use
// the ipcRenderer without exposing the entire object
contextBridge.exposeInMainWorld('electronAPI', {
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
  
  // External links
  openExternal: (url) => ipcRenderer.invoke('open-external', url),
  
  // Version checking
  checkForUpdates: () => ipcRenderer.invoke('check-for-updates'),
  getCurrentVersion: () => ipcRenderer.invoke('get-current-version'),
  
  // Events
  onSpeedTestResult: (callback) => ipcRenderer.on('speed-test-result', callback),
  onSpeedTestStarted: (callback) => ipcRenderer.on('speed-test-started', callback),
  onMonitoringStatus: (callback) => ipcRenderer.on('monitoring-status', callback),
  
  // Remove listeners
  removeAllListeners: (channel) => ipcRenderer.removeAllListeners(channel),
  
  // Direct ipcRenderer access for update notifications
  ipcRenderer: {
    on: (channel, callback) => ipcRenderer.on(channel, callback),
    removeAllListeners: (channel) => ipcRenderer.removeAllListeners(channel)
  }
});
