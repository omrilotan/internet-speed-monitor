# Contributing to Internet Speed Monitor

Thank you for your interest in contributing! This document describes how the application is put together, how to build and test it, and how releases are made.

## Project Structure

```
internet-speed-monitor/
├── Cargo.toml                  # Package manifest, dependencies, bundle metadata
├── src/
│   ├── main.rs                 # Executable entry point: window, async runtime, logging
│   ├── lib.rs                  # Library crate root (modules below, shared constants)
│   ├── app/
│   │   ├── mod.rs              # Application state, event handling, user actions
│   │   ├── view.rs             # Main window layout, styling, chart and table
│   │   ├── dialogs.rs          # Modal dialogs (alerts, export, clear data, about, debug log)
│   │   └── format.rs           # Date/number formatting helpers
│   ├── monitor.rs              # Background scheduler and test runner
│   ├── speedtest.rs            # Fast.com download measurement, TCP ping, result assembly
│   ├── schedule.rs             # Interval and cron schedules, descriptions, presets
│   ├── data_store.rs           # speed_tests.json persistence, queries, CSV export
│   ├── settings.rs             # settings.json persistence
│   ├── paths.rs                # Data directory resolution
│   ├── logger.rs               # stdout + debug.log logging
│   ├── updates.rs              # GitHub release version check
│   ├── launch.rs               # "Start with computer" (login item)
│   ├── network.rs              # WiFi / Ethernet detection
│   └── isp.rs                  # ISP lookup (ipify + ip-api)
├── examples/
│   ├── screenshot.rs           # Headless render of the UI to PNG (uses egui_kittest)
│   └── speedtest.rs            # Run one speed test from the terminal
├── fonts/                      # Bundled Inter font (SIL OFL 1.1)
├── assets -> docs/assets       # App icons (icns/png/svg)
├── docs/                       # Project website (GitHub Pages)
└── .github/workflows/          # CI: checks, multi-platform builds, releases
```

## Technical Architecture

### Rust + egui

The app is a single native process written in Rust:

