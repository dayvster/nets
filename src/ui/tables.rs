//! Table rendering utilities for nets TUI.
//!
//! This module provides functions to render the main tables (ports and LAN devices)
//! in both scrollable and non-scrollable forms, as well as the header bar.

/// Render a scrollable table of local listening ports and their owning processes.
///
/// # Arguments
/// * `ports` - List of port/process mappings to display.
/// * `f` - The TUI frame to render into.
/// * `area` - The area of the screen to draw the table.
/// * `filter` - Optional filter string (process name, port, etc).
/// * `scroll` - Scroll offset for vertical paging.
/// * `focused` - Whether this table is currently focused (for highlight).
pub fn draw_ports_table_scroll(
    ports: &[PortProcess],
    f: &mut Frame,
    area: Rect,
    filter: Option<&str>,
    scroll: usize,
    focused: bool,
) {
    let header = Row::new(["PID", "Process", "Proto", "Local Address", "Remote Address"]).style(
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    );
    let filtered: Vec<_> = ports
        .iter()
        .filter(|p| {
            if let Some(f) = filter {
                if let Ok(port) = f.parse::<u16>() {
                    p.local_port == port
                } else {
                    let f = f.to_lowercase();
                    p.process_name.to_lowercase().contains(&f)
                        || p.pid.to_string().contains(&f)
                        || p.protocol.to_lowercase().contains(&f)
                        || format!("{}", p.local_addr).contains(&f)
                        || format!("{}", p.local_port).contains(&f)
                }
            } else {
                true
            }
        })
        .collect();
    let rows: Vec<Row> = if filtered.is_empty() {
        vec![Row::new([
            Cell::from("-"),
            Cell::from("-"),
            Cell::from("-"),
            Cell::from("-"),
            Cell::from("-"),
        ])
        .style(Style::default().fg(Color::DarkGray))]
    } else {
        filtered
            .iter()
            .map(|p| {
                let proto_color = match p.protocol.as_str() {
                    "TCP" => Color::Green,
                    "UDP" => Color::Blue,
                    _ => Color::White,
                };
                Row::new([
                    Cell::from(p.pid.to_string()),
                    Cell::from(Span::styled(
                        &p.process_name,
                        Style::default().fg(Color::Magenta),
                    )),
                    Cell::from(Span::styled(&p.protocol, Style::default().fg(proto_color))),
                    Cell::from(Span::styled(
                        format!("{}:{}", p.local_addr, p.local_port),
                        Style::default().fg(Color::Yellow),
                    )),
                    match (&p.remote_addr, &p.remote_port) {
                        (Some(addr), Some(port)) => Cell::from(Span::styled(
                            format!("{}:{}", addr, port),
                            Style::default().fg(Color::Cyan),
                        )),
                        _ => Cell::from("-"),
                    },
                ])
            })
            .collect()
    };
    let widths = [
        Constraint::Length(7),
        Constraint::Length(18),
        Constraint::Length(7),
        Constraint::Length(24),
        Constraint::Length(24),
    ];
    let block = Block::default().borders(Borders::ALL).title(Span::styled(
        "Listening Ports",
        if focused {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Cyan)
        },
    ));
    let table = Table::new(rows, widths)
        .header(header)
        .block(block)
        .column_spacing(1)
        .highlight_style(Style::default().bg(Color::DarkGray));
    let visible_height = area.height.saturating_sub(3) as usize; // header + borders
    let scroll = scroll.min(filtered.len().saturating_sub(visible_height));
    f.render_stateful_widget(table, area, &mut TableState::default().with_offset(scroll));
}

