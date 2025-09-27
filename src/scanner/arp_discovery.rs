use dns_lookup;
use get_if_addrs::get_if_addrs;
use serde::Serialize;
use std::net::IpAddr;

/// Represents a device discovered on the local network.
#[derive(Debug, Clone, Serialize)]
pub struct LanDevice {
    pub ip: IpAddr,
    pub mac: Option<String>,
    pub vendor: Option<String>, // Used for hostname if available
}

/// Scans the local network for devices using the ARP table and network interfaces.
pub async fn scan_lan() -> Vec<LanDevice> {
    let mut devices = Vec::new();
    if let Ok(lines) = std::fs::read_to_string("/proc/net/arp") {
        for line in lines.lines().skip(1) {
            let parts: Vec<_> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                let ip = parts[0].parse().ok();
                let mac = Some(parts[3].to_string()).filter(|m| m != "00:00:00:00:00:00");
                if let (Some(ip), Some(mac)) = (ip, mac) {
                    // Try reverse DNS lookup for hostname
                    let hostname = match dns_lookup::lookup_addr(&ip) {
                        Ok(name) => Some(name),
                        Err(_) => None,
                    };
                    devices.push(LanDevice {
                        ip,
                        mac: Some(mac),
                        vendor: hostname,
                    });
                }
            }
        }
    }
    // Also add local interface IPs (no MAC)
    if let Ok(ifaces) = get_if_addrs() {
        for iface in ifaces {
            let ip = match iface.addr {
                get_if_addrs::IfAddr::V4(a) => Some(IpAddr::V4(a.ip)),
                get_if_addrs::IfAddr::V6(a) => Some(IpAddr::V6(a.ip)),
            };
            if let Some(ip) = ip {
                let hostname = match dns_lookup::lookup_addr(&ip) {
                    Ok(name) => Some(name),
                    Err(_) => None,
                };
                devices.push(LanDevice {
                    ip,
                    mac: None,
                    vendor: hostname,
                });
            }
        }
    }
    devices
}
