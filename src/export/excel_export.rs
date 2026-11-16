use crate::db::models::{Entry, Project};
use anyhow::Result;
use rust_xlsxwriter::{Format, Workbook};
use std::collections::HashMap;
use std::path::Path;

pub fn export_entries(
    path: &Path,
    entries: &[Entry],
    projects: &HashMap<i64, Project>,
) -> Result<()> {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    worksheet.set_name("Timesheet")?;

    let header_fmt = Format::new().set_bold();
    let headers = [
        "Date",
        "Project",
        "Client",
        "Start",
        "End",
        "Minutes",
        "Hours",
        "Billable",
        "Tags",
        "Description",
    ];
    for (col, h) in headers.iter().enumerate() {
        worksheet.write_string_with_format(0, col as u16, *h, &header_fmt)?;
    }

    let mut total_minutes: i64 = 0;
    for (i, e) in entries.iter().enumerate() {
        let row = (i + 1) as u32;
        let (proj_name, client) = e
            .project_id
            .and_then(|id| projects.get(&id))
            .map(|p| (p.name.clone(), p.client.clone().unwrap_or_default()))
            .unwrap_or_else(|| ("".to_string(), "".to_string()));
        worksheet.write_string(row, 0, e.date.format("%Y-%m-%d").to_string())?;
        worksheet.write_string(row, 1, proj_name)?;
        worksheet.write_string(row, 2, client)?;
        worksheet.write_string(row, 3, e.start_time.clone().unwrap_or_default())?;
        worksheet.write_string(row, 4, e.end_time.clone().unwrap_or_default())?;
        worksheet.write_number(row, 5, e.duration_minutes as f64)?;
        worksheet.write_number(row, 6, e.duration_minutes as f64 / 60.0)?;
        worksheet.write_string(row, 7, if e.billable { "Yes" } else { "No" })?;
        worksheet.write_string(row, 8, &e.tags)?;
        worksheet.write_string(row, 9, &e.description)?;
        total_minutes += e.duration_minutes;
    }

    let total_row = (entries.len() + 1) as u32;
    worksheet.write_string_with_format(total_row, 4, "Total", &header_fmt)?;
    worksheet.write_number_with_format(total_row, 5, total_minutes as f64, &header_fmt)?;
    worksheet.write_number_with_format(total_row, 6, total_minutes as f64 / 60.0, &header_fmt)?;

    worksheet.set_column_width(0, 12)?;
    worksheet.set_column_width(1, 20)?;
    worksheet.set_column_width(2, 18)?;
    worksheet.set_column_width(3, 10)?;
    worksheet.set_column_width(4, 10)?;
    worksheet.set_column_width(5, 10)?;
    worksheet.set_column_width(6, 10)?;
    worksheet.set_column_width(7, 10)?;
    worksheet.set_column_width(8, 20)?;
    worksheet.set_column_width(9, 40)?;

    workbook.save(path)?;
    Ok(())
}
