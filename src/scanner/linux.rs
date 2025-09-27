use get_if_addrs::get_if_addrs;
use serde::Serialize;
use std::net::IpAddr;

#[derive(Debug, Clone, Serialize)]
pub struct LanDevice {
    pub ip: IpAddr,
    pub mac: Option<String>,
    pub vendor: Option<String>,
}

pub async fn scan_lan() -> Vec<LanDevice> {
    let mut devices = Vec::new();
    // Read /proc/net/arp for ARP table
    if let Ok(lines) = std::fs::read_to_string("/proc/net/arp") {
        for line in lines.lines().skip(1) {
            let parts: Vec<_> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                let ip = parts[0].parse().ok();
                let mac = Some(parts[3].to_string()).filter(|m| m != "00:00:00:00:00:00");
                if let Some(ip) = ip {
                    devices.push(LanDevice {
                        ip,
                        mac,
                        vendor: None, // Vendor lookup could be added
                    });
                }
            }
        }
    }
    // Optionally, add local interfaces
    if let Ok(ifaces) = get_if_addrs() {
        for iface in ifaces {
            if let Some(ip) = match iface.addr {
                get_if_addrs::IfAddr::V4(a) => Some(IpAddr::V4(a.ip)),
                get_if_addrs::IfAddr::V6(a) => Some(IpAddr::V6(a.ip)),
                _ => None,
            } {
                let mac = iface.mac.map(|m| m.to_string());
                devices.push(LanDevice {
                    ip,
                    mac,
                    vendor: None,
                });
            }
        }
    }
    devices
}