/// Render a scrollable table of discovered LAN devices.
///
/// # Arguments
/// * `devices` - List of LAN devices to display.
/// * `f` - The TUI frame to render into.
/// * `area` - The area of the screen to draw the table.
/// * `scroll` - Scroll offset for vertical paging.
/// * `focused` - Whether this table is currently focused (for highlight).
pub fn draw_lan_table_scroll(
    devices: &[LanDevice],
    f: &mut Frame,
    area: Rect,
    scroll: usize,
    focused: bool,
) {
    let header = Row::new(["IP Address", "MAC Address", "Vendor"]).style(
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD),
    );
    let rows: Vec<Row> = if devices.is_empty() {
        vec![Row::new([
            Cell::from("-"),
            Cell::from("No devices found"),
            Cell::from("-"),
        ])
        .style(Style::default().fg(Color::DarkGray))]
    } else {
        devices
            .iter()
            .map(|d| {
                Row::new([
                    Cell::from(Span::styled(
                        d.ip.to_string(),
                        Style::default().fg(Color::Cyan),
                    )),
                    Cell::from(Span::styled(
                        d.mac.clone().unwrap_or_else(|| "-".to_string()),
                        Style::default().fg(Color::Yellow),
                    )),
                    Cell::from(Span::styled(
                        d.vendor.clone().unwrap_or_else(|| "-".to_string()),
                        Style::default().fg(Color::Magenta),
                    )),
                ])
            })
            .collect()
    };
    let widths = [
        Constraint::Length(20),
        Constraint::Length(20),
        Constraint::Length(20),
    ];
    let block = Block::default().borders(Borders::ALL).title(Span::styled(
        "LAN Devices",
        if focused {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Green)
        },
    ));
    let table = Table::new(rows, widths)
        .header(header)
        .block(block)
        .column_spacing(1)
        .highlight_style(Style::default().bg(Color::DarkGray));
    let visible_height = area.height.saturating_sub(3) as usize;
    let scroll = scroll.min(devices.len().saturating_sub(visible_height));
    f.render_stateful_widget(table, area, &mut TableState::default().with_offset(scroll));
}
use crate::ports::PortProcess;
use crate::scanner::LanDevice;
use ratatui::{
    prelude::*,
    style::{Color, Style},
    widgets::*,
};
/// Render the header bar with stats and controls at the top of the TUI.
///
/// # Arguments
/// * `f` - The TUI frame to render into.
/// * `area` - The area of the screen to draw the header.
/// * `port_count` - Number of listening ports.
/// * `device_count` - Number of discovered LAN devices.
/// Enhanced header: show local IP and hostname if available.
pub fn draw_header(
    f: &mut Frame,
    area: Rect,
    port_count: usize,
    device_count: usize,
    local_ip: Option<String>,
    local_host: Option<String>,
) {
    let title = Line::from(vec![Span::styled(
        " nets ",
        Style::default()
            .fg(Color::Black)
            .bg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )]);
    let mut stats_vec = vec![Span::styled(
        format!(" Ports: {}  Devices: {}  ", port_count, device_count),
        Style::default().fg(Color::Yellow),
    )];
    if let Some(ip) = local_ip {
        stats_vec.push(Span::styled(
            format!("IP: {}  ", ip),
            Style::default().fg(Color::Green),
        ));
    }
    if let Some(host) = local_host {
        stats_vec.push(Span::styled(
            format!("Host: {}  ", host),
            Style::default().fg(Color::Cyan),
        ));
    }
    stats_vec.push(Span::raw("Press 'q' to quit | '/' to filter"));
    let stats = Line::from(stats_vec);

    let block = Block::default().borders(Borders::ALL).title(title);
    let header = Paragraph::new(vec![stats])
        .block(block)
        .alignment(Alignment::Left);
    f.render_widget(header, area);
}

/// Render a non-scrollable table of local listening ports and their owning processes.
///
/// This is a legacy function; prefer [`draw_ports_table_scroll`] for interactive UIs.
pub fn draw_ports_table(ports: &[PortProcess], f: &mut Frame, area: Rect, filter: Option<&str>) {
    let header = Row::new(["PID", "Process", "Proto", "Local Address", "Remote Address"]).style(
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    );
    let rows = ports
        .iter()
        .filter(|p| {
            if let Some(f) = filter {
                // If filter is a valid port number, filter by local_port
                if let Ok(port) = f.parse::<u16>() {
                    p.local_port == port
                } else {
                    let f = f.to_lowercase();
                    p.process_name.to_lowercase().contains(&f)
                        || p.pid.to_string().contains(&f)
                        || p.protocol.to_lowercase().contains(&f)
                        || format!("{}", p.local_addr).contains(&f)
                        || format!("{}", p.local_port).contains(&f)
                }
            } else {
                true
            }
        })
        .map(|p| {
            Row::new([
                p.pid.to_string(),
                p.process_name.clone(),
                p.protocol.clone(),
                format!("{}:{}", p.local_addr, p.local_port),
                match (&p.remote_addr, p.remote_port) {
                    (Some(addr), Some(port)) => format!("{}:{}", addr, port),
                    _ => "-".to_string(),
                },
            ])
            .style(Style::default().fg(Color::White))
        });
    let widths = [
        Constraint::Length(7),
        Constraint::Length(18),
        Constraint::Length(7),
        Constraint::Length(24),
        Constraint::Length(24),
    ];
    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title(Span::styled(
            "Listening Ports",
            Style::default().fg(Color::Cyan),
        )))
        .column_spacing(1);
    f.render_widget(table, area);
}

/// Render a non-scrollable table of discovered LAN devices.
///
/// This is a legacy function; prefer [`draw_lan_table_scroll`] for interactive UIs.
pub fn draw_lan_table(devices: &[LanDevice], f: &mut Frame, area: Rect) {
    let header = Row::new(["IP Address", "MAC Address", "Vendor"]).style(
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD),
    );
    let rows = devices.iter().map(|d| {
        Row::new([
            d.ip.to_string(),
            d.mac.clone().unwrap_or_else(|| "-".to_string()),
            d.vendor.clone().unwrap_or_else(|| "-".to_string()),
        ])
        .style(Style::default().fg(Color::White))
    });
    let widths = [
        Constraint::Length(20),
        Constraint::Length(20),
        Constraint::Length(20),
    ];
    let table = Table::new(rows, widths).header(header).block(
        Block::default().borders(Borders::ALL).title(Span::styled(
            "LAN Devices",
            Style::default().fg(Color::Green),
        )),
    );
    f.render_widget(table, area);
}
