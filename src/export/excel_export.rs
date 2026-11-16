use crate::db::models::{Entry, Project};
use crate::export::{ExportOptions, client_name, project_name};
use anyhow::Result;
use rust_xlsxwriter::{Format, FormatAlign, FormatBorder, Workbook};
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
    } else {
        rows.sort_by(|a, b| a.date.cmp(&b.date).then(a.start_time.cmp(&b.start_time)));
    }

    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    worksheet.set_name("Timesheet")?;

    let title_fmt = Format::new().set_bold().set_font_size(14);
    let meta_label = Format::new().set_bold();
    let header_fmt = Format::new()
        .set_bold()
        .set_background_color(rust_xlsxwriter::Color::RGB(0xE8EEF7))
        .set_border(FormatBorder::Thin)
        .set_align(FormatAlign::Center);
    let text_fmt = Format::new().set_border(FormatBorder::Thin);
    let num_fmt = Format::new()
        .set_border(FormatBorder::Thin)
        .set_num_format("0.00");
    let int_fmt = Format::new()
        .set_border(FormatBorder::Thin)
        .set_num_format("0");
    let subtotal_fmt = Format::new()
        .set_bold()
        .set_background_color(rust_xlsxwriter::Color::RGB(0xF5F5F5))
        .set_border(FormatBorder::Thin);
    let total_fmt = Format::new()
        .set_bold()
        .set_background_color(rust_xlsxwriter::Color::RGB(0xDDE8D8))
        .set_border(FormatBorder::Thin);

    let mut row: u32 = 0;
    worksheet.write_string_with_format(
        row,
        0,
        if options.title.is_empty() {
            "Timesheet Report"
        } else {
            &options.title
        },
        &title_fmt,
    )?;
    row += 1;
    let prepared = options.prepared_by();
    if !prepared.is_empty() {
        worksheet.write_string_with_format(row, 0, "Prepared by:", &meta_label)?;
        worksheet.write_string(row, 1, prepared)?;
        row += 1;
    }
    if !options.period_text.is_empty() {
        worksheet.write_string_with_format(row, 0, "Period:", &meta_label)?;
        worksheet.write_string(row, 1, &options.period_text)?;
        row += 1;
    }
    if !options.filter_text.is_empty() {
        worksheet.write_string_with_format(row, 0, "Filters:", &meta_label)?;
        worksheet.write_string(row, 1, &options.filter_text)?;
        row += 1;
    }
    if options.include_summary {
        let total: i64 = rows.iter().map(|e| e.duration_minutes).sum();
        let billable: i64 = rows
            .iter()
            .filter(|e| e.billable)
            .map(|e| e.duration_minutes)
            .sum();
        worksheet.write_string_with_format(row, 0, "Summary:", &meta_label)?;
        worksheet.write_string(
            row,
            1,
            format!(
                "Entries: {}, Total: {:.2} h, Billable: {:.2} h, Non-billable: {:.2} h",
                rows.len(),
                total as f64 / 60.0,
                billable as f64 / 60.0,
                (total - billable) as f64 / 60.0
            ),
        )?;
        row += 1;
    }
    row += 1; // blank line

    let header_row = row;
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
        worksheet.write_string_with_format(header_row, col as u16, *h, &header_fmt)?;
    }
    worksheet.set_freeze_panes(header_row + 1, 0)?;
    worksheet.autofilter(header_row, 0, header_row, (headers.len() - 1) as u16)?;
    row = header_row + 1;

    let mut current_project: Option<String> = None;
    let mut project_minutes: i64 = 0;
    let mut grand_minutes: i64 = 0;
    for e in &rows {
        let pname = project_name(e, projects).to_string();
        if options.group_by_project {
            if current_project.as_deref() != Some(pname.as_str()) {
                if let Some(prev) = &current_project {
                    worksheet.write_string_with_format(
                        row,
                        3,
                        format!("Subtotal: {}", prev),
                        &subtotal_fmt,
                    )?;
                    worksheet.write_number_with_format(
                        row,
                        5,
                        project_minutes as f64,
                        &subtotal_fmt,
                    )?;
                    worksheet.write_number_with_format(
                        row,
                        6,
                        project_minutes as f64 / 60.0,
                        &subtotal_fmt,
                    )?;
                    for c in [0u16, 1, 2, 4, 7, 8, 9] {
                        worksheet.write_blank(row, c, &subtotal_fmt)?;
                    }
                    row += 1;
                }
                // section header
                let section_fmt = Format::new()
                    .set_bold()
                    .set_background_color(rust_xlsxwriter::Color::RGB(0xEDF3FF))
                    .set_border(FormatBorder::Thin);
                worksheet.write_string_with_format(
                    row,
                    0,
                    format!("Project: {}", pname),
                    &section_fmt,
                )?;
                for c in 1..10u16 {
                    worksheet.write_blank(row, c, &section_fmt)?;
                }
                row += 1;
                current_project = Some(pname.clone());
                project_minutes = 0;
            }
            project_minutes += e.duration_minutes;
        }
        grand_minutes += e.duration_minutes;
        worksheet.write_string_with_format(
            row,
            0,
            e.date.format("%Y-%m-%d").to_string(),
            &text_fmt,
        )?;
        worksheet.write_string_with_format(row, 1, &pname, &text_fmt)?;
        worksheet.write_string_with_format(row, 2, client_name(e, projects), &text_fmt)?;
        worksheet.write_string_with_format(
            row,
            3,
            e.start_time.clone().unwrap_or_default(),
            &text_fmt,
        )?;
        worksheet.write_string_with_format(
            row,
            4,
            e.end_time.clone().unwrap_or_default(),
            &text_fmt,
        )?;
        worksheet.write_number_with_format(row, 5, e.duration_minutes as f64, &int_fmt)?;
        worksheet.write_number_with_format(row, 6, e.duration_minutes as f64 / 60.0, &num_fmt)?;
        worksheet.write_string_with_format(
            row,
            7,
            if e.billable { "Yes" } else { "No" },
            &text_fmt,
        )?;
        worksheet.write_string_with_format(row, 8, &e.tags, &text_fmt)?;
        worksheet.write_string_with_format(row, 9, &e.description, &text_fmt)?;
        row += 1;
    }
    if options.group_by_project {
        if let Some(prev) = &current_project {
            worksheet.write_string_with_format(
                row,
                3,
                format!("Subtotal: {}", prev),
                &subtotal_fmt,
            )?;
            worksheet.write_number_with_format(row, 5, project_minutes as f64, &subtotal_fmt)?;
            worksheet.write_number_with_format(
                row,
                6,
                project_minutes as f64 / 60.0,
                &subtotal_fmt,
            )?;
            for c in [0u16, 1, 2, 4, 7, 8, 9] {
                worksheet.write_blank(row, c, &subtotal_fmt)?;
            }
            row += 1;
        }
    }

    worksheet.write_string_with_format(row, 4, "Grand Total", &total_fmt)?;
    worksheet.write_number_with_format(row, 5, grand_minutes as f64, &total_fmt)?;
    worksheet.write_number_with_format(row, 6, grand_minutes as f64 / 60.0, &total_fmt)?;
    for c in [0u16, 1, 2, 3, 7, 8, 9] {
        worksheet.write_blank(row, c, &total_fmt)?;
    }

    worksheet.set_column_width(0, 12)?;
    worksheet.set_column_width(1, 22)?;
    worksheet.set_column_width(2, 18)?;
    worksheet.set_column_width(3, 10)?;
    worksheet.set_column_width(4, 10)?;
    worksheet.set_column_width(5, 10)?;
    worksheet.set_column_width(6, 10)?;
    worksheet.set_column_width(7, 10)?;
    worksheet.set_column_width(8, 18)?;
    worksheet.set_column_width(9, 45)?;

    workbook.save(path)?;
    Ok(())
}
