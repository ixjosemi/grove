use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher, event::EventKind};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;

pub struct FileWatcher {
    #[allow(dead_code)]
    watcher: RecommendedWatcher,
}

pub fn start_watcher(root: &Path) -> anyhow::Result<(FileWatcher, Receiver<PathBuf>)> {
    let (tx, rx) = mpsc::channel();

    let watcher = RecommendedWatcher::new(
        move |res: Result<notify::Event, notify::Error>| {
            if let Ok(event) = res {
                if should_process_event(&event.kind) {
                    for path in event.paths {
                        if !should_ignore_path(&path) {
                            let _ = tx.send(path);
                        }
                    }
                }
            }
        },
        Config::default().with_poll_interval(Duration::from_millis(300)),
    )?;

    let mut file_watcher = FileWatcher { watcher };
    file_watcher
        .watcher
        .watch(root, RecursiveMode::Recursive)?;

    Ok((file_watcher, rx))
}

fn should_process_event(kind: &EventKind) -> bool {
    matches!(
        kind,
        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
    )
}

fn should_ignore_path(path: &Path) -> bool {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy())
        .unwrap_or_default();

    // Ignore editor temp files and system files
    name.ends_with(".swp")
        || name.ends_with(".swo")
        || name.ends_with('~')
        || name.starts_with(".#")
        || name == ".DS_Store"
        || name.ends_with(".tmp")
        || name.contains(".git")
}
