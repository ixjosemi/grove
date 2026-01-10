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
