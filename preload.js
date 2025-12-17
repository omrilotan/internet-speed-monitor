const { contextBridge, ipcRenderer } = require('electron');

// Expose protected methods that allow the renderer process to use
// the ipcRenderer without exposing the entire object
contextBridge.exposeInMainWorld('electronAPI', {
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
  
  // External links
  openExternal: (url) => ipcRenderer.invoke('open-external', url),
  
  // Version checking
  checkForUpdates: () => ipcRenderer.invoke('check-for-updates'),
  getCurrentVersion: () => ipcRenderer.invoke('get-current-version'),
  getNextTestTime: (schedule) => ipcRenderer.invoke('get-next-test-time', schedule),
  
  // Settings management
  loadSettings: () => ipcRenderer.invoke('load-settings'),
  saveSettings: (settings) => ipcRenderer.invoke('save-settings', settings),
  setLaunchAtStartup: (enable) => ipcRenderer.invoke('set-launch-at-startup', enable),
  getLaunchAtStartup: () => ipcRenderer.invoke('get-launch-at-startup'),
  
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
