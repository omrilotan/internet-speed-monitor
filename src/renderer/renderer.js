console.log('=== RENDERER.JS LOADED ===');

let speedChart;
// Global variables for next test tracking
let isMonitoring = false;
let nextTestTimer = null;
let nextTestTimestamp = null;
let allSpeedTests = []; // Store all tests for median calculation
let lastChartDate = null; // Track last date shown on chart for date labeling
let currentRunningSchedule = null; // Store the actual schedule being used for monitoring

// DOM elements
const startStopBtn = document.getElementById('start-stop-btn');
const testNowBtn = document.getElementById('test-now-btn');
const clearHistoryBtn = document.getElementById('clear-history-btn');
const clearSelectiveBtn = document.getElementById('clear-selective-btn');
const exportCsvBtn = document.getElementById('export-csv-btn');
const hamburgerBtn = document.getElementById('hamburger-btn');
const hamburgerDropdown = document.getElementById('hamburger-dropdown');
const autoStartToggle = document.getElementById('auto-start-toggle');
const launchAtStartupToggle = document.getElementById('launch-at-startup-toggle');

// Schedule controls
const scheduleTypeSelect = document.getElementById('schedule-type');
const intervalControls = document.getElementById('interval-controls');
const cronControls = document.getElementById('cron-controls');
const intervalInput = document.getElementById('interval');
const intervalDisplay = document.getElementById('interval-display');
const cronExpressionInput = document.getElementById('cron-expression');
const cronDescription = document.getElementById('cron-description');

// Status elements
const statusText = document.getElementById('status-text');
const statusDot = document.getElementById('status-dot');
const downloadSpeed = document.getElementById('download-speed');
const uploadSpeed = document.getElementById('upload-speed');
const ping = document.getElementById('ping');
const lastTest = document.getElementById('last-test');
const testRunning = document.getElementById('test-running');
const nextTest = document.getElementById('next-test');
const nextTestTime = document.getElementById('next-test-time-display');
const medianDownload = document.getElementById('median-download');
const medianUpload = document.getElementById('median-upload');

// Modal elements
const clearDataModal = document.getElementById('clear-data-modal');
const modalClose = clearDataModal.querySelector('.modal-close');
const cutoffDateInput = document.getElementById('cutoff-date');
const previewClearBtn = document.getElementById('preview-clear-btn');
const confirmClearBtn = document.getElementById('confirm-clear-btn');
const cancelClearBtn = document.getElementById('cancel-clear-btn');
const clearPreview = document.getElementById('clear-preview');

// Export date range modal elements
const exportDateRangeModal = document.getElementById('export-date-range-modal');
const exportModalClose = exportDateRangeModal.querySelector('.modal-close');
const exportStartDateInput = document.getElementById('export-start-date');
const exportEndDateInput = document.getElementById('export-end-date');
const previewExportBtn = document.getElementById('preview-export-btn');
const confirmExportBtn = document.getElementById('confirm-export-btn');
const cancelExportBtn = document.getElementById('cancel-export-btn');
const exportDateInfo = document.getElementById('export-date-info');
const exportRecordCount = document.getElementById('export-record-count');

// Chart view controls
const chartSubtitle = document.getElementById('chart-subtitle');
const chartViewLastDay = document.getElementById('chart-view-last-day');
const chartViewLast7 = document.getElementById('chart-view-last7');
const chartViewLast20 = document.getElementById('chart-view-last20');
const chartViewRange = document.getElementById('chart-view-range');
const chartRangeControls = document.getElementById('chart-range-controls');
const chartStartDateInput = document.getElementById('chart-start-date');
const chartEndDateInput = document.getElementById('chart-end-date');
const chartApplyBtn = document.getElementById('chart-apply-btn');

let chartViewMode = 'range'; // 'lastDay' | 'last7' | 'last20' | 'range'

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
    loadAndApplySettings(); // Load settings and apply them
    setupChartViewControls();
});

// Event listeners
startStopBtn.addEventListener('click', toggleMonitoring);
testNowBtn.addEventListener('click', testOnceNow);
hamburgerBtn.addEventListener('click', toggleHamburgerMenu);
clearHistoryBtn.addEventListener('click', clearHistory);
clearSelectiveBtn.addEventListener('click', showClearDataModal);
exportCsvBtn.addEventListener('click', exportCSV);

// Schedule control event listeners
scheduleTypeSelect.addEventListener('change', handleScheduleTypeChange);
intervalInput.addEventListener('input', updateIntervalDisplay);
cronExpressionInput.addEventListener('input', updateCronDescription);

// Save schedule settings when they change
scheduleTypeSelect.addEventListener('change', saveScheduleSettings);
intervalInput.addEventListener('change', saveScheduleSettings);
cronExpressionInput.addEventListener('change', saveScheduleSettings);

// Settings toggle event listeners
autoStartToggle.addEventListener('change', handleAutoStartToggle);
launchAtStartupToggle.addEventListener('change', handleLaunchAtStartupToggle);

