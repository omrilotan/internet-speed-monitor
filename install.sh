#!/bin/bash
# Internet Speed Monitor - Easy Install Script for macOS
# This script downloads and installs the latest version

set -e

echo "🚀 Internet Speed Monitor - Easy Installer"
echo "==========================================="

# Get latest release info
LATEST_RELEASE=$(curl -s https://api.github.com/repos/omrilotan/internet-speed-monitor/releases/latest)
DOWNLOAD_URL=$(echo "$LATEST_RELEASE" | grep "browser_download_url.*dmg" | cut -d '"' -f 4)
VERSION=$(echo "$LATEST_RELEASE" | grep '"tag_name"' | cut -d '"' -f 4)

if [ -z "$DOWNLOAD_URL" ]; then
    echo "❌ Error: Could not find download URL"
    exit 1
fi

echo "📦 Downloading Internet Speed Monitor $VERSION..."
TEMP_DMG="/tmp/internet-speed-monitor.dmg"
curl -L -o "$TEMP_DMG" "$DOWNLOAD_URL"

echo "💿 Mounting DMG..."
MOUNT_POINT=$(hdiutil attach "$TEMP_DMG" | grep "/Volumes" | awk '{print $3}')

echo "📁 Installing application..."
cp -R "$MOUNT_POINT/Internet Speed Monitor.app" /Applications/

echo "🧹 Cleaning up..."
hdiutil detach "$MOUNT_POINT"
rm "$TEMP_DMG"

echo "✅ Installation complete!"
echo ""
echo "🛡️  Note: When you first run the app, macOS may show a security dialog."
echo "   Simply right-click the app and select 'Open' to bypass this."
echo ""
echo "🚀 You can now find 'Internet Speed Monitor' in your Applications folder!"