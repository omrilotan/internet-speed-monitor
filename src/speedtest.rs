//! Speed measurement against Netflix's Fast.com.
//!
//! The download algorithm mirrors the `fast-speedtest-api` package the Electron build
//! used: fetch `urlCount` target URLs, download them all in parallel, sample the byte
//! counter every `interval` and report the average of the last `buffer_size` samples,
//! stopping at `timeout` or as soon as the first target finishes downloading.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use serde::Deserialize;
use tokio::sync::Notify;

use crate::data_store::SpeedTest;
use crate::isp;
use crate::network;

pub const SERVER_NAME: &str = "Netflix Fast.com";
pub const FAST_TOKEN: &str = "YXNkZmFzZGxmbnNkYWZoYXNkZmhrYWxm";
pub const USER_AGENT: &str = concat!("Internet-Speed-Monitor-App/", env!("CARGO_PKG_VERSION"));

#[derive(Debug, Clone)]
pub struct FastConfig {
    pub timeout: Duration,
    pub url_count: usize,
    pub buffer_size: usize,
}

impl Default for FastConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_millis(10_000),
            url_count: 5,
            buffer_size: 8,
        }
    }
}

#[derive(Debug, Deserialize)]
struct Target {
    url: String,
}

#[derive(Debug)]
pub struct SpeedTestError(pub String);

impl std::fmt::Display for SpeedTestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for SpeedTestError {}

impl From<reqwest::Error> for SpeedTestError {
    fn from(error: reqwest::Error) -> Self {
        SpeedTestError(error.to_string())
    }
}

pub fn build_client() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .build()
        .expect("HTTP client")
}

pub async fn fetch_targets(
    client: &reqwest::Client,
    url_count: usize,
) -> Result<Vec<String>, SpeedTestError> {
    let mut targets: Vec<String> = Vec::new();
    while targets.len() < url_count {
        let remaining = url_count - targets.len();
        let url = format!(
            "https://api.fast.com/netflix/speedtest?https=true&token={FAST_TOKEN}&urlCount={remaining}"
        );
        let response = client
            .get(&url)
            .timeout(Duration::from_secs(10))
            .send()
            .await?;
        match response.status().as_u16() {
            200 => {}
            403 => return Err(SpeedTestError("Fast.com rejected the app token".into())),
            status => {
                return Err(SpeedTestError(format!(
                    "Fast.com API returned HTTP {status}"
                )));
            }
        }
        let batch: Vec<Target> = response.json().await?;
        if batch.is_empty() {
            return Err(SpeedTestError("Fast.com returned no test targets".into()));
        }
        targets.extend(batch.into_iter().map(|target| target.url));
    }
    Ok(targets)
}

/// Download speed in megabits per second.
pub async fn measure_download(
    client: &reqwest::Client,
    config: &FastConfig,
) -> Result<f64, SpeedTestError> {
    let targets = fetch_targets(client, config.url_count).await?;
    let bytes = Arc::new(AtomicU64::new(0));
    let finished = Arc::new(Notify::new());

    let mut handles = Vec::with_capacity(targets.len());
    for url in targets {
        let client = client.clone();
        let bytes = Arc::clone(&bytes);
        let finished = Arc::clone(&finished);
        handles.push(tokio::spawn(async move {
            let Ok(mut response) = client.get(&url).send().await else {
                return;
            };
            loop {
                match response.chunk().await {
                    Ok(Some(chunk)) => {
                        bytes.fetch_add(chunk.len() as u64, Ordering::Relaxed);
                    }
                    Ok(None) => {
                        // First target fully downloaded: stop the whole measurement.
                        finished.notify_one();
                        return;
                    }
                    Err(_) => return,
                }
            }
        }));
    }

    let interval = (config.timeout / config.buffer_size as u32).min(Duration::from_millis(200));
    let mut recents: Vec<Option<f64>> = vec![None; config.buffer_size.max(1)];
    let mut index = 0usize;
    let started = Instant::now();
    let deadline = started + config.timeout;

    loop {
        let tick = tokio::time::sleep(interval);
        tokio::select! {
            _ = tick => {
                index = (index + 1) % recents.len();
                let count = bytes.swap(0, Ordering::Relaxed);
                recents[index] = Some(count as f64 / interval.as_secs_f64());
                if Instant::now() >= deadline {
                    break;
                }
            }
            _ = finished.notified() => break,
            _ = tokio::time::sleep_until(tokio::time::Instant::from_std(deadline)) => break,
        }
    }

    for handle in handles {
        handle.abort();
    }

    Ok(bytes_per_second_to_mbps(average(&recents)))
}