- **[eframe/egui](https://github.com/emilk/egui)** draws the user interface (immediate mode, OpenGL via `glow`).
- **[egui_plot](https://github.com/emilk/egui_plot)** renders the speed chart; **egui_extras** provides the table and date pickers.
- **[tokio](https://tokio.rs/)** runs network and scheduling work on a background runtime so the UI thread never blocks.
- **[reqwest](https://github.com/seanmonstar/reqwest)** performs HTTP requests (Fast.com, ipify, ip-api, GitHub).
- **[cron](https://github.com/zslayton/cron)** parses cron expressions; **chrono** and **jiff** handle time.
- **rfd** shows the native save dialog, **open** opens URLs and folders, **auto-launch** manages login items, **dirs** finds the data directory.

### Threading model

- `main.rs` builds a multi-threaded tokio runtime and hands its `Handle` to the app.
- `monitor::SpeedMonitor` spawns a scheduler task per monitoring session. The task sleeps until the next due time (interval: run now, then every N minutes; cron: next matching boundary in local time), runs the test, saves it, and sends `monitor::Event`s over a `std::sync::mpsc` channel.
- The UI (`app::App::frame`) drains the channel at the start of every frame, updates its state, and requests a repaint every second so clocks stay fresh.
- The data store is shared as `Arc<Mutex<DataStore>>`; writes happen on the background task, reads on the UI thread.

### Speed testing (`src/speedtest.rs`)

The download measurement mirrors the `fast-speedtest-api` npm package used by the previous (Electron) version:

1. Fetch 5 target URLs from `https://api.fast.com/netflix/speedtest?https=true&token=…&urlCount=5`.
2. Download all targets concurrently, counting bytes in an atomic counter.
3. Every 200 ms sample bytes/second into a ring buffer of 8 samples.
4. Stop after 10 seconds or as soon as the first target finishes; report the average of the non-zero samples in Mbps.

Other fields:

- **Upload** is estimated as 10% of download (Fast.com's public API only measures download), as before.
- **Ping** is the median of three TCP connect round-trips to a Fast.com edge host. The previous version only simulated this number.
- **Network interface** uses `route`/`networksetup` (macOS), `netsh` (Windows), `ip route`/`iwconfig` (Linux).
- **ISP** is looked up via ipify then ip-api with a 5 second budget.

Failed tests are stored as zero-value records with `server: "Error"` so they show in history and charts.

### Scheduling (`src/schedule.rs`)

- `Schedule::Interval { minutes }` – first test immediately, then every N minutes.
- `Schedule::Cron { expression }` – standard 5-field expressions (`minute hour day month weekday`), evaluated in local time. Validation requires exactly 5 fields and a parseable expression.
- `classify_cron` provides the description shown under the input (known patterns, custom, invalid).
- Interval stepping sequence for the arrow keys / scroll wheel / ± buttons: 1, 5, 10, 15, 30, 60.

### Data storage (`src/data_store.rs`)

#### Storage format

`speed_tests.json` keeps the exact shape the Electron version used, so existing history carries over:

```json
{
  "speedTests": [
    {
      "id": 1760383548000,
      "timestamp": "2026-10-13T19:25:48.000Z",
      "download": 93.14,
      "upload": 9.31,
      "ping": 5.7,
      "server": "Netflix Fast.com",
      "isp": "Example ISP",
      "networkInterface": "WiFi",
      "created_at": "2026-10-13T19:25:48.000Z"
    }
  ]
}
```

Records are kept newest-first and written atomically (temp file + rename).

#### Storage location

| Platform | Directory |
|---|---|
| macOS | `~/Library/Application Support/internet-speed-monitor` |
| Windows | `%APPDATA%\internet-speed-monitor` |
| Linux | `$XDG_CONFIG_HOME/internet-speed-monitor` (defaults to `~/.config/…`) |

Files: `speed_tests.json`, `settings.json`, `debug.log`. Set `ISM_DATA_DIR=/some/dir` to use another directory (handy for development and screenshots).

#### Settings

`settings.json` (camelCase, unchanged from before): `scheduleType`, `interval`, `cronExpression`, `autoStart`, `launchAtStartup`, `lastCheckDate`, `lastCheckedVersion`.

### User interface (`src/app/`)

- `view.rs` draws the menu bar (File / View / Help), header with status and ☰ menu, schedule controls, the four stat cards, the chart with its view modes, and the recent tests table.
- `dialogs.rs` draws modals: alerts, clear-all confirmation, "Clear Data Until…" with preview, CSV export with date range, About (with "Check for Updates"), and the debug log window.
- `install_style` sets a light theme that matches the previous CSS (colours, card shadows, Inter font) and registers the `bold` / `semibold` font families.
- The chart smooths lines with the same cardinal-spline tension (0.4) Chart.js used, draws point markers, and places grid lines on clock boundaries.

## Development Setup

### Prerequisites

- Rust stable 1.88+ via [rustup](https://rustup.rs/)
- Linux only: `libgtk-3-dev libxkbcommon-dev libwayland-dev libssl-dev pkg-config`
- Optional: `cargo install cargo-bundle` for `.app` / `.deb` bundles

### Common commands

```bash
cargo run                      # run in debug mode
cargo run --release            # optimised build
cargo test                     # unit tests
cargo fmt                      # format
cargo clippy --all-targets -- -D warnings
cargo bundle --release         # macOS .app in target/release/bundle/

# Run one real speed test and print JSON
cargo run --example speedtest

# Render the main window headlessly (no display needed) to inspect layout changes
ISM_DATA_DIR=/tmp/ism-data cargo run --example screenshot -- out.png 1200 1400 main
# scenes: main, cron, sleeping, monitoring, menu, cron-menu, export, clear, about, help, last7, range
# options: --scale 2 (crisp output), --crop x,y,w,h (in points)
```

### Updating the README screenshots

The README images are rendered with the `screenshot` example against a data directory with a few weeks of history. Each run writes `settings.json` into that directory, so start every render from a fresh copy of the data and a `settings.json` with a recent `lastCheckDate` (this skips the update check). The commands used for the current images:

```bash
run() { ISM_DATA_DIR=/tmp/ism-data cargo run -q --example screenshot -- "$@"; }
run docs/screenshot.png 1200 1000 sleeping --scale 2
run docs/screenshots/history-last-7-days.png 1200 1000 last7 --scale 2 --crop 28,420,1144,485
run docs/screenshots/recent-tests.png 1200 1560 main --scale 2 --crop 28,905,1144,424
run docs/screenshots/options-menu.png 1200 1000 cron-menu --scale 2 --crop 540,48,652,292
run docs/screenshots/export-csv.png 640 620 export --scale 2
run docs/screenshots/clear-data.png 640 620 clear --scale 2
```

The crop rectangles assume an interval schedule in `settings.json`; adjust them if the layout changes.

### Logging

Everything logged with `log::info!` / `log::error!` goes to stdout and to `debug.log` in the data directory (`<timestamp>: message`, errors prefixed with `ERROR:`). Help → Show Debug Log displays it in-app.

## Build Configuration

- **macOS**: `[package.metadata.bundle]` in `Cargo.toml` configures `cargo-bundle`: product name, bundle identifier `com.omrilotan.internet-speed-monitor` (the same as 1.x, so the new app replaces the old one), icons from `docs/assets/`, category and minimum macOS version.
- **Windows**: `build.rs` embeds `docs/assets/icon.ico` and version info in the exe. `scripts/package-windows/installer.nsi` builds the NSIS installer (see the comments at the top for the `makensis` command).
- **Linux**: `scripts/package-deb/procedure.sh <binary> <version> <arch> <out-dir>` builds the `.deb` with `dpkg-deb`.
- The release profile uses LTO and strips symbols for a small binary.

Icons are referenced through `docs/assets/`, not the `assets` symlink, because Windows checkouts do not always create symlinks.

### Upgrading from 1.x (Electron)

Installing 2.x must replace 1.x and keep the user's data. What makes that work:

| | 1.x left behind | 2.x handling |
|---|---|---|
| Data | `speed_tests.json`, `settings.json`, `debug.log` in the `internet-speed-monitor` user data folder | Same folder (`paths.rs`). Parsing is lenient and lossless (`data_store::lenient`, unknown fields kept, unreadable records written back). A file without the `writtenBy` marker is copied to `speed_tests.backup-1.x.json` once. An unparseable file is copied aside before anything can overwrite it. |
| Electron caches | `Cache`, `GPUCache`, `Local Storage`, `Preferences`, … in the same folder | Removed once on first launch (`legacy.rs`), never the data files. |
| macOS app | `/Applications/Internet Speed Monitor.app`, bundle id `com.omrilotan.internet-speed-monitor` | Same name and bundle id: the DMG replaces it. |
| macOS login item | SMAppService registration | Reported as on; removed when the user toggles the setting (`launch.rs`). |
| Windows install | electron-builder NSIS, uninstall key `7e80438c-3e51-5232-9119-5f227d0b6b72` (UUID v5 of the app id), `Uninstall Internet Speed Monitor.exe` | The installer runs that uninstaller silently (it keeps user data) after backing up the history. |
| Windows autostart | `HKCU\…\Run` value pointing at `Internet Speed Monitor.exe` | Replaced by the 2.x value `Internet Speed Monitor`. |
| Linux .deb | Package `internet-speed-monitor`, files in `/opt/Internet Speed Monitor`, `/usr/bin` link via `update-alternatives` | Same package name, so it upgrades in place. `preinst` removes the alternatives link group first. |

The unit tests in `data_store.rs`, `settings.rs` and `legacy.rs` cover the data side with 1.x-format fixtures.

## CI/CD Pipeline

`.github/workflows/automated-release.yml` runs on pushes and pull requests to `main`:

1. **check-version-and-release** (Ubuntu): `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test`, then compares the `Cargo.toml` version with existing git tags.
2. **build** (only when the tag is missing): builds release binaries for
   - macOS arm64 and x64 (`.dmg` + `.zip` of the `.app` bundle),
   - Windows x64 (portable `.exe` + `.zip`),
   - Linux x64 (`.deb` + `.tar.gz`).
3. **release** (main branch only): creates the `vX.Y.Z` tag and a GitHub Release with the artifacts.

### Release process

1. Update `CHANGELOG.md`.
2. Bump `version` in `Cargo.toml` (`Cargo.lock` updates on the next build).
3. Commit and push to `main`. CI tags, builds and publishes automatically.

Builds are unsigned. See the README for the macOS Gatekeeper workarounds; Windows SmartScreen may also warn on first run.

## Adding Features

- **New data in results**: extend `data_store::SpeedTest` (keep serde names backward compatible), fill it in `speedtest::run_speed_test`, show it in `view.rs`.
- **New schedule types**: add a variant to `schedule::Schedule` and implement `next_run_after`, `validate` and the UI controls.
- **New dialogs**: add state to `App`, draw it in `dialogs.rs` with `egui::Modal`, and open it from a menu or button.
- Add unit tests next to the code (`#[cfg(test)] mod tests`) for anything with logic.

## Troubleshooting

### Build failures

```bash
rustup update stable
cargo clean && cargo build
```

On Linux, missing system libraries show up as `pkg-config` or linker errors; install the packages listed under Prerequisites.

### Speed test failures

- Verify internet connectivity and that `https://api.fast.com` is reachable.
- Run `cargo run --example speedtest` to see the raw result and error.
- Check `debug.log` (Help → Show Debug Log).

### Blank window or GPU errors

eframe uses OpenGL through `glow`. On Linux make sure Mesa/GPU drivers are installed; on remote desktops try `WINIT_UNIX_BACKEND=x11`.

### "Start with computer" does not stick

The login item points at the executable that was running when the toggle was enabled. If you move the app, disable and re-enable the toggle.

## Version Management

This project follows [Semantic Versioning](https://semver.org/):

- **MAJOR**: breaking changes (2.0.0 was the Rust rewrite)
- **MINOR**: new features (backward compatible)
- **PATCH**: bug fixes (backward compatible)

## Code Style

- `cargo fmt` formatting, `cargo clippy` clean with `-D warnings`.
- Prefer small modules with unit tests; keep the UI thread free of blocking work.
- Keep the on-disk JSON formats compatible with existing user data.
