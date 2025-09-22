const { contextBridge, ipcRenderer } = require('electron');

// Expose protected methods that allow the renderer process to use
// the ipcRenderer without exposing the entire object
contextBridge.exposeInMainWorld('electronAPI', {
  // Speed monitoring
  startMonitoring: (interval) => ipcRenderer.invoke('start-monitoring', interval),
  stopMonitoring: () => ipcRenderer.invoke('stop-monitoring'),
  getMonitoringStatus: () => ipcRenderer.invoke('get-monitoring-status'),
  
  // Data management
  getSpeedTests: (limit) => ipcRenderer.invoke('get-speed-tests', limit),
  getHistoricalData: (limit) => ipcRenderer.invoke('get-historical-data', limit),
  clearHistory: () => ipcRenderer.invoke('clear-history'),
  exportCSV: () => ipcRenderer.invoke('export-csv'),
  
  // Debug
  getDebugLog: () => ipcRenderer.invoke('get-debug-log'),
  
  // Events
  onSpeedTestResult: (callback) => ipcRenderer.on('speed-test-result', callback),
  onMonitoringStatus: (callback) => ipcRenderer.on('monitoring-status', callback),
  
  // Remove listeners
  removeAllListeners: (channel) => ipcRenderer.removeAllListeners(channel)
});
