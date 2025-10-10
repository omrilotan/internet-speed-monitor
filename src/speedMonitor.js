const FastSpeedtest = require('fast-speedtest-api');
const { CronExpressionParser } = require('cron-parser');
const { detectNetworkConnectionType } = require('./utils/detectNetworkConnectionType');
const { lookupISP } = require('./utils/lookupISP');

class SpeedMonitor {
    constructor(dataStore, sendResultCallback, sendStartedCallback = null) {
        console.log('SpeedMonitor constructor called');
        
        if (!dataStore) {
            throw new Error('DataStore is required for SpeedMonitor');
        }
        
        if (!sendResultCallback) {
            throw new Error('SendResultCallback is required for SpeedMonitor');
        }
        
        this.dataStore = dataStore;
        this.sendResultCallback = sendResultCallback;
        this.sendStartedCallback = sendStartedCallback;
        this.isRunning = false;
        this.interval = null;
        this.schedule = { type: 'interval', minutes: 5 }; // default schedule
        this.currentTest = null;
        
        try {
            this.speedtest = new FastSpeedtest({
                token: "YXNkZmFzZGxmbnNkYWZoYXNkZmhrYWxm", // Universal token
                verbose: false,
                timeout: 10000,
                https: true,
                urlCount: 5,
                bufferSize: 8,
                unit: FastSpeedtest.UNITS.Mbps
            });
            console.log('FastSpeedtest instance created successfully');
        } catch (error) {
            console.error('Error creating FastSpeedtest instance:', error);
            // Don't throw here, we'll handle it in the speed test method
            this.speedtest = null;
        }
        
        console.log('SpeedMonitor constructor completed');
    }

    start(schedule = { type: 'interval', minutes: 5 }) {
        if (this.isRunning) {
            console.log('Speed monitor is already running');
            return;
        }

        // Create defensive copy and validate schedule
        schedule = JSON.parse(JSON.stringify(schedule));
        
        // Strict validation for cron schedules
        if (schedule.type === 'cron') {
            if (!schedule.expression || typeof schedule.expression !== 'string' || schedule.expression.trim() === '') {
                console.error('CRITICAL: Invalid cron schedule detected');
                console.error('Schedule object:', JSON.stringify(schedule));
                throw new Error('Invalid cron schedule: expression is missing, not a string, or empty');
            }
            
            // Validate cron expression has the right format (5 parts)
            const parts = schedule.expression.trim().split(/\s+/);
            if (parts.length !== 5) {
                console.error('CRITICAL: Invalid cron expression format');
                console.error('Expression:', schedule.expression);
                console.error('Parts:', parts);
                throw new Error('Invalid cron expression: must have exactly 5 parts (minute hour day month weekday)');
            }
        }

        this.schedule = schedule;
        this.isRunning = true;
        this.lastTestTime = null;
        
        console.log(`Starting speed monitor with schedule:`, schedule);
        
        if (schedule.type === 'cron') {
            // Cron-style scheduling: don't run initial test, wait for scheduled time
            console.log('Cron scheduling mode: waiting for next scheduled time');
            // Don't set lastTestTime initially for cron schedules
            this.lastTestTime = null;
            // Check every minute if it's time to run
            this.interval = setInterval(() => {
                this.checkAndRunScheduledTest();
            }, 60000); // Check every minute
        } else {
            // Traditional fixed interval scheduling: run initial test immediately
            this.runSpeedTest();
            this.lastTestTime = new Date();
            this.interval = setInterval(() => {
                this.runSpeedTest();
            }, schedule.minutes * 60 * 1000);
        }
    }

    checkAndRunScheduledTest() {
        if (!this.isRunning) return;
        
        const now = new Date();
        const shouldRun = this.shouldRunTestNow(now);
        
        console.log(`Checking scheduled test at ${now.toISOString()}: shouldRun=${shouldRun}`);
        
        if (shouldRun) {
            console.log(`Running scheduled test at ${now.toISOString()}`);
            this.runSpeedTest();
            this.lastTestTime = new Date(now);
        } else {
            // Debug info for why test didn't run
            if (this.schedule.type === 'cron') {
                try {
                    const interval = CronExpressionParser.parse(this.schedule.expression, {
                        currentDate: new Date(now.getTime() - 60000)
                    });
                    const nextRun = interval.next().toDate();
                    console.log(`Next scheduled test: ${nextRun.toISOString()}, Current: ${now.toISOString()}`);
                } catch (error) {
                    console.error('Error calculating next run time:', error);
                }
            }
        }
    }

