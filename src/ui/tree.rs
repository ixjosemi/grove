use crate::app::{App, AppMode};
use crate::icons::get_icon;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(frame.area());

    render_tree(frame, app, chunks[0]);
    render_input_or_status(frame, app, chunks[1]);
    render_help_bar(frame, app, chunks[2]);
}

fn render_tree(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .entries
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let indent = "  ".repeat(entry.depth);
            let icon = get_icon(&entry.name, entry.is_dir(), entry.is_expanded);
            let name = &entry.name;

            let style = if i == app.cursor {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else if entry.is_dir() {
                Style::default().fg(Color::Blue)
            } else if entry.is_executable {
                Style::default().fg(Color::Green)
            } else if entry.is_hidden {
                Style::default().fg(Color::DarkGray)
            } else if matches!(entry.entry_type, crate::fs::entry::EntryType::Symlink) {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default()
            };

            let line = Line::from(vec![
                Span::raw(indent),
                Span::styled(format!("{icon}{name}"), style),
            ]);

            ListItem::new(line)
        })
        .collect();

    let title = app
        .root_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| app.root_path.to_string_lossy().to_string());

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(format!(" {} ", title)));

    frame.render_widget(list, area);
}

fn render_input_or_status(frame: &mut Frame, app: &App, area: Rect) {
    let widget = match &app.mode {
        AppMode::Input(kind) => {
            let label = match kind {
                crate::app::InputKind::CreateFile => "New file: ",
                crate::app::InputKind::CreateDir => "New directory: ",
                crate::app::InputKind::Rename => "Rename: ",
            };
            Paragraph::new(format!("{}{}", label, app.input_buffer))
                .style(Style::default().fg(Color::Yellow))
        }
        AppMode::Search => {
            let count = app.search_results.len();
            let idx = if count > 0 { app.search_index + 1 } else { 0 };
            Paragraph::new(format!("/{} ({}/{})", app.search_query, idx, count))
                .style(Style::default().fg(Color::Yellow))
        }
        AppMode::Confirm(kind) => {
            let msg = match kind {
                crate::app::ConfirmKind::Delete => {
                    let name = app.current_entry().map(|e| e.name.as_str()).unwrap_or("");
                    format!("Delete \"{}\"? [y/N]", name)
                }
                crate::app::ConfirmKind::Overwrite => "File exists. Overwrite? [y/N]".to_string(),
            };
            Paragraph::new(msg).style(Style::default().fg(Color::Red))
        }
        _ => {
            if let Some((msg, _)) = &app.status_message {
                Paragraph::new(msg.as_str()).style(Style::default().fg(Color::Green))
            } else {
                Paragraph::new("")
            }
        }
    };

    frame.render_widget(widget, area);
}

fn render_help_bar(frame: &mut Frame, app: &App, area: Rect) {
    let help_text = match &app.mode {
        AppMode::Normal => "[a]dd [A]dd dir [r]ename [d]elete [y]ank [x]cut [p]aste [/]search [H]idden [?]help [q]uit",
        AppMode::Search => "[Enter]confirm [n]ext [N]prev [Esc]cancel",
        AppMode::Input(_) => "[Enter]confirm [Esc]cancel",
        AppMode::Confirm(_) => "[y]es [n]o",
        AppMode::Help => "[Esc]close [q]uit",
    };

    let paragraph = Paragraph::new(help_text).style(Style::default().fg(Color::DarkGray));

    frame.render_widget(paragraph, area);
}
