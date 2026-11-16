use anyhow::Result;
use chrono::NaiveDate;
use rusqlite::{OptionalExtension, params};

use super::Database;
use super::models::{Entry, EntryFilter, NewEntry, NewProject, Project};

impl Database {
    pub fn list_projects(&self, include_inactive: bool) -> Result<Vec<Project>> {
        let sql = if include_inactive {
            "SELECT id, name, client, rate, color, active, budget_hours FROM projects ORDER BY active DESC, name ASC"
        } else {
            "SELECT id, name, client, rate, color, active, budget_hours FROM projects WHERE active = 1 ORDER BY name ASC"
        };
        let mut stmt = self.conn.prepare(sql)?;
        let rows = stmt.query_map([], |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                client: row.get(2)?,
                rate: row.get(3)?,
                color: row.get(4)?,
                active: row.get::<_, i64>(5)? != 0,
                budget_hours: row.get(6)?,
            })
        })?;
        Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
    }

    pub fn create_project(&self, p: &NewProject) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO projects (name, client, rate, color, active, budget_hours) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                p.name,
                p.client,
                p.rate,
                p.color,
                if p.active { 1 } else { 0 },
                p.budget_hours
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn update_project(&self, p: &Project) -> Result<()> {
        self.conn.execute(
            "UPDATE projects SET name=?1, client=?2, rate=?3, color=?4, active=?5, budget_hours=?6 WHERE id=?7",
            params![
                p.name,
                p.client,
                p.rate,
                p.color,
                if p.active { 1 } else { 0 },
                p.budget_hours,
                p.id
            ],
        )?;
        Ok(())
    }

    pub fn delete_project(&self, id: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM projects WHERE id=?1", params![id])?;
        Ok(())
    }

    pub fn list_entries(&self, filter: &EntryFilter) -> Result<Vec<Entry>> {
        let mut sql = String::from(
            "SELECT id, project_id, date, start_time, end_time, duration_minutes, description, billable, tags \
             FROM entries WHERE 1=1",
        );
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        if let Some(from) = filter.from {
            sql.push_str(" AND date >= ?");
            params_vec.push(Box::new(from.format("%Y-%m-%d").to_string()));
        }
        if let Some(to) = filter.to {
            sql.push_str(" AND date <= ?");
            params_vec.push(Box::new(to.format("%Y-%m-%d").to_string()));
        }
        if let Some(pid) = filter.project_id {
            sql.push_str(" AND project_id = ?");
            params_vec.push(Box::new(pid));
        }
        if let Some(b) = filter.billable {
            sql.push_str(" AND billable = ?");
            params_vec.push(Box::new(if b { 1i64 } else { 0i64 }));
        }
        sql.push_str(" ORDER BY date DESC, id DESC");

        let mut stmt = self.conn.prepare(&sql)?;
        let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|b| &**b).collect();
        let rows = stmt.query_map(params_refs.as_slice(), |row| {
            let date_str: String = row.get(2)?;
            let date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    2,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?;
            Ok(Entry {
                id: row.get(0)?,
                project_id: row.get(1)?,
                date,
                start_time: row.get(3)?,
                end_time: row.get(4)?,
                duration_minutes: row.get(5)?,
                description: row.get(6)?,
                billable: row.get::<_, i64>(7)? != 0,
                tags: row.get(8)?,
            })
        })?;
        Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
    }

    pub fn create_entry(&self, e: &NewEntry) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO entries (project_id, date, start_time, end_time, duration_minutes, description, billable, tags)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                e.project_id,
                e.date.format("%Y-%m-%d").to_string(),
                e.start_time,
                e.end_time,
                e.duration_minutes,
                e.description,
                if e.billable { 1 } else { 0 },
                e.tags,
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn update_entry(&self, e: &Entry) -> Result<()> {
        self.conn.execute(
            "UPDATE entries SET project_id=?1, date=?2, start_time=?3, end_time=?4, duration_minutes=?5, description=?6, billable=?7, tags=?8, updated_at=datetime('now') WHERE id=?9",
            params![
                e.project_id,
                e.date.format("%Y-%m-%d").to_string(),
                e.start_time,
                e.end_time,
                e.duration_minutes,
                e.description,
                if e.billable { 1 } else { 0 },
                e.tags,
                e.id,
            ],
        )?;
        Ok(())
    }

    pub fn delete_entry(&self, id: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM entries WHERE id=?1", params![id])?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn get_project(&self, id: i64) -> Result<Option<Project>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, client, rate, color, active, budget_hours FROM projects WHERE id=?1",
        )?;
        let p = stmt
            .query_row(params![id], |row| {
                Ok(Project {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    client: row.get(2)?,
                    rate: row.get(3)?,
                    color: row.get(4)?,
                    active: row.get::<_, i64>(5)? != 0,
                    budget_hours: row.get(6)?,
                })
            })
            .optional()?;
        Ok(p)
    }
}
