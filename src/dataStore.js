const path = require('path');
const fs = require('fs');
const { app } = require('electron');

class DataStore {
    constructor() {
        console.log('DataStore constructor called');
        try {
            this.dataPath = this.getDataPath();
            this.speedTestsFile = path.join(this.dataPath, 'speed_tests.json');
            this.data = {
                speedTests: []
            };
            console.log('DataStore constructor completed, dataPath:', this.dataPath);
        } catch (error) {
            console.error('Error in DataStore constructor:', error);
            throw error;
        }
    }

    getDataPath() {
        // Use user data directory for storing the data
        const userDataPath = app ? app.getPath('userData') : './data';
        
        // Ensure directory exists
        if (!fs.existsSync(userDataPath)) {
            fs.mkdirSync(userDataPath, { recursive: true });
        }
        
        return userDataPath;
    }

    initialize() {
        try {
            // Load existing data if file exists
            if (fs.existsSync(this.speedTestsFile)) {
                const fileData = fs.readFileSync(this.speedTestsFile, 'utf8');
                this.data = JSON.parse(fileData);
                console.log('Loaded existing data from:', this.speedTestsFile);
            } else {
                console.log('No existing data file found, starting fresh');
            }
            
            // Ensure speedTests array exists
            if (!Array.isArray(this.data.speedTests)) {
                this.data.speedTests = [];
            }
            
            console.log('Data store initialized with', this.data.speedTests.length, 'speed tests');
            return Promise.resolve();
        } catch (error) {
            console.error('Error initializing data store:', error);
            this.data = { speedTests: [] };
            return Promise.resolve(); // Don't fail completely
        }
    }

    saveData() {
        try {
            fs.writeFileSync(this.speedTestsFile, JSON.stringify(this.data, null, 2));
        } catch (error) {
            console.error('Error saving data:', error);
            throw error;
        }
    }

    async saveSpeedTest(testResult) {
        try {
            const speedTest = {
                id: Date.now(), // Simple ID using timestamp
                timestamp: testResult.timestamp,
                download: testResult.download,
                upload: testResult.upload,
                ping: testResult.ping,
                server: testResult.server || null,
                location: testResult.location || null,
                isp: testResult.isp || null,
                created_at: new Date().toISOString()
            };

            this.data.speedTests.push(speedTest);
            this.saveData();
            
            console.log('Speed test saved with ID:', speedTest.id);
            return speedTest.id;
        } catch (error) {
            console.error('Error saving speed test:', error);
            throw error;
        }
    }

    async getSpeedTests(limit = 50) {
        try {
            // Sort by created_at descending and limit results
            const sortedTests = this.data.speedTests
                .sort((a, b) => new Date(b.created_at) - new Date(a.created_at))
                .slice(0, limit);
            
            return sortedTests;
        } catch (error) {
            console.error('Error retrieving speed tests:', error);
            return [];
        }
    }

    async getSpeedTestsByDateRange(startDate, endDate) {
        try {
            const start = new Date(startDate);
            const end = new Date(endDate);
            
            const filteredTests = this.data.speedTests.filter(test => {
                const testDate = new Date(test.timestamp);
                return testDate >= start && testDate <= end;
            });
            
            return filteredTests.sort((a, b) => new Date(b.created_at) - new Date(a.created_at));
        } catch (error) {
            console.error('Error retrieving speed tests by date range:', error);
            return [];
        }
    }

    async getAverageSpeedsByDay(days = 7) {
        try {
            const cutoffDate = new Date();
            cutoffDate.setDate(cutoffDate.getDate() - days);
            
            const recentTests = this.data.speedTests.filter(test => 
                new Date(test.timestamp) >= cutoffDate
            );
            
            // Group by date
            const groupedByDate = {};
            recentTests.forEach(test => {
                const date = new Date(test.timestamp).toISOString().split('T')[0];
                if (!groupedByDate[date]) {
                    groupedByDate[date] = [];
                }
                groupedByDate[date].push(test);
            });
            
            // Calculate averages
            const averages = Object.keys(groupedByDate).map(date => {
                const tests = groupedByDate[date];
                const avgDownload = tests.reduce((sum, test) => sum + test.download, 0) / tests.length;
                const avgUpload = tests.reduce((sum, test) => sum + test.upload, 0) / tests.length;
                const avgPing = tests.reduce((sum, test) => sum + test.ping, 0) / tests.length;
                
                return {
                    date,
                    avg_download: avgDownload,
                    avg_upload: avgUpload,
                    avg_ping: avgPing,
                    test_count: tests.length
                };
            });
            
            return averages.sort((a, b) => b.date.localeCompare(a.date));
        } catch (error) {
            console.error('Error retrieving average speeds:', error);
            return [];
        }
    }

