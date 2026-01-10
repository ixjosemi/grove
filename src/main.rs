mod app;
mod fs;
mod icons;
mod ui;

use app::App;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers, MouseEventKind, MouseButton},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{env, io};

fn main() -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let root_path = env::args()
        .nth(1)
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| ".".into()));

    let mut app = App::new(root_path);
    app.refresh()?;

    let res = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {err}");
    }

    Ok(())
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> anyhow::Result<()> {
    loop {
        terminal.draw(|f| ui::draw(f, app))?;

        if event::poll(std::time::Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) => {
                    handle_key(app, key.code, key.modifiers)?;
                }
                Event::Mouse(mouse) => {
                    handle_mouse(app, mouse.kind, mouse.row, mouse.column)?;
                }
                _ => {}
            }
        }

        // Handle pending editor file open
        if let Some(path) = app.pending_editor_file.take() {
            open_in_editor(terminal, &path)?;
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

fn handle_key(app: &mut App, key: KeyCode, _modifiers: KeyModifiers) -> anyhow::Result<()> {
    use app::AppMode;

    match &app.mode {
        AppMode::Normal => handle_normal_mode(app, key),
        AppMode::Search => handle_search_mode(app, key),
        AppMode::Input(_) => handle_input_mode(app, key),
        AppMode::Confirm(_) => handle_confirm_mode(app, key),
        AppMode::Help => handle_help_mode(app, key),
    }
}

fn handle_normal_mode(app: &mut App, key: KeyCode) -> anyhow::Result<()> {
    match key {
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Char('j') | KeyCode::Down => app.move_cursor_down(),
        KeyCode::Char('k') | KeyCode::Up => app.move_cursor_up(),
        KeyCode::Char('h') | KeyCode::Left => app.collapse_or_parent()?,
        KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => {
            if let Some(entry) = app.current_entry() {
                if entry.is_dir() {
                    app.toggle_expand()?;
                } else {
                    // Queue file for opening - handled in main loop
                    app.pending_editor_file = Some(entry.path.clone());
                }
            }
        }
        KeyCode::Char('g') => app.go_to_top(),
        KeyCode::Char('G') => app.go_to_bottom(),
        KeyCode::Char('H') => app.toggle_hidden()?,
        KeyCode::Char('R') => {
            app.refresh()?;
            app.set_status("Refreshed");
        }
        KeyCode::Char('E') => app.expand_all()?,
        KeyCode::Char('W') => app.collapse_all()?,
        KeyCode::Char('O') => open_in_file_manager(app)?,
        KeyCode::Char('/') => {
            app.mode = app::AppMode::Search;
            app.search_query.clear();
            app.search_results.clear();
        }
        KeyCode::Char('a') => {
            app.mode = app::AppMode::Input(app::InputKind::CreateFile);
            app.input_buffer.clear();
        }
        KeyCode::Char('A') => {
            app.mode = app::AppMode::Input(app::InputKind::CreateDir);
            app.input_buffer.clear();
        }
        KeyCode::Char('r') => {
            if let Some(entry) = app.current_entry() {
                app.input_buffer = entry.name.clone();
                app.mode = app::AppMode::Input(app::InputKind::Rename);
            }
        }
        KeyCode::Char('d') => {
            if app.current_entry().is_some() {
                app.mode = app::AppMode::Confirm(app::ConfirmKind::Delete);
            }
        }
        KeyCode::Char('y') => yank_entry(app),
        KeyCode::Char('x') => cut_entry(app),
        KeyCode::Char('p') => paste_entry(app)?,
        KeyCode::Char('?') => app.mode = app::AppMode::Help,
        _ => {}
    }
    Ok(())
}

fn handle_search_mode(app: &mut App, key: KeyCode) -> anyhow::Result<()> {
    match key {
        KeyCode::Esc => {
            app.mode = app::AppMode::Normal;
            app.search_query.clear();
            app.search_results.clear();
        }
        KeyCode::Enter => {
            app.mode = app::AppMode::Normal;
        }
        KeyCode::Char('n') if app.search_query.is_empty() => {}
        KeyCode::Char('n') => {
            if !app.search_results.is_empty() {
                app.search_index = (app.search_index + 1) % app.search_results.len();
                app.cursor = app.search_results[app.search_index];
            }
        }
        KeyCode::Char('N') => {
            if !app.search_results.is_empty() {
                app.search_index = app
                    .search_index
                    .checked_sub(1)
                    .unwrap_or(app.search_results.len() - 1);
                app.cursor = app.search_results[app.search_index];
            }
        }
        KeyCode::Backspace => {
            app.search_query.pop();
            update_search_results(app);
        }
        KeyCode::Char(c) => {
            app.search_query.push(c);
            update_search_results(app);
        }
        _ => {}
    }
    Ok(())
}

fn handle_input_mode(app: &mut App, key: KeyCode) -> anyhow::Result<()> {
    match key {
        KeyCode::Esc => {
            app.mode = app::AppMode::Normal;
            app.input_buffer.clear();
        }
        KeyCode::Enter => {
            let input = app.input_buffer.clone();
            match &app.mode {
                app::AppMode::Input(kind) => match kind {
                    app::InputKind::CreateFile => {
                        if !input.is_empty() {
                            create_file(app, &input)?;
                        }
                    }
                    app::InputKind::CreateDir => {
                        if !input.is_empty() {
                            create_dir(app, &input)?;
                        }
                    }
                    app::InputKind::Rename => {
                        if !input.is_empty() {
                            rename_entry(app, &input)?;
                        }
                    }
                    app::InputKind::ConfirmDelete => {
                        if input == "yes" {
                            delete_entry(app)?;
                        } else {
                            app.set_status("Delete cancelled");
                        }
                    }
                },
                _ => {}
            }
            app.mode = app::AppMode::Normal;
            app.input_buffer.clear();
        }
        KeyCode::Backspace => {
            app.input_buffer.pop();
        }
        KeyCode::Char(c) => {
            app.input_buffer.push(c);
        }
        _ => {}
    }
    Ok(())
}

fn handle_confirm_mode(app: &mut App, key: KeyCode) -> anyhow::Result<()> {
    match key {
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            if let app::AppMode::Confirm(kind) = &app.mode {
                match kind {
                    app::ConfirmKind::Delete => {
                        // Second confirmation: require typing "yes"
                        app.input_buffer.clear();
                        app.mode = app::AppMode::Input(app::InputKind::ConfirmDelete);
                        return Ok(());
                    }
                    app::ConfirmKind::Overwrite => {}
                }
            }
            app.mode = app::AppMode::Normal;
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            app.mode = app::AppMode::Normal;
        }
        _ => {}
    }
    Ok(())
}

