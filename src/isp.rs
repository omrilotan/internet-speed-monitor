//! ISP lookup: public IP via ipify, then ISP name via ip-api. Five second budget overall.

use std::time::Duration;

use serde::Deserialize;

pub const UNKNOWN: &str = "Unknown";

#[derive(Deserialize)]
struct IpResponse {
    ip: Option<String>,
}

#[derive(Deserialize)]
struct IspResponse {
    isp: Option<String>,
}

pub async fn lookup_isp(client: &reqwest::Client) -> String {
    match tokio::time::timeout(Duration::from_secs(5), lookup(client)).await {
        Ok(Ok(isp)) => isp,
        Ok(Err(error)) => {
            log::error!("Error in ISP lookup: {error}");
            UNKNOWN.to_string()
        }
        Err(_) => {
            log::info!("ISP lookup timed out");
            UNKNOWN.to_string()
        }
    }
}

async fn lookup(client: &reqwest::Client) -> Result<String, String> {
    let ip_response = client
        .get("https://api.ipify.org?format=json")
        .timeout(Duration::from_secs(3))
        .send()
        .await
        .map_err(|error| error.to_string())?;
    if !ip_response.status().is_success() {
        return Err(format!(
            "IP lookup failed: {}",
            ip_response.status().as_u16()
        ));
    }
    let ip = ip_response
        .json::<IpResponse>()
        .await
        .map_err(|error| error.to_string())?
        .ip
        .filter(|ip| !ip.is_empty())
        .ok_or_else(|| "No IP address received".to_string())?;

    let isp_response = client
        .get(format!("http://ip-api.com/json/{ip}?fields=isp"))
        .timeout(Duration::from_secs(4))
        .send()
        .await
        .map_err(|error| error.to_string())?;
    if !isp_response.status().is_success() {
        return Err(format!(
            "ISP lookup failed: {}",
            isp_response.status().as_u16()
        ));
    }
    let isp = isp_response
        .json::<IspResponse>()
        .await
        .map_err(|error| error.to_string())?
        .isp
        .filter(|isp| !isp.is_empty())
        .unwrap_or_else(|| UNKNOWN.to_string());
    Ok(isp)
}
