use procfs::net::{TcpNetEntry, UdpNetEntry};
use procfs::process::{all_processes, Process};
use serde::Serialize;
use std::net::IpAddr;

/// Represents a process bound to a network port.
#[derive(Debug, Clone, Serialize)]
pub struct PortProcess {
    pub pid: u32,
    pub process_name: String,
    pub local_addr: IpAddr,
    pub local_port: u16,
    pub remote_addr: Option<IpAddr>,
    pub remote_port: Option<u16>,
    pub protocol: String,
}

/// Looks up the process name for a given PID.
fn get_process_name(pid: i32) -> String {
    Process::new(pid)
        .and_then(|p| p.stat())
        .map(|stat| stat.comm)
        .unwrap_or_else(|_| "?".to_string())
}

/// Converts a TCP entry to a PortProcess (pid and name will be filled in later).
fn tcp_entry_to_portproc(entry: &TcpNetEntry) -> PortProcess {
    PortProcess {
        pid: entry.inode as u32,
        process_name: "?".to_string(),
        local_addr: entry.local_address.ip(),
        local_port: entry.local_address.port(),
        remote_addr: Some(entry.remote_address.ip()),
        remote_port: Some(entry.remote_address.port()),
        protocol: "TCP".to_string(),
    }
}

/// Converts a UDP entry to a PortProcess (pid and name will be filled in later).
fn udp_entry_to_portproc(entry: &UdpNetEntry) -> PortProcess {
    PortProcess {
        pid: entry.inode as u32,
        process_name: "?".to_string(),
        local_addr: entry.local_address.ip(),
        local_port: entry.local_address.port(),
        remote_addr: Some(entry.remote_address.ip()),
        remote_port: Some(entry.remote_address.port()),
        protocol: "UDP".to_string(),
    }
}

/// Finds the PID and process name for a given socket inode.
fn find_pid_by_inode(inode: u64) -> Option<(i32, String)> {
    let procs = all_processes().ok()?;
    for proc_res in procs {
        let proc = match proc_res {
            Ok(p) => p,
            Err(_) => continue,
        };
        if let Ok(fds) = proc.fd() {
            for fd in fds.flatten() {
                if let procfs::process::FDTarget::Socket(fd_inode) = fd.target {
                    if fd_inode == inode {
                        let name = proc
                            .stat()
                            .map(|s| s.comm)
                            .unwrap_or_else(|_| "?".to_string());
                        return Some((proc.pid, name));
                    }
                }
            }
        }
    }
    None
}

/// Lists all processes with open TCP/UDP ports.
pub async fn list_ports() -> Vec<PortProcess> {
    let mut results = Vec::new();
    if let Ok(tcp) = procfs::net::tcp() {
        for entry in tcp {
            let mut proc = tcp_entry_to_portproc(&entry);
            if let Some((pid, name)) = find_pid_by_inode(entry.inode) {
                proc.pid = pid as u32;
                proc.process_name = name;
            }
            results.push(proc);
        }
    }
    if let Ok(udp) = procfs::net::udp() {
        for entry in udp {
            let mut proc = udp_entry_to_portproc(&entry);
            if let Some((pid, name)) = find_pid_by_inode(entry.inode) {
                proc.pid = pid as u32;
                proc.process_name = name;
            }
            results.push(proc);
        }
    }
    results
}
