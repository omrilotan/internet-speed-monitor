//! Render the app headlessly to a PNG. Used to check layout without a display and to
//! produce the README screenshots.
//!
//! ```sh
//! ISM_DATA_DIR=/path/to/data cargo run --example screenshot -- \
//!     out.png [width height] [scene] [--scale N] [--crop x,y,w,h]
//! ```
//!
//! - `width` / `height`: window size in points (default 1200 x 1400).
//! - `--scale`: pixels per point (default 1; use 2 for crisp README images).
//! - `--crop`: keep only this rectangle, given in points.
//!
//! Scenes:
//! - `main` (default): window as it opens.
//! - `cron`: cron schedule controls.
//! - `sleeping`: cron schedule started, waiting for the next run (no network test).
//! - `monitoring`: interval schedule started; waits for the first real speed test.
//! - `menu`: the ☰ options menu (`cron-menu`: same, over the cron controls).
//! - `export`, `clear`, `about`: the matching dialogs (`clear` also clicks Preview).
//! - `help`: the Help menu.
//! - `last7`: chart in "Last 7 days" mode.
//! - `range`: chart in "Date range" mode with the full range applied.

use std::sync::{Arc, Mutex};

use egui_kittest::kittest::Queryable as _;
use internet_speed_monitor::{app, data_store, paths};

struct Options {
    out: String,
    width: f32,
    height: f32,
    scene: String,
    scale: f32,
    crop: Option<[f32; 4]>,
}

fn parse_args() -> Options {
    let mut positional = Vec::new();
    let mut scale = 1.0;
    let mut crop = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--scale" => {
                scale = args
                    .next()
                    .and_then(|v| v.parse().ok())
                    .expect("--scale needs a number");
            }
            "--crop" => {
                let values: Vec<f32> = args
                    .next()
                    .expect("--crop needs x,y,w,h")
                    .split(',')
                    .map(|v| v.trim().parse().expect("--crop values must be numbers"))
                    .collect();
                assert_eq!(values.len(), 4, "--crop needs exactly x,y,w,h");
                crop = Some([values[0], values[1], values[2], values[3]]);
            }
            _ => positional.push(arg),
        }
    }
    let mut positional = positional.into_iter();
    Options {
        out: positional
            .next()
            .unwrap_or_else(|| "screenshot.png".to_string()),
        width: positional
            .next()
            .and_then(|v| v.parse().ok())
            .unwrap_or(1200.0),
        height: positional
            .next()
            .and_then(|v| v.parse().ok())
            .unwrap_or(1400.0),
        scene: positional.next().unwrap_or_else(|| "main".to_string()),
        scale,
        crop,
    }
}

fn main() {
    let options = parse_args();

    let runtime = tokio::runtime::Runtime::new().expect("runtime");
    let paths = paths::AppPaths::resolve();
    let store = data_store::DataStore::open(paths.speed_tests_file.clone())
        .unwrap_or_else(|_| data_store::DataStore::empty(paths.speed_tests_file.clone()));
    let store = Arc::new(Mutex::new(store));
    let handle = runtime.handle().clone();

    let mut app: Option<app::App> = None;
    let mut harness = egui_kittest::Harness::builder()
        .with_size(egui::Vec2::new(options.width, options.height))
        .with_pixels_per_point(options.scale)
        .wgpu()
        .build_ui(move |ui| {
            // Fonts registered by `App::new` only apply from the next frame on, so the
            // frame that creates the app draws nothing (eframe creates the app before
            // its first frame, so the real binary never hits this).
            match app.as_mut() {
                Some(app) => app.frame(ui),
                None => {
                    app = Some(app::App::new(
                        ui.ctx(),
                        handle.clone(),
                        paths.clone(),
                        Arc::clone(&store),
                    ));
                }
            }
        });
    harness.run_steps(4);

    let select_cron = |harness: &mut egui_kittest::Harness<'_>| {
        harness
            .get_by(|node| node.role() == egui::accesskit::Role::ComboBox)
            .click();
        harness.run_steps(2);
        harness.get_by_label("Cron").click();
        harness.run_steps(2);
    };

    match options.scene.as_str() {
        "menu" => harness.get_by_label("☰").click(),
        "cron-menu" => {
            select_cron(&mut harness);
            harness.get_by_label("☰").click();
        }
        "export" => {
            harness.get_by_label("☰").click();
            harness.run_steps(2);
            harness.get_by_label_contains("Export as CSV").click();
        }
        "clear" => {
            harness.get_by_label("☰").click();
            harness.run_steps(2);
            harness.get_by_label_contains("Clear Data Until").click();
            harness.run_steps(2);
            harness.get_by_label("Preview").click();
        }
        "about" => {
            harness.get_by_label("Help").click();
            harness.run_steps(2);
            harness.get_by_label("About Internet Speed Monitor").click();
        }
        "help" => harness.get_by_label("Help").click(),
        "cron" => select_cron(&mut harness),
        "sleeping" => {
            select_cron(&mut harness);
            harness
                .get_by_label("Start automatic speed testing")
                .click();
        }
        "last7" => harness.get_by_label("Last 7 days").click(),
        "range" => {
            harness.get_by_label("Date range").click();
            harness.run_steps(2);
            harness.get_by_label("Apply").click();
        }
        "monitoring" => {
            harness
                .get_by_label("Start automatic speed testing")
                .click();
            // Let the first (real) speed test finish: roughly 15 seconds of wall time.
            for _ in 0..30 {
                std::thread::sleep(std::time::Duration::from_millis(500));
                harness.step();
            }
        }
        _ => {}
    }
    harness.run_steps(2);
    // Hide the painted mouse cursor and hover highlights, except where the hover is the
    // point of the picture.
    if !matches!(options.scene.as_str(), "menu" | "cron-menu" | "help") {
        harness.remove_cursor();
    }
    harness.run_steps(3);

    let mut image = harness.render().expect("render");
    if let Some([x, y, w, h]) = options.crop {
        let s = options.scale;
        image = image::imageops::crop_imm(
            &image,
            (x * s).round() as u32,
            (y * s).round() as u32,
            (w * s).round() as u32,
            (h * s).round() as u32,
        )
        .to_image();
    }
    image.save(&options.out).expect("save png");
    println!(
        "wrote {} ({}x{})",
        options.out,
        image.width(),
        image.height()
    );
}
