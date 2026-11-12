#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::{Arc, Mutex};

use internet_speed_monitor::instance::{self, Instance};
use internet_speed_monitor::{APP_NAME, CURRENT_VERSION, data_store, launch, logger, paths, shell};

fn main() -> eframe::Result {
    let paths = paths::AppPaths::resolve();
    logger::init(paths.debug_log.clone());

    // The login item starts the app in the background; the user starts it to see it.
    let background = launch::started_in_background();
    let listener = match instance::acquire(&paths.data_dir, !background) {
        Instance::Primary(listener) => listener,
        Instance::AlreadyRunning => {
            log::info!("{APP_NAME} is already running; left it to that copy");
            return Ok(());
        }
    };

    log::info!("=== {APP_NAME} v{CURRENT_VERSION} STARTED ===");
    log::info!(
        "Platform: {} {}",
        std::env::consts::OS,
        std::env::consts::ARCH
    );
    log::info!("Data directory: {}", paths.data_dir.display());
    log::info!("Log file location: {}", paths.debug_log.display());

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_name("ism-worker")
        .build()
        .expect("failed to start async runtime");

    let store = match data_store::DataStore::open(paths.speed_tests_file.clone()) {
        Ok(store) => store,
        Err(error) => {
            log::error!("Failed to open data store: {error}");
            data_store::DataStore::empty(paths.speed_tests_file.clone())
        }
    };
    let store = Arc::new(Mutex::new(store));

    let result = shell::run(shell::Launch {
        handle: runtime.handle().clone(),
        paths,
        store,
        listener,
        background,
    });

    log::info!("=== APP EXIT ===");
    runtime.shutdown_timeout(std::time::Duration::from_secs(2));
    result
}
