use crate::db::Database;
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Copy the live SQLite database to `dest` after checkpointing the WAL.
pub fn backup_database(db: &Database, dest: &Path) -> Result<()> {
    db.checkpoint().context("failed to checkpoint database")?;
    let src = Database::default_path()?;
    copy_db_files(&src, dest)?;
    Ok(())
}

/// Restore is applied at startup; this copies a backup over the live DB path.
/// The app should be restarted after restoring.
pub fn stage_restore(backup_path: &Path) -> Result<PathBuf> {
    let target = Database::default_path()?;
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }
    // only copy the main db file; remove stale WAL/SHM so the backup is used as-is
    for suffix in ["-wal", "-shm"] {
        let p = PathBuf::from(format!("{}{}", target.display(), suffix));
        if p.exists() {
            let _ = fs::remove_file(p);
        }
    }
    fs::copy(backup_path, &target)
        .with_context(|| format!("failed to copy backup into {}", target.display()))?;
    Ok(target)
}

fn copy_db_files(src: &Path, dest: &Path) -> Result<()> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(src, dest)
        .with_context(|| format!("failed to copy {} to {}", src.display(), dest.display()))?;
    Ok(())
}
