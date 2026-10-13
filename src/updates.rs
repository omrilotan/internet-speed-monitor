//! Version check against the latest GitHub release.

use std::time::Duration;

use serde::Deserialize;

pub const VERSION_CHECK_URL: &str =
    "https://api.github.com/repos/omrilotan/internet-speed-monitor/releases/latest";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateInfo {
    pub latest_version: String,
    pub current_version: String,
    pub check_date: String,
}

#[derive(Deserialize)]
struct Release {
    tag_name: Option<String>,
}

pub async fn fetch_latest_version(client: &reqwest::Client) -> Result<String, String> {
    let response = client
        .get(VERSION_CHECK_URL)
        .header("User-Agent", "Internet-Speed-Monitor-App")
        .header("Accept", "application/vnd.github+json")
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .map_err(|error| error.to_string())?;
    if !response.status().is_success() {
        return Err(format!(
            "HTTP error! status: {}",
            response.status().as_u16()
        ));
    }
    let release: Release = response.json().await.map_err(|error| error.to_string())?;
    let tag = release
        .tag_name
        .ok_or_else(|| "Release has no tag name".to_string())?;
    Ok(normalise_version(&tag))
}

pub fn normalise_version(tag: &str) -> String {
    tag.trim().trim_start_matches(['v', 'V']).to_string()
}

pub fn is_newer_version(latest: &str, current: &str) -> bool {
    let parse = |version: &str| -> Vec<u64> {
        version
            .split('.')
            .map(|part| {
                part.chars()
                    .take_while(|c| c.is_ascii_digit())
                    .collect::<String>()
                    .parse()
                    .unwrap_or(0)
            })
            .collect()
    };
    let latest = parse(latest);
    let current = parse(current);
    for i in 0..latest.len().max(current.len()) {
        let l = latest.get(i).copied().unwrap_or(0);
        let c = current.get(i).copied().unwrap_or(0);
        if l > c {
            return true;
        }
        if l < c {
            return false;
        }
    }
    false
}

/// Returns update info when the latest release is newer than `current`.
pub async fn check_for_updates(
    client: &reqwest::Client,
    current: &str,
) -> Result<Option<UpdateInfo>, String> {
    log::info!("Checking for updates...");
    let latest = fetch_latest_version(client).await?;
    if is_newer_version(&latest, current) {
        log::info!("New version available: {latest} (current: {current})");
        Ok(Some(UpdateInfo {
            latest_version: latest,
            current_version: current.to_string(),
            check_date: chrono::Utc::now().to_rfc3339(),
        }))
    } else {
        log::info!("No updates available. Current version {current} is latest.");
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compares_versions() {
        assert!(is_newer_version("1.5.0", "1.4.9"));
        assert!(is_newer_version("2.0.0", "1.99.99"));
        assert!(!is_newer_version("1.4.9", "1.4.9"));
        assert!(!is_newer_version("1.4.8", "1.4.9"));
        assert!(is_newer_version("1.4.9.1", "1.4.9"));
        assert!(!is_newer_version("1.4", "1.4.0"));
    }

    #[test]
    fn strips_tag_prefix() {
        assert_eq!(normalise_version("v1.4.9"), "1.4.9");
        assert_eq!(normalise_version(" 2.0.0 "), "2.0.0");
    }
}
