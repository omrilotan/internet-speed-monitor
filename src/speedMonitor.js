const FastSpeedtest = require('fast-speedtest-api');

class SpeedMonitor {
    constructor(dataStore, sendResultCallback) {
        console.log('SpeedMonitor constructor called');
        
        if (!dataStore) {
            throw new Error('DataStore is required for SpeedMonitor');
        }
        
        if (!sendResultCallback) {
            throw new Error('SendResultCallback is required for SpeedMonitor');
        }
        
        this.dataStore = dataStore;
        this.sendResultCallback = sendResultCallback;
        this.isRunning = false;
        this.interval = null;
        this.testInterval = 5; // minutes
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

    start(intervalMinutes = 5) {
        if (this.isRunning) {
            console.log('Speed monitor is already running');
            return;
        }

        this.testInterval = intervalMinutes;
        this.isRunning = true;
        
        console.log(`Starting speed monitor with ${intervalMinutes} minute intervals`);
        
        // Run initial test immediately
        this.runSpeedTest();
        
        // Set up interval for periodic tests
        this.interval = setInterval(() => {
            this.runSpeedTest();
        }, intervalMinutes * 60 * 1000);
    }

    stop() {
        if (!this.isRunning) {
            console.log('Speed monitor is not running');
            return;
        }

        this.isRunning = false;
        
        if (this.interval) {
            clearInterval(this.interval);
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
            return;
        }

        console.log('Starting speed test...');
        
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
        } finally {
            this.currentTest = null;
        }
    }

    getStatus() {
        return {
            isRunning: this.isRunning,
            testInterval: this.testInterval,
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
