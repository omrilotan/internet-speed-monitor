# Electron Internet Speed Monitor

This is an Electron application that monitors internet connectivity speed at set intervals and stores the data.

## Project Structure
- Main Electron process files in root directory
- Renderer process files for UI
- Speed testing module for connectivity monitoring
- Data storage module for persisting results

## Development Guidelines
- Use Node.js modules for speed testing functionality
- Use JSON format for local data storage (stored in speed_tests.json)
- Create simple UI for displaying current speeds and historical data
- Follow Electron security best practices with context isolation and secure IPC

## Key Features
- Periodic internet speed testing
- Data persistence and retrieval
- Real-time speed monitoring display
- Historical data visualization
