use procfs::net::{NetEntry, TcpNetEntry, UdpNetEntry};
use procfs::process::{all_processes, Process};
use serde::Serialize;
use std::net::IpAddr;

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

fn get_process_name(pid: i32) -> String {
    Process::new(pid)
        .and_then(|p| p.stat())
        .map(|stat| stat.comm)
        .unwrap_or_else(|_| "?".to_string())
}

fn entry_to_portproc<E: NetEntry>(entry: &E, proto: &str) -> PortProcess {
    PortProcess {
        pid: entry.inode().unwrap_or(0) as u32, // Will be replaced below
        process_name: "?".to_string(),          // Will be replaced below
        local_addr: entry.local_address().ip(),
        local_port: entry.local_address().port(),
        remote_addr: Some(entry.remote_address().ip()),
        remote_port: Some(entry.remote_address().port()),
        protocol: proto.to_string(),
    }
}

fn find_pid_by_inode(inode: u64) -> Option<(i32, String)> {
    for proc in all_processes().flatten() {
        if let Ok(fds) = proc.fd() {
            for fd in fds.flatten() {
                if let Ok(procfs::process::FDTarget::Socket(fd_inode)) = fd.target() {
                    if fd_inode == inode {
                        let name = proc
                            .stat()
                            .map(|s| s.comm)
                            .unwrap_or_else(|_| "?".to_string());
                        return Some((proc.pid(), name));
                    }
                }
            }
        }
    }
    None
}

pub async fn list_ports() -> Vec<PortProcess> {
    let mut results = Vec::new();
    // TCP
    if let Ok(tcp) = procfs::net::tcp() {
        for entry in tcp {
            let mut proc = entry_to_portproc(&entry, "TCP");
            if let Some((pid, name)) = find_pid_by_inode(entry.inode()) {
                proc.pid = pid as u32;
                proc.process_name = name;
            }
            results.push(proc);
        }
    }
    // UDP
    if let Ok(udp) = procfs::net::udp() {
        for entry in udp {
            let mut proc = entry_to_portproc(&entry, "UDP");
            if let Some((pid, name)) = find_pid_by_inode(entry.inode()) {
                proc.pid = pid as u32;
                proc.process_name = name;
            }
            results.push(proc);
        }
    }
    results
}
