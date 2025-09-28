console.log('=== RENDERER.JS LOADED ===');

let speedChart;
// Global variables for next test tracking
let isMonitoring = false;
let nextTestTimer = null;
let nextTestTimestamp = null;
let allSpeedTests = []; // Store all tests for median calculation
let lastChartDate = null; // Track last date shown on chart for date labeling

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
const updateNotification = document.getElementById('update-notification');
const downloadUpdateBtn = document.getElementById('download-update-btn');
const dismissUpdateBtn = document.getElementById('dismiss-update-btn');
const updateMessage = document.getElementById('update-message');
const appVersion = document.getElementById('app-version');

// Initialize the app
document.addEventListener('DOMContentLoaded', () => {
    console.log('=== DOM CONTENT LOADED ===');
    initializeChart();
    loadHistoricalData();
    updateMonitoringStatus();
    loadAppVersion();
    setupIntervalStepping(); // Setup smart stepping for interval input
});

// Event listeners
startBtn.addEventListener('click', startMonitoring);
stopBtn.addEventListener('click', stopMonitoring);
testNowBtn.addEventListener('click', testOnceNow);
debugBtn.addEventListener('click', showDebugLog);
clearDebugBtn.addEventListener('click', clearDebugLog);
clearHistoryBtn.addEventListener('click', clearHistory);
exportCsvBtn.addEventListener('click', exportCSV);

// Update notification event listeners
dismissUpdateBtn.addEventListener('click', () => {
    updateNotification.style.display = 'none';
});

downloadUpdateBtn.addEventListener('click', () => {
    window.electronAPI.openExternal('https://github.com/omrilotan/internet-speed-monitor/releases/latest');
});

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

