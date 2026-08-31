use crate::models::{SnapshotFile, SnapshotManifest, UndoResult};
use crate::storage;
use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

fn hash_bytes(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let out = hasher.finalize();
    let mut s = String::with_capacity(64);
    for b in out {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

/// Walk the workspace and record every file + directory that exists at
/// session start. Contents are stored (deduplicated by hash) under
/// ~/.actionguard/snapshots/<session_id>/blobs/ so undo can restore them.
pub fn create_snapshot(
    workspace: &Path,
    session_id: &str,
    cfg: &crate::models::AppConfig,
) -> Result<SnapshotManifest> {
    let snap_root = storage::snapshot_dir(session_id);
    let blobs_dir = snap_root.join("blobs");
    fs::create_dir_all(&blobs_dir).context("create snapshot blobs dir")?;

    let mut manifest = SnapshotManifest::default();

    for entry in WalkDir::new(workspace)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            if e.depth() == 0 {
                return true;
            }
            let rel = e.path().strip_prefix(workspace).unwrap_or(e.path());
            !storage::is_ignored(&rel.to_string_lossy(), cfg)
        })
    {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        if entry.depth() == 0 {
            continue;
        }
        let Ok(rel) = entry.path().strip_prefix(workspace) else {
            continue;
        };
        let rel_str = rel.to_string_lossy().replace('\\', "/");
        if entry.file_type().is_dir() {
            manifest.dirs.push(rel_str);
            continue;
        }
        if !entry.file_type().is_file() {
            continue;
        }
        let bytes = match fs::read(entry.path()) {
            Ok(b) => b,
            Err(_) => continue,
        };
        let hash = hash_bytes(&bytes);
        let blob_path = blobs_dir.join(&hash);
        if !blob_path.exists() {
            let _ = fs::write(&blob_path, &bytes);
        }
        manifest.files.push(SnapshotFile {
            path: rel_str,
            hash,
            size: bytes.len() as u64,
        });
    }

    manifest.file_count = manifest.files.len() as u32;

    let manifest_path = snap_root.join("manifest.json");
    fs::write(&manifest_path, serde_json::to_string_pretty(&manifest)?)
        .context("write snapshot manifest")?;
    Ok(manifest)
}

