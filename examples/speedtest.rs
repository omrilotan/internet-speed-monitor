//! Run one real speed test from the command line and print the result as JSON.
//!
//! ```sh
//! cargo run --example speedtest
//! ```

use internet_speed_monitor::speedtest::{self, FastConfig};

#[tokio::main]
async fn main() {
    let client = speedtest::build_client();
    let started = std::time::Instant::now();
    let result = speedtest::run_speed_test(&client, &FastConfig::default()).await;
    println!("{}", serde_json::to_string_pretty(&result).expect("json"));
    if let Some(error) = &result.error {
        eprintln!("error: {error}");
    }
    eprintln!("took {:.1}s", started.elapsed().as_secs_f64());
}
