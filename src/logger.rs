//! Logging to stdout and to `debug.log` in the data directory.
//!
//! The file format matches the previous build: `<ISO timestamp>: <message>`,
//! with errors prefixed by `ERROR:`.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

use log::{Level, LevelFilter, Log, Metadata, Record};

struct FileLogger {
    file: Mutex<PathBuf>,
}

impl Log for FileLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        // Only our own crate's logs go to the file; dependencies stay on stderr at warn+.
        metadata.target().starts_with("internet_speed_monitor") || metadata.level() <= Level::Warn
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let message = match record.level() {
            Level::Error => format!("ERROR: {}", record.args()),
            Level::Warn => format!("WARN: {}", record.args()),
            _ => record.args().to_string(),
        };

        if record.level() <= Level::Warn {
            eprintln!("{message}");
        } else {
            println!("{message}");
        }

        if !record.target().starts_with("internet_speed_monitor") {
            return;
        }
        let timestamp = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        let line = format!("{timestamp}: {message}\n");
        if let Ok(path) = self.file.lock()
            && let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&*path)
        {
            // Ignore file write errors, same as before.
            let _ = file.write_all(line.as_bytes());
        }
    }

    fn flush(&self) {}
}

pub fn init(path: PathBuf) {
    let logger = Box::new(FileLogger {
        file: Mutex::new(path),
    });
    if log::set_boxed_logger(logger).is_ok() {
        log::set_max_level(LevelFilter::Info);
    }
}

pub fn read(path: &std::path::Path) -> Result<String, String> {
    if !path.exists() {
        return Err("Log file does not exist".to_string());
    }
    std::fs::read_to_string(path).map_err(|error| error.to_string())
}

pub fn clear(path: &std::path::Path) -> Result<(), String> {
    if !path.exists() {
        return Ok(());
    }
    std::fs::write(path, "").map_err(|error| error.to_string())?;
    log::info!("Debug log cleared by user");
    Ok(())
}
