use crate::fs::FileEntry;
use crate::preview::PreviewData;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};

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
    ConfirmDelete,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConfirmKind {
    Delete,
    #[allow(dead_code)]
    Overwrite,
}

#[derive(Debug, Clone)]
pub struct ClipboardEntry {
    pub path: PathBuf,
    pub is_cut: bool,
}

const RECENT_CHANGE_DURATION: Duration = Duration::from_secs(5);

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
    pub status_message: Option<(String, Instant)>,
    pub should_quit: bool,
    pub pending_editor_file: Option<PathBuf>,
    pub last_click: Option<(Instant, usize)>,
    // Live file monitoring
    pub watcher_rx: Option<Receiver<PathBuf>>,
    pub recent_changes: HashMap<PathBuf, Instant>,
    pub watcher_active: bool,
    // Preview
    pub preview_cache: HashMap<PathBuf, PreviewData>,
    pub show_preview: bool,
    pub preview_scroll: usize,
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
            pending_editor_file: None,
            last_click: None,
            watcher_rx: None,
            recent_changes: HashMap::new(),
            watcher_active: false,
            preview_cache: HashMap::new(),
            show_preview: false,
            preview_scroll: 0,
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

    pub fn expand_all(&mut self) -> anyhow::Result<()> {
        self.entries = crate::fs::build_tree_fully_expanded(&self.root_path, self.show_hidden)?;

        // Ensure cursor is within bounds
        if self.cursor >= self.entries.len() {
            self.cursor = self.entries.len().saturating_sub(1);
        }

        let status = if self.entries.len() >= 5000 {
            format!("Expanded all (limited to {} entries)", self.entries.len())
        } else {
            format!("Expanded all ({} entries)", self.entries.len())
        };
        self.set_status(status);
        Ok(())
    }

    pub fn collapse_all(&mut self) -> anyhow::Result<()> {
        for entry in &mut self.entries {
            if entry.is_dir() {
                entry.is_expanded = false;
            }
        }
        self.refresh()?;
        self.set_status("Collapsed all directories");
        Ok(())
    }

    // Watcher methods
    pub fn check_watcher(&mut self) {
        if let Some(rx) = &self.watcher_rx {
            // Non-blocking: drain all pending events
            while let Ok(path) = rx.try_recv() {
                self.recent_changes.insert(path.clone(), Instant::now());
                // Invalidate preview cache for this path
                self.preview_cache.remove(&path);
            }
        }
    }

    pub fn cleanup_old_changes(&mut self) {
        self.recent_changes
            .retain(|_, instant| instant.elapsed() < RECENT_CHANGE_DURATION);
    }

    pub fn is_recently_changed(&self, path: &Path) -> bool {
        self.recent_changes
            .get(path)
            .map(|instant| instant.elapsed() < RECENT_CHANGE_DURATION)
            .unwrap_or(false)
    }

    // Preview methods
    pub fn get_cached_preview(&self) -> Option<&PreviewData> {
        self.current_entry()
            .and_then(|entry| self.preview_cache.get(&entry.path))
    }

    pub fn toggle_preview(&mut self) {
        if self.show_preview {
            self.show_preview = false;
            self.preview_scroll = 0;
        } else {
            self.show_preview = true;
            self.preview_scroll = 0;
            self.generate_current_preview();
        }
    }

    pub fn generate_current_preview(&mut self) {
        if let Some(entry) = self.current_entry() {
            let path = entry.path.clone();
            if !self.preview_cache.contains_key(&path) {
                if let Ok(preview) = crate::preview::generate_preview(&path) {
                    self.preview_cache.insert(path, preview);
                }
            }
        }
    }

    pub fn scroll_preview_up(&mut self) {
        self.preview_scroll = self.preview_scroll.saturating_sub(5);
    }

    pub fn scroll_preview_down(&mut self) {
        self.preview_scroll = self.preview_scroll.saturating_add(5);
    }
}
