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
