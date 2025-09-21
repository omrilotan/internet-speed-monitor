const { ipcRenderer } = require('electron');

console.log('=== RENDERER.JS LOADED ===');

let speedChart;
let isMonitoring = false;

// DOM elements
const startBtn = document.getElementById('start-btn');
const stopBtn = document.getElementById('stop-btn');
const debugBtn = document.getElementById('debug-btn');
const intervalInput = document.getElementById('interval');
const statusText = document.getElementById('status-text');
const statusDot = document.getElementById('status-dot');
const downloadSpeed = document.getElementById('download-speed');
const uploadSpeed = document.getElementById('upload-speed');
const ping = document.getElementById('ping');
const lastTest = document.getElementById('last-test');
const resultsTable = document.getElementById('results-tbody');

// Initialize the app
document.addEventListener('DOMContentLoaded', () => {
    console.log('=== DOM CONTENT LOADED ===');
    initializeChart();
    loadHistoricalData();
    updateMonitoringStatus();
});

// Event listeners
startBtn.addEventListener('click', startMonitoring);
stopBtn.addEventListener('click', stopMonitoring);
debugBtn.addEventListener('click', showDebugLog);

// IPC listeners
ipcRenderer.on('speed-test-result', (event, result) => {
    updateCurrentStats(result);
    addToTable(result);
    updateChart(result);
});

async function startMonitoring() {
    console.log('=== START MONITORING CLICKED ===');
    const interval = parseInt(intervalInput.value);
    console.log('Requested interval:', interval);
    
    if (interval < 1 || interval > 60) {
        console.error('Invalid interval:', interval);
        alert('Please enter a valid interval between 1 and 60 minutes.');
        return;
    }

    try {
        console.log('Setting status to initializing...');
        showStatus('Initializing...', 'running');
        
        console.log('Adding delay before IPC call...');
        // Add a small delay to ensure main process is ready
        await new Promise(resolve => setTimeout(resolve, 1000));
        
        console.log('Invoking start-monitoring IPC...');
        const response = await ipcRenderer.invoke('start-monitoring', interval);
        console.log('IPC response received:', response);
        
        if (response.success) {
            console.log('Monitoring started successfully');
            isMonitoring = true;
            updateUI();
            showStatus('Starting monitoring...', 'running');
        } else {
            console.error('Monitoring failed to start:', response.error);
            showStatus('Stopped', 'stopped');
            alert('Failed to start monitoring: ' + response.error);
        }
    } catch (error) {
        console.error('Error starting monitoring:', error);
        console.error('Error stack:', error.stack);
        showStatus('Error', 'stopped');
        alert('Failed to start monitoring: ' + error.message);
    }
}

async function stopMonitoring() {
    try {
        const response = await ipcRenderer.invoke('stop-monitoring');
        if (response.success) {
            isMonitoring = false;
            updateUI();
            showStatus('Stopped', 'stopped');
        } else {
            alert('Failed to stop monitoring: ' + response.error);
        }
    } catch (error) {
        console.error('Error stopping monitoring:', error);
        alert('Failed to stop monitoring');
    }
}

async function showDebugLog() {
    try {
        console.log('Fetching debug log...');
        const response = await ipcRenderer.invoke('get-debug-log');
        if (response.success) {
            // Create a popup window to show the log
            const popup = window.open('', '_blank', 'width=800,height=600,scrollbars=yes');
            popup.document.write('<html><head><title>Debug Log</title></head><body>');
            popup.document.write('<h1>Debug Log</h1>');
            popup.document.write('<pre style="font-family: monospace; white-space: pre-wrap;">' + 
                                response.content.replace(/</g, '&lt;').replace(/>/g, '&gt;') + '</pre>');
            popup.document.write('</body></html>');
            popup.document.close();
        } else {
            alert('Failed to get debug log: ' + response.error);
        }
    } catch (error) {
        console.error('Error getting debug log:', error);
        alert('Failed to get debug log: ' + error.message);
    }
}

