//! "Start with computer" via the OS login-item mechanism.
//!
//! - Windows: `HKCU\...\Run` value named `Internet Speed Monitor`. The 2.x installer
//!   moves a 1.x entry to this name, so the setting carries over.
//! - macOS: a LaunchAgent. Version 1.x (Electron) registered the app with SMAppService;
//!   that registration follows the bundle identifier, so it now starts 2.x. It is
//!   reported as "on" and removed when the user turns the setting off.
//! - Linux: an XDG autostart entry.

use auto_launch::{AutoLaunch, AutoLaunchBuilder};

fn launcher() -> Result<AutoLaunch, String> {
    let exe = std::env::current_exe().map_err(|error| error.to_string())?;
    let app_path = app_path_for(&exe);
    let app_path = app_path.to_string_lossy();
    // The Run value is a command line: quote the path, the install folder has spaces.
    let app_path = if cfg!(windows) {
        format!("\"{app_path}\"")
    } else {
        app_path.into_owned()
    };
    let mut builder = AutoLaunchBuilder::new();
    builder
        .set_app_name(crate::APP_NAME)
        .set_app_path(&app_path);
    #[cfg(target_os = "macos")]
    builder.set_macos_launch_mode(auto_launch::MacOSLaunchMode::LaunchAgent);
    builder.build().map_err(|error| error.to_string())
}

/// The SMAppService login item that version 1.x created (macOS 13+).
#[cfg(target_os = "macos")]
fn legacy_login_item() -> Option<AutoLaunch> {
    let mut builder = AutoLaunchBuilder::new();
    builder
        .set_app_name(crate::APP_NAME)
        .set_app_path("")
        .set_macos_launch_mode(auto_launch::MacOSLaunchMode::SMAppService);
    builder.build().ok()
}

#[cfg(target_os = "macos")]
fn legacy_login_item_enabled() -> bool {
    legacy_login_item()
        .and_then(|item| item.is_enabled().ok())
        .unwrap_or(false)
}

#[cfg(not(target_os = "macos"))]
fn legacy_login_item_enabled() -> bool {
    false
}

#[cfg(target_os = "macos")]
fn disable_legacy_login_item() {
    if legacy_login_item_enabled()
        && let Some(item) = legacy_login_item()
    {
        match item.disable() {
            Ok(()) => log::info!("Removed the login item created by version 1.x"),
            Err(error) => log::error!("Could not remove the 1.x login item: {error}"),
        }
    }
}

#[cfg(not(target_os = "macos"))]
fn disable_legacy_login_item() {}

/// On macOS prefer the `.app` bundle over the binary inside it when available.
fn app_path_for(exe: &std::path::Path) -> std::path::PathBuf {
    if cfg!(target_os = "macos") {
        let mut current = exe;
        while let Some(parent) = current.parent() {
            if parent.extension().is_some_and(|ext| ext == "app") {
                return parent.to_path_buf();
            }
            current = parent;
        }
    }
    exe.to_path_buf()
}

pub fn get_launch_at_startup() -> bool {
    match launcher().and_then(|launch| launch.is_enabled().map_err(|error| error.to_string())) {
        Ok(enabled) => enabled || legacy_login_item_enabled(),
        Err(error) => {
            log::error!("Error getting launch at startup setting: {error}");
            false
        }
    }
}

pub fn set_launch_at_startup(enable: bool) -> Result<(), String> {
    let launch = launcher()?;
    // Never leave two login entries: the 1.x one goes either way.
    disable_legacy_login_item();
    let outcome = if enable {
        launch.enable()
    } else if launch.is_enabled().unwrap_or(true) {
        launch.disable()
    } else {
        Ok(())
    };
    match outcome {
        Ok(()) => {
            log::info!(
                "Launch at startup {}",
                if enable { "enabled" } else { "disabled" }
            );
            Ok(())
        }
        Err(error) => {
            log::error!("Error setting launch at startup: {error}");
            Err(error.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_bundle_on_macos() {
        let exe = std::path::Path::new(
            "/Applications/Internet Speed Monitor.app/Contents/MacOS/internet-speed-monitor",
        );
        let path = app_path_for(exe);
        if cfg!(target_os = "macos") {
            assert_eq!(
                path,
                std::path::Path::new("/Applications/Internet Speed Monitor.app")
            );
        } else {
            assert_eq!(path, exe);
        }
    }
}