/// Restore a workspace to the state captured by the snapshot.
///  - files that changed / were deleted are restored from blobs
///  - files that were created after the snapshot are deleted
///  - directories that were created after the snapshot are removed (when empty)
pub fn restore_snapshot(
    workspace: &Path,
    session_id: &str,
    cfg: &crate::models::AppConfig,
) -> Result<UndoResult> {
    let snap_root = storage::snapshot_dir(session_id);
    let manifest_path = snap_root.join("manifest.json");
    let manifest: SnapshotManifest = serde_json::from_str(
        &fs::read_to_string(&manifest_path).context("read snapshot manifest")?,
    )
    .context("parse snapshot manifest")?;

    let mut result = UndoResult::default();
    let mut snapshot_files: HashSet<String> = HashSet::new();
    let mut snapshot_dirs: HashSet<String> = HashSet::new();

    for f in &manifest.files {
        snapshot_files.insert(f.path.clone());
        let target = workspace.join(&f.path);
        let blob = snap_root.join("blobs").join(&f.hash);

        if target.exists() {
            // The path was replaced by a directory (or symlink): remove it so
            // the original file can be restored.
            if target.is_dir() {
                let _ = fs::remove_dir_all(&target);
            } else {
                match fs::read(&target) {
                    Ok(cur) if hash_bytes(&cur) == f.hash => {
                        result.skipped += 1;
                        continue;
                    }
                    _ => {}
                }
            }
        }
        if let Some(parent) = target.parent() {
            let _ = fs::create_dir_all(parent);
        }
        match fs::read(&blob) {
            Ok(content) => {
                let _ = fs::write(&target, content);
                result.restored_files += 1;
            }
            Err(_) => {
                // Missing blob: the file was empty in the snapshot.
                let _ = fs::write(&target, b"");
                result.restored_files += 1;
            }
        }
    }
    for d in &manifest.dirs {
        snapshot_dirs.insert(d.clone());
    }

    // Remove anything created after the snapshot.
    let mut remove_dirs: Vec<PathBuf> = Vec::new();
    for entry in WalkDir::new(workspace)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            if e.depth() == 0 {
                return true;
            }
            let rel = e.path().strip_prefix(workspace).unwrap_or(e.path());
            !storage::is_ignored(&rel.to_string_lossy(), cfg)
        })
    {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        if entry.depth() == 0 {
            continue;
        }
        let Ok(rel) = entry.path().strip_prefix(workspace) else {
            continue;
        };
        let rel_str = rel.to_string_lossy().replace('\\', "/");
        if entry.file_type().is_file() {
            if !snapshot_files.contains(&rel_str) {
                let _ = fs::remove_file(entry.path());
                result.deleted_files += 1;
            }
        } else if entry.file_type().is_dir() && !snapshot_dirs.contains(&rel_str) {
            remove_dirs.push(entry.path().to_path_buf());
        }
    }
    // Remove new dirs deepest-first so parents become empty before removal.
    remove_dirs.sort_by_key(|p| std::cmp::Reverse(p.components().count()));
    for d in remove_dirs {
        if fs::remove_dir(&d).is_ok() {
            result.removed_dirs += 1;
        } // not empty or in use — leave it
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::AppConfig;

    fn tmp_workspace(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("actionguard-test-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn snapshot_and_undo_roundtrip() {
        let ws = tmp_workspace("roundtrip");
        let session_id = "90001";
        let cfg = AppConfig::default();

        // Seed the workspace.
        fs::write(ws.join("a.txt"), "hello world").unwrap();
        fs::create_dir_all(ws.join("src/lib")).unwrap();
        fs::write(ws.join("src/lib/util.ts"), "export const u = 1;").unwrap();
        fs::write(ws.join(".env"), "TOKEN=secret").unwrap();

        // Snapshot.
        let manifest = create_snapshot(&ws, session_id, &cfg).unwrap();
        assert_eq!(manifest.file_count, 3);
        assert!(storage::snapshot_dir(session_id).join("manifest.json").exists());

        // Simulate agent work: modify, delete, create, rename.
        fs::write(ws.join("a.txt"), "changed content").unwrap();
        fs::remove_file(ws.join("src/lib/util.ts")).unwrap();
        fs::write(ws.join("new-file.md"), "# new").unwrap();
        fs::create_dir_all(ws.join("src/lib/util.ts")).unwrap();
        fs::write(ws.join("src/lib/util.ts/index.ts"), "// moved").unwrap();
        fs::remove_file(ws.join(".env")).unwrap();

        // Undo.
        let res = restore_snapshot(&ws, session_id, &cfg).unwrap();
        assert_eq!(res.restored_files, 3); // a.txt + util.ts + .env
        assert_eq!(res.deleted_files, 1); // new-file.md (index.ts removed with its dir)
        assert_eq!(res.removed_dirs, 0); // src/lib/util.ts dir already removed while restoring the file

        // Verify state restored.
        assert_eq!(fs::read_to_string(ws.join("a.txt")).unwrap(), "hello world");
        assert_eq!(
            fs::read_to_string(ws.join("src/lib/util.ts")).unwrap(),
            "export const u = 1;"
        );
        assert_eq!(fs::read_to_string(ws.join(".env")).unwrap(), "TOKEN=secret");
        assert!(!ws.join("new-file.md").exists());
        assert!(!ws.join("src/lib/util.ts/index.ts").exists());

        let _ = fs::remove_dir_all(&ws);
        let _ = fs::remove_dir_all(storage::snapshot_dir(session_id));
    }
}
