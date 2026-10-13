//! Locations of the files the app owns on disk.
//!
//! The data directory matches the one the previous Electron build used
//! (`app.getPath('userData')`), so existing history and settings carry over:
//!
//! - macOS: `~/Library/Application Support/internet-speed-monitor`
//! - Windows: `%APPDATA%\internet-speed-monitor`
//! - Linux: `$XDG_CONFIG_HOME/internet-speed-monitor` (defaults to `~/.config/...`)
//!
//! Set `ISM_DATA_DIR` to override the directory (handy for development).

use std::path::PathBuf;

pub const DATA_DIR_NAME: &str = "internet-speed-monitor";

#[derive(Debug, Clone)]
pub struct AppPaths {
    pub data_dir: PathBuf,
    pub speed_tests_file: PathBuf,
    pub settings_file: PathBuf,
    pub debug_log: PathBuf,
}

impl AppPaths {
    pub fn resolve() -> Self {
        if let Some(dir) = std::env::var_os("ISM_DATA_DIR").filter(|v| !v.is_empty()) {
            return Self::in_dir(PathBuf::from(dir));
        }
        let base = dirs::config_dir()
            .or_else(dirs::home_dir)
            .unwrap_or_else(|| PathBuf::from("."));
        Self::in_dir(base.join(DATA_DIR_NAME))
    }

    pub fn in_dir(data_dir: PathBuf) -> Self {
        if let Err(error) = std::fs::create_dir_all(&data_dir) {
            eprintln!(
                "Cannot create data directory {}: {error}",
                data_dir.display()
            );
        }
        Self {
            speed_tests_file: data_dir.join("speed_tests.json"),
            settings_file: data_dir.join("settings.json"),
            debug_log: data_dir.join("debug.log"),
            data_dir,
        }
    }
}