// Cron preset button listeners
document.addEventListener('click', (e) => {
    if (e.target.classList.contains('preset-btn')) {
        const cronExpression = e.target.dataset.cron;
        if (e.target.classList.contains('custom-btn')) {
            // For custom button, clear the input and focus it
            cronExpressionInput.value = '';
            cronExpressionInput.focus();
            updateCronDescription();
        } else {
            cronExpressionInput.value = cronExpression;
            updateCronDescription();
            // Programmatically updating the input doesn't fire the 'change' event,
            // so dispatch a change to trigger the existing saveScheduleSettings handler
            // which listens for cronExpressionInput 'change'. Do not dispatch for the
            // custom button since that intentionally focuses the input for manual edits.
            cronExpressionInput.dispatchEvent(new Event('change'));
        }
    }
});

// Close hamburger menu when clicking outside
document.addEventListener('click', (e) => {
    if (!hamburgerBtn.contains(e.target) && !hamburgerDropdown.contains(e.target)) {
        hamburgerDropdown.style.display = 'none';
    }
});

// Modal event listeners
modalClose.addEventListener('click', hideClearDataModal);
cancelClearBtn.addEventListener('click', hideClearDataModal);
previewClearBtn.addEventListener('click', previewClearData);
confirmClearBtn.addEventListener('click', confirmClearData);

// Close modal when clicking outside
clearDataModal.addEventListener('click', (e) => {
    if (e.target === clearDataModal) {
        hideClearDataModal();
    }
});

// Export date range modal event listeners
exportModalClose.addEventListener('click', hideExportDateRangeModal);
cancelExportBtn.addEventListener('click', hideExportDateRangeModal);
previewExportBtn.addEventListener('click', previewExportRecordCount);
confirmExportBtn.addEventListener('click', exportCSVDateRange);
exportStartDateInput.addEventListener('change', previewExportRecordCount);
exportEndDateInput.addEventListener('change', previewExportRecordCount);

// Close modal when clicking outside
exportDateRangeModal.addEventListener('click', (e) => {
    if (e.target === exportDateRangeModal) {
        hideExportDateRangeModal();
    }
});

// Update notification event listeners
dismissUpdateBtn.addEventListener('click', () => {
    updateNotification.style.display = 'none';
});

downloadUpdateBtn.addEventListener('click', () => {
    window.electronAPI.openExternal('https://github.com/omrilotan/internet-speed-monitor/releases/latest');
});

// Initialize the app

// IPC listeners using electronAPI
window.electronAPI.onSpeedTestStarted((event) => {
    showTestRunningIndicator(); // Show the running indicator when any test starts
    // Show "Running" status when test starts
    showStatus('Running', 'running');
});

window.electronAPI.onSpeedTestResult((event, result) => {
    hideTestRunningIndicator(); // Hide the running indicator when test completes
    
    // Return to sleeping status if monitoring is active
    if (isMonitoring) {
        showStatus('Sleeping', 'sleeping');
    }
    
    // Add the new result to the allSpeedTests array at the beginning (most recent first)
    allSpeedTests.unshift(result);
    
    // Keep only the last 100 results to match loadHistoricalData behavior
    if (allSpeedTests.length > 100) {
        allSpeedTests = allSpeedTests.slice(0, 100);
    }
    
    updateCurrentStatsEnhanced(result);
    addToTable(result);
    // Re-render chart according to current view selection
    refreshChartForCurrentView();
    
    // Update median stats with the new data
    updateMedianStats();
});

// Listen for update notifications
window.electronAPI.ipcRenderer.on('update-available', (event, updateInfo) => {
    showUpdateNotification(updateInfo);
});

// Listen for menu-triggered debug actions
window.electronAPI.ipcRenderer.on('show-debug-log', () => {
    showDebugLog();
});

window.electronAPI.ipcRenderer.on('clear-debug-log', () => {
    clearDebugLog();
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
            // Use async/await but don't block the function
            updateNextTestDisplay().catch(error => {
                console.error('Error updating next test display:', error);
            });
        }
    }
}

async function startMonitoring() {
    console.log('=== START MONITORING ===');
    
    try {
        const schedule = getCurrentSchedule();
        showStatus('Initializing...', 'running');
        
        // Add a small delay to ensure main process is ready
        await new Promise(resolve => setTimeout(resolve, 1000));
        
        const response = await window.electronAPI.startMonitoring(schedule);
        
        if (response.success) {
            console.log('Monitoring started successfully');
            isMonitoring = true;
            currentRunningSchedule = schedule; // Store the running schedule
            updateUI();
            showStatus('Sleeping', 'sleeping');
            await startNextTestTimer(schedule);
        } else {
            console.error('Monitoring failed to start:', response.error);
            showStatus('Stopped', 'stopped');
            alert('Failed to start monitoring: ' + response.error);
        }
        
    } catch (error) {
        console.error('Error starting monitoring:', error);
        showStatus('Stopped', 'stopped');
        alert('Error: ' + error.message);
    }
}