fn handle_help_mode(app: &mut App, key: KeyCode) -> anyhow::Result<()> {
    match key {
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') => {
            app.mode = app::AppMode::Normal;
        }
        _ => {}
    }
    Ok(())
}

fn handle_mouse(app: &mut App, kind: MouseEventKind, row: u16, _column: u16) -> anyhow::Result<()> {
    // Only handle mouse in Normal mode
    if !matches!(app.mode, app::AppMode::Normal) {
        return Ok(());
    }

    // Tree area starts at row 1 (after border)
    let tree_start_row: u16 = 1;

    match kind {
        MouseEventKind::Down(MouseButton::Left) => {
            if row >= tree_start_row {
                let clicked_index = (row - tree_start_row) as usize;
                if clicked_index < app.entries.len() {
                    // Check for double click
                    let now = std::time::Instant::now();
                    let is_double_click = if let Some((last_time, last_index)) = app.last_click {
                        last_index == clicked_index && now.duration_since(last_time).as_millis() < 400
                    } else {
                        false
                    };

                    app.cursor = clicked_index;

                    if is_double_click {
                        // Double click: toggle directory
                        if let Some(entry) = app.current_entry() {
                            if entry.is_dir() {
                                app.toggle_expand()?;
                            }
                        }
                        app.last_click = None;
                    } else {
                        app.last_click = Some((now, clicked_index));
                    }
                }
            }
        }
        MouseEventKind::Down(MouseButton::Right) => {
            if row >= tree_start_row {
                let clicked_index = (row - tree_start_row) as usize;
                if clicked_index < app.entries.len() {
                    app.cursor = clicked_index;
                    // Open file or toggle directory
                    if let Some(entry) = app.current_entry() {
                        if entry.is_dir() {
                            app.toggle_expand()?;
                        } else {
                            app.pending_editor_file = Some(entry.path.clone());
                        }
                    }
                }
            }
        }
        MouseEventKind::ScrollUp => {
            for _ in 0..3 {
                app.move_cursor_up();
            }
        }
        MouseEventKind::ScrollDown => {
            for _ in 0..3 {
                app.move_cursor_down();
            }
        }
        _ => {}
    }

    Ok(())
}

fn update_search_results(app: &mut App) {
    app.search_results.clear();
    if app.search_query.is_empty() {
        return;
    }

    let query = app.search_query.to_lowercase();
    for (i, entry) in app.entries.iter().enumerate() {
        if entry.name.to_lowercase().contains(&query) {
            app.search_results.push(i);
        }
    }

    if !app.search_results.is_empty() {
        app.search_index = 0;
        app.cursor = app.search_results[0];
    }
}

