# Internet Speed Monitor

A beautiful, easy-to-use desktop application that monitors your internet connection speed automatically and helps you track your network performance over time.

![Internet Speed Monitor Screenshot](https://github.com/user-attachments/assets/9ec86a95-2764-457e-ad6c-210abf822ae2)

## ✨ Features

- **🚀 Automatic Monitoring** - Set intervals from 1-60 minutes for hands-free speed testing
- **⚡ Instant Testing** - "Test Once Now" button for on-demand speed tests without starting monitoring
- **📊 Real-time Charts** - Beautiful visualizations of your internet speed over time  
- **📈 Performance Tracking** - Keep detailed history of download speeds, upload speeds, and ping
- **📐 Median Statistics** - Track median performance across all your historical data for better insights
- **⏱️ Smart Status Indicators** - Real-time display of test status with countdown to next scheduled test
- **💾 Local Storage** - All your data stays on your device - no cloud required
- **📤 Easy Export** - Export your speed data to CSV for further analysis
- **🧹 Data Management** - Clear history and debug logs with dedicated buttons
- **🎨 Clean Interface** - Modern, responsive design that works great on desktop, tablet, and mobile
- **🔄 Cross-platform** - Available for Windows, macOS, and Linux

## 🚀 Quick Start

### Download & Install

#### Option 1: Direct Download (Most Users)
1. **Go to [Releases](https://github.com/omrilotan/internet-speed-monitor/releases)**
2. **Download for your platform:**
   - 🍎 **macOS**: Download `.dmg` file
   - 🪟 **Windows**: Download `.exe` installer  
   - 🐧 **Linux**: Download `.AppImage` file

3. **Install and run** - Double-click the downloaded file and follow the installation prompts

#### Option 2: Homebrew (macOS - Recommended for avoiding security dialogs)
```bash
# Add this repository as a tap
brew tap omrilotan/internet-speed-monitor

# Install the app
brew install --cask internet-speed-monitor
```

#### Option 3: Build from Source
```bash
# Clone the repository
git clone https://github.com/omrilotan/internet-speed-monitor.git
cd internet-speed-monitor

# Install dependencies
npm install

# Run in development
npm start

# Or build for your platform
npm run build:mac    # macOS
npm run build:win    # Windows
npm run build:linux  # Linux
```

### 🍎 macOS Security Dialogs

macOS may show security warnings like "cannot verify the app is free of malware" or "app is damaged". This is normal for apps not distributed through the Mac App Store. The app is completely safe and open source.

**📋 Solutions (choose any one):**

**🎯 Best Solution: Use Homebrew** (No security dialogs)
```bash
brew tap omrilotan/internet-speed-monitor
brew install --cask internet-speed-monitor
```

**Method 1: Right-click to Open**
1. **Right-click** (or Control-click) the app in Finder
2. **Select "Open"** from the context menu
3. **Click "Open"** in the security dialog

**Method 2: System Preferences**
1. Open **System Preferences** → **Security & Privacy** → **General**
2. Click **"Open Anyway"** next to the blocked app message
3. **Confirm** by clicking "Open"

**Method 3: Terminal Command** (Advanced Users)
```bash
sudo xattr -rd com.apple.quarantine "/Applications/Internet Speed Monitor.app"
```

**Method 4: Build from Source** (No security warnings)
```bash
git clone https://github.com/omrilotan/internet-speed-monitor.git
cd internet-speed-monitor && npm install && npm start
```

This is normal for apps not distributed through the Mac App Store. The app is safe and open source.

### Using the App

1. **Set Your Interval** - Choose how often you want to test your speed (1-60 minutes)
2. **Start Monitoring** - Click the "Start Monitoring" button to begin automatic testing
3. **Test Instantly** - Use "Test Once Now" for immediate speed testing without starting monitoring
4. **Watch Status** - Monitor real-time status with countdown timers and running indicators
5. **View Your Data** - See current speeds, median statistics, charts, and detailed history
6. **Manage Data** - Export results, clear history, or manage debug logs as needed

## 🎯 Perfect For

- **Home Users** - Monitor your ISP's actual performance vs. promised speeds
- **Remote Workers** - Ensure stable internet for video calls and productivity
- **Gamers** - Track connection quality for optimal gaming performance  
- **Content Creators** - Monitor upload speeds for streaming and file sharing
- **IT Professionals** - Document network issues with historical data
- **Students** - Ensure reliable internet for online classes and research

## 📋 What You'll See

- **Current Speed Stats** - Live download speed, upload speed, and ping measurements
- **Median Performance** - Median statistics showing your typical internet performance over time
- **Smart Status Display** - Real-time indicators showing monitoring status with visual feedback
- **Next Test Countdown** - Timer showing exactly when your next automatic test will run
- **Test Running Indicator** - Visual feedback when speed tests are actively running
- **Historical Charts** - Trend graphs showing your internet performance over time
- **Recent Test Results** - Detailed table of your most recent speed tests with server information
- **Export & Management** - Save data as CSV files and manage your test history and debug logs

## 🔒 Privacy & Security

- **Local Storage Only** - All data stays on your computer
- **No Tracking** - We don't collect any personal information
- **Open Source** - Full transparency with public source code
- **Secure Testing** - Uses Netflix's Fast.com service for reliable speed measurements

## 🛠️ System Requirements

- **Windows**: Windows 10 or later
- **macOS**: macOS 10.15 (Catalina) or later
- **Linux**: Most modern distributions (Ubuntu 18.04+, etc.)
- **Memory**: 100MB RAM
- **Storage**: 50MB free space

## 📞 Support & Help

Having issues? We're here to help!

1. **Check Common Solutions** - Most issues are covered in our [troubleshooting guide](CONTRIBUTING.md#troubleshooting)
2. **Report Bugs** - Open an [issue on GitHub](https://github.com/omrilotan/internet-speed-monitor/issues) with details
3. **Request Features** - We love hearing your ideas for improvements!

## 🤝 Contributing

Want to help make Internet Speed Monitor even better? We welcome contributions!

- **Report Bugs** - Found something broken? Let us know!
- **Suggest Features** - Have ideas for improvements? Share them!
- **Contribute Code** - Check our [Contributing Guide](CONTRIBUTING.md) for technical details

## 📜 License

This project is released into the public domain under the UNLICENSE - feel free to use, modify, and distribute without any restrictions!

---

**Download now and take control of your internet monitoring!** 🌐✨