    async deleteOldTests(daysToKeep = 30) {
        try {
            const cutoffDate = new Date();
            cutoffDate.setDate(cutoffDate.getDate() - daysToKeep);
            
            const originalLength = this.data.speedTests.length;
            this.data.speedTests = this.data.speedTests.filter(test => 
                new Date(test.timestamp) >= cutoffDate
            );
            
            const deletedCount = originalLength - this.data.speedTests.length;
            
            if (deletedCount > 0) {
                this.saveData();
                console.log('Deleted', deletedCount, 'old speed test records');
            }
            
            return deletedCount;
        } catch (error) {
            console.error('Error deleting old tests:', error);
            throw error;
        }
    }

    async getStatistics() {
        try {
            const tests = this.data.speedTests;
            
            if (tests.length === 0) {
                return {
                    total_tests: 0,
                    avg_download: 0,
                    max_download: 0,
                    min_download: 0,
                    avg_upload: 0,
                    max_upload: 0,
                    min_upload: 0,
                    avg_ping: 0,
                    max_ping: 0,
                    min_ping: 0
                };
            }
            
            const downloads = tests.map(test => test.download);
            const uploads = tests.map(test => test.upload);
            const pings = tests.map(test => test.ping);
            
            return {
                total_tests: tests.length,
                avg_download: downloads.reduce((sum, val) => sum + val, 0) / downloads.length,
                max_download: Math.max(...downloads),
                min_download: Math.min(...downloads),
                avg_upload: uploads.reduce((sum, val) => sum + val, 0) / uploads.length,
                max_upload: Math.max(...uploads),
                min_upload: Math.min(...uploads),
                avg_ping: pings.reduce((sum, val) => sum + val, 0) / pings.length,
                max_ping: Math.max(...pings),
                min_ping: Math.min(...pings)
            };
        } catch (error) {
            console.error('Error retrieving statistics:', error);
            throw error;
        }
    }

    /**
     * Clear all speed test data
     * @returns {Promise<boolean>} Success status
     */
    async clearAllData() {
        try {
            this.speedTests = [];
            await this.saveData();
            console.log('All speed test data cleared');
            return true;
        } catch (error) {
            console.error('Error clearing data:', error);
            return false;
        }
    }

    /**
     * Export all data as CSV format
     * @returns {string} CSV formatted data
     */
    exportAsCSV() {
        try {
            console.log('Exporting CSV, speedTests count:', this.data.speedTests.length);
            
            const headers = ['Date', 'Time', 'Download Speed (Mbps)', 'Upload Speed (Mbps)', 'Ping (ms)'];
            const csvRows = [headers.join(',')];

            if (this.data.speedTests.length === 0) {
                // Add a row indicating no data
                csvRows.push('"No data available","Please run some speed tests first","","",""');
            } else {
                this.data.speedTests.forEach(test => {
                    const date = new Date(test.timestamp);
                    const dateStr = date.toLocaleDateString();
                    const timeStr = date.toLocaleTimeString();
                    const row = [
                        `"${dateStr}"`,
                        `"${timeStr}"`,
                        test.download.toFixed(2),
                        test.upload.toFixed(2),
                        test.ping.toFixed(0)
                    ];
                    csvRows.push(row.join(','));
                });
            }

            return csvRows.join('\n');
        } catch (error) {
            console.error('Error exporting CSV:', error);
            return '';
        }
    }

    close() {
        try {
            this.saveData();
            console.log('Data store closed and saved');
        } catch (error) {
            console.error('Error closing data store:', error);
        }
    }
}

module.exports = DataStore;
