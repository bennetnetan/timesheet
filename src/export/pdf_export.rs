use crate::db::models::{Entry, Project};
use crate::export::{ExportOptions, project_name};
use anyhow::Result;
use printpdf::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufWriter;
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

    let (doc, page1, layer1) = PdfDocument::new(
        if options.title.is_empty() {
            "Timesheet Report"
        } else {
            &options.title
        },
        Mm(210.0),
        Mm(297.0),
        "Layer 1",
    );
    let font = doc.add_builtin_font(BuiltinFont::Helvetica)?;
    let font_bold = doc.add_builtin_font(BuiltinFont::HelveticaBold)?;
    let mut page_index = 1usize;
    let mut current_layer = doc.get_page(page1).get_layer(layer1);

    let left = 12.0_f32;
    let right = 198.0_f32;
    let bottom = 15.0_f32;

    let total_minutes: i64 = rows.iter().map(|e| e.duration_minutes).sum();
    let billable_minutes: i64 = rows
        .iter()
        .filter(|e| e.billable)
        .map(|e| e.duration_minutes)
        .sum();

    let draw_header = |layer: &PdfLayerReference, page_no: usize| {
        let mut yy = 282.0_f32;
        layer.use_text(
            if options.title.is_empty() {
                "Timesheet Report"
            } else {
                &options.title
            },
            16.0,
            Mm(left),
            Mm(yy),
            &font_bold,
        );
        yy -= 7.0;
        let prepared = options.prepared_by();
        if !prepared.is_empty() {
            layer.use_text(prepared, 9.5, Mm(left), Mm(yy), &font);
            yy -= 5.0;
        }
        layer.use_text(
            format!(
                "Generated: {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M")
            ),
            9.0,
            Mm(left),
            Mm(yy),
            &font,
        );
        yy -= 5.0;
        if !options.period_text.is_empty() {
            layer.use_text(
                format!("Period: {}", options.period_text),
                9.0,
                Mm(left),
                Mm(yy),
                &font,
            );
            yy -= 5.0;
        }
        if !options.filter_text.is_empty() {
            let ft: String = options.filter_text.chars().take(120).collect();
            layer.use_text(format!("Filters: {}", ft), 9.0, Mm(left), Mm(yy), &font);
            yy -= 5.0;
        }
        if options.include_summary {
            layer.use_text(
                format!(
                    "Entries: {} | Total: {} ({:.2} h) | Billable: {} ({:.2} h) | Non-billable: {} ({:.2} h)",
                    rows.len(),
                    crate::domain::format_minutes(total_minutes), total_minutes as f64 / 60.0,
                    crate::domain::format_minutes(billable_minutes), billable_minutes as f64 / 60.0,
                    crate::domain::format_minutes(total_minutes - billable_minutes), (total_minutes - billable_minutes) as f64 / 60.0,
                ),
                9.0, Mm(left), Mm(yy), &font,
            );
            yy -= 7.0;
        } else {
            yy -= 2.0;
        }
        // table header
        layer.use_text("Date", 8.5, Mm(left), Mm(yy), &font_bold);
        layer.use_text("Project", 8.5, Mm(left + 22.0), Mm(yy), &font_bold);
        layer.use_text("Start", 8.5, Mm(left + 62.0), Mm(yy), &font_bold);
        layer.use_text("End", 8.5, Mm(left + 75.0), Mm(yy), &font_bold);
        layer.use_text("Dur", 8.5, Mm(left + 88.0), Mm(yy), &font_bold);
        layer.use_text("Bill", 8.5, Mm(left + 100.0), Mm(yy), &font_bold);
        layer.use_text("Description", 8.5, Mm(left + 112.0), Mm(yy), &font_bold);
        let line = Line {
            points: vec![
                (Point::new(Mm(left), Mm(yy - 1.5)), false),
                (Point::new(Mm(right), Mm(yy - 1.5)), false),
            ],
            is_closed: false,
        };
        layer.add_line(line);
        layer.use_text(format!("Page {}", page_no), 8.0, Mm(180.0), Mm(10.0), &font);
        yy - 6.0
    };

    let mut y = draw_header(&current_layer, page_index);

    let mut current_project: Option<String> = None;
    let mut project_total: i64 = 0;
    let mut grand_total: i64 = 0;

    for e in &rows {
        if y < bottom + 12.0 {
            // footer/signature space on last page handled at end; page break now
            let (page, layer) = doc.add_page(Mm(210.0), Mm(297.0), "Layer 1");
            page_index += 1;
            current_layer = doc.get_page(page).get_layer(layer);
            y = draw_header(&current_layer, page_index);
        }

        let pname = project_name(e, projects).to_string();
        if options.group_by_project && current_project.as_deref() != Some(pname.as_str()) {
            if let Some(prev) = &current_project {
                current_layer.use_text(
                    format!(
                        "Subtotal {}: {} ({:.2} h)",
                        prev,
                        crate::domain::format_minutes(project_total),
                        project_total as f64 / 60.0
                    ),
                    8.5,
                    Mm(left),
                    Mm(y),
                    &font_bold,
                );
                y -= 5.0;
            }
            current_layer.use_text(
                format!("Project: {}", pname),
                9.5,
                Mm(left),
                Mm(y),
                &font_bold,
            );
            y -= 5.0;
            current_project = Some(pname.clone());
            project_total = 0;
            if y < bottom + 12.0 {
                let (page, layer) = doc.add_page(Mm(210.0), Mm(297.0), "Layer 1");
                page_index += 1;
                current_layer = doc.get_page(page).get_layer(layer);
                y = draw_header(&current_layer, page_index);
            }
        }

        project_total += e.duration_minutes;
        grand_total += e.duration_minutes;

        let proj_short: String = pname.chars().take(20).collect();
        let desc = e.description.replace('\n', " ");
        let desc_short: String = desc.chars().take(45).collect();
        current_layer.use_text(
            e.date.format("%Y-%m-%d").to_string(),
            8.0,
            Mm(left),
            Mm(y),
            &font,
        );
        current_layer.use_text(proj_short, 8.0, Mm(left + 22.0), Mm(y), &font);
        current_layer.use_text(
            e.start_time.clone().unwrap_or("-".into()),
            8.0,
            Mm(left + 62.0),
            Mm(y),
            &font,
        );
        current_layer.use_text(
            e.end_time.clone().unwrap_or("-".into()),
            8.0,
            Mm(left + 75.0),
            Mm(y),
            &font,
        );
        current_layer.use_text(
            crate::domain::format_minutes(e.duration_minutes),
            8.0,
            Mm(left + 88.0),
            Mm(y),
            &font,
        );
        current_layer.use_text(
            if e.billable { "Yes" } else { "No" },
            8.0,
            Mm(left + 100.0),
            Mm(y),
            &font,
        );
        current_layer.use_text(desc_short, 8.0, Mm(left + 112.0), Mm(y), &font);
        y -= 4.8;

        // wrap long description onto next line if needed
        if desc.chars().count() > 45 {
            let rest: String = desc.chars().skip(45).take(70).collect();
            if y < bottom + 12.0 {
                let (page, layer) = doc.add_page(Mm(210.0), Mm(297.0), "Layer 1");
                page_index += 1;
                current_layer = doc.get_page(page).get_layer(layer);
                y = draw_header(&current_layer, page_index);
            }
            current_layer.use_text(rest, 7.5, Mm(left + 112.0), Mm(y), &font);
            y -= 4.2;
        }
    }

    if options.group_by_project {
        if let Some(prev) = &current_project {
            if y < bottom + 20.0 {
                let (page, layer) = doc.add_page(Mm(210.0), Mm(297.0), "Layer 1");
                page_index += 1;
                current_layer = doc.get_page(page).get_layer(layer);
                y = draw_header(&current_layer, page_index);
            }
            current_layer.use_text(
                format!(
                    "Subtotal {}: {} ({:.2} h)",
                    prev,
                    crate::domain::format_minutes(project_total),
                    project_total as f64 / 60.0
                ),
                8.5,
                Mm(left),
                Mm(y),
                &font_bold,
            );
            y -= 6.0;
        }
    }

    if y < bottom + 25.0 {
        let (page, layer) = doc.add_page(Mm(210.0), Mm(297.0), "Layer 1");
        page_index += 1;
        current_layer = doc.get_page(page).get_layer(layer);
        y = draw_header(&current_layer, page_index);
    }
    current_layer.use_text(
        format!(
            "Grand Total: {} ({:.2} h)",
            crate::domain::format_minutes(grand_total),
            grand_total as f64 / 60.0
        ),
        10.0,
        Mm(left),
        Mm(y),
        &font_bold,
    );
    y -= 12.0;
    // signature line
    current_layer.use_text(
        "Signature: ________________________________",
        9.0,
        Mm(left),
        Mm(y),
        &font,
    );
    current_layer.use_text("Date: ____________", 9.0, Mm(120.0), Mm(y), &font);

    doc.save(&mut BufWriter::new(File::create(path)?))?;
    Ok(())
}
