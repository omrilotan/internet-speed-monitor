const { exec } = require('child_process');
const { promisify } = require('util');

const execAsync = promisify(exec);

module.exports = { detectNetworkConnectionType };

/**
 * Detects the network connection type (WiFi, Ethernet, etc.) using OS-specific commands
 * @returns {Promise<string>} Connection type
 */
async function detectNetworkConnectionType() {
    try {
        const platform = process.platform;
        
        switch (platform) {
            case 'darwin': {
                // macOS - use route and networksetup commands
                const { stdout } = await execAsync('route -n get default');
                const interfaceMatch = stdout.match(/interface: (\w+)/);
                
                if (interfaceMatch) {
                    const interfaceName = interfaceMatch[1];
                    
                    // Check if it's WiFi using networksetup
                    try {
                        const { stdout: wifiCheck } = await execAsync(`networksetup -getairportnetwork ${interfaceName}`);
                        if (wifiCheck.includes('Current Wi-Fi Network') || !wifiCheck.includes('not a Wi-Fi interface')) {
                            return 'WiFi';
                        }
                    } catch (error) {
                        // If networksetup fails, fall back to interface name detection
                        if (interfaceName.startsWith('en1') || interfaceName.includes('wi')) {
                            return 'WiFi';
                        }
                    }
                    
                    // Check for Ethernet
                    if (interfaceName.startsWith('en0') || interfaceName.includes('eth')) {
                        return 'Ethernet';
                    }
                    
                    return 'Network';
                }
                break;
            }
            
            case 'win32': {
                // Windows - use netsh command
                const { stdout } = await execAsync('netsh interface show interface');
                const lines = stdout.split('\n');
                
                for (const line of lines) {
                    if (line.includes('Connected') && line.includes('Dedicated')) {
                        const lowerLine = line.toLowerCase();
                        if (lowerLine.includes('wi-fi') || lowerLine.includes('wireless') || lowerLine.includes('wifi')) {
                            return 'WiFi';
                        }
                        if (lowerLine.includes('ethernet') || lowerLine.includes('local area connection')) {
                            return 'Ethernet';
                        }
                    }
                }
                break;
            }
            
            case 'linux': {
                // Linux - use ip route command
                const { stdout } = await execAsync('ip route show default');
                const interfaceMatch = stdout.match(/dev (\w+)/);
                
                if (interfaceMatch) {
                    const interfaceName = interfaceMatch[1];
                    
                    // Check if wireless using iwconfig or checking interface name
                    try {
                        await execAsync(`iwconfig ${interfaceName}`);
                        return 'WiFi'; // If iwconfig succeeds, it's a wireless interface
                    } catch (error) {
                        // iwconfig failed, so it's likely not WiFi
                        if (interfaceName.startsWith('wl') || interfaceName.includes('wifi')) {
                            return 'WiFi';
                        }
                        if (interfaceName.startsWith('eth') || interfaceName.startsWith('en')) {
                            return 'Ethernet';
                        }
                    }
                    
                    return 'Network';
                }
                break;
            }
            
            default:
                // Unsupported platform
                break;
        }
        
        return 'Unknown';
    } catch (error) {
        console.error('Error detecting network connection type:', error);
        return 'Unknown';
    }
}