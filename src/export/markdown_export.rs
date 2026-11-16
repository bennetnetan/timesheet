use crate::db::models::{Entry, Project};
use crate::export::{ExportOptions, client_name, project_name};
use anyhow::Result;
use std::collections::HashMap;
use std::fs;
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

    let mut out = String::new();
    out.push_str(&format!(
        "# {}\n\n",
        if options.title.is_empty() {
            "Timesheet Report"
        } else {
            &options.title
        }
    ));
    if !options.period_text.is_empty() {
        out.push_str(&format!("- **Period:** {}\n", options.period_text));
    }
    if !options.filter_text.is_empty() {
        out.push_str(&format!("- **Filters:** {}\n", options.filter_text));
    }
    if options.include_summary {
        let total: i64 = rows.iter().map(|e| e.duration_minutes).sum();
        let billable: i64 = rows
            .iter()
            .filter(|e| e.billable)
            .map(|e| e.duration_minutes)
            .sum();
        out.push_str(&format!("- **Entries:** {}\n", rows.len()));
        out.push_str(&format!(
            "- **Total:** {} ({:.2} h)\n",
            crate::domain::format_minutes(total),
            total as f64 / 60.0
        ));
        out.push_str(&format!(
            "- **Billable:** {} ({:.2} h)\n",
            crate::domain::format_minutes(billable),
            billable as f64 / 60.0
        ));
        out.push_str(&format!(
            "- **Non-billable:** {} ({:.2} h)\n",
            crate::domain::format_minutes(total - billable),
            (total - billable) as f64 / 60.0
        ));
    }
    out.push('\n');

    out.push_str(
        "| Date | Project | Client | Start | End | Duration | Billable | Tags | Description |\n",
    );
    out.push_str("|---|---|---|---:|---:|---:|:---:|---|---|\n");

    let mut current_project: Option<String> = None;
    let mut project_total: i64 = 0;
    let mut grand_total: i64 = 0;
    let flush_project = |out: &mut String, name: &str, total: i64| {
        out.push_str(&format!(
            "|  | **Subtotal: {}** |  |  |  | **{} ({:.2} h)** |  |  |  |\n\n",
            name,
            crate::domain::format_minutes(total),
            total as f64 / 60.0
        ));
    };

    for e in &rows {
        let pname = project_name(e, projects).to_string();
        if options.group_by_project {
            if current_project.as_deref() != Some(pname.as_str()) {
                if let Some(prev) = &current_project {
                    flush_project(&mut out, prev, project_total);
                }
                current_project = Some(pname.clone());
                project_total = 0;
                out.push_str(&format!("## {}\n\n", pname));
                out.push_str("| Date | Project | Client | Start | End | Duration | Billable | Tags | Description |\n");
                out.push_str("|---|---|---|---:|---:|---:|:---:|---|---|\n");
            }
            project_total += e.duration_minutes;
        }
        grand_total += e.duration_minutes;
        let desc = e.description.replace('|', "\\|").replace('\n', " ");
        let tags = e.tags.replace('|', "\\|");
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
            e.date.format("%Y-%m-%d"),
            pname.replace('|', "\\|"),
            client_name(e, projects).replace('|', "\\|"),
            e.start_time.clone().unwrap_or_default(),
            e.end_time.clone().unwrap_or_default(),
            crate::domain::format_minutes(e.duration_minutes),
            if e.billable { "Yes" } else { "No" },
            tags,
            desc,
        ));
    }
    if options.group_by_project {
        if let Some(prev) = &current_project {
            flush_project(&mut out, prev, project_total);
        }
    }
    if options.include_summary {
        out.push_str(&format!(
            "\n**Grand total:** {} ({:.2} h)\n",
            crate::domain::format_minutes(grand_total),
            grand_total as f64 / 60.0
        ));
    }

    fs::write(path, out)?;
    Ok(())
}
