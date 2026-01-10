# FileTree TUI Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a terminal file explorer with Vim navigation, file operations, and Nerd Font icons in Rust.

**Architecture:** Event-driven TUI using ratatui for rendering, crossterm for terminal handling. Lazy-loaded file tree with internal clipboard for copy/paste operations. Async file operations to keep UI responsive.

**Tech Stack:** Rust, ratatui, crossterm, tokio, ignore crate

---

### Task 1: Project Setup

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`
- Create: `.gitignore`

**Step 1: Initialize Cargo project**

Run: `cargo init --name filetree`

**Step 2: Configure Cargo.toml with dependencies**

```toml
[package]
name = "filetree"
version = "0.1.0"
edition = "2021"

[dependencies]
ratatui = "0.29"
crossterm = "0.28"
tokio = { version = "1", features = ["full"] }
ignore = "0.4"
dirs = "5"
anyhow = "1"

[profile.release]
lto = true
codegen-units = 1
panic = "abort"
strip = true
```

**Step 3: Create .gitignore**

```
/target
```

**Step 4: Verify build**

Run: `cargo build`
Expected: Build succeeds

**Step 5: Initialize git and commit**

```bash
git init
git add .
git commit -m "chore: initial project setup with dependencies"
```

---

### Task 2: File Entry Model

**Files:**
- Create: `src/fs/mod.rs`
- Create: `src/fs/entry.rs`
- Modify: `src/main.rs`

**Step 1: Create fs module structure**

Create `src/fs/mod.rs`:
```rust
pub mod entry;

pub use entry::FileEntry;
```

**Step 2: Create FileEntry struct**

Create `src/fs/entry.rs`:
```rust
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub enum EntryType {
    File,
    Directory,
    Symlink,
}

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub name: String,
    pub path: PathBuf,
    pub entry_type: EntryType,
    pub is_hidden: bool,
    pub is_expanded: bool,
    pub depth: usize,
    pub is_executable: bool,
}

impl FileEntry {
    pub fn new(path: PathBuf, depth: usize) -> anyhow::Result<Self> {
        let metadata = path.symlink_metadata()?;
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string());

        let entry_type = if metadata.is_symlink() {
            EntryType::Symlink
        } else if metadata.is_dir() {
            EntryType::Directory
        } else {
            EntryType::File
        };

        let is_hidden = name.starts_with('.');

        #[cfg(unix)]
        let is_executable = {
            use std::os::unix::fs::PermissionsExt;
            metadata.permissions().mode() & 0o111 != 0
        };
        #[cfg(not(unix))]
        let is_executable = false;

        Ok(Self {
            name,
            path,
            entry_type,
            is_hidden,
            is_expanded: false,
            depth,
            is_executable,
        })
    }

    pub fn is_dir(&self) -> bool {
        matches!(self.entry_type, EntryType::Directory)
    }
}
```

**Step 3: Update main.rs to use module**

```rust
mod fs;

fn main() {
    println!("FileTree TUI");
}
```

**Step 4: Verify build**

Run: `cargo build`
Expected: Build succeeds

**Step 5: Commit**

```bash
git add .
git commit -m "feat: add FileEntry model with metadata handling"
```

---

### Task 3: Icon Mapping

**Files:**
- Create: `src/icons.rs`
- Modify: `src/main.rs`

**Step 1: Create icons module**

Create `src/icons.rs`:
```rust
use std::collections::HashMap;
use std::sync::LazyLock;

