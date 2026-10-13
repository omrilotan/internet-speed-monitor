//! Detects whether the default route goes over WiFi or Ethernet using OS tools.

use std::process::Command;

pub const UNKNOWN: &str = "Unknown";

pub async fn detect_network_connection_type() -> String {
    match tokio::task::spawn_blocking(detect_blocking).await {
        Ok(kind) => kind,
        Err(error) => {
            log::error!("Error detecting network connection type: {error}");
            UNKNOWN.to_string()
        }
    }
}

fn run(program: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(program).args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Like [`run`] but returns stdout even when the command exits non-zero.
fn run_lenient(program: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(program).args(args).output().ok()?;
    Some(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn detect_blocking() -> String {
    let detected = match std::env::consts::OS {
        "macos" => detect_macos(),
        "windows" => detect_windows(),
        "linux" => detect_linux(),
        _ => None,
    };
    detected.unwrap_or_else(|| UNKNOWN.to_string())
}

fn detect_macos() -> Option<String> {
    let route = run("route", &["-n", "get", "default"])?;
    let interface = parse_macos_interface(&route)?;

    if let Some(output) = run_lenient("networksetup", &["-getairportnetwork", &interface])
        && output.contains("Current Wi-Fi Network")
    {
        return Some("WiFi".to_string());
    }

    // The hardware port list names the adapter ("Wi-Fi", "Ethernet", "USB 10/100 LAN", ...).
    if let Some(ports) = run("networksetup", &["-listallhardwareports"])
        && let Some(kind) = classify_macos_port(&ports, &interface)
    {
        return Some(kind);
    }

    if interface.starts_with("en1") || interface.contains("wi") {
        return Some("WiFi".to_string());
    }
    if interface.starts_with("en0") || interface.contains("eth") {
        return Some("Ethernet".to_string());
    }
    Some("Network".to_string())
}

pub fn classify_macos_port(ports_output: &str, interface: &str) -> Option<String> {
    let mut current_port: Option<&str> = None;
    for line in ports_output.lines() {
        if let Some(port) = line.trim().strip_prefix("Hardware Port:") {
            current_port = Some(port.trim());
        } else if let Some(device) = line.trim().strip_prefix("Device:")
            && device.trim() == interface
        {
            let port = current_port?.to_lowercase();
            if port.contains("wi-fi") || port.contains("airport") || port.contains("wireless") {
                return Some("WiFi".to_string());
            }
            if port.contains("ethernet") || port.contains("lan") || port.contains("thunderbolt") {
                return Some("Ethernet".to_string());
            }
            return None;
        }
    }
    None
}

pub fn parse_macos_interface(route_output: &str) -> Option<String> {
    route_output
        .lines()
        .find_map(|line| line.trim().strip_prefix("interface:"))
        .map(|name| name.trim().to_string())
        .filter(|name| !name.is_empty())
}

fn detect_windows() -> Option<String> {
    let output = run("netsh", &["interface", "show", "interface"])?;
    parse_windows_interfaces(&output)
}

pub fn parse_windows_interfaces(output: &str) -> Option<String> {
    for line in output.lines() {
        if line.contains("Connected") && line.contains("Dedicated") {
            let lower = line.to_lowercase();
            if lower.contains("wi-fi") || lower.contains("wireless") || lower.contains("wifi") {
                return Some("WiFi".to_string());
            }
            if lower.contains("ethernet") || lower.contains("local area connection") {
                return Some("Ethernet".to_string());
            }
        }
    }
    None
}

fn detect_linux() -> Option<String> {
    let output = run("ip", &["route", "show", "default"])?;
    let interface = parse_linux_interface(&output)?;

    if run("iwconfig", &[&interface]).is_some() {
        return Some("WiFi".to_string());
    }
    if interface.starts_with("wl") || interface.contains("wifi") {
        return Some("WiFi".to_string());
    }
    if interface.starts_with("eth") || interface.starts_with("en") {
        return Some("Ethernet".to_string());
    }
    Some("Network".to_string())
}

pub fn parse_linux_interface(output: &str) -> Option<String> {
    let mut words = output.split_whitespace();
    while let Some(word) = words.next() {
        if word == "dev" {
            return words.next().map(str::to_string);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_route_output() {
        let output = "   route to: default\ndestination: default\n       mask: default\n    gateway: 192.168.1.1\n  interface: en0\n      flags: <UP,GATEWAY>\n";
        assert_eq!(parse_macos_interface(output).as_deref(), Some("en0"));
        assert_eq!(parse_macos_interface("nothing"), None);
    }

    #[test]
    fn classifies_hardware_ports() {
        let ports = "Hardware Port: Wi-Fi\nDevice: en0\nEthernet Address: aa\n\nHardware Port: USB 10/100 LAN\nDevice: en7\nEthernet Address: bb\n";
        assert_eq!(classify_macos_port(ports, "en0").as_deref(), Some("WiFi"));
        assert_eq!(
            classify_macos_port(ports, "en7").as_deref(),
            Some("Ethernet")
        );
        assert_eq!(classify_macos_port(ports, "en9"), None);
    }

    #[test]
    fn parses_ip_route() {
        assert_eq!(
            parse_linux_interface("default via 10.0.0.1 dev wlp3s0 proto dhcp metric 600\n")
                .as_deref(),
            Some("wlp3s0")
        );
    }

    #[test]
    fn parses_netsh() {
        let output = "Admin State    State          Type             Interface Name\n-------------------------------------------------------------------------\nEnabled        Connected      Dedicated        Wi-Fi\nEnabled        Disconnected   Dedicated        Ethernet\n";
        assert_eq!(parse_windows_interfaces(output).as_deref(), Some("WiFi"));
    }
}