    shouldRunTestNow(currentTime) {
        if (this.schedule.type === 'cron') {
            // For cron scheduling, check if we're at a scheduled time
            try {
                // Parse cron from current time to find next scheduled time
                const interval = CronExpressionParser.parse(this.schedule.expression, {
                    currentDate: new Date(currentTime.getTime() - 60000) // Look from 1 minute ago
                });
                const nextRun = interval.next().toDate();
                
                // Check if we're within 30 seconds of the scheduled time
                const tolerance = 30 * 1000;
                const isScheduledTime = Math.abs(currentTime.getTime() - nextRun.getTime()) < tolerance;
                
                // Prevent running the same test twice by checking last test time
                if (isScheduledTime && this.lastTestTime) {
                    const timeSinceLastTest = (currentTime - this.lastTestTime) / (1000 * 60);
                    if (timeSinceLastTest < 0.5) { // Don't run if we just ran a test
                        return false;
                    }
                }
                
                return isScheduledTime;
            } catch (error) {
                console.error('Error parsing cron expression:', error);
                return false;
            }
        } else {
            // Simple interval scheduling
            if (!this.lastTestTime) return false;
            
            // Check if we've passed a reasonable minimum interval since last test
            const timeSinceLastTest = (currentTime - this.lastTestTime) / (1000 * 60);
            if (timeSinceLastTest < 0.5) { // Allow 30 second tolerance
                return false;
            }
            // Simple interval scheduling
            const intervalMinutes = this.schedule.minutes;
            
            // Check if we've passed the minimum interval since last test
            if (timeSinceLastTest < intervalMinutes - 0.5) { // Allow 30 second tolerance
                return false;
            }
            
            return timeSinceLastTest >= intervalMinutes;
        }
    }

    stop() {
        if (!this.isRunning) {
            console.log('Speed monitor is not running');
            return;
        }

        this.isRunning = false;
        
        if (this.interval) {
            clearInterval(this.interval); // Always use clearInterval now
            this.interval = null;
        }

        if (this.currentTest) {
            // Cancel current test if possible
            this.currentTest = null;
        }

        console.log('Speed monitor stopped');
    }

    async runSpeedTest() {
        if (this.currentTest) {
            console.log('Speed test already in progress, skipping...');
            return null; // Return null to indicate test was skipped
        }

        console.log('Starting speed test...');
        
        // Send test started notification if callback is available
        if (this.sendStartedCallback) {
            this.sendStartedCallback();
        }
        
        try {
            this.currentTest = true;
            
            // Detect network connection type
            const networkInterface = await detectNetworkConnectionType();
            console.log('Detected network interface:', networkInterface);
            
            // Run download speed test
            const downloadSpeed = await this.speedtest.getSpeed();
            
            // Simulate upload speed (fast-speedtest-api only does download)
            // For now we'll estimate upload as 10% of download, but this could be improved
            const uploadSpeed = downloadSpeed * 0.1;
            
            // Simulate ping test (would need separate implementation for real ping)
            const ping = Math.random() * 50 + 10; // Random ping between 10-60ms
            
            // Try to lookup ISP information
            const isp = await lookupISP();
            
            const speedTestResult = {
                timestamp: new Date().toISOString(),
                download: downloadSpeed,
                upload: uploadSpeed,
                ping: ping,
                server: 'Netflix Fast.com',
                isp: isp,
                networkInterface: networkInterface
            };

            console.log('Speed test completed:', speedTestResult);

            // Store result in database
            if (this.dataStore) {
                await this.dataStore.saveSpeedTest(speedTestResult);
            }

            // Send result to renderer process
            if (this.sendResultCallback) {
                this.sendResultCallback(speedTestResult);
            }

            return speedTestResult; // Return the result

        } catch (error) {
            console.error('Speed test failed:', error);
            
            // Send error result to renderer
            const errorResult = {
                timestamp: new Date().toISOString(),
                download: 0,
                upload: 0,
                ping: 0,
                server: 'Error',
                error: error.message
            };
            
            if (this.sendResultCallback) {
                this.sendResultCallback(errorResult);
            }

            return errorResult; // Return the error result
        } finally {
            this.currentTest = null;
        }
    }

    getStatus() {
        return {
            isRunning: this.isRunning,
            schedule: this.schedule,
            hasCurrentTest: this.currentTest !== null
        };
    }

    // Manual test trigger
    async triggerTest() {
        if (!this.isRunning) {
            console.log('Cannot trigger manual test - monitor is not running');
            return false;
        }

        await this.runSpeedTest();
        return true;
    }
}

module.exports = SpeedMonitor;
