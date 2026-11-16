use crate::db::models::{Entry, Project};
use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;

pub fn export_entries(
    path: &Path,
    entries: &[Entry],
    projects: &HashMap<i64, Project>,
) -> Result<()> {
    let mut wtr = csv::Writer::from_path(path)?;
    wtr.write_record([
        "Date",
        "Project",
        "Client",
        "Start",
        "End",
        "Duration (min)",
        "Hours",
        "Billable",
        "Tags",
        "Description",
    ])?;
    for e in entries {
        let (proj_name, client) = e
            .project_id
            .and_then(|id| projects.get(&id))
            .map(|p| (p.name.clone(), p.client.clone().unwrap_or_default()))
            .unwrap_or_else(|| ("".to_string(), "".to_string()));
        let hours = format!("{:.2}", e.duration_minutes as f64 / 60.0);
        wtr.write_record([
            e.date.format("%Y-%m-%d").to_string(),
            proj_name,
            client,
            e.start_time.clone().unwrap_or_default(),
            e.end_time.clone().unwrap_or_default(),
            e.duration_minutes.to_string(),
            hours,
            if e.billable {
                "Yes".into()
            } else {
                "No".into()
            },
            e.tags.clone(),
            e.description.clone(),
        ])?;
    }
    wtr.flush()?;
    Ok(())
}
