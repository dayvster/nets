//! LAN device discovery (ARP/ICMP sweep)
use get_if_addrs::get_if_addrs;
use serde::Serialize;
use std::net::IpAddr;

pub use self::arp_discovery::*;

mod arp_discovery;