// Listen for update notifications
window.electronAPI.ipcRenderer.on('update-available', (event, updateInfo) => {
    showUpdateNotification(updateInfo);
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
    const timestamp = new Date(result.timestamp);
    const currentDate = timestamp.toDateString(); // "Mon Sep 25 2025"
    const time = timestamp.toLocaleTimeString([], { 
        hour: '2-digit', 
        minute: '2-digit',
        hour12: false 
    });
    
    // Check if this is the first measurement of a new day
    let label;
    if (lastChartDate !== currentDate) {
        // Include date for first measurement of a new day
        label = `${timestamp.toLocaleDateString([], { 
            month: 'short', 
            day: 'numeric' 
        })} ${time}`;
        lastChartDate = currentDate;
    } else {
        // Just time for subsequent measurements of the same day
        label = time;
    }
    
    speedChart.data.labels.push(label);
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
        
        // Reset chart date tracking when loading historical data
        lastChartDate = null;
        
        // Clear existing chart data
        speedChart.data.labels = [];
        speedChart.data.datasets[0].data = [];
        speedChart.data.datasets[1].data = [];
        
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
        
        // Add to chart (reverse for chronological order - oldest to newest, limit to 20)
        const chartData = data.slice(-20).reverse();
        chartData.forEach(result => {
            updateChart(result);
        });
        
        // Update median stats
        updateMedianStats();
        
        // Update the "Last Test" display with the most recent test (24-hour format)
        if (data.length > 0) {
            const mostRecentTest = data[0]; // First item in descending order is most recent
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

// Smart stepping for interval input
function setupIntervalStepping() {
    const intervalInput = document.getElementById('interval');
    const roundToNearest = document.getElementById('round-to-nearest');
    
    // Define the step sequence
    const stepSequence = [1, 5, 10, 15, 30, 60];
    
    // Function to suggest appropriate interval for rounding
    function suggestRoundingInterval(currentValue) {
        if (!roundToNearest.checked) return currentValue;
        
        // Find the closest value in step sequence for better rounding behavior
        let closest = stepSequence[0];
        let minDiff = Math.abs(currentValue - closest);
        
        for (let i = 1; i < stepSequence.length; i++) {
            const diff = Math.abs(currentValue - stepSequence[i]);
            if (diff < minDiff) {
                minDiff = diff;
                closest = stepSequence[i];
            }
        }
        
        return closest;
    }
    
    // Update input value when rounding checkbox changes
    roundToNearest.addEventListener('change', () => {
        if (roundToNearest.checked) {
            const currentValue = parseInt(intervalInput.value) || 5;
            const suggestedValue = suggestRoundingInterval(currentValue);
            if (suggestedValue !== currentValue) {
                intervalInput.value = suggestedValue;
                intervalInput.dispatchEvent(new Event('input'));
            }
        }
    });
    
    // Handle arrow key presses and wheel events
    intervalInput.addEventListener('keydown', (e) => {
        if (e.key === 'ArrowUp' || e.key === 'ArrowDown') {
            e.preventDefault();
            const currentValue = parseInt(intervalInput.value) || 1;
            const isIncrement = e.key === 'ArrowUp';
            const newValue = getNextStepValue(currentValue, isIncrement, stepSequence);
            intervalInput.value = newValue;
            intervalInput.dispatchEvent(new Event('input'));
        }
    });
    
    // Handle mouse wheel
    intervalInput.addEventListener('wheel', (e) => {
        if (intervalInput === document.activeElement) {
            e.preventDefault();
            const currentValue = parseInt(intervalInput.value) || 1;
            const isIncrement = e.deltaY < 0; // Scroll up = increment
            const newValue = getNextStepValue(currentValue, isIncrement, stepSequence);
            intervalInput.value = newValue;
            intervalInput.dispatchEvent(new Event('input'));
        }
    });
}

// Get the next value in the step sequence
function getNextStepValue(currentValue, isIncrement, stepSequence) {
    // Find current position in sequence
    let currentIndex = -1;
    for (let i = 0; i < stepSequence.length; i++) {
        if (stepSequence[i] === currentValue) {
            currentIndex = i;
            break;
        }
    }
    
    if (currentIndex !== -1) {
        // Current value is in sequence, move to next/previous
        if (isIncrement) {
            return currentIndex < stepSequence.length - 1 ? stepSequence[currentIndex + 1] : stepSequence[currentIndex];
        } else {
            return currentIndex > 0 ? stepSequence[currentIndex - 1] : stepSequence[currentIndex];
        }
    } else {
        // Current value not in sequence, find closest
        if (isIncrement) {
            // Find next higher value in sequence
            for (let i = 0; i < stepSequence.length; i++) {
                if (stepSequence[i] > currentValue) {
                    return stepSequence[i];
                }
            }
            return stepSequence[stepSequence.length - 1]; // Return max if no higher value
        } else {
            // Find next lower value in sequence
            for (let i = stepSequence.length - 1; i >= 0; i--) {
                if (stepSequence[i] < currentValue) {
                    return stepSequence[i];
                }
            }
            return stepSequence[0]; // Return min if no lower value
        }
    }
}

// Rounding helper function
function getRoundedNextTestTime(intervalMinutes) {
    const roundToNearest = document.getElementById('round-to-nearest').checked;
    const intervalInput = document.getElementById('interval');
    
    if (!roundToNearest) {
        return new Date(Date.now() + intervalMinutes * 60 * 1000);
    }
    
    // Suggest better interval values for rounding if current value isn't optimal
    const stepSequence = [1, 5, 10, 15, 30, 60];
    const currentInputValue = parseInt(intervalInput.value) || intervalMinutes;
    
    // If the current interval isn't in our step sequence, suggest the closest one
    if (!stepSequence.includes(currentInputValue) && currentInputValue === intervalMinutes) {
        let closest = stepSequence[0];
        let minDiff = Math.abs(currentInputValue - closest);
        
        for (let i = 1; i < stepSequence.length; i++) {
            const diff = Math.abs(currentInputValue - stepSequence[i]);
            if (diff < minDiff) {
                minDiff = diff;
                closest = stepSequence[i];
            }
        }
        
        // Update the input value to the suggested one
        if (closest !== currentInputValue) {
            intervalInput.value = closest;
            intervalMinutes = closest; // Use the new value for calculation
        }
    }
    
    const now = new Date();
    const nextTest = new Date(now.getTime() + intervalMinutes * 60 * 1000);
    
    // Define rounding rules based on interval
    if (intervalMinutes >= 60) {
        // Round to nearest hour
        nextTest.setMinutes(0, 0, 0);
        if (now.getMinutes() >= 30) {
            nextTest.setHours(nextTest.getHours() + 1);
        }
    } else if (intervalMinutes >= 30) {
        // Round to nearest half hour (00 or 30 minutes)
        const minutes = nextTest.getMinutes();
        if (minutes < 15) {
            nextTest.setMinutes(0, 0, 0);
        } else if (minutes < 45) {
            nextTest.setMinutes(30, 0, 0);
        } else {
            nextTest.setMinutes(0, 0, 0);
            nextTest.setHours(nextTest.getHours() + 1);
        }
    } else if (intervalMinutes >= 15) {
        // Round to nearest quarter hour (00, 15, 30, 45)
        const minutes = nextTest.getMinutes();
        if (minutes < 7.5) {
            nextTest.setMinutes(0, 0, 0);
        } else if (minutes < 22.5) {
            nextTest.setMinutes(15, 0, 0);
        } else if (minutes < 37.5) {
            nextTest.setMinutes(30, 0, 0);
        } else if (minutes < 52.5) {
            nextTest.setMinutes(45, 0, 0);
        } else {
            nextTest.setMinutes(0, 0, 0);
            nextTest.setHours(nextTest.getHours() + 1);
        }
    } else if (intervalMinutes >= 10) {
        // Round to nearest 10 minutes (00, 10, 20, 30, 40, 50)
        const minutes = nextTest.getMinutes();
        const rounded = Math.round(minutes / 10) * 10;
        if (rounded >= 60) {
            nextTest.setMinutes(0, 0, 0);
            nextTest.setHours(nextTest.getHours() + 1);
        } else {
            nextTest.setMinutes(rounded, 0, 0);
        }
    } else if (intervalMinutes >= 5) {
        // Round to nearest 5 minutes (00, 05, 10, 15, 20, 25, 30, 35, 40, 45, 50, 55)
        const minutes = nextTest.getMinutes();
        const rounded = Math.round(minutes / 5) * 5;
        if (rounded >= 60) {
            nextTest.setMinutes(0, 0, 0);
            nextTest.setHours(nextTest.getHours() + 1);
        } else {
            nextTest.setMinutes(rounded, 0, 0);
        }
    } else {
        // Round to nearest whole minute
        nextTest.setSeconds(0, 0);
    }
    
    return nextTest;
}

// Next test timer functions
function startNextTestTimer(intervalMinutes) {
    stopNextTestTimer(); // Clear any existing timer
    
    // Calculate next test time with optional rounding
    nextTestTimestamp = getRoundedNextTestTime(intervalMinutes);
    
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

// Load and display app version
async function loadAppVersion() {
    try {
        const version = await window.electronAPI.getCurrentVersion();
        if (appVersion && version) {
            appVersion.textContent = `Version ${version}`;
        }
    } catch (error) {
        console.error('Failed to load app version:', error);
        if (appVersion) {
            appVersion.textContent = 'Version --';
        }
    }
}

// Function to show inline update notification within version info
function showUpdateNotification(updateInfo) {
    if (!updateNotification || !updateMessage) {
        console.error('Update notification elements not found');
        return;
    }
    
    updateMessage.textContent = `🎉 v${updateInfo.latestVersion} available!`;
    updateNotification.style.display = 'block';
    
    console.log(`Update notification shown: ${updateInfo.currentVersion} -> ${updateInfo.latestVersion}`);
}