pub static ICONS: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    // Directories
    m.insert("dir_open", "󰉋 ");
    m.insert("dir_closed", "󰉖 ");

    // Programming languages
    m.insert("rs", " ");
    m.insert("py", " ");
    m.insert("js", " ");
    m.insert("ts", " ");
    m.insert("jsx", " ");
    m.insert("tsx", " ");
    m.insert("go", " ");
    m.insert("rb", " ");
    m.insert("php", " ");
    m.insert("java", " ");
    m.insert("c", " ");
    m.insert("cpp", " ");
    m.insert("h", " ");
    m.insert("hpp", " ");
    m.insert("cs", "󰌛 ");
    m.insert("swift", " ");
    m.insert("kt", " ");
    m.insert("scala", " ");
    m.insert("hs", " ");
    m.insert("lua", " ");
    m.insert("vim", " ");
    m.insert("sh", " ");
    m.insert("bash", " ");
    m.insert("zsh", " ");
    m.insert("fish", " ");

    // Web
    m.insert("html", " ");
    m.insert("css", " ");
    m.insert("scss", " ");
    m.insert("sass", " ");
    m.insert("less", " ");
    m.insert("vue", " ");
    m.insert("svelte", " ");

    // Data/Config
    m.insert("json", " ");
    m.insert("yaml", " ");
    m.insert("yml", " ");
    m.insert("toml", " ");
    m.insert("xml", "󰗀 ");
    m.insert("csv", " ");
    m.insert("sql", " ");

    // Documentation
    m.insert("md", " ");
    m.insert("txt", " ");
    m.insert("pdf", " ");
    m.insert("doc", "󰈬 ");
    m.insert("docx", "󰈬 ");

    // Images
    m.insert("png", " ");
    m.insert("jpg", " ");
    m.insert("jpeg", " ");
    m.insert("gif", " ");
    m.insert("svg", "󰜡 ");
    m.insert("ico", " ");
    m.insert("webp", " ");

    // Archives
    m.insert("zip", " ");
    m.insert("tar", " ");
    m.insert("gz", " ");
    m.insert("rar", " ");
    m.insert("7z", " ");

    // Git
    m.insert("git", " ");
    m.insert("gitignore", " ");

    // Docker
    m.insert("dockerfile", " ");
    m.insert("docker", " ");

    // Misc
    m.insert("lock", " ");
    m.insert("env", " ");
    m.insert("log", " ");

    // Default
    m.insert("default", " ");
    m
});

pub fn get_icon(filename: &str, is_dir: bool, is_expanded: bool) -> &'static str {
    if is_dir {
        return if is_expanded {
            ICONS.get("dir_open").unwrap_or(&"󰉋 ")
        } else {
            ICONS.get("dir_closed").unwrap_or(&"󰉖 ")
        };
    }

    // Check special filenames first
    let lower_name = filename.to_lowercase();
    if lower_name == "dockerfile" {
        return ICONS.get("dockerfile").unwrap_or(&" ");
    }
    if lower_name.contains(".git") {
        return ICONS.get("git").unwrap_or(&" ");
    }
    if lower_name.ends_with(".lock") {
        return ICONS.get("lock").unwrap_or(&" ");
    }

    // Get extension
    let ext = filename
        .rsplit('.')
        .next()
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    ICONS.get(ext.as_str()).unwrap_or(ICONS.get("default").unwrap_or(&" "))
}
```

**Step 2: Add module to main.rs**

```rust
mod fs;
mod icons;

fn main() {
    println!("FileTree TUI");
}
```

**Step 3: Verify build**

Run: `cargo build`
Expected: Build succeeds

**Step 4: Commit**

```bash
git add .
git commit -m "feat: add Nerd Font icon mapping for file types"
```

---

### Task 4: Application State

**Files:**
- Create: `src/app.rs`
- Modify: `src/main.rs`

**Step 1: Create app state module**

Create `src/app.rs`:
```rust
use crate::fs::FileEntry;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    Normal,
    Search,
    Input(InputKind),
    Confirm(ConfirmKind),
    Help,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputKind {
    CreateFile,
    CreateDir,
    Rename,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConfirmKind {
    Delete,
    Overwrite,
}

#[derive(Debug, Clone)]
pub struct ClipboardEntry {
    pub path: PathBuf,
    pub is_cut: bool,
}

pub struct App {
    pub entries: Vec<FileEntry>,
    pub cursor: usize,
    pub mode: AppMode,
    pub show_hidden: bool,
    pub root_path: PathBuf,
    pub input_buffer: String,
    pub search_query: String,
    pub search_results: Vec<usize>,
    pub search_index: usize,
    pub clipboard: Option<ClipboardEntry>,
    pub status_message: Option<(String, std::time::Instant)>,
    pub should_quit: bool,
}

impl App {
    pub fn new(root_path: PathBuf) -> Self {
        Self {
            entries: Vec::new(),
            cursor: 0,
            mode: AppMode::Normal,
            show_hidden: false,
            root_path,
            input_buffer: String::new(),
            search_query: String::new(),
            search_results: Vec::new(),
            search_index: 0,
            clipboard: None,
            status_message: None,
            should_quit: false,
        }
    }

    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status_message = Some((msg.into(), std::time::Instant::now()));
    }

    pub fn clear_old_status(&mut self) {
        if let Some((_, instant)) = &self.status_message {
            if instant.elapsed().as_secs() >= 3 {
                self.status_message = None;
            }
        }
    }

    pub fn current_entry(&self) -> Option<&FileEntry> {
        self.entries.get(self.cursor)
    }

    pub fn move_cursor_up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    pub fn move_cursor_down(&mut self) {
        if self.cursor < self.entries.len().saturating_sub(1) {
            self.cursor += 1;
        }
    }

    pub fn go_to_top(&mut self) {
        self.cursor = 0;
    }

    pub fn go_to_bottom(&mut self) {
        self.cursor = self.entries.len().saturating_sub(1);
    }
}
```

**Step 2: Update main.rs**

```rust
mod app;
mod fs;
mod icons;

