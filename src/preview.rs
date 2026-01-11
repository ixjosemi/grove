use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime};

const MAX_PREVIEW_LINES: usize = 25;
const MAX_PREVIEW_SIZE: u64 = 50 * 1024; // 50KB
const BINARY_CHECK_SIZE: usize = 512;

#[derive(Debug, Clone)]
pub struct PreviewData {
    pub path: PathBuf,
    pub content: PreviewContent,
    pub metadata: PreviewMetadata,
    pub cached_at: Instant,
}

#[derive(Debug, Clone)]
pub struct PreviewMetadata {
    pub size: u64,
    pub modified: Option<SystemTime>,
    pub permissions: u32,
}

#[derive(Debug, Clone)]
pub struct DirChild {
    pub name: String,
    pub is_dir: bool,
}

#[derive(Debug, Clone)]
pub enum PreviewContent {
    Text(Vec<String>),
    Directory(Vec<DirChild>),
    Binary,
    TooLarge,
    Empty,
    Error(String),
}

pub fn generate_preview(path: &Path) -> anyhow::Result<PreviewData> {
    let metadata = fs::metadata(path)?;
    let preview_metadata = PreviewMetadata {
        size: metadata.len(),
        modified: metadata.modified().ok(),
        permissions: get_permissions(&metadata),
    };

    let content = if metadata.is_dir() {
        generate_dir_preview(path)
    } else {
        generate_file_preview(path, metadata.len())
    };

    Ok(PreviewData {
        path: path.to_path_buf(),
        content,
        metadata: preview_metadata,
        cached_at: Instant::now(),
    })
}

fn generate_dir_preview(path: &Path) -> PreviewContent {
    match fs::read_dir(path) {
        Ok(entries) => {
            let mut children: Vec<DirChild> = entries
                .filter_map(|e| e.ok())
                .map(|e| DirChild {
                    name: e.file_name().to_string_lossy().to_string(),
                    is_dir: e.path().is_dir(),
                })
                .collect();

            // Sort: directories first, then alphabetically
            children.sort_by(|a, b| match (a.is_dir, b.is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            });

            if children.is_empty() {
                PreviewContent::Empty
            } else {
                PreviewContent::Directory(children)
            }
        }
        Err(e) => PreviewContent::Error(e.to_string()),
    }
}

fn generate_file_preview(path: &Path, size: u64) -> PreviewContent {
    if size == 0 {
        return PreviewContent::Empty;
    }

    if size > MAX_PREVIEW_SIZE {
        return PreviewContent::TooLarge;
    }

    // Check if binary
    if let Ok(bytes) = fs::read(path) {
        let check_len = bytes.len().min(BINARY_CHECK_SIZE);
        if bytes[..check_len].contains(&0) {
            return PreviewContent::Binary;
        }
    }

    // Read text lines
    match fs::File::open(path) {
        Ok(file) => {
            let reader = BufReader::new(file);
            let lines: Vec<String> = reader
                .lines()
                .take(MAX_PREVIEW_LINES)
                .filter_map(|l| l.ok())
                .map(|l| {
                    // Truncate very long lines
                    if l.len() > 200 {
                        format!("{}...", &l[..200])
                    } else {
                        l
                    }
                })
                .collect();

            if lines.is_empty() {
                PreviewContent::Empty
            } else {
                PreviewContent::Text(lines)
            }
        }
        Err(e) => PreviewContent::Error(e.to_string()),
    }
}

#[cfg(unix)]
fn get_permissions(metadata: &fs::Metadata) -> u32 {
    use std::os::unix::fs::PermissionsExt;
    metadata.permissions().mode()
}

#[cfg(not(unix))]
fn get_permissions(_metadata: &fs::Metadata) -> u32 {
    0
}

pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

pub fn format_permissions(mode: u32) -> String {
    if mode == 0 {
        "---".to_string()
    } else {
        format!("{:o}", mode & 0o777)
    }
}
