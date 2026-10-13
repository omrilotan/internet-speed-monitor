//! Internet Speed Monitor library crate: everything except the executable entry point.
//!
//! The binary in `main.rs` wires this together; examples and tests reuse these modules.

pub mod app;
pub mod data_store;
pub mod isp;
pub mod launch;
pub mod legacy;
pub mod logger;
pub mod monitor;
pub mod network;
pub mod paths;
pub mod schedule;
pub mod settings;
pub mod speedtest;
pub mod updates;

pub const APP_NAME: &str = "Internet Speed Monitor";
pub const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const REPOSITORY_URL: &str = "https://github.com/omrilotan/internet-speed-monitor";
pub const DOCUMENTATION_URL: &str = "https://omrilotan.github.io/internet-speed-monitor/";
pub const RELEASES_URL: &str =
    "https://github.com/omrilotan/internet-speed-monitor/releases/latest";
