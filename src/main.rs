use clap::Parser;
/// Command-line arguments for nets.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = 2)]
    refresh: u64,
    #[arg(long, default_value_t = false)]
    no_lan: bool,
    #[arg(short, long)]
    port: Option<u16>,
}
mod ports;
mod scanner;
mod ui;

use crossterm::event::{self, Event, KeyCode};
use ports::PortProcess;
use ratatui::{backend::CrosstermBackend, Terminal};
use scanner::LanDevice;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

/// Entry point for nets.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let ports_data = Arc::new(Mutex::new(Vec::new()));
    let lan_data = Arc::new(Mutex::new(Vec::new()));

    let ports_data_clone = ports_data.clone();
    let refresh = args.refresh;
    tokio::spawn(async move {
        loop {
            let list = ports::list_ports().await;
            let mut lock = ports_data_clone.lock().await;
            *lock = list;
            tokio::time::sleep(Duration::from_secs(refresh)).await;
        }
    });

    if !args.no_lan {
        let lan_data_clone = lan_data.clone();
        let refresh = args.refresh;
        tokio::spawn(async move {
            let mut tick: u64 = 0;
            loop {
                if tick % 10 == 0 {
                    let list = scanner::scan_lan().await;
                    let mut lock = lan_data_clone.lock().await;
                    *lock = list;
                }
                tick = tick.wrapping_add(1);
                tokio::time::sleep(Duration::from_secs(refresh)).await;
            }
        });
    }

    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = ratatui::Terminal::new(backend)?;

    tui_main_loop(&mut terminal, &ports_data, &lan_data, args.port).await?;
    Ok(())
}

/// Main TUI event/render loop.
async fn tui_main_loop(
    terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
    ports_data: &tokio::sync::Mutex<Vec<PortProcess>>,
    lan_data: &tokio::sync::Mutex<Vec<LanDevice>>,
    port_filter: Option<u16>,
) -> anyhow::Result<()> {
    use ratatui::layout::Constraint;
    use ratatui::widgets::Paragraph;

    let mut filter: Option<String> = None;
    let mut filter_input = String::new();
    let mut filtering = false;
    let mut port_scroll: usize = 0;
    let mut lan_scroll: usize = 0;
    let mut focus_ports = true; // true: ports, false: lan
    let mut last_ports: Vec<PortProcess> = Vec::new();
    let mut last_devices: Vec<LanDevice> = Vec::new();
    let mut force_redraw = true;
    let mut ping_modal = false;
    let mut ping_input = String::new();
    let mut ping_output: Option<String> = None;
    let mut ping_in_progress = false;
    let mut ping_rx: Option<std::sync::mpsc::Receiver<String>> = None;
    let mut ping_anim_frame: u8 = 0;
    let mut ping_anim_tick: u8 = 0;
    loop {
        let mut redraw = false;
        // Check for ping result if in progress (every loop, not just after Enter)
        if ping_modal && ping_in_progress {
            use std::sync::mpsc::TryRecvError;
            if let Some(ref rx) = ping_rx {
                match rx.try_recv() {
                    Ok(result) => {
                        ping_output = Some(result);
                        ping_in_progress = false;
                        ping_rx = None;
                        redraw = true;
                    }
                    Err(TryRecvError::Empty) => {}
                    Err(TryRecvError::Disconnected) => {
                        ping_output = Some("Ping failed: channel closed".to_string());
                        ping_in_progress = false;
                        ping_rx = None;
                        redraw = true;
                    }
                }
            }
            // Slow down animation: only advance frame every 2 ticks (~200ms)
            ping_anim_tick = (ping_anim_tick + 1) % 2;
            if ping_anim_tick == 0 {
                ping_anim_frame = (ping_anim_frame + 1) % 3;
            }
        }
        // Redraw on input or every 200ms
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if ping_modal {
                    match key.code {
                        KeyCode::Esc => {
                            ping_modal = false;
                            ping_input.clear();
                            ping_output = None;
                            ping_in_progress = false;
                            ping_rx = None;
                            redraw = true;
                        }
                        KeyCode::Enter => {
                            let host = ping_input.trim().to_string();
                            if !host.is_empty() && !ping_in_progress {
                                ping_in_progress = true;
                                ping_output = None;
                                // Drop any previous receiver
                                ping_rx = None;
                                let (tx, rx) = std::sync::mpsc::channel();
                                let host_clone = host.clone();
                                std::thread::spawn(move || {
                                    let output = std::process::Command::new("ping")
                                        .arg("-c")
                                        .arg("4")
                                        .arg(&host_clone)
                                        .output();
                                    let result = match output {
                                        Ok(out) => String::from_utf8_lossy(&out.stdout).to_string(),
                                        Err(e) => format!("Ping failed: {}", e),
                                    };
                                    let _ = tx.send(result);
                                });
                                ping_rx = Some(rx);
                            }
                            redraw = true;
                        }
                        KeyCode::Char(c) => {
                            ping_input.push(c);
                            redraw = true;
                        }
                        KeyCode::Backspace => {
                            ping_input.pop();
                            redraw = true;
                        }
                        _ => {}
                    }
                } else if key.code == KeyCode::Char('q') {
                    break;
                } else if port_filter.is_none() {
                    if filtering {
                        match key.code {
                            KeyCode::Esc => {
                                filtering = false;
                                filter_input.clear();
                                redraw = true;
                            }
                            KeyCode::Enter => {
                                filter = if filter_input.is_empty() {
                                    None
                                } else {
                                    Some(filter_input.clone())
                                };
                                filtering = false;
                                redraw = true;
                            }
                            KeyCode::Char(c) => {
                                filter_input.push(c);
                                redraw = true;
                            }
                            KeyCode::Backspace => {
                                filter_input.pop();
                                redraw = true;
                            }
                            _ => {}
                        }
                    } else {
                        match key.code {
                            KeyCode::Char('/') => {
                                filtering = true;
                                filter_input.clear();
                                redraw = true;
                            }
                            KeyCode::Char('p') => {
                                ping_modal = true;
                                ping_input.clear();
                                ping_output = None;
                                redraw = true;
                            }
                            KeyCode::Tab | KeyCode::BackTab => {
                                focus_ports = !focus_ports;
                                redraw = true;
                            }
                            KeyCode::Down => {
                                if focus_ports {
                                    port_scroll = port_scroll.saturating_add(1);
                                } else {
                                    lan_scroll = lan_scroll.saturating_add(1);
                                }
                                redraw = true;
                            }
                            KeyCode::Up => {
                                if focus_ports {
                                    port_scroll = port_scroll.saturating_sub(1);
                                } else {
                                    lan_scroll = lan_scroll.saturating_sub(1);
                                }
                                redraw = true;
                            }
                            KeyCode::PageDown => {
                                if focus_ports {
                                    port_scroll = port_scroll.saturating_add(10);
                                } else {
                                    lan_scroll = lan_scroll.saturating_add(10);
                                }
                                redraw = true;
                            }
                            KeyCode::PageUp => {
                                if focus_ports {
                                    port_scroll = port_scroll.saturating_sub(10);
                                } else {
                                    lan_scroll = lan_scroll.saturating_sub(10);
                                }
                                redraw = true;
                            }
                            _ => {}
                        }
                    }
                }
            }
        } else {
            // No input, periodic redraw
            redraw = true;
        }
        if redraw || force_redraw {
            force_redraw = false;
            // Try to get new data, but never block UI
            if let Ok(lock) = ports_data.try_lock() {
                last_ports = lock.clone();
            }
            if let Ok(lock) = lan_data.try_lock() {
                last_devices = lock.clone();
            }
            let ports = &last_ports;
            let devices = &last_devices;
            terminal.draw(|f| {
                let size = f.size();
                let header_height = 5;
                let chunks = ratatui::layout::Layout::default()
                    .direction(ratatui::layout::Direction::Vertical)
                    .constraints([
                        Constraint::Length(header_height),
                        Constraint::Percentage(60),
                        Constraint::Percentage(40),
                    ])
                    .split(size);
                // Try to get local IP and hostname from first LAN device with a MAC (likely local interface)
                let (local_ip, local_host) = devices
                    .iter()
                    .find(|d| d.mac.is_none())
                    .map(|d| (Some(d.ip.to_string()), d.vendor.clone()))
                    .unwrap_or((None, None));
                ui::draw_header(
                    f,
                    chunks[0],
                    ports.len(),
                    devices.len(),
                    local_ip,
                    local_host,
                );
                let effective_filter = if let Some(port) = port_filter {
                    Some(port.to_string())
                } else {
                    filter.clone()
                };
                ui::draw_ports_table_scroll(
                    ports,
                    f,
                    chunks[1],
                    effective_filter.as_deref(),
                    port_scroll,
                    focus_ports,
                );
                ui::draw_lan_table_scroll(devices, f, chunks[2], lan_scroll, !focus_ports);
                if filtering && port_filter.is_none() {
                    let area = ratatui::layout::Rect {
                        x: chunks[0].x + 2,
                        y: chunks[0].y + 1,
                        width: chunks[0].width - 4,
                        height: 3,
                    };
                    let p = Paragraph::new(format!("Filter: {}", filter_input)).block(
                        ratatui::widgets::Block::default()
                            .borders(ratatui::widgets::Borders::ALL)
                            .title("Filter"),
                    );
                    f.render_widget(p, area);
                }
                if ping_modal {
                    use ratatui::style::{Color, Modifier, Style};
                    use ratatui::text::{Line, Span};
                    use ratatui::widgets::{Block, Borders, Clear, Paragraph};
                    // Modal area
                    let area = ratatui::layout::Rect {
                        x: size.width / 4,
                        y: size.height / 4,
                        width: size.width / 2,
                        height: size.height / 2,
                    };
                    // Clear the area (draws a solid background)
                    f.render_widget(Clear, area);
                    // Draw modal border and title
                    let bg_block = Block::default()
                        .style(Style::default().bg(Color::Rgb(30, 32, 40)))
                        .borders(Borders::ALL)
                        .border_type(ratatui::widgets::BorderType::Thick)
                        .title(Span::styled(
                            " Ping ",
                            Style::default()
                                .fg(Color::Yellow)
                                .bg(Color::Rgb(30, 32, 40))
                                .add_modifier(Modifier::BOLD),
                        ));
                    f.render_widget(bg_block, area);

                    // Input box area (top of modal)
                    let input_area = ratatui::layout::Rect {
                        x: area.x + 2,
                        y: area.y + 2,
                        width: area.width - 4,
                        height: 3,
                    };
                    let input_label = Span::styled("Host:", Style::default().fg(Color::Cyan));
                    let input_value = Span::styled(
                        if ping_input.is_empty() {
                            "<type host>"
                        } else {
                            &ping_input
                        },
                        Style::default().fg(Color::White).bg(Color::Rgb(40, 44, 54)),
                    );
                    let input_line = Line::from(vec![input_label, Span::raw(" "), input_value]);
                    let input_box = Paragraph::new(vec![input_line])
                        .block(
                            Block::default()
                                .borders(Borders::ALL)
                                .border_style(Style::default().fg(Color::Cyan))
                                .title("Ping host"),
                        )
                        .style(Style::default().bg(Color::Rgb(40, 44, 54)));
                    f.render_widget(input_box, input_area);

                    // Show cursor in input
                    let cursor_x = input_area.x + 7 + ping_input.len() as u16;
                    let cursor_y = input_area.y + 1;
                    f.set_cursor(cursor_x, cursor_y);

                    // Output area (below input)
                    let output_area = ratatui::layout::Rect {
                        x: area.x + 2,
                        y: area.y + 6,
                        width: area.width - 4,
                        height: area.height - 8,
                    };
                    let mut text: Vec<Line> = vec![Line::from(vec![Span::styled(
                        "[Enter] to run, [ESC] to cancel",
                        Style::default().fg(Color::DarkGray),
                    )])];
                    if ping_in_progress {
                        text.push(Line::from(""));
                        let dots = match ping_anim_frame {
                            0 => ".  ",
                            1 => ".. ",
                            _ => "...",
                        };
                        text.push(Line::from(vec![Span::styled(
                            format!("Pinging{}", dots),
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::ITALIC),
                        )]));
                    } else if let Some(ref out) = ping_output {
                        text.push(Line::from(""));
                        text.push(Line::from(vec![Span::styled(
                            "--- Output ---",
                            Style::default().fg(Color::Magenta),
                        )]));
                        text.extend(
                            out.lines()
                                .take((output_area.height as usize) - 2)
                                .map(|s| Line::from(Span::raw(s.to_string()))),
                        );
                    }
                    let p = Paragraph::new(text)
                        .style(Style::default().fg(Color::White).bg(Color::Rgb(30, 32, 40)))
                        .alignment(ratatui::layout::Alignment::Left);
                    f.render_widget(p, output_area);
                }
            })?;
        }
    }

    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen
    )?;
    crossterm::terminal::disable_raw_mode()?;
    Ok(())
}
