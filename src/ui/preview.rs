use crate::app::App;
use crate::icons::get_icon;
use crate::preview::{format_permissions, format_size, PreviewContent, PreviewData};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use std::time::SystemTime;

pub fn render_preview_overlay(frame: &mut Frame, app: &App, preview: &PreviewData) {
    let area = centered_rect(60, 70, frame.area());

    // Clear the area behind the overlay
    frame.render_widget(Clear, area);

    let filename = preview
        .path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| preview.path.to_string_lossy().to_string());

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" Preview: {} ", filename));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Split inner area: metadata (2 lines) + content
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(1)])
        .split(inner);

    render_metadata(frame, preview, chunks[0]);
    render_content(frame, app, preview, chunks[1]);
}

fn render_metadata(frame: &mut Frame, preview: &PreviewData, area: Rect) {
    let size_str = format_size(preview.metadata.size);
    let perms_str = format_permissions(preview.metadata.permissions);
    let modified_str = preview
        .metadata
        .modified
        .map(format_time)
        .unwrap_or_else(|| "---".to_string());

    let is_dir = matches!(preview.content, PreviewContent::Directory(_));
    let type_info = if is_dir {
        match &preview.content {
            PreviewContent::Directory(children) => format!("{} items", children.len()),
            _ => "directory".to_string(),
        }
    } else {
        size_str
    };

    let meta_line = format!(
        "{}  |  Modified: {}  |  Permissions: {}",
        type_info, modified_str, perms_str
    );

    let paragraph = Paragraph::new(meta_line).style(Style::default().fg(Color::DarkGray));
    frame.render_widget(paragraph, area);
}

fn render_content(frame: &mut Frame, app: &App, preview: &PreviewData, area: Rect) {
    let lines: Vec<Line> = match &preview.content {
        PreviewContent::Text(text_lines) => {
            let total = text_lines.len();
            let visible_height = area.height as usize;
            let start = app.preview_scroll.min(total.saturating_sub(1));
            let end = (start + visible_height).min(total);

            let mut result: Vec<Line> = text_lines[start..end]
                .iter()
                .map(|l| Line::from(l.as_str()))
                .collect();

            // Add scroll indicator if needed
            if total > visible_height {
                let indicator = format!("[{}-{}/{}]", start + 1, end, total);
                if result.len() < visible_height {
                    result.push(Line::from(""));
                }
                result.push(Line::from(Span::styled(
                    indicator,
                    Style::default().fg(Color::DarkGray),
                )));
            }

            result
        }
        PreviewContent::Directory(children) => {
            let total = children.len();
            let visible_height = area.height as usize;
            let start = app.preview_scroll.min(total.saturating_sub(1));
            let end = (start + visible_height).min(total);

            let mut result: Vec<Line> = children[start..end]
                .iter()
                .map(|child| {
                    let icon = get_icon(&child.name, child.is_dir, false);
                    let style = if child.is_dir {
                        Style::default().fg(Color::Blue)
                    } else {
                        Style::default()
                    };
                    Line::from(Span::styled(format!("{}{}", icon, child.name), style))
                })
                .collect();

            // Add scroll indicator if needed
            if total > visible_height {
                let indicator = format!("[{}-{}/{}]", start + 1, end, total);
                result.push(Line::from(Span::styled(
                    indicator,
                    Style::default().fg(Color::DarkGray),
                )));
            }

            result
        }
        PreviewContent::Binary => {
            vec![Line::from(Span::styled(
                "[Binary file]",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::ITALIC),
            ))]
        }
        PreviewContent::TooLarge => {
            vec![Line::from(Span::styled(
                "[File too large to preview (>50KB)]",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::ITALIC),
            ))]
        }
        PreviewContent::Empty => {
            vec![Line::from(Span::styled(
                "[Empty]",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
            ))]
        }
        PreviewContent::Error(msg) => {
            vec![Line::from(Span::styled(
                format!("[Error: {}]", msg),
                Style::default().fg(Color::Red),
            ))]
        }
    };

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, area);
}

fn format_time(time: SystemTime) -> String {
    let datetime: chrono::DateTime<chrono::Local> = time.into();
    datetime.format("%Y-%m-%d %H:%M").to_string()
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
