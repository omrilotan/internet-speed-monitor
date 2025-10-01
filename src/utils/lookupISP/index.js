module.exports = { lookupISP };

/**
 * Attempts to lookup ISP information using a public API
 * @returns {Promise<string>} ISP name or 'Unknown'
 */
async function lookupISP() {
    try {
        // Create AbortController for timeout handling
        const controller = new AbortController();
        const timeoutId = setTimeout(() => controller.abort(), 5000);

        try {
            // Get public IP using ipify
            const ipResponse = await fetch('https://api.ipify.org?format=json', {
                signal: controller.signal,
                timeout: 3000
            });
            
            if (!ipResponse.ok) {
                throw new Error(`IP lookup failed: ${ipResponse.status}`);
            }
            
            const ipData = await ipResponse.json();
            const ip = ipData.ip;
            
            if (!ip) {
                throw new Error('No IP address received');
            }
            
            // Lookup ISP info using the IP
            const ispResponse = await fetch(`http://ip-api.com/json/${ip}?fields=isp`, {
                signal: controller.signal,
                timeout: 4000
            });
            
            if (!ispResponse.ok) {
                throw new Error(`ISP lookup failed: ${ispResponse.status}`);
            }
            
            const ispData = await ispResponse.json();
            clearTimeout(timeoutId);
            
            return ispData.isp || 'Unknown';
            
        } finally {
            clearTimeout(timeoutId);
        }
        
    } catch (error) {
        if (error.name === 'AbortError') {
            console.log('ISP lookup timed out');
        } else {
            console.error('Error in ISP lookup:', error.message);
        }
        return 'Unknown';
    }
}