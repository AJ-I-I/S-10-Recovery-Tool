use crate::tui::app::{App, AppMode};
use ratatui::prelude::*;
use ratatui::widgets::*;

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(f.size());

    // Title bar
    let title = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Cyan))
        .title("S-10 Recovery Tool");

    let title_text = match app.mode {
        AppMode::Browser => "File Recovery Browser",
        AppMode::Search => "Search Mode",
        AppMode::Forensic => "Forensic Analysis Mode",
        AppMode::Command => "Command Mode",
    };
    
    // Show current directory in title
    let current_path = format!(" | Current: {}", app.current_dir.display());
    let full_title = format!("{}{}", title_text, current_path);

    let title_widget = Paragraph::new(full_title)
        .block(title)
        .style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);

    f.render_widget(title_widget, chunks[0]);

    // Main content area
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(chunks[1]);

    draw_file_list(f, app, main_chunks[0]);
    draw_info_panel(f, app, main_chunks[1]);

    // Status bar
    draw_status_bar(f, app, chunks[2]);
}

fn draw_file_list(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(if app.is_scanning { "Scan Results" } else { "Directory Contents" })
        .style(Style::default().fg(Color::White));

    // Show directory items when not scanning, scan results when scanning
    let items: Vec<ListItem> = if app.is_scanning && !app.items.is_empty() {
        // Show scan results
        app.items
            .iter()
            .enumerate()
            .skip(app.scroll_offset)
            .take(20)
            .map(|(i, entry)| {
                let style = if i + app.scroll_offset == app.selected_index {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD | Modifier::REVERSED)
                } else if entry.is_deleted {
                    Style::default().fg(Color::Red)
                } else {
                    Style::default().fg(Color::Green)
                };

                let icon = if entry.is_deleted {
                    "[DEL]"
                } else {
                    "[OK]"
                };

                let size_str = format_size(entry.size);
                let display_text = format!("{} {} ({})", icon, entry.path.display(), size_str);

                ListItem::new(display_text).style(style)
            })
            .collect()
    } else {
        // Show directory listing
        app.directory_items
            .iter()
            .enumerate()
            .skip(app.scroll_offset)
            .take(20)
            .map(|(i, path)| {
                let style = if i + app.scroll_offset == app.selected_index {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD | Modifier::REVERSED)
                } else if path.is_dir() {
                    Style::default().fg(Color::Cyan)
                } else {
                    Style::default().fg(Color::White)
                };

                let icon = if path.is_dir() {
                    "[DIR]"
                } else {
                    "[FILE]"
                };

                let name = path.file_name()
                    .and_then(|n| n.to_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| path.to_string_lossy().to_string());
                
                let size_str = if path.is_file() {
                    if let Ok(metadata) = std::fs::metadata(path) {
                        format!(" ({})", format_size(metadata.len()))
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                };

                let display_text = format!("{} {}{}", icon, name, size_str);

                ListItem::new(display_text).style(style)
            })
            .collect()
    };

    let list = List::new(items).block(block);
    f.render_widget(list, area);

    // Render scrollbar
    let total_items = if app.is_scanning && !app.items.is_empty() {
        app.items.len()
    } else {
        app.directory_items.len()
    };
    
    if total_items > 20 {
        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));
        let mut scrollbar_state = ScrollbarState::default()
            .content_length(total_items)
            .position(app.scroll_offset);
        f.render_stateful_widget(scrollbar, area, &mut scrollbar_state);
    }
}

