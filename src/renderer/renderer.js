console.log('=== RENDERER.JS LOADED ===');

let speedChart;
// Global variables for next test tracking
let isMonitoring = false;
let nextTestTimer = null;
let nextTestTimestamp = null;
let allSpeedTests = []; // Store all tests for median calculation

// DOM elements
const startBtn = document.getElementById('start-btn');
const stopBtn = document.getElementById('stop-btn');
const testNowBtn = document.getElementById('test-now-btn');
const debugBtn = document.getElementById('debug-btn');
const clearDebugBtn = document.getElementById('clear-debug-btn');
const clearHistoryBtn = document.getElementById('clear-history-btn');
const exportCsvBtn = document.getElementById('export-csv-btn');
const intervalInput = document.getElementById('interval');
const statusText = document.getElementById('status-text');
const statusDot = document.getElementById('status-dot');
const downloadSpeed = document.getElementById('download-speed');
const uploadSpeed = document.getElementById('upload-speed');
const ping = document.getElementById('ping');
const lastTest = document.getElementById('last-test');
const testRunning = document.getElementById('test-running');
const nextTest = document.getElementById('next-test');
const nextTestTime = document.getElementById('next-test-time');
const medianDownload = document.getElementById('median-download');
const medianUpload = document.getElementById('median-upload');
const medianPing = document.getElementById('median-ping');
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
testNowBtn.addEventListener('click', testOnceNow);
debugBtn.addEventListener('click', showDebugLog);
clearDebugBtn.addEventListener('click', clearDebugLog);
clearHistoryBtn.addEventListener('click', clearHistory);
exportCsvBtn.addEventListener('click', exportCSV);

// Footer links
document.addEventListener('DOMContentLoaded', () => {
    document.getElementById('github-link').addEventListener('click', (e) => {
        e.preventDefault();
        window.electronAPI.openExternal('https://github.com/omrilotan/internet-speed-monitor');
    });
    
    document.getElementById('website-link').addEventListener('click', (e) => {
        e.preventDefault();
        window.electronAPI.openExternal('https://omrilotan.github.io/internet-speed-monitor/');
    });
});

// Initialize the app

// IPC listeners using electronAPI
window.electronAPI.onSpeedTestStarted((event) => {
    showTestRunningIndicator(); // Show the running indicator when any test starts
    // Show "Running" status when test starts
    if (isMonitoring) {
        showStatus('Running', 'running');
    } else {
        showStatus('Testing...', 'running');
    }
});

window.electronAPI.onSpeedTestResult((event, result) => {
    hideTestRunningIndicator(); // Hide the running indicator when test completes
    updateCurrentStatsEnhanced(result);
    addToTable(result);
    updateChart(result);
});

// Functions to manage test running indicator
function showTestRunningIndicator() {
    if (testRunning) {
        testRunning.style.display = 'block';
        // Hide next test info when showing test running
        if (nextTest) {
            nextTest.style.display = 'none';
        }
    }
}

function hideTestRunningIndicator() {
    if (testRunning) {
        testRunning.style.display = 'none';
        // Only show next test info if monitoring is active and there's a valid timestamp
        // Let updateNextTestDisplay handle the actual display logic
        if (isMonitoring && nextTestTimestamp) {
            updateNextTestDisplay();
        }
    }
}

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
        
        console.log('Invoking start-monitoring via electronAPI...');
        const response = await window.electronAPI.startMonitoring(interval);
        console.log('API response received:', response);
        
        if (response.success) {
            console.log('Monitoring started successfully');
            isMonitoring = true;
            updateUI();
            showStatus('Starting monitoring...', 'running');
            startNextTestTimer(interval);
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
        const response = await window.electronAPI.stopMonitoring();
        if (response.success) {
            isMonitoring = false;
            updateUI();
            showStatus('Stopped', 'stopped');
            stopNextTestTimer();
        } else {
            alert('Failed to stop monitoring: ' + response.error);
        }
    } catch (error) {
        console.error('Error stopping monitoring:', error);
        alert('Failed to stop monitoring');
    }
}

async function testOnceNow() {
    console.log('=== TEST ONCE NOW CLICKED ===');
    
    try {
        // Disable button to prevent multiple clicks
        testNowBtn.disabled = true;
        testNowBtn.textContent = 'Testing...';
        
        // Show the test running indicator
        showTestRunningIndicator();
        
        // Show temporary testing status if not monitoring
        const wasShowingStatus = !isMonitoring;
        if (wasShowingStatus) {
            showStatus('Testing...', 'running');
        }
        
        console.log('Invoking test-once-now via electronAPI...');
        const response = await window.electronAPI.testOnceNow();
        console.log('Test once response:', response);
        
        if (response.success) {
            console.log('Manual test completed successfully');
            // The result will come through the speed-test-result event
        } else {
            console.error('Manual test failed:', response.error);
            alert('Failed to run test: ' + response.error);
            
            // Hide test running indicator on failure
            hideTestRunningIndicator();
            
            // Restore status if we were showing testing status
            if (wasShowingStatus) {
                showStatus('Stopped', 'stopped');
            }
        }
    } catch (error) {
        console.error('Error running manual test:', error);
        alert('Failed to run test: ' + error.message);
        
        // Hide test running indicator on error
        hideTestRunningIndicator();
        
        // Restore status if not monitoring
        if (!isMonitoring) {
            showStatus('Stopped', 'stopped');
        }
    } finally {
        // Re-enable button
        testNowBtn.disabled = false;
        testNowBtn.textContent = 'Test Once Now';
    }
}

