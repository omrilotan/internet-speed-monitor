# Internet Speed Monitor

This is a native desktop application written in Rust that monitors internet connectivity speed at set intervals and stores the data locally.

## Project Structure
- `src/main.rs` — executable entry point (window setup, async runtime, logging)
- `src/lib.rs` — library crate root exposing the modules below
- `src/app/` — egui user interface (`mod.rs` state and actions, `view.rs` main layout, `dialogs.rs` modals)
- `src/monitor.rs` — background scheduler that runs tests and reports events to the UI
- `src/speedtest.rs` — Fast.com download measurement, TCP ping, result assembly
- `src/data_store.rs` — `speed_tests.json` persistence, queries, CSV export
- `src/schedule.rs` — interval and cron schedules
- `src/settings.rs`, `src/paths.rs`, `src/logger.rs`, `src/updates.rs`, `src/launch.rs`, `src/network.rs`, `src/isp.rs`

## Development Guidelines
- Keep the JSON file formats (`speed_tests.json`, `settings.json`) backwards compatible
- Network and file work runs on the tokio runtime; the UI thread only reads shared state and drains the event channel
- Run `cargo fmt`, `cargo clippy --all-targets -- -D warnings` and `cargo test` before pushing
- Bump the version in `Cargo.toml` to publish a release; CI tags and builds when the tag does not exist yet

## Key Features
- Periodic internet speed testing (interval or cron schedule)
- Data persistence and retrieval
- Real-time speed monitoring display
- Historical data visualization
