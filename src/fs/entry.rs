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