fn draw_info_panel(f: &mut Frame, app: &App, area: Rect) {
    let info_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(10), Constraint::Min(0)])
        .split(area);

    // Stats panel
    let stats_block = Block::default()
        .borders(Borders::ALL)
        .title("Scan Statistics")
        .style(Style::default().fg(Color::Cyan));

    let stats_text = vec![
        Line::from(vec![
            Span::styled("Files Found: ", Style::default().fg(Color::White)),
            Span::styled(
                format!("{}", app.scan_stats.files_found),
                Style::default().fg(Color::Green),
            ),
        ]),
        Line::from(vec![
            Span::styled("Deleted: ", Style::default().fg(Color::White)),
            Span::styled(
                format!("{}", app.scan_stats.deleted_found),
                Style::default().fg(Color::Red),
            ),
        ]),
        Line::from(vec![
            Span::styled("Bytes Scanned: ", Style::default().fg(Color::White)),
            Span::styled(
                format_size(app.scan_stats.bytes_scanned),
                Style::default().fg(Color::Yellow),
            ),
        ]),
        Line::from(vec![
            Span::styled("Progress: ", Style::default().fg(Color::White)),
            Span::styled(
                format!("{:.1}%", app.scan_progress * 100.0),
                Style::default().fg(Color::Cyan),
            ),
        ]),
    ];

    let stats_widget = Paragraph::new(stats_text).block(stats_block);
    f.render_widget(stats_widget, info_chunks[0]);

    // File details panel
    let details_block = Block::default()
        .borders(Borders::ALL)
        .title("Details")
        .style(Style::default().fg(Color::Cyan));

    let details_text = if app.is_scanning && !app.items.is_empty() {
        // Show scan result details
        if let Some(entry) = app.items.get(app.selected_index) {
            vec![
                Line::from(vec![
                    Span::styled("Path: ", Style::default().fg(Color::White)),
                    Span::raw(entry.path.to_string_lossy()),
                ]),
                Line::from(vec![
                    Span::styled("Size: ", Style::default().fg(Color::White)),
                    Span::styled(format_size(entry.size), Style::default().fg(Color::Yellow)),
                ]),
                Line::from(vec![
                    Span::styled("Status: ", Style::default().fg(Color::White)),
                    Span::styled(
                        if entry.is_deleted { "DELETED" } else { "ACTIVE" },
                        if entry.is_deleted {
                            Style::default().fg(Color::Red)
                        } else {
                            Style::default().fg(Color::Green)
                        },
                    ),
                ]),
            ]
        } else {
            vec![Line::from("No item selected")]
        }
    } else if !app.directory_items.is_empty() {
        // Show directory item details
        if let Some(path) = app.directory_items.get(app.selected_index) {
            let mut lines = vec![
                Line::from(vec![
                    Span::styled("Path: ", Style::default().fg(Color::White)),
                    Span::raw(path.to_string_lossy()),
                ]),
                Line::from(vec![
                    Span::styled("Type: ", Style::default().fg(Color::White)),
                    Span::styled(
                        if path.is_dir() { "Directory" } else { "File" },
                        if path.is_dir() {
                            Style::default().fg(Color::Cyan)
                        } else {
                            Style::default().fg(Color::White)
                        },
                    ),
                ]),
            ];
            
            if path.is_file() {
                if let Ok(metadata) = std::fs::metadata(path) {
                    lines.push(Line::from(vec![
                        Span::styled("Size: ", Style::default().fg(Color::White)),
                        Span::styled(
                            format_size(metadata.len()),
                            Style::default().fg(Color::Yellow),
                        ),
                    ]));
                }
            }
            
            lines
        } else {
            vec![Line::from("No item selected")]
        }
    } else {
        vec![Line::from("No items to display")]
    };

    let details_widget = Paragraph::new(details_text).block(details_block);
    f.render_widget(details_widget, info_chunks[1]);
}

fn draw_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let status_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White));

    let status_line = if app.mode == AppMode::Command {
        format!(":{}", app.command_input)
    } else if !app.status_message.is_empty() {
        app.status_message.clone()
    } else {
        "Commands: [s]can [q]uit [↑↓]nav [Enter]open/recover [:]command [f]orensic [p]ause".to_string()
    };

    let status_style = if app.mode == AppMode::Command {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else if app.is_scanning {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::Green)
    };

    let status_widget = Paragraph::new(status_line)
        .block(status_block)
        .style(status_style)
        .alignment(Alignment::Left);

    f.render_widget(status_widget, area);
}

fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.2} {}", size, UNITS[unit_index])
}

