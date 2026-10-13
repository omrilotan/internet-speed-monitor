//! User settings persisted to `settings.json` (same schema as the Electron build).

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::data_store::lenient;

pub const DEFAULT_INTERVAL_MINUTES: u32 = 5;
pub const DEFAULT_CRON_EXPRESSION: &str = "*/5 * * * *";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    #[serde(default = "default_schedule_type", deserialize_with = "schedule_type")]
    pub schedule_type: String,
    #[serde(default = "default_interval", deserialize_with = "interval")]
    pub interval: u32,
    #[serde(default = "default_cron", deserialize_with = "cron_expression")]
    pub cron_expression: String,
    #[serde(default, deserialize_with = "lenient::bool")]
    pub auto_start: bool,
    #[serde(default, deserialize_with = "lenient::bool")]
    pub launch_at_startup: bool,
    #[serde(default, deserialize_with = "lenient::opt_string")]
    pub last_check_date: Option<String>,
    #[serde(default, deserialize_with = "lenient::opt_string")]
    pub last_checked_version: Option<String>,
    /// Set once the files Electron left in the data folder have been removed.
    #[serde(default, deserialize_with = "lenient::bool")]
    pub legacy_cleanup_done: bool,
    /// Keys this version does not know about, kept so saving never drops them.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

fn schedule_type<'de, D: serde::Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    let value = lenient::opt_string(d)?;
    Ok(match value.as_deref() {
        Some("cron") => "cron".to_string(),
        _ => default_schedule_type(),
    })
}

fn interval<'de, D: serde::Deserializer<'de>>(d: D) -> Result<u32, D::Error> {
    Ok(lenient::u32(d)?
        .filter(|minutes| *minutes >= 1)
        .unwrap_or(DEFAULT_INTERVAL_MINUTES))
}

fn cron_expression<'de, D: serde::Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    Ok(lenient::opt_string(d)?
        .filter(|expression| !expression.trim().is_empty())
        .unwrap_or_else(default_cron))
}

fn default_schedule_type() -> String {
    "interval".to_string()
}
fn default_interval() -> u32 {
    DEFAULT_INTERVAL_MINUTES
}
fn default_cron() -> String {
    DEFAULT_CRON_EXPRESSION.to_string()
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            schedule_type: default_schedule_type(),
            interval: default_interval(),
            cron_expression: default_cron(),
            auto_start: false,
            launch_at_startup: false,
            last_check_date: None,
            last_checked_version: None,
            legacy_cleanup_done: false,
            extra: serde_json::Map::new(),
        }
    }
}

impl Settings {
    pub fn load(path: &Path) -> Self {
        match std::fs::read_to_string(path) {
            Ok(contents) => match serde_json::from_str::<Settings>(&contents) {
                Ok(settings) => settings,
                Err(error) => {
                    // Keep the unreadable file: the next save would otherwise replace it.
                    let stamp = chrono::Utc::now().format("%Y%m%d-%H%M%S");
                    let backup = path.with_file_name(format!("settings.unreadable-{stamp}.json"));
                    let _ = std::fs::copy(path, &backup);
                    log::error!(
                        "Error loading settings: {error}. Kept a copy at {}",
                        backup.display()
                    );
                    Settings::default()
                }
            },
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Settings::default(),
            Err(error) => {
                log::error!("Error loading settings: {error}");
                Settings::default()
            }
        }
    }

    pub fn save(&self, path: &Path) -> bool {
        let json = match serde_json::to_string_pretty(self) {
            Ok(json) => json,
            Err(error) => {
                log::error!("Error serialising settings: {error}");
                return false;
            }
        };
        match std::fs::write(path, json) {
            Ok(()) => {
                log::info!("Settings saved successfully");
                true
            }
            Err(error) => {
                log::error!("Error saving settings: {error}");
                false
            }
        }
    }

    /// Whether the daily update check is due.
    pub fn should_check_for_updates(&self, now: chrono::DateTime<chrono::Utc>) -> bool {
        let Some(last) = self.last_check_date.as_deref() else {
            return true;
        };
        match chrono::DateTime::parse_from_rfc3339(last) {
            Ok(last) => (now - last.with_timezone(&chrono::Utc)) >= chrono::Duration::days(1),
            Err(_) => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_previous_build() {
        let settings = Settings::default();
        assert_eq!(settings.schedule_type, "interval");
        assert_eq!(settings.interval, 5);
        assert_eq!(settings.cron_expression, "*/5 * * * *");
        assert!(!settings.auto_start);
    }

    #[test]
    fn parses_legacy_file() {
        let json = r#"{
            "scheduleType": "cron",
            "interval": 15,
            "cronExpression": "0 * * * *",
            "autoStart": true,
            "launchAtStartup": false,
            "lastCheckDate": "2026-10-01T10:00:00.000Z",
            "lastCheckedVersion": "1.4.9"
        }"#;
        let settings: Settings = serde_json::from_str(json).unwrap();
        assert_eq!(settings.schedule_type, "cron");
        assert_eq!(settings.interval, 15);
        assert!(settings.auto_start);
        assert_eq!(settings.last_checked_version.as_deref(), Some("1.4.9"));
    }

    #[test]
    fn tolerates_loose_values_and_keeps_unknown_keys() {
        let json = r#"{
            "scheduleType": null,
            "interval": "15",
            "cronExpression": "",
            "autoStart": "true",
            "launchAtStartup": null,
            "lastCheckDate": null,
            "windowBounds": {"width": 900}
        }"#;
        let settings: Settings = serde_json::from_str(json).unwrap();
        assert_eq!(settings.schedule_type, "interval");
        assert_eq!(settings.interval, 15);
        assert_eq!(settings.cron_expression, "*/5 * * * *");
        assert!(settings.auto_start);
        assert!(!settings.launch_at_startup);
        let out = serde_json::to_value(&settings).unwrap();
        assert_eq!(out["windowBounds"]["width"], 900);
        assert_eq!(out["scheduleType"], "interval");
    }

    #[test]
    fn update_check_is_daily() {
        let mut settings = Settings::default();
        let now = chrono::Utc::now();
        assert!(settings.should_check_for_updates(now));
        settings.last_check_date = Some((now - chrono::Duration::hours(2)).to_rfc3339());
        assert!(!settings.should_check_for_updates(now));
        settings.last_check_date = Some((now - chrono::Duration::hours(25)).to_rfc3339());
        assert!(settings.should_check_for_updates(now));
        settings.last_check_date = Some("garbage".to_string());
        assert!(settings.should_check_for_updates(now));
    }
}