fn main() {
    println!("FileTree TUI");
}
```

**Step 3: Verify build**

Run: `cargo build`
Expected: Build succeeds

**Step 4: Commit**

```bash
git add .
git commit -m "feat: add application state management"
```

---

### Task 5: File Tree Loading

**Files:**
- Modify: `src/app.rs`
- Create: `src/fs/tree.rs`
- Modify: `src/fs/mod.rs`

**Step 1: Create tree loading logic**

Create `src/fs/tree.rs`:
```rust
use super::FileEntry;
use std::path::Path;

pub fn load_directory(path: &Path, depth: usize, show_hidden: bool) -> anyhow::Result<Vec<FileEntry>> {
    let mut entries = Vec::new();

    let read_dir = std::fs::read_dir(path)?;
    let mut items: Vec<_> = read_dir.filter_map(|e| e.ok()).collect();

    // Sort: directories first, then alphabetically (case-insensitive)
    items.sort_by(|a, b| {
        let a_is_dir = a.path().is_dir();
        let b_is_dir = b.path().is_dir();

        match (a_is_dir, b_is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a
                .file_name()
                .to_string_lossy()
                .to_lowercase()
                .cmp(&b.file_name().to_string_lossy().to_lowercase()),
        }
    });

    for item in items {
        let entry = FileEntry::new(item.path(), depth)?;

        if !show_hidden && entry.is_hidden {
            continue;
        }

        entries.push(entry);
    }

    Ok(entries)
}

pub fn build_tree(
    root: &Path,
    expanded_paths: &[std::path::PathBuf],
    show_hidden: bool,
) -> anyhow::Result<Vec<FileEntry>> {
    fn recurse(
        path: &Path,
        depth: usize,
        expanded_paths: &[std::path::PathBuf],
        show_hidden: bool,
        entries: &mut Vec<FileEntry>,
    ) -> anyhow::Result<()> {
        let children = load_directory(path, depth, show_hidden)?;

        for mut child in children {
            let is_expanded = expanded_paths.contains(&child.path);
            child.is_expanded = is_expanded;
            let child_path = child.path.clone();
            let is_dir = child.is_dir();
            entries.push(child);

            if is_dir && is_expanded {
                recurse(&child_path, depth + 1, expanded_paths, show_hidden, entries)?;
            }
        }

        Ok(())
    }

    let mut entries = Vec::new();
    recurse(root, 0, expanded_paths, show_hidden, &mut entries)?;
    Ok(entries)
}
```

**Step 2: Update fs/mod.rs**

```rust
pub mod entry;
pub mod tree;