async function stopMonitoring() {
    try {
        const response = await window.electronAPI.stopMonitoring();
        if (response.success) {
            isMonitoring = false;
            currentRunningSchedule = null; // Clear the running schedule
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

async function toggleMonitoring() {
    if (isMonitoring) {
        await stopMonitoring();
    } else {
        await startMonitoring();
    }
}

async function testOnceNow() {
    console.log('=== TEST ONCE NOW CLICKED ===');
    
    try {
        // Disable button to prevent multiple clicks
        testNowBtn.disabled = true;
        testNowBtn.textContent = '⏺️';
        testNowBtn.title = 'Test in progress...';
        
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
        testNowBtn.textContent = '⏺️';
        testNowBtn.title = 'Run a single speed test now';
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

// Clear data modal functions
function showClearDataModal() {
    // Set default cutoff date to 30 days ago
    const defaultDate = new Date();
    defaultDate.setDate(defaultDate.getDate() - 30);
    cutoffDateInput.valueAsDate = defaultDate;
    clearDataModal.style.display = 'flex';
}

function hideClearDataModal() {
    clearDataModal.style.display = 'none';
    clearPreview.style.display = 'none';
    confirmClearBtn.disabled = true;
}

// Export date range modal functions
function showExportDateRangeModal() {
    // Find earliest and latest dates from allSpeedTests
    let earliestDate = null;
    let latestDate = null;
    
    if (allSpeedTests && allSpeedTests.length > 0) {
        // allSpeedTests is sorted with newest first, so:
        // Latest is the first element
        latestDate = new Date(allSpeedTests[0].created_at || allSpeedTests[0].timestamp);
        // Earliest is the last element
        earliestDate = new Date(allSpeedTests[allSpeedTests.length - 1].created_at || allSpeedTests[allSpeedTests.length - 1].timestamp);
    } else {
        // If no data, use today as both start and end
        latestDate = new Date();
        earliestDate = new Date();
    }
    
    // Set date inputs to earliest and latest dates
    exportStartDateInput.valueAsDate = earliestDate;
    exportEndDateInput.valueAsDate = latestDate;
    
    // Set min and max attributes to restrict picker to data range
    // Both inputs should allow selecting any date within the full data range
    const earliestDateStr = earliestDate.toISOString().split('T')[0];
    const latestDateStr = latestDate.toISOString().split('T')[0];
    
    // Both start and end date inputs can select any date in the data range
    exportStartDateInput.min = earliestDateStr;
    exportStartDateInput.max = latestDateStr;
    exportEndDateInput.min = earliestDateStr;
    exportEndDateInput.max = latestDateStr;
    
    exportDateRangeModal.style.display = 'flex';
    previewExportRecordCount();
}

function hideExportDateRangeModal() {
    exportDateRangeModal.style.display = 'none';
    exportDateInfo.style.display = 'none';
    confirmExportBtn.disabled = true;
}

async function previewExportRecordCount() {
    try {
        let startDate = exportStartDateInput.value;
        let endDate = exportEndDateInput.value;
        
        if (!startDate || !endDate) {
            exportDateInfo.style.display = 'none';
            confirmExportBtn.disabled = true;
            return;
        }
        
        // Ensure start date is not after end date
        if (startDate > endDate) {
            // Swap them
            const temp = startDate;
            startDate = endDate;
            endDate = temp;
            exportStartDateInput.value = startDate;
            exportEndDateInput.value = endDate;
        }
        
        // Count records in the selected range
        const start = new Date(startDate);
        const end = new Date(endDate);
        end.setHours(23, 59, 59, 999); // Include entire end date
        
        const filtered = allSpeedTests.filter(test => {
            const testDate = new Date(test.created_at || test.timestamp);
            return testDate >= start && testDate <= end;
        });
        
        exportRecordCount.textContent = filtered.length;
        exportDateInfo.style.display = 'block';
        confirmExportBtn.disabled = filtered.length === 0;
    } catch (error) {
        console.error('Error previewing export record count:', error);
    }
}

async function exportCSVDateRange() {
    try {
        const startDate = exportStartDateInput.value;
        const endDate = exportEndDateInput.value;
        
        if (!startDate || !endDate) {
            alert('Please select both start and end dates');
            return;
        }
        
        console.log('Exporting CSV for date range:', startDate, 'to', endDate);
        const response = await window.electronAPI.exportCSVDateRange(startDate, endDate);
        
        if (response.success) {
            const message = response.message || 'Data exported successfully';
            alert(message + '\nFile saved to: ' + response.filePath);
            hideExportDateRangeModal();
        } else {
            alert('Failed to export data: ' + response.error);
        }
    } catch (error) {
        console.error('Error exporting CSV:', error);
        alert('Failed to export data: ' + error.message);
    }
}

async function exportCSV() {
    // Show the date range modal instead of exporting directly
    showExportDateRangeModal();
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
        if (isMonitoring && currentRunningSchedule) {
            await startNextTestTimer(currentRunningSchedule);
        } else if (isMonitoring && !currentRunningSchedule) {
            // Fallback: try to get schedule from UI if we don't have stored schedule
            try {
                const schedule = getCurrentSchedule();
                currentRunningSchedule = schedule;
                await startNextTestTimer(schedule);
            } catch (error) {
                console.warn('Could not determine schedule:', error.message);
                // Stop monitoring if we can't get a valid schedule
                isMonitoring = false;
                updateUI();
                showStatus('Error - Invalid Schedule', 'stopped');
            }
        }
    } catch (error) {
        console.error('Error getting monitoring status:', error);
        showStatus('Error', 'stopped');
    }
}

function updateUI() {
    // Update start/stop button icon and tooltip
    if (isMonitoring) {
        startStopBtn.textContent = '⏹️';
        startStopBtn.title = 'Stop automatic speed testing';
    } else {
        startStopBtn.textContent = '▶️';
        startStopBtn.title = 'Start automatic speed testing';
    }
    
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
        <td>${result.networkInterface || 'Unknown'}</td>
        <td>${result.isp || 'Unknown'}</td>
    `;
    
    resultsTable.insertBefore(row, resultsTable.firstChild);
    
    // Keep only the last 20 rows
    while (resultsTable.children.length > 20) {
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
            animation: false,
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

function setupChartViewControls() {
    // Initialize radio buttons and range controls
    if (chartViewLastDay) chartViewLastDay.checked = true;
    chartViewMode = 'lastDay';
    if (chartRangeControls) chartRangeControls.style.display = 'none';
    updateChartSubtitle();
    updateChartDateRangeBounds();

    // Event listeners
    if (chartViewLastDay) {
        chartViewLastDay.addEventListener('change', () => {
            if (chartViewLastDay.checked) {
                chartViewMode = 'lastDay';
                if (chartRangeControls) chartRangeControls.style.display = 'none';
                updateChartSubtitle();
                refreshChartForCurrentView();
            }
        });
    }
    if (chartViewLast7) {
        chartViewLast7.addEventListener('change', () => {
            if (chartViewLast7.checked) {
                chartViewMode = 'last7';
                if (chartRangeControls) chartRangeControls.style.display = 'none';
                updateChartSubtitle();
                refreshChartForCurrentView();
            }
        });
    }
    if (chartViewLast20) {
        chartViewLast20.addEventListener('change', () => {
            if (chartViewLast20.checked) {
                chartViewMode = 'last20';
                if (chartRangeControls) chartRangeControls.style.display = 'none';
                updateChartSubtitle();
                refreshChartForCurrentView();
            }
        });
    }
    if (chartViewRange) {
        chartViewRange.addEventListener('change', () => {
            if (chartViewRange.checked) {
                chartViewMode = 'range';
                if (chartRangeControls) chartRangeControls.style.display = 'flex';
                updateChartDateRangeBounds();
                updateChartSubtitle();
                refreshChartForCurrentView();
            }
        });
    }

    if (chartApplyBtn) {
        chartApplyBtn.addEventListener('click', () => {
            refreshChartForCurrentView();
        });
    }

    // Keep date range valid on change
    chartStartDateInput?.addEventListener('change', () => {
        if (chartEndDateInput.value && chartStartDateInput.value > chartEndDateInput.value) {
            chartEndDateInput.value = chartStartDateInput.value;
        }
    });
    chartEndDateInput?.addEventListener('change', () => {
        if (chartStartDateInput.value && chartEndDateInput.value < chartStartDateInput.value) {
            chartStartDateInput.value = chartEndDateInput.value;
        }
    });
}

// Format a Date to local YYYY-MM-DD (avoids timezone off-by-one with toISOString)
function formatDateYYYYMMDD(date) {
    const y = date.getFullYear();
    const m = String(date.getMonth() + 1).padStart(2, '0');
    const d = String(date.getDate()).padStart(2, '0');
    return `${y}-${m}-${d}`;
}

// Update the chart date range inputs' min/max according to available data
function updateChartDateRangeBounds() {
    if (!chartStartDateInput || !chartEndDateInput) return;

    let earliestDate, latestDate;
    if (allSpeedTests && allSpeedTests.length > 0) {
        const newest = allSpeedTests[0];
        const oldest = allSpeedTests[allSpeedTests.length - 1];
        // Prefer measurement timestamp; fallback to created_at
        latestDate = new Date(newest.timestamp || newest.created_at);
        earliestDate = new Date(oldest.timestamp || oldest.created_at);
    } else {
        // Fallback to today if no data yet
        earliestDate = new Date();
        latestDate = new Date();
    }

    const minStr = formatDateYYYYMMDD(earliestDate);
    const maxStr = formatDateYYYYMMDD(latestDate);

    chartStartDateInput.min = minStr;
    chartStartDateInput.max = maxStr;
    chartEndDateInput.min = minStr;
    chartEndDateInput.max = maxStr;

    // Initialize values if empty or clamp if out of bounds
    if (!chartStartDateInput.value) chartStartDateInput.value = minStr;
    if (!chartEndDateInput.value) chartEndDateInput.value = maxStr;
    if (chartStartDateInput.value < minStr) chartStartDateInput.value = minStr;
    if (chartStartDateInput.value > maxStr) chartStartDateInput.value = maxStr;
    if (chartEndDateInput.value < minStr) chartEndDateInput.value = minStr;
    if (chartEndDateInput.value > maxStr) chartEndDateInput.value = maxStr;
}

function updateChartSubtitle() {
    if (!chartSubtitle) return;
    if (chartViewMode === 'lastDay') {
        chartSubtitle.textContent = '(last day)';
    } else if (chartViewMode === 'last7') {
        chartSubtitle.textContent = '(last 7 days)';
    } else if (chartViewMode === 'last20') {
        chartSubtitle.textContent = '(last twenty data points)';
    } else {
        chartSubtitle.textContent = '(date range)';
    }
}

function renderChartFromData(items) {
    if (!speedChart) return;
    // items expected in chronological order for x-axis readability
    const labels = [];
    const downloads = [];
    const uploads = [];
    items.forEach(item => {
        const d = new Date(item.timestamp || item.created_at);
        const datePart = d.toLocaleDateString([], { month: 'short', day: 'numeric' });
        const timePart = d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', hour12: false });
        labels.push(`${datePart} ${timePart}`);
        downloads.push(item.download);
        uploads.push(item.upload);
    });

    speedChart.data.labels = labels;
    speedChart.data.datasets[0].data = downloads;
    speedChart.data.datasets[1].data = uploads;
    speedChart.update();
}

function refreshChartForCurrentView() {
    if (!allSpeedTests) return;
    let items = [];
    const now = new Date();
    if (chartViewMode === 'lastDay') {
        const cutoff = new Date(now.getTime() - 24 * 60 * 60 * 1000);
        const filtered = allSpeedTests.filter(test => {
            const t = new Date(test.created_at || test.timestamp);
            return t >= cutoff && t <= now;
        });
        items = filtered.reverse();
    } else if (chartViewMode === 'last7') {
        const cutoff = new Date(now.getTime() - 7 * 24 * 60 * 60 * 1000);
        const filtered = allSpeedTests.filter(test => {
            const t = new Date(test.created_at || test.timestamp);
            return t >= cutoff && t <= now;
        });
        items = filtered.reverse();
    } else if (chartViewMode === 'last20') {
        // Take newest twenty tests, then reverse to chronological
        items = allSpeedTests.slice(0, 20).reverse();
    } else {
        // Date range
        const startVal = chartStartDateInput.value;
        const endVal = chartEndDateInput.value;
        if (!startVal || !endVal) {
            // If not set yet, show all available (up to 100 kept)
            items = [...allSpeedTests].reverse();
        } else {
            const start = new Date(startVal);
            const end = new Date(endVal);
            end.setHours(23, 59, 59, 999);
            const filtered = allSpeedTests.filter(test => {
                const t = new Date(test.created_at || test.timestamp);
                return t >= start && t <= end;
            });
            items = filtered.reverse();
        }
    }
    renderChartFromData(items);
}

async function loadHistoricalData() {
    try {
        const data = await window.electronAPI.getSpeedTests(100); // Get more data for median calculation
        
        console.log('=== LOAD HISTORICAL DATA ===');
        console.log('Received data count:', data ? data.length : 0);
        if (data && data.length > 0) {
            console.log('Most recent test:', data[0]);
            console.log('Oldest test:', data[data.length - 1]);
        }
        
        // Store all data for median calculation
        allSpeedTests = [...data];

        // After data arrives, update date-range bounds for chart controls
        updateChartDateRangeBounds();
        
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
        
        // Add data to table (newest first, limit to 20)
        // Reverse the order so oldest gets added first and newest ends up on top
        const recentData = data.slice(0, 20);
        recentData.reverse().forEach(result => {
            addToTable(result);
        });
        
        // Render chart according to selected view
        refreshChartForCurrentView();
        
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
    
    // Define the step sequence
    const stepSequence = [1, 5, 10, 15, 30, 60];
    
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

// Next test time calculation function
async function getNextTestTime(schedule) {
    try {
        console.log('=== getNextTestTime called with:', JSON.stringify(schedule));
        
        // Validate schedule before sending to main process
        if (!schedule || !schedule.type) {
            console.error('Invalid schedule:', schedule);
            throw new Error('Invalid schedule object');
        }
        
        if (schedule.type === 'cron' && (!schedule.expression || schedule.expression.trim() === '')) {
            console.error('Cron schedule with empty expression:', schedule);
            throw new Error('Cron schedule missing expression');
        }
        
        // Use the main process to calculate next test time (returns ms since epoch)
        const result = await window.electronAPI.getNextTestTime(schedule);
        if (typeof result === 'number') {
            return new Date(result);
        }
        // If main returned a serialized Date string, try to parse it
        if (typeof result === 'string') {
            const parsed = new Date(result);
            if (!isNaN(parsed.getTime())) return parsed;
        }
        console.warn('Unexpected getNextTestTime response, falling back:', result);
        // Fall through to fallback logic below
    } catch (error) {
        console.error('Error getting next test time from main process:', error);
        // Fallback calculation in renderer
        if (schedule.type === 'interval') {
            const now = new Date();
            return new Date(now.getTime() + schedule.minutes * 60 * 1000);
        }
        
        // Fallback to 5 minutes for cron expressions that fail
        const now = new Date();
        return new Date(now.getTime() + 5 * 60 * 1000);
    }
}

// Next test timer functions
async function startNextTestTimer(schedule) {
    console.log('Starting timer for schedule:', JSON.stringify(schedule));
    
    stopNextTestTimer(); // Clear any existing timer
    
    try {
        // Calculate next test time based on schedule type
        nextTestTimestamp = await getNextTestTime(schedule);
        
        // Show next test info
        nextTest.style.display = 'block';
        await updateNextTestDisplay();
        
        // Update display every 30 seconds
        nextTestTimer = setInterval(async () => {
            await updateNextTestDisplay();
        }, 30000);
    } catch (error) {
        console.error('Error starting next test timer:', error);
        // Hide next test display on error
        nextTest.style.display = 'none';
    }
}

function stopNextTestTimer() {
    if (nextTestTimer) {
        clearInterval(nextTestTimer);
        nextTestTimer = null;
    }
    nextTest.style.display = 'none';
    nextTestTimestamp = null;
}

async function updateNextTestDisplay() {
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
        try {
            if (currentRunningSchedule) {
                // Create a deep copy to prevent any reference issues
                const scheduleCopy = JSON.parse(JSON.stringify(currentRunningSchedule));
                nextTestTimestamp = await getNextTestTime(scheduleCopy);
            } else {
                // Fallback: try to get from UI but warn about potential issue
                console.warn('No stored schedule, reading from UI');
                try {
                    const schedule = getCurrentSchedule();
                    nextTestTimestamp = await getNextTestTime(schedule);
                } catch (fallbackError) {
                    console.error('Fallback failed, using 5-minute default:', fallbackError.message);
                    // If UI schedule is also broken, just set next test to 5 minutes from now
                    nextTestTimestamp = new Date(Date.now() + 5 * 60 * 1000);
                }
            }
        } catch (error) {
            console.error('Error updating next test time:', error);
            // Keep current timestamp if update fails
        }
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
    const medianInfoElements = document.querySelectorAll('.median-info');
    
    if (allSpeedTests.length === 0) {
        medianDownload.textContent = '-- Mbps';
        medianUpload.textContent = '-- Mbps';
        medianPing.textContent = '-- ms';
        // Hide median info when no data
        medianInfoElements.forEach(el => el.style.display = 'none');
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
    
    // Show median info when there's data
    medianInfoElements.forEach(el => el.style.display = 'block');
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
    if (isMonitoring && currentRunningSchedule) {
        startNextTestTimer(currentRunningSchedule);
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

// Clear data until date functions
async function showClearDataModal() {
    try {
        // Get all data to find the earliest and latest test dates
        const allData = await window.electronAPI.getSpeedTests(10000);
        
        if (allData.length === 0) {
            alert('No data available to clear.');
            return;
        }
        
        // Find the earliest test date (last item in descending order)
        const earliestTest = allData[allData.length - 1];
        const earliestDate = new Date(earliestTest.created_at || earliestTest.timestamp);
        
        // Find the latest test date (first item in descending order)
        const latestTest = allData[0];
        const latestDate = new Date(latestTest.created_at || latestTest.timestamp);
        
        // Set minimum and maximum dates
        const minDate = earliestDate.toISOString().slice(0, 16);
        const maxDate = latestDate.toISOString().slice(0, 16);
        cutoffDateInput.min = minDate;
        cutoffDateInput.max = maxDate;
        
        // Set default date to 30 days ago or earliest date, whichever is later
        // but not later than the latest test
        const thirtyDaysAgo = new Date();
        thirtyDaysAgo.setDate(thirtyDaysAgo.getDate() - 30);
        
        let defaultDate;
        if (thirtyDaysAgo < earliestDate) {
            defaultDate = earliestDate;
        } else if (thirtyDaysAgo > latestDate) {
            defaultDate = latestDate;
        } else {
            defaultDate = thirtyDaysAgo;
        }
        
        cutoffDateInput.value = defaultDate.toISOString().slice(0, 16);
        
        // Reset modal state
        clearPreview.style.display = 'none';
        confirmClearBtn.disabled = true;
        
        clearDataModal.style.display = 'block';
        
        // Update the hint text to show the valid range
        const hintElement = cutoffDateInput.nextElementSibling;
        if (hintElement && hintElement.classList.contains('input-hint')) {
            hintElement.textContent = `Valid range: ${earliestDate.toLocaleString()} to ${latestDate.toLocaleString()}`;
        }
        
    } catch (error) {
        console.error('Error loading data for modal:', error);
        alert('Error loading data. Please try again.');
    }
}

function hideClearDataModal() {
    clearDataModal.style.display = 'none';
    clearPreview.style.display = 'none';
    confirmClearBtn.disabled = true;
}

async function previewClearData() {
    const cutoffDate = cutoffDateInput.value;
    
    if (!cutoffDate) {
        alert('Please select a date and time.');
        return;
    }
    
    try {
        previewClearBtn.disabled = true;
        previewClearBtn.textContent = 'Calculating...';
        
        // Get current data to calculate what would be removed
        const allData = await window.electronAPI.getSpeedTests(10000); // Get all data
        const cutoff = new Date(cutoffDate);
        
        if (allData.length === 0) {
            clearPreview.innerHTML = '<strong style="color: #666;">No data available.</strong>';
            clearPreview.style.display = 'block';
            confirmClearBtn.disabled = true;
            return;
        }
        
        // Get earliest and latest test dates
        const earliestTest = allData[allData.length - 1];
        const earliestDate = new Date(earliestTest.created_at || earliestTest.timestamp);
        const latestTest = allData[0];
        const latestDate = new Date(latestTest.created_at || latestTest.timestamp);
        
        // Check if cutoff is outside the valid range
        if (cutoff < earliestDate) {
            clearPreview.innerHTML = `
                <strong style="color: #dc3545;">Invalid Date Range</strong><br>
                Selected date is before your earliest test.<br>
                <strong>Earliest test:</strong> ${earliestDate.toLocaleString()}<br>
                <strong>Selected cutoff:</strong> ${cutoff.toLocaleString()}
            `;
            clearPreview.style.display = 'block';
            confirmClearBtn.disabled = true;
            return;
        }
        
        if (cutoff > latestDate) {
            clearPreview.innerHTML = `
                <strong style="color: #dc3545;">Invalid Date Range</strong><br>
                Selected date is after your latest test.<br>
                <strong>Latest test:</strong> ${latestDate.toLocaleString()}<br>
                <strong>Selected cutoff:</strong> ${cutoff.toLocaleString()}<br>
                <br>
                <em>Note: This would clear ALL your data.</em>
            `;
            clearPreview.style.display = 'block';
            confirmClearBtn.disabled = true;
            return;
        }
        
        const toRemove = allData.filter(test => {
            const testDate = new Date(test.created_at || test.timestamp);
            return testDate <= cutoff;
        });
        
        const toKeep = allData.length - toRemove.length;
        
        // Show preview
        clearPreview.innerHTML = `
            <strong>Preview:</strong><br>
            • Tests to be removed: <strong>${toRemove.length}</strong><br>
            • Tests to be kept: <strong>${toKeep}</strong><br>
            • Cutoff date: <strong>${cutoff.toLocaleString()}</strong><br>
            <br>
            <em>This action cannot be undone.</em>
        `;
        
        clearPreview.style.display = 'block';
        confirmClearBtn.disabled = toRemove.length === 0;
        
        if (toRemove.length === 0) {
            clearPreview.innerHTML += '<br><strong style="color: #666;">No data to remove before this date.</strong>';
        } else if (toRemove.length === allData.length) {
            clearPreview.innerHTML += '<br><strong style="color: #dc3545;">⚠️ This will remove ALL your data!</strong>';
        }
        
    } catch (error) {
        console.error('Error previewing clear data:', error);
        alert('Error calculating preview: ' + error.message);
    } finally {
        previewClearBtn.disabled = false;
        previewClearBtn.textContent = 'Preview';
    }
}

async function confirmClearData() {
    const cutoffDate = cutoffDateInput.value;
    
    if (!cutoffDate) {
        alert('Please select a date and time.');
        return;
    }
    
    try {
        confirmClearBtn.disabled = true;
        confirmClearBtn.textContent = 'Clearing...';
        
        const response = await window.electronAPI.clearDataUntilDate(cutoffDate);
        
        if (response.success) {
            alert(`Successfully cleared ${response.removed} speed tests before ${new Date(cutoffDate).toLocaleString()}.\n\nRemaining tests: ${response.remaining}`);
            
            // Refresh the display
            await loadHistoricalData();
            
            hideClearDataModal();
        } else {
            alert('Failed to clear data: ' + (response.error || 'Unknown error'));
        }
        
    } catch (error) {
        console.error('Error clearing data until date:', error);
        alert('Failed to clear data: ' + error.message);
    } finally {
        confirmClearBtn.disabled = false;
        confirmClearBtn.textContent = 'Clear Data';
    }
}

// Hamburger menu toggle function
function toggleHamburgerMenu() {
    const isVisible = hamburgerDropdown.style.display === 'block';
    hamburgerDropdown.style.display = isVisible ? 'none' : 'block';
}

// Cron interface functions
function handleScheduleTypeChange() {
    const scheduleType = scheduleTypeSelect.value;
    
    if (scheduleType === 'cron') {
        intervalControls.style.display = 'none';
        cronControls.style.display = 'flex';
        cronExpressionInput.value = cronExpressionInput.value || '*/5 * * * *';
        updateCronDescription();
    } else {
        intervalControls.style.display = 'flex';
        cronControls.style.display = 'none';
        updateIntervalDisplay();
    }
}

function updateIntervalDisplay() {
    const interval = parseInt(intervalInput.value) || 5;
    intervalDisplay.textContent = interval;
}

function updateCronDescription() {
    const cronExpression = cronExpressionInput.value.trim();
    
    if (!cronExpression) {
        cronDescription.innerHTML = 'Enter a cron expression - <a href="https://crontab.guru/" target="_blank" style="color: #007bff;">Create a custom schedule</a>';
        cronDescription.style.color = '#666'; // Gray for empty
        return;
    }
    
    try {
        // Simple cron description logic
        const description = describeCronExpression(cronExpression);
        if (description === 'Custom schedule') {
            cronDescription.innerHTML = 'Custom schedule - <a href="https://crontab.guru/" target="_blank" style="color: #007bff;">Create a custom schedule</a>';
        } else {
            cronDescription.textContent = description;
        }
        cronDescription.style.color = '#28a745'; // Green for valid
    } catch (error) {
        cronDescription.textContent = 'Invalid cron expression';
        cronDescription.style.color = '#dc3545'; // Red for invalid
    }
}

function describeCronExpression(cronExpr) {
    // Simple descriptions for common patterns
    const patterns = {
        '* * * * *': 'Every minute',
        '*/1 * * * *': 'Every minute',
        '*/5 * * * *': 'Every 5 minutes',
        '*/10 * * * *': 'Every 10 minutes', 
        '*/15 * * * *': 'Every 15 minutes',
        '*/30 * * * *': 'Every 30 minutes',
        '0 * * * *': 'Every hour',
        '0 */2 * * *': 'Every 2 hours',
        '0 */4 * * *': 'Every 4 hours',
        '0 */6 * * *': 'Every 6 hours',
        '0 */12 * * *': 'Every 12 hours',
        '0 0 * * *': 'Daily at midnight',
        '0 9 * * *': 'Daily at 9 AM',
        '0 17 * * *': 'Daily at 5 PM',
        '0 9-17 * * 1-5': 'Every hour, 9am-5pm, Mon-Fri',
        '*/30 9-17 * * 1-5': 'Every 30 minutes, 9am-5pm, Mon-Fri',
        '0 9,12,15,17 * * 1-5': 'At 9am, 12pm, 3pm, 5pm, Mon-Fri'
    };
    
    if (patterns[cronExpr]) {
        return patterns[cronExpr];
    }
    
    // Basic validation - should have 5 parts
    const parts = cronExpr.split(' ');
    if (parts.length !== 5) {
        throw new Error('Invalid format');
    }
    
    return 'Custom schedule';
}

function getCurrentSchedule() {
    const scheduleType = scheduleTypeSelect.value;
    
    if (scheduleType === 'cron') {
        if (!cronExpressionInput) {
            console.error('CRITICAL: cronExpressionInput element is null/undefined!');
            throw new Error('Cron input field not found');
        }
        
        const rawValue = cronExpressionInput.value;
        const cronExpression = rawValue ? rawValue.trim() : '';
        
        if (!cronExpression || cronExpression.length === 0) {
            throw new Error('Please enter a cron expression');
        }
        
        const schedule = {
            type: 'cron',
            expression: cronExpression
        };
        
        return schedule;
    } else {
        const interval = parseInt(intervalInput.value);
        if (!interval || interval < 1) {
            throw new Error('Please enter a valid interval');
        }
        return {
            type: 'interval',
            minutes: interval
        };
    }
}

// Settings management functions
async function loadAndApplySettings() {
    try {
        console.log('Loading settings...');
        const settings = await window.electronAPI.loadSettings();
        
        if (settings) {
            console.log('Settings loaded:', settings);
            
            // Apply schedule settings
            if (settings.scheduleType === 'cron') {
                scheduleTypeSelect.value = 'cron';
                cronExpressionInput.value = settings.cronExpression || '*/5 * * * *';
                handleScheduleTypeChange(); // Switch to cron view
                updateCronDescription();
            } else {
                scheduleTypeSelect.value = 'interval';
                intervalInput.value = settings.interval || 5;
                handleScheduleTypeChange(); // Switch to interval view
                updateIntervalDisplay();
            }
            
            // Apply toggle settings
            autoStartToggle.checked = settings.autoStart || false;
            
            // Get launch at startup status from system
            const launchAtStartup = await window.electronAPI.getLaunchAtStartup();
            launchAtStartupToggle.checked = launchAtStartup;
            
            // Auto-start monitoring if enabled
            if (settings.autoStart && !isMonitoring) {
                console.log('Auto-start is enabled, starting monitoring...');
                // Add a small delay to ensure everything is initialized
                setTimeout(() => {
                    startMonitoring();
                }, 1000);
            }
        }
    } catch (error) {
        console.error('Error loading settings:', error);
    }
}

async function saveScheduleSettings() {
    try {
        const scheduleType = scheduleTypeSelect.value;
        const settings = {
            scheduleType: scheduleType,
            interval: parseInt(intervalInput.value) || 5,
            cronExpression: cronExpressionInput.value.trim() || '*/5 * * * *',
            autoStart: autoStartToggle.checked,
            launchAtStartup: launchAtStartupToggle.checked
        };
        
        await window.electronAPI.saveSettings(settings);
        console.log('Schedule settings saved');
    } catch (error) {
        console.error('Error saving schedule settings:', error);
    }
}

async function handleAutoStartToggle() {
    try {
        const settings = await window.electronAPI.loadSettings();
        settings.autoStart = autoStartToggle.checked;
        await window.electronAPI.saveSettings(settings);
        console.log('Auto-start setting saved:', autoStartToggle.checked);
    } catch (error) {
        console.error('Error saving auto-start setting:', error);
        // Revert checkbox on error
        autoStartToggle.checked = !autoStartToggle.checked;
    }
}

async function handleLaunchAtStartupToggle() {
    try {
        const enable = launchAtStartupToggle.checked;
        const result = await window.electronAPI.setLaunchAtStartup(enable);
        
        if (result.success) {
            console.log('Launch at startup setting saved:', enable);
        } else {
            console.error('Failed to set launch at startup:', result.error);
            // Revert checkbox on error
            launchAtStartupToggle.checked = !enable;
            alert('Failed to change startup setting: ' + (result.error || 'Unknown error'));
        }
    } catch (error) {
        console.error('Error setting launch at startup:', error);
        // Revert checkbox on error
        launchAtStartupToggle.checked = !launchAtStartupToggle.checked;
        alert('Failed to change startup setting: ' + error.message);
    }
}
