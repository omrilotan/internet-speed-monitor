# Publishing Your Internet Speed Monitor App

## Quick Start - Build for Your Platform

1. **Build for macOS** (if you're on Mac):
   ```bash
   npm run build:mac
   ```

2. **Build for all platforms**:
   ```bash
   npm run build:all
   ```

The built applications will be in the `dist/` folder.

## Publishing Methods

### 1. Direct Distribution
- Build the app using the commands above
- Share the installer files from the `dist/` folder
- Users can download and install directly

### 2. GitHub Releases (Recommended) - AUTOMATED

#### Setup (One-time):
1. **Create a GitHub repository**:
   ```bash
   git init
   git add .
   git commit -m "Initial commit"
   git branch -M main
   git remote add origin https://github.com/omrilotan/internet-speed-monitor.git
   git push -u origin main
   ```

2. **Automated releases are now configured!** 
   - GitHub Actions workflows are included in `.github/workflows/`
   - No additional setup required - GitHub provides the necessary tokens automatically

#### Publishing Methods:

**Method A: Manual Version Release (Recommended)**
1. Go to your GitHub repository
2. Click "Actions" tab
3. Select "Auto Release" workflow
4. Click "Run workflow"
5. Choose version type (patch/minor/major)
6. Click "Run workflow"

This will:
- ✅ Automatically bump the version
- ✅ Update CHANGELOG.md
- ✅ Create a git tag
- ✅ Build for all platforms (macOS, Windows, Linux)
- ✅ Create a GitHub release with all installers

**Method B: Tag-based Release**
```bash
# Create and push a version tag
git tag v1.0.1
git push origin v1.0.1
```

**Method C: Manual Release**
```bash
# Update version manually
npm version patch  # or minor, major
git push
git push --tags
```

#### What Gets Built Automatically:
- **macOS**: `.dmg` installers (Intel + Apple Silicon) + `.zip` files
- **Windows**: `.exe` NSIS installer + portable version
- **Linux**: `.AppImage` + `.deb` packages

#### Auto-Update Support:
The builds include update metadata files (`latest-mac.yml`, `latest.yml`, `latest-linux.yml`) for electron-updater integration.

### 3. Mac App Store

#### Requirements:
- Apple Developer Account ($99/year)
- Code signing certificates
- App Store guidelines compliance

#### Setup:
```json
"mas": {
  "category": "public.app-category.utilities",
  "hardenedRuntime": true,
  "entitlements": "assets/entitlements.mas.plist",
  "entitlementsInherit": "assets/entitlements.mas.inherit.plist"
}
```

### 4. Microsoft Store

#### Requirements:
- Microsoft Developer Account
- Windows Store certification

#### Setup:
```json
"appx": {
  "applicationId": "InternetSpeedMonitor",
  "backgroundColor": "transparent",
  "showNameOnTiles": false
}
```

### 5. Snap Store (Linux)

#### Setup:
```json
"snap": {
  "summary": "Monitor your internet speed",
  "description": "An Electron app that monitors internet connectivity speed at set intervals"
}
```

## Auto-Update Setup

Add auto-updater capability:

```bash
npm install electron-updater
```

Then add to your main.js:
```javascript
const { autoUpdater } = require('electron-updater');

app.whenReady().then(() => {
  autoUpdater.checkForUpdatesAndNotify();
});
```

## Code Signing (For macOS/Windows)

### macOS:
1. Get Apple Developer certificates
2. Add to package.json:
```json
"mac": {
  "identity": "Developer ID Application: Your Name"
}
```

### Windows:
1. Get code signing certificate
2. Add to package.json:
```json
"win": {
  "certificateFile": "path/to/certificate.p12",
  "certificatePassword": "password"
}
```

## Build Commands Summary

| Command | Description |
|---------|-------------|
| `npm run build:mac` | Build for macOS only |
| `npm run build:win` | Build for Windows only |
| `npm run build:linux` | Build for Linux only |
| `npm run build:all` | Build for all platforms |
| `npm run pack` | Create unpacked directory |

## Distribution Checklist

- [ ] Update version in package.json
- [ ] Test the app thoroughly
- [ ] Build for target platforms
- [ ] Test the built installers
- [ ] Create release notes
- [ ] Upload to distribution platform
- [ ] Announce the release

## File Sizes (Approximate)
- macOS DMG: ~150-200MB
- Windows NSIS: ~120-150MB
- Linux AppImage: ~130-170MB

## Next Steps
1. Choose your distribution method
2. Set up the necessary accounts/tokens
3. Run the build commands
4. Test the installers
5. Distribute!