pub use entry::FileEntry;
pub use tree::{build_tree, load_directory};
```

**Step 3: Add refresh method to App**

Add to `src/app.rs` after the `go_to_bottom` method:

```rust
    pub fn get_expanded_paths(&self) -> Vec<PathBuf> {
        self.entries
            .iter()
            .filter(|e| e.is_expanded)
            .map(|e| e.path.clone())
            .collect()
    }

    pub fn refresh(&mut self) -> anyhow::Result<()> {
        let expanded = self.get_expanded_paths();
        self.entries = crate::fs::build_tree(&self.root_path, &expanded, self.show_hidden)?;

        // Ensure cursor is within bounds
        if self.cursor >= self.entries.len() {
            self.cursor = self.entries.len().saturating_sub(1);
        }

        Ok(())
    }

    pub fn toggle_expand(&mut self) -> anyhow::Result<()> {
        if let Some(entry) = self.entries.get_mut(self.cursor) {
            if entry.is_dir() {
                entry.is_expanded = !entry.is_expanded;
                self.refresh()?;
            }
        }
        Ok(())
    }

    pub fn collapse_or_parent(&mut self) -> anyhow::Result<()> {
        if let Some(entry) = self.entries.get(self.cursor) {
            if entry.is_dir() && entry.is_expanded {
                // Collapse current directory
                if let Some(e) = self.entries.get_mut(self.cursor) {
                    e.is_expanded = false;
                }
                self.refresh()?;
            } else if entry.depth > 0 {
                // Go to parent directory
                let current_depth = entry.depth;
                for i in (0..self.cursor).rev() {
                    if self.entries[i].is_dir() && self.entries[i].depth < current_depth {
                        self.cursor = i;
                        break;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn toggle_hidden(&mut self) -> anyhow::Result<()> {
        self.show_hidden = !self.show_hidden;
        self.refresh()?;
        self.set_status(if self.show_hidden {
            "Showing hidden files"
        } else {
            "Hiding hidden files"
        });
        Ok(())
    }
```

**Step 4: Verify build**

Run: `cargo build`
Expected: Build succeeds

**Step 5: Commit**

```bash
git add .
git commit -m "feat: add file tree loading with expand/collapse support"
```

---

### Task 6: Terminal UI Setup

**Files:**
- Create: `src/ui/mod.rs`
- Create: `src/ui/tree.rs`
- Modify: `src/main.rs`

**Step 1: Create UI module**

Create `src/ui/mod.rs`:
```rust
pub mod tree;

use crate::app::App;
use ratatui::Frame;

pub fn draw(frame: &mut Frame, app: &mut App) {
    app.clear_old_status();
    tree::render(frame, app);
}
```

**Step 2: Create tree renderer**

Create `src/ui/tree.rs`:
```rust
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
```

**Step 3: Update main.rs with TUI setup**

```rust
mod app;
mod fs;
mod icons;
mod ui;

use app::App;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{env, io};

fn main() -> anyhow::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let root_path = env::args()
        .nth(1)
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| ".".into()));

    let mut app = App::new(root_path);
    app.refresh()?;

    // Main loop
    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
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
            if let Event::Key(key) = event::read()? {
                handle_key(app, key.code, key.modifiers)?;
            }
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
                    open_in_editor(app)?;
                }
            }
        }
        KeyCode::Char('g') => app.go_to_top(),
        KeyCode::Char('G') => app.go_to_bottom(),
        KeyCode::Char('H') => app.toggle_hidden()?,
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
            if !input.is_empty() {
                match &app.mode {
                    app::AppMode::Input(kind) => match kind {
                        app::InputKind::CreateFile => create_file(app, &input)?,
                        app::InputKind::CreateDir => create_dir(app, &input)?,
                        app::InputKind::Rename => rename_entry(app, &input)?,
                    },
                    _ => {}
                }
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
                    app::ConfirmKind::Delete => delete_entry(app)?,
                    app::ConfirmKind::Overwrite => {
                        // Handle overwrite confirmation
                    }
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
        app.clipboard = Some(app::ClipboardEntry {
            path: entry.path.clone(),
            is_cut: false,
        });
        app.set_status(format!("Copied: {}", entry.name));
    }
}