async function showDebugLog() {
    try {
        console.log('Fetching debug log...');
        const response = await window.electronAPI.getDebugLog();
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

async function clearDebugLog() {
    try {
        console.log('Clearing debug log...');
        const response = await window.electronAPI.clearDebugLog();
        if (response.success) {
            alert('Debug log cleared successfully');
        } else {
            alert('Failed to clear debug log: ' + response.error);
        }
    } catch (error) {
        console.error('Error clearing debug log:', error);
        alert('Failed to clear debug log: ' + error.message);
    }
}

async function clearHistory() {
    try {
        // Confirm action with user
        const confirmed = confirm('Are you sure you want to clear all speed test history? This action cannot be undone.');
        if (!confirmed) {
            return;
        }

        console.log('Clearing history...');
        const response = await window.electronAPI.clearHistory();
        if (response.success) {
            // Reset all data arrays
            allSpeedTests = [];
            
            // Clear current stats display
            document.getElementById('download-speed').textContent = '-- Mbps';
            document.getElementById('upload-speed').textContent = '-- Mbps';
            document.getElementById('ping').textContent = '-- ms';
            document.getElementById('last-test').textContent = 'Never';
            
            // Hide test running indicator
            hideTestRunningIndicator();
            
            // Clear median stats
            updateMedianStats();
            
            // Clear table
            resultsTable.innerHTML = '';
            
            // Clear and reset chart
            if (speedChart) {
                speedChart.data.labels = [];
                speedChart.data.datasets[0].data = [];
                speedChart.data.datasets[1].data = [];
                speedChart.update();
            }
            
            // Refresh the UI to show empty data
            await loadHistoricalData();
            
            alert('History cleared successfully!');
        } else {
            alert('Failed to clear history: ' + response.error);
        }
    } catch (error) {
        console.error('Error clearing history:', error);
        alert('Failed to clear history: ' + error.message);
    }
}

async function exportCSV() {
    try {
        console.log('Exporting CSV...');
        const response = await window.electronAPI.exportCSV();
        if (response.success) {
            const message = response.message || 'Data exported successfully';
            alert(message + '\nFile saved to: ' + response.filePath);
        } else {
            alert('Failed to export data: ' + response.error);
        }
    } catch (error) {
        console.error('Error exporting CSV:', error);
        alert('Failed to export data: ' + error.message);
    }
}

async function updateMonitoringStatus() {
    try {
        // Add retry logic for initialization
        let attempts = 0;
        let status;
        
        while (attempts < 3) {
            try {
                status = await window.electronAPI.getMonitoringStatus();
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
        showStatus(isMonitoring ? 'Sleeping' : 'Stopped', isMonitoring ? 'sleeping' : 'stopped');
        
        // If monitoring is active, start the next test timer
        if (isMonitoring) {
            const interval = parseInt(intervalInput.value) || 5;
            startNextTestTimer(interval);
        }
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
    // Use 24-hour format for last test time
    lastTest.textContent = new Date(result.timestamp).toLocaleTimeString([], { 
        hour: '2-digit', 
        minute: '2-digit', 
        second: '2-digit',
        hour12: false 
    });
    
    // Show "Sleeping" status if monitoring is active but test just completed
    if (isMonitoring) {
        showStatus('Sleeping', 'sleeping');
    } else {
        showStatus('Stopped', 'stopped');
    }
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
        const data = await window.electronAPI.getSpeedTests(100); // Get more data for median calculation
        
        // Store all data for median calculation
        allSpeedTests = [...data];
        
        // Clear existing table data
        resultsTable.innerHTML = '';
        
        // Reset current stats display if no data
        if (!data || data.length === 0) {
            document.getElementById('download-speed').textContent = '-- Mbps';
            document.getElementById('upload-speed').textContent = '-- Mbps';
            document.getElementById('ping').textContent = '-- ms';
            document.getElementById('last-test').textContent = 'Never';
            
            // Hide median stats if no data
            updateMedianStats();
            return;
        }
        
        // Add data to table (reverse to show newest first, but limit to 10)
        const recentData = data.slice(-10).reverse();
        recentData.forEach(result => {
            addToTable(result);
        });
        
        // Add to chart (keep original order for chronological chart, limit to 20)
        const chartData = data.slice(-20);
        chartData.forEach(result => {
            updateChart(result);
        });
        
        // Update median stats
        updateMedianStats();
        
        // Update the "Last Test" display with the most recent test (24-hour format)
        if (data.length > 0) {
            const mostRecentTest = data[data.length - 1];
            lastTest.textContent = new Date(mostRecentTest.timestamp).toLocaleTimeString([], { 
                hour: '2-digit', 
                minute: '2-digit', 
                second: '2-digit',
                hour12: false 
            });
            
            // Also update current stats with most recent data
            downloadSpeed.textContent = `${mostRecentTest.download.toFixed(2)} Mbps`;
            uploadSpeed.textContent = `${mostRecentTest.upload.toFixed(2)} Mbps`;
            ping.textContent = `${mostRecentTest.ping.toFixed(2)} ms`;
        }
        
    } catch (error) {
        console.error('Error loading historical data:', error);
    }
}

// Next test timer functions
function startNextTestTimer(intervalMinutes) {
    stopNextTestTimer(); // Clear any existing timer
    
    // Calculate next test time
    nextTestTimestamp = new Date(Date.now() + intervalMinutes * 60 * 1000);
    
    // Show next test info
    nextTest.style.display = 'block';
    updateNextTestDisplay();
    
    // Update display every 30 seconds (less frequent since we show actual time, not countdown)
    nextTestTimer = setInterval(() => {
        updateNextTestDisplay();
    }, 30000); // 30 seconds instead of 1 second
}

function stopNextTestTimer() {
    if (nextTestTimer) {
        clearInterval(nextTestTimer);
        nextTestTimer = null;
    }
    nextTest.style.display = 'none';
    nextTestTimestamp = null;
}

function updateNextTestDisplay() {
    if (!nextTestTimestamp || !isMonitoring) {
        nextTest.style.display = 'none';
        return;
    }
    
    // Don't show next test display if test running indicator is visible
    if (testRunning && testRunning.style.display === 'block') {
        nextTest.style.display = 'none';
        return;
    }
    
    const now = Date.now();
    const timeLeft = nextTestTimestamp.getTime() - now;
    
    if (timeLeft <= 0) {
        // Test should have run, update for next cycle
        const intervalInput = document.getElementById('interval');
        const interval = parseInt(intervalInput.value) || 5;
        nextTestTimestamp = new Date(Date.now() + interval * 60 * 1000);
    }
    
    // Display the actual time when the test will run (24-hour format)
    const nextTestTime_element = nextTestTime;
    const timeString = nextTestTimestamp.toLocaleTimeString([], { 
        hour: '2-digit', 
        minute: '2-digit',
        hour12: false 
    });
    
    nextTestTime_element.textContent = timeString;
    
    // Show the next test display (it will only show if not blocked by earlier checks)
    nextTest.style.display = 'block';
}

// Median calculation functions
function calculateMedian(values) {
    if (values.length === 0) return 0;
    
    const sorted = [...values].sort((a, b) => a - b);
    const mid = Math.floor(sorted.length / 2);
    
    if (sorted.length % 2 === 0) {
        return (sorted[mid - 1] + sorted[mid]) / 2;
    } else {
        return sorted[mid];
    }
}

function updateMedianStats() {
    if (allSpeedTests.length === 0) {
        medianDownload.textContent = '-- Mbps';
        medianUpload.textContent = '-- Mbps';
        medianPing.textContent = '-- ms';
        return;
    }
    
    const downloadSpeeds = allSpeedTests.map(test => test.download);
    const uploadSpeeds = allSpeedTests.map(test => test.upload);
    const pings = allSpeedTests.map(test => test.ping);
    
    const medianDownloadSpeed = calculateMedian(downloadSpeeds);
    const medianUploadSpeed = calculateMedian(uploadSpeeds);
    const medianPingValue = calculateMedian(pings);
    
    medianDownload.textContent = `${medianDownloadSpeed.toFixed(2)} Mbps`;
    medianUpload.textContent = `${medianUploadSpeed.toFixed(2)} Mbps`;
    medianPing.textContent = `${medianPingValue.toFixed(2)} ms`;
}

// Update the existing updateCurrentStats function to also update medians
function updateCurrentStatsEnhanced(result) {
    updateCurrentStats(result);
    
    // Add to all tests array for median calculation
    allSpeedTests.push(result);
    
    // Keep only last 100 tests for median calculation
    if (allSpeedTests.length > 100) {
        allSpeedTests = allSpeedTests.slice(-100);
    }
    
    updateMedianStats();
    
    // Reset next test timer if monitoring is active
    if (isMonitoring) {
        const intervalInput = document.getElementById('interval');
        const interval = parseInt(intervalInput.value) || 5;
        startNextTestTimer(interval);
    }
}

// Override the existing speed test result handler
// (This is already handled above in the IPC listeners section)
