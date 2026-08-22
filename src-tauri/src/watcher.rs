use crate::models::AppConfig;
use crate::storage;
use anyhow::Result;
use chrono::Local;
use notify::event::{ModifyKind, RenameMode};
use notify::{Event, EventKind, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub enum FsEventKind {
    Create,
    Modify,
    Delete,
    Rename { from: String },
}

#[derive(Debug, Clone)]
pub struct FsEvent {
    pub path: String,
    pub kind: FsEventKind,
    pub is_dir: bool,
    pub outside: bool,
    /// Wall-clock "%Y-%m-%d %H:%M:%S" when the event was observed.
    pub timestamp: String,
}

fn now_str() -> String {
    Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

pub type SharedWatcher = Arc<Mutex<Option<notify::RecommendedWatcher>>>;

fn is_ignored_path(path: &Path, workspace: &Path, cfg: &AppConfig) -> bool {
    let Some(rel) = path.strip_prefix(workspace).ok() else {
        return false;
    };
    storage::is_ignored(&rel.to_string_lossy(), cfg)
}

/// Map a notify event to one or more normalized FsEvents.
fn normalize(event: &Event, workspace: &Path, cfg: &AppConfig, detect_outside: bool) -> Vec<FsEvent> {
    let mut out = Vec::new();

    let is_dir = event
        .paths
        .first()
        .and_then(|p| p.metadata().ok())
        .map(|m| m.is_dir())
        .unwrap_or(false);

    for path in &event.paths {
        let path = path.clone();
        let (rel, outside) = if let Ok(rel) = path.strip_prefix(workspace) {
            let rel_str = rel.to_string_lossy().replace('\\', "/");
            if rel_str.is_empty() {
                continue;
            }
            (rel_str, false)
        } else if detect_outside && !path.starts_with(workspace) {
            let rel_str = path.to_string_lossy().replace('\\', "/");
            (rel_str, true)
        } else {
            continue;
        };

        if !outside && is_ignored_path(&path, workspace, cfg) {
            continue;
        }

        let kind = match &event.kind {
            EventKind::Create(_) => FsEventKind::Create,
            EventKind::Remove(_) => FsEventKind::Delete,
            EventKind::Modify(ModifyKind::Name(mode)) => match mode {
                RenameMode::Both => {
                    // paths[0] = from, paths[1] = to
                    if event.paths.len() >= 2 && event.paths[0] == path {
                        if let Ok(from) = event.paths[0].strip_prefix(workspace) {
                            FsEventKind::Rename {
                                from: from.to_string_lossy().replace('\\', "/"),
                            }
                        } else {
                            FsEventKind::Create
                        }
                    } else if event.paths.len() >= 2 && event.paths[1] == path {
                        FsEventKind::Create
                    } else {
                        FsEventKind::Create
                    }
                }
                RenameMode::From => FsEventKind::Delete,
                RenameMode::To => FsEventKind::Create,
                RenameMode::Any | RenameMode::Other => {
                    if outside {
                        FsEventKind::Delete
                    } else {
                        FsEventKind::Modify
                    }
                }
            },
            EventKind::Modify(_) => FsEventKind::Modify,
            EventKind::Access(_) => continue,
            _ => continue,
        };
        out.push(FsEvent {
            path: rel,
            kind,
            is_dir,
            outside,
            timestamp: now_str(),
        });
    }
    out
}

/// Start watching:
///   - workspace root (non-recursive)           -> direct children of the workspace
///   - every non-ignored top-level subdirectory (recursive)
///   - parent of the workspace (non-recursive)  -> "outside workspace" detection
pub fn start_watcher(
    workspace: &Path,
    cfg: &AppConfig,
    tx: Sender<FsEvent>,
) -> Result<SharedWatcher> {
    let tx2 = tx.clone();
    let ws = workspace.to_path_buf();
    let cfg2 = cfg.clone();
    let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
        let Ok(event) = res else { return };
        let events = normalize(&event, &ws, &cfg2, cfg2.detect_outside);
        for e in events {
            let _ = tx2.send(e);
        }
    })?;

    // Root: non-recursive so we control what is watched below.
    watcher.watch(workspace, RecursiveMode::NonRecursive)?;

    // Recursive watches on each non-ignored top-level directory.
    if let Ok(entries) = std::fs::read_dir(workspace) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && !is_ignored_path(&path, workspace, cfg) {
                let _ = watcher.watch(&path, RecursiveMode::Recursive);
            }
        }
    }

    // Parent watch for outside-workspace detection (non-recursive).
    if cfg.detect_outside {
        if let Some(parent) = workspace.parent() {
            if !parent.starts_with(workspace) {
                let _ = watcher.watch(parent, RecursiveMode::NonRecursive);
            }
        }
    }

    Ok(Arc::new(Mutex::new(Some(watcher))))
}

/// Add a recursive watch on a newly created directory.
pub fn add_dir_watch(shared: &SharedWatcher, dir: &Path) {
    if let Ok(mut guard) = shared.lock() {
        if let Some(w) = guard.as_mut() {
            let _ = w.watch(dir, RecursiveMode::Recursive);
        }
    }
}