fn cut_entry(app: &mut App) {
    if let Some(entry) = app.current_entry() {
        app.clipboard = Some(app::ClipboardEntry {
            path: entry.path.clone(),
            is_cut: true,
        });
        app.set_status(format!("Cut: {}", entry.name));
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
            app.clipboard = Some(clip); // Keep in clipboard for multiple pastes
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

fn open_in_editor(app: &mut App) -> anyhow::Result<()> {
    if let Some(entry) = app.current_entry() {
        if !entry.is_dir() {
            let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());
            let path = entry.path.clone();

            // Restore terminal before opening editor
            disable_raw_mode()?;
            execute!(
                std::io::stdout(),
                LeaveAlternateScreen,
                DisableMouseCapture
            )?;

            std::process::Command::new(&editor)
                .arg(&path)
                .status()?;

            // Re-enable TUI mode
            enable_raw_mode()?;
            execute!(
                std::io::stdout(),
                EnterAlternateScreen,
                EnableMouseCapture
            )?;
        }
    }
    Ok(())
}
```

**Step 4: Verify build and run**

Run: `cargo build && cargo run`
Expected: TUI launches, shows current directory tree

**Step 5: Commit**

```bash
git add .
git commit -m "feat: complete TUI with navigation and file operations"
```

---

### Task 7: Help Screen

**Files:**
- Modify: `src/ui/tree.rs`

**Step 1: Add help overlay rendering**

Add this function to `src/ui/tree.rs` after `render_help_bar`:

```rust
pub fn render_help_overlay(frame: &mut Frame) {
    let area = centered_rect(60, 80, frame.area());

    let help_text = vec![
        Line::from("Navigation").style(Style::default().add_modifier(Modifier::BOLD)),
        Line::from("  j/↓       Move down"),
        Line::from("  k/↑       Move up"),
        Line::from("  h/←       Collapse / go to parent"),
        Line::from("  l/→/Enter Expand / open file"),
        Line::from("  g         Go to top"),
        Line::from("  G         Go to bottom"),
        Line::from(""),
        Line::from("File Operations").style(Style::default().add_modifier(Modifier::BOLD)),
        Line::from("  a         Create file"),
        Line::from("  A         Create directory"),
        Line::from("  r         Rename"),
        Line::from("  d         Delete"),
        Line::from("  y         Copy (yank)"),
        Line::from("  x         Cut"),
        Line::from("  p         Paste"),
        Line::from(""),
        Line::from("Other").style(Style::default().add_modifier(Modifier::BOLD)),
        Line::from("  /         Search"),
        Line::from("  H         Toggle hidden files"),
        Line::from("  ?         Show this help"),
        Line::from("  q         Quit"),
        Line::from(""),
        Line::from("Press Esc or ? to close").style(Style::default().fg(Color::DarkGray)),
    ];

    let paragraph = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title(" Help "))
        .style(Style::default());

    frame.render_widget(ratatui::widgets::Clear, area);
    frame.render_widget(paragraph, area);
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
```

**Step 2: Update render function**

Modify the `render` function in `src/ui/tree.rs`:

```rust
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

    if matches!(app.mode, AppMode::Help) {
        render_help_overlay(frame);
    }
}
```

**Step 3: Verify build and test help screen**

Run: `cargo run`
Expected: Press `?` to see help overlay, `Esc` to close

**Step 4: Commit**

```bash
git add .
git commit -m "feat: add help overlay screen"
```

---

### Task 8: Polish and Release Build

**Files:**
- Modify: `Cargo.toml`
- Create: `README.md` (optional, user requested)

**Step 1: Build release binary**

Run: `cargo build --release`
Expected: Optimized binary at `target/release/filetree`

**Step 2: Test release binary**

Run: `./target/release/filetree`
Expected: Fast startup, smooth navigation

**Step 3: Create install alias (optional)**

Run: `cp target/release/filetree ~/.local/bin/ft`
(Assumes ~/.local/bin is in PATH)

**Step 4: Final commit**

```bash
git add .
git commit -m "chore: release build configuration"
```

---

## Summary

**Total Tasks:** 8
**Estimated Implementation:** 8 task blocks using TDD approach

**Key Files:**
- `src/main.rs` - Entry point, event handling
- `src/app.rs` - Application state
- `src/fs/entry.rs` - File entry model
- `src/fs/tree.rs` - Tree loading
- `src/ui/tree.rs` - TUI rendering
- `src/icons.rs` - Nerd Font mapping
