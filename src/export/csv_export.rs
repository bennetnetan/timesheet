use crate::db::models::{Entry, Project};
use crate::export::{ExportOptions, client_name, project_name};
use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;

pub fn export_entries(
    path: &Path,
    entries: &[Entry],
    projects: &HashMap<i64, Project>,
    options: &ExportOptions,
) -> Result<()> {
    let mut rows: Vec<&Entry> = entries.iter().collect();
    if options.group_by_project {
        rows.sort_by(|a, b| {
            project_name(a, projects)
                .cmp(project_name(b, projects))
                .then(a.date.cmp(&b.date))
                .then(a.start_time.cmp(&b.start_time))
        });
    }

    let mut wtr = csv::Writer::from_path(path)?;
    // metadata comments-like rows in first column when requested (simple approach)
    if options.include_summary || !options.period_text.is_empty() || !options.filter_text.is_empty()
    {
        if !options.title.is_empty() {
            wtr.write_record(["Report", &options.title, "", "", "", "", "", "", "", ""])?;
        }
        let prepared = options.prepared_by();
        if !prepared.is_empty() {
            wtr.write_record(["Prepared by", &prepared, "", "", "", "", "", "", "", ""])?;
        }
        if !options.period_text.is_empty() {
            wtr.write_record([
                "Period",
                &options.period_text,
                "",
                "",
                "",
                "",
                "",
                "",
                "",
                "",
            ])?;
        }
        if !options.filter_text.is_empty() {
            wtr.write_record([
                "Filters",
                &options.filter_text,
                "",
                "",
                "",
                "",
                "",
                "",
                "",
                "",
            ])?;
        }
        if options.include_summary {
            let total: i64 = rows.iter().map(|e| e.duration_minutes).sum();
            let billable: i64 = rows
                .iter()
                .filter(|e| e.billable)
                .map(|e| e.duration_minutes)
                .sum();
            wtr.write_record([
                "Summary",
                &format!("Entries: {}", rows.len()),
                &format!("Total min: {}", total),
                &format!("Total h: {:.2}", total as f64 / 60.0),
                &format!("Billable h: {:.2}", billable as f64 / 60.0),
                "",
                "",
                "",
                "",
                "",
            ])?;
        }
        wtr.write_record(["", "", "", "", "", "", "", "", "", ""])?;
    }

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
    for e in rows {
        let hours = format!("{:.2}", e.duration_minutes as f64 / 60.0);
        wtr.write_record([
            e.date.format("%Y-%m-%d").to_string(),
            project_name(e, projects).to_string(),
            client_name(e, projects).to_string(),
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