fn get_target_dir(app: &App) -> std::path::PathBuf {
    app.current_entry()
        .map(|e| {
            if e.is_dir() {
                e.path.clone()
            } else {
                e.path.parent().unwrap_or(&app.root_path).to_path_buf()
            }
        })
        .unwrap_or_else(|| app.root_path.clone())
}

fn create_file(app: &mut App, name: &str) -> anyhow::Result<()> {
    let dir = get_target_dir(app);
    let path = dir.join(name);
    std::fs::File::create(&path)?;
    app.refresh()?;
    app.set_status(format!("Created: {}", name));
    Ok(())
}

fn create_dir(app: &mut App, name: &str) -> anyhow::Result<()> {
    let dir = get_target_dir(app);
    let path = dir.join(name);
    std::fs::create_dir(&path)?;
    app.refresh()?;
    app.set_status(format!("Created directory: {}", name));
    Ok(())
}

fn rename_entry(app: &mut App, new_name: &str) -> anyhow::Result<()> {
    if let Some(entry) = app.current_entry() {
        let old_path = entry.path.clone();
        let new_path = old_path.parent().unwrap().join(new_name);
        std::fs::rename(&old_path, &new_path)?;
        app.refresh()?;
        app.set_status(format!("Renamed to: {}", new_name));
    }
    Ok(())
}

fn delete_entry(app: &mut App) -> anyhow::Result<()> {
    if let Some(entry) = app.current_entry() {
        let path = entry.path.clone();
        let name = entry.name.clone();
        if entry.is_dir() {
            std::fs::remove_dir_all(&path)?;
        } else {
            std::fs::remove_file(&path)?;
        }
        app.refresh()?;
        app.set_status(format!("Deleted: {}", name));
    }
    Ok(())
}

fn yank_entry(app: &mut App) {
    if let Some(entry) = app.current_entry() {
        let path = entry.path.clone();
        let name = entry.name.clone();
        app.clipboard = Some(app::ClipboardEntry {
            path,
            is_cut: false,
        });
        app.set_status(format!("Copied: {}", name));
    }
}

fn cut_entry(app: &mut App) {
    if let Some(entry) = app.current_entry() {
        let path = entry.path.clone();
        let name = entry.name.clone();
        app.clipboard = Some(app::ClipboardEntry {
            path,
            is_cut: true,
        });
        app.set_status(format!("Cut: {}", name));
    }
}

fn paste_entry(app: &mut App) -> anyhow::Result<()> {
    if let Some(clip) = app.clipboard.take() {
        let target_dir = get_target_dir(app);
        let file_name = clip.path.file_name().unwrap();
        let dest = target_dir.join(file_name);

        if clip.is_cut {
            std::fs::rename(&clip.path, &dest)?;
            app.set_status(format!("Moved: {}", file_name.to_string_lossy()));
        } else {
            if clip.path.is_dir() {
                copy_dir_recursive(&clip.path, &dest)?;
            } else {
                std::fs::copy(&clip.path, &dest)?;
            }
            app.set_status(format!("Pasted: {}", file_name.to_string_lossy()));
            app.clipboard = Some(clip);
        }
        app.refresh()?;
    }
    Ok(())
}

fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path) -> anyhow::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let dest_path = dst.join(entry.file_name());

        if path.is_dir() {
            copy_dir_recursive(&path, &dest_path)?;
        } else {
            std::fs::copy(&path, &dest_path)?;
        }
    }
    Ok(())
}

fn open_in_editor(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    path: &std::path::Path,
) -> anyhow::Result<()> {
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());

    // Leave TUI mode
    disable_raw_mode()?;
    execute!(
        io::stdout(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;

    // Run editor
    std::process::Command::new(&editor)
        .arg(path)
        .status()?;

    // Restore TUI mode
    enable_raw_mode()?;
    execute!(
        io::stdout(),
        EnterAlternateScreen,
        EnableMouseCapture
    )?;

    // Drain any pending input events
    while event::poll(std::time::Duration::from_millis(50))? {
        let _ = event::read()?;
    }

    // Force full terminal refresh
    terminal.clear()?;

    Ok(())
}

fn open_in_file_manager(app: &mut App) -> anyhow::Result<()> {
    let path = if let Some(entry) = app.current_entry() {
        if entry.is_dir() {
            entry.path.clone()
        } else {
            entry.path.parent().unwrap_or(&app.root_path).to_path_buf()
        }
    } else {
        app.root_path.clone()
    };

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&path)
            .spawn()?;
        app.set_status(format!("Opened in Finder: {}", path.display()));
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&path)
            .spawn()?;
        app.set_status(format!("Opened in file manager: {}", path.display()));
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&path)
            .spawn()?;
        app.set_status(format!("Opened in Explorer: {}", path.display()));
    }

    Ok(())
}
