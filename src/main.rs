#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::{Arc, Mutex};

use internet_speed_monitor::{APP_NAME, CURRENT_VERSION, app, data_store, logger, paths};

fn main() -> eframe::Result {
    let paths = paths::AppPaths::resolve();
    logger::init(paths.debug_log.clone());

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

    let icon = eframe::icon_data::from_png_bytes(include_bytes!("../docs/assets/icon-256x256.png"))
        .expect("bundled icon must be a valid PNG");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(APP_NAME)
            .with_app_id("com.omrilotan.internet-speed-monitor")
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([640.0, 480.0])
            .with_icon(icon),
        ..Default::default()
    };

    let handle = runtime.handle().clone();
    let result = eframe::run_native(
        APP_NAME,
        options,
        Box::new(move |cc| Ok(Box::new(app::App::new(&cc.egui_ctx, handle, paths, store)))),
    );

    log::info!("=== APP EXIT ===");
    runtime.shutdown_timeout(std::time::Duration::from_secs(2));
    result
}