pub fn average(values: &[Option<f64>]) -> f64 {
    let present: Vec<f64> = values
        .iter()
        .flatten()
        .copied()
        .filter(|v| *v > 0.0)
        .collect();
    if present.is_empty() {
        0.0
    } else {
        present.iter().sum::<f64>() / present.len() as f64
    }
}

pub fn bytes_per_second_to_mbps(bytes_per_second: f64) -> f64 {
    bytes_per_second * 8.0 / 1_000_000.0
}

/// Round-trip latency to a Fast.com edge in milliseconds, measured as the median of
/// three TCP connects. The Electron build only simulated this number.
pub async fn measure_ping(target_url: &str) -> Option<f64> {
    let url = reqwest::Url::parse(target_url).ok()?;
    let host = url.host_str()?.to_string();
    let port = url.port_or_known_default()?;
    let mut samples = Vec::new();
    for _ in 0..3 {
        let started = Instant::now();
        let connect = tokio::net::TcpStream::connect((host.as_str(), port));
        if let Ok(Ok(_stream)) = tokio::time::timeout(Duration::from_secs(3), connect).await {
            samples.push(started.elapsed().as_secs_f64() * 1000.0);
        }
    }
    if samples.is_empty() {
        return None;
    }
    samples.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    Some(samples[samples.len() / 2])
}

/// Run a full speed test: interface detection, download, ping, ISP lookup.
pub async fn run_speed_test(client: &reqwest::Client, config: &FastConfig) -> SpeedTest {
    log::info!("Starting speed test...");
    let network_interface = network::detect_network_connection_type().await;
    log::info!("Detected network interface: {network_interface}");

    let download = match measure_download(client, config).await {
        Ok(speed) => speed,
        Err(error) => {
            log::error!("Speed test failed: {error}");
            return SpeedTest {
                id: 0,
                timestamp: now_iso(),
                download: 0.0,
                upload: 0.0,
                ping: 0.0,
                server: Some("Error".to_string()),
                isp: None,
                network_interface: None,
                created_at: String::new(),
                extra: Default::default(),
                error: Some(error.0),
            };
        }
    };

    // Fast.com's public API only measures download; the upload figure is an estimate
    // of 10% of download, as in the previous build.
    let upload = download * 0.1;

    let ping = match fetch_targets(client, 1).await {
        Ok(targets) if !targets.is_empty() => measure_ping(&targets[0]).await.unwrap_or(0.0),
        _ => 0.0,
    };

    let isp_name = isp::lookup_isp(client).await;

    let result = SpeedTest {
        id: 0,
        timestamp: now_iso(),
        download,
        upload,
        ping,
        server: Some(SERVER_NAME.to_string()),
        isp: Some(isp_name),
        network_interface: Some(network_interface),
        created_at: String::new(),
        extra: Default::default(),
        error: None,
    };
    log::info!(
        "Speed test completed: download={:.2} Mbps upload={:.2} Mbps ping={:.0} ms isp={} iface={}",
        result.download,
        result.upload,
        result.ping,
        result.isp.as_deref().unwrap_or("-"),
        result.network_interface.as_deref().unwrap_or("-")
    );
    result
}

pub fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn average_skips_empty_and_zero_samples() {
        assert_eq!(average(&[None, Some(0.0), Some(10.0), Some(20.0)]), 15.0);
        assert_eq!(average(&[None, None]), 0.0);
    }

    #[test]
    fn converts_bytes_to_megabits() {
        assert_eq!(bytes_per_second_to_mbps(1_250_000.0), 10.0);
    }

    #[test]
    fn parses_target_json() {
        let json =
            r#"[{"url":"https://example.net/speedtest?c=gb"},{"url":"https://example.org/x"}]"#;
        let targets: Vec<Target> = serde_json::from_str(json).unwrap();
        assert_eq!(targets.len(), 2);
        assert!(targets[0].url.starts_with("https://example.net"));
    }
}
