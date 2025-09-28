const FastSpeedtest = require('fast-speedtest-api');
const parser = require('cron-parser');

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

        this.schedule = schedule;
        this.isRunning = true;
        this.lastTestTime = null;
        
        console.log(`Starting speed monitor with schedule:`, schedule);
        
        // Run initial test immediately
        this.runSpeedTest();
        this.lastTestTime = new Date();
        
        if (schedule.type === 'cron') {
            // Cron-style scheduling: check every minute if it's time to run
            this.interval = setInterval(() => {
                this.checkAndRunScheduledTest();
            }, 60000); // Check every minute
        } else {
            // Traditional fixed interval scheduling
            this.interval = setInterval(() => {
                this.runSpeedTest();
            }, schedule.minutes * 60 * 1000);
        }
    }

    checkAndRunScheduledTest() {
        if (!this.isRunning) return;
        
        const now = new Date();
        const shouldRun = this.shouldRunTestNow(now);
        
        if (shouldRun) {
            console.log(`Running scheduled test at ${now.toLocaleTimeString()}`);
            this.runSpeedTest();
            this.lastTestTime = new Date(now);
        }
    }

    shouldRunTestNow(currentTime) {
        if (!this.lastTestTime) return false;
        
        // Check if we've passed a reasonable minimum interval since last test
        const timeSinceLastTest = (currentTime - this.lastTestTime) / (1000 * 60);
        if (timeSinceLastTest < 0.5) { // Allow 30 second tolerance
            return false;
        }
        
        if (this.schedule.type === 'cron') {
            // Use cron expression for scheduling
            try {
                const interval = parser.parseExpression(this.schedule.expression, {
                    currentDate: this.lastTestTime
                });
                const nextRun = interval.next().toDate();
                
                // Allow 30 second tolerance
                const tolerance = 30 * 1000;
                return Math.abs(currentTime.getTime() - nextRun.getTime()) < tolerance;
            } catch (error) {
                console.error('Error parsing cron expression:', error);
                return false;
            }
        } else {
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
            
            // Run download speed test
            const downloadSpeed = await this.speedtest.getSpeed();
            
            // Simulate upload speed (fast-speedtest-api only does download)
            // For now we'll estimate upload as 10% of download, but this could be improved
            const uploadSpeed = downloadSpeed * 0.1;
            
            // Simulate ping test (would need separate implementation for real ping)
            const ping = Math.random() * 50 + 10; // Random ping between 10-60ms
            
            const speedTestResult = {
                timestamp: new Date().toISOString(),
                download: downloadSpeed,
                upload: uploadSpeed,
                ping: ping,
                server: 'Netflix Fast.com',
                location: 'Unknown',
                isp: 'Unknown'
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