async function updateMonitoringStatus() {
    try {
        // Add retry logic for initialization
        let attempts = 0;
        let status;
        
        while (attempts < 3) {
            try {
                status = await ipcRenderer.invoke('get-monitoring-status');
                break;
            } catch (error) {
                attempts++;
                if (attempts < 3) {
                    await new Promise(resolve => setTimeout(resolve, 1000));
                } else {
                    throw error;
                }
            }
        }
        
        isMonitoring = status.isRunning;
        updateUI();
        showStatus(isMonitoring ? 'Running' : 'Stopped', isMonitoring ? 'running' : 'stopped');
    } catch (error) {
        console.error('Error getting monitoring status:', error);
        showStatus('Error', 'stopped');
    }
}

function updateUI() {
    startBtn.disabled = isMonitoring;
    stopBtn.disabled = !isMonitoring;
    intervalInput.disabled = isMonitoring;
}

function showStatus(text, type) {
    statusText.textContent = text;
    statusDot.className = `status-dot ${type}`;
}

function updateCurrentStats(result) {
    downloadSpeed.textContent = `${result.download.toFixed(2)} Mbps`;
    uploadSpeed.textContent = `${result.upload.toFixed(2)} Mbps`;
    ping.textContent = `${result.ping.toFixed(2)} ms`;
    lastTest.textContent = new Date(result.timestamp).toLocaleTimeString();
    
    showStatus('Running', 'running');
}

function addToTable(result) {
    const row = document.createElement('tr');
    row.innerHTML = `
        <td>${new Date(result.timestamp).toLocaleString()}</td>
        <td>${result.download.toFixed(2)}</td>
        <td>${result.upload.toFixed(2)}</td>
        <td>${result.ping.toFixed(2)}</td>
        <td>${result.server || 'Unknown'}</td>
    `;
    
    resultsTable.insertBefore(row, resultsTable.firstChild);
    
    // Keep only the last 50 rows
    while (resultsTable.children.length > 50) {
        resultsTable.removeChild(resultsTable.lastChild);
    }
}

function initializeChart() {
    const ctx = document.getElementById('speedChart').getContext('2d');
    speedChart = new Chart(ctx, {
        type: 'line',
        data: {
            labels: [],
            datasets: [
                {
                    label: 'Download Speed (Mbps)',
                    data: [],
                    borderColor: '#3498db',
                    backgroundColor: 'rgba(52, 152, 219, 0.1)',
                    tension: 0.4
                },
                {
                    label: 'Upload Speed (Mbps)',
                    data: [],
                    borderColor: '#e74c3c',
                    backgroundColor: 'rgba(231, 76, 60, 0.1)',
                    tension: 0.4
                }
            ]
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            scales: {
                y: {
                    beginAtZero: true,
                    title: {
                        display: true,
                        text: 'Speed (Mbps)'
                    }
                },
                x: {
                    title: {
                        display: true,
                        text: 'Time'
                    }
                }
            },
            plugins: {
                legend: {
                    display: true
                }
            }
        }
    });
}

function updateChart(result) {
    const time = new Date(result.timestamp).toLocaleTimeString();
    
    speedChart.data.labels.push(time);
    speedChart.data.datasets[0].data.push(result.download);
    speedChart.data.datasets[1].data.push(result.upload);
    
    // Keep only the last 20 data points
    if (speedChart.data.labels.length > 20) {
        speedChart.data.labels.shift();
        speedChart.data.datasets[0].data.shift();
        speedChart.data.datasets[1].data.shift();
    }
    
    speedChart.update();
}

async function loadHistoricalData() {
    try {
        const data = await ipcRenderer.invoke('get-historical-data', 10);
        
        // Clear existing table data
        resultsTable.innerHTML = '';
        
        // Add data to table (reverse to show newest first)
        data.reverse().forEach(result => {
            addToTable(result);
        });
        
        // Add to chart (keep original order for chronological chart)
        data.reverse().forEach(result => {
            updateChart(result);
        });
        
    } catch (error) {
        console.error('Error loading historical data:', error);
    }
}
