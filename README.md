# nets

nets is a cross-platform, real-time network dashboard for the terminal, inspired by htop but focused on networking. It provides a live, interactive view of local listening ports, running processes, and devices on your LAN, with powerful filtering, paging, and network testing features—all in a modern TUI.

## Features

- **Live Port & Process Table:** See all listening ports and the owning processes, with protocol, local/remote addresses, and filtering.
- **LAN Device Discovery:** Scan your local network for devices, including hostnames and MAC addresses.
- **Interactive TUI:** Navigate with keyboard, scroll, filter, and switch focus between tables.
- **Ping Modal:** Press `p` to open a modal and ping any host, with animated feedback.
- **Header with Network Info:** Displays your IP, router IP, hostname, and stats.
- **Colorful, Modern UI:** Uses ratatui for a beautiful, responsive terminal experience.
- **Cross-Platform:** Works on Linux, macOS, and Windows (with some feature limitations).
- **Keyboard Shortcuts:** Fast navigation, filtering, paging, and modal actions.
- **Animated Feedback:** Visual cues for pinging and modal actions.
- **Process/Port Filtering:** Filter by process name, port, or protocol.
- **LAN Hostname Lookup:** Reverse DNS for LAN devices.
- **Paging & Scrolling:** View large tables with smooth navigation.
- **Customizable Refresh Rate:** Choose how often data updates.
- **Graceful Error Handling:** Robust against missing permissions or partial data.
- **Extensible Architecture:** Modular codebase for easy feature addition.

### Potential/Future Features

- Copy IP/host/ping results to clipboard
- Traceroute modal
- Export tables to CSV/JSON
- Service name lookup for ports
- Network interface stats (live RX/TX)
- Notifications for new devices/ports
- Theme toggle (light/dark/high-contrast)
- Block/allowlist for devices/ports

## Usage

```
cargo run --release -- [OPTIONS]
```

### Options

- `-r, --refresh <SECONDS>`   Refresh interval (default: 2)
- `--no-lan`                  Disable LAN device scanning
- `-p, --port <PORT>`         Filter by port number

### Keyboard Shortcuts

- `/`         Start text filter
- `p`         Open ping modal
- `Tab`       Switch focus between tables
- `Up/Down`   Scroll
- `PageUp/PageDown`   Page scroll
- `q`         Quit

## Building

Requires Rust 2021+. Clone the repo and run:

```
cargo build --release
```

## License

nets is licensed under the MIT License. See [LICENSE](LICENSE) for details.

## Contributing

Contributions, bug reports, and feature requests are welcome! Please open an issue or pull request.

## Acknowledgments

- [ratatui](https://github.com/ratatui-org/ratatui) for the TUI framework
- [tokio](https://tokio.rs/) for async runtime
- [procfs](https://github.com/eminence/procfs) for process/port info
- [get_if_addrs](https://github.com/maidsafe-archive/get_if_addrs) for network interfaces

---

*__nets__: The network dashboard your terminal deserves.*
