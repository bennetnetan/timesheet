pub mod models;
pub mod queries;

use anyhow::{Context, Result};
use directories::ProjectDirs;
use rusqlite::Connection;
use std::fs;
use std::path::PathBuf;

pub struct Database {
    pub conn: Connection,
}

impl Database {
    pub fn init() -> Result<Self> {
        let db_path = default_db_path()?;
        if let Some(parent) = db_path.parent() {
            fs::create_dir_all(parent).context("failed to create app data directory")?;
        }
        let conn = Connection::open(&db_path)
            .with_context(|| format!("failed to open database at {}", db_path.display()))?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        let db = Self { conn };
        db.create_schema()?;
        Ok(db)
    }

    pub fn default_path() -> Result<PathBuf> {
        default_db_path()
    }

    /// Flush WAL contents into the main database file so copies are self-contained.
    pub fn checkpoint(&self) -> Result<()> {
        self.conn
            .execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")?;
        Ok(())
    }

    pub fn counts(&self) -> Result<(i64, i64)> {
        let entries: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM entries", [], |r| r.get(0))?;
        let projects: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM projects", [], |r| r.get(0))?;
        Ok((entries, projects))
    }

    fn create_schema(&self) -> Result<()> {
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS projects (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                client TEXT,
                rate REAL,
                color TEXT,
                active INTEGER NOT NULL DEFAULT 1
            );

            CREATE TABLE IF NOT EXISTS entries (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id INTEGER REFERENCES projects(id) ON DELETE SET NULL,
                date TEXT NOT NULL,
                start_time TEXT,
                end_time TEXT,
                duration_minutes INTEGER NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                billable INTEGER NOT NULL DEFAULT 1,
                tags TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE INDEX IF NOT EXISTS idx_entries_date ON entries(date);
            CREATE INDEX IF NOT EXISTS idx_entries_project ON entries(project_id);
            ",
        )?;
        Ok(())
    }
}

fn default_db_path() -> Result<PathBuf> {
    let proj = ProjectDirs::from("com", "timesheet", "Timesheet")
        .context("could not determine application data directory")?;
    Ok(proj.data_dir().join("timesheet.db"))
}
