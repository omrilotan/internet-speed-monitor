//! Clean-up after the 1.x (Electron) version, which shared this data folder.
//!
//! Electron stores its browser caches and state next to the app's own files. 2.x does not
//! use any of them, so they are removed once. The history (`speed_tests.json`), settings
//! and debug log are never touched.

use std::path::Path;

/// Files and folders Chromium/Electron creates in the user data folder.
/// None of these names are used by 2.x.
const ELECTRON_ENTRIES: &[&str] = &[
    "Cache",
    "Code Cache",
    "GPUCache",
    "DawnCache",
    "DawnGraphiteCache",
    "DawnWebGPUCache",
    "GraphiteDawnCache",
    "ShaderCache",
    "Local Storage",
    "Session Storage",
    "IndexedDB",
    "WebStorage",
    "databases",
    "blob_storage",
    "Service Worker",
    "Shared Dictionary",
    "SharedStorage",
    "SharedStorage-wal",
    "shared_proto_db",
    "VideoDecodeStats",
    "Crashpad",
    "Dictionaries",
    "Partitions",
    "Network",
    "component_crx_cache",
    "extensions_crx_cache",
    "Cookies",
    "Cookies-journal",
    "DIPS",
    "DIPS-journal",
    "DIPS-wal",
    "Local State",
    "Network Persistent State",
    "Preferences",
    "QuotaManager",
    "QuotaManager-journal",
    "TransportSecurity",
    "Trust Tokens",
    "Trust Tokens-journal",
    "SingletonCookie",
    "SingletonLock",
    "SingletonSocket",
    "lockfile",
];

/// Remove Electron leftovers from `data_dir`. Returns how many entries were removed.
pub fn remove_electron_leftovers(data_dir: &Path) -> usize {
    let mut removed = 0;
    for name in ELECTRON_ENTRIES {
        let path = data_dir.join(name);
        // symlink_metadata: never follow a link out of the data folder.
        let Ok(metadata) = std::fs::symlink_metadata(&path) else {
            continue;
        };
        let result = if metadata.is_dir() {
            std::fs::remove_dir_all(&path)
        } else {
            std::fs::remove_file(&path)
        };
        match result {
            Ok(()) => removed += 1,
            Err(error) => log::error!("Could not remove {}: {error}", path.display()),
        }
    }
    if removed > 0 {
        log::info!("Removed {removed} files and folders left by version 1.x");
    }
    removed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_only_electron_entries() {
        let dir = std::env::temp_dir().join(format!("ism-legacy-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("GPUCache")).unwrap();
        std::fs::write(dir.join("GPUCache").join("data_0"), "x").unwrap();
        std::fs::create_dir_all(dir.join("Local Storage/leveldb")).unwrap();
        std::fs::write(dir.join("Preferences"), "{}").unwrap();
        for keep in [
            "speed_tests.json",
            "settings.json",
            "debug.log",
            "speed_tests.backup-1.x.json",
        ] {
            std::fs::write(dir.join(keep), "keep").unwrap();
        }

        assert_eq!(remove_electron_leftovers(&dir), 3);
        let mut left: Vec<String> = std::fs::read_dir(&dir)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        left.sort();
        assert_eq!(
            left,
            [
                "debug.log",
                "settings.json",
                "speed_tests.backup-1.x.json",
                "speed_tests.json"
            ]
        );
        assert_eq!(remove_electron_leftovers(&dir), 0);
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
