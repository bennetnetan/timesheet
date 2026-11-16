use crate::db::models::{Entry, Project};
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
) -> Result<()> {
    let (doc, page1, layer1) = PdfDocument::new("Timesheet", Mm(210.0), Mm(297.0), "Layer 1");
    let font = doc.add_builtin_font(BuiltinFont::Helvetica)?;
    let font_bold = doc.add_builtin_font(BuiltinFont::HelveticaBold)?;
    let mut current_layer = doc.get_page(page1).get_layer(layer1);
    let mut y = 280.0_f32;
    let left = 15.0_f32;

    current_layer.use_text("Timesheet Report", 18.0, Mm(left), Mm(y), &font_bold);
    y -= 10.0;
    let total_minutes: i64 = entries.iter().map(|e| e.duration_minutes).sum();
    let summary = format!(
        "Entries: {}    Total: {} ({:.2} h)",
        entries.len(),
        crate::domain::format_minutes(total_minutes),
        total_minutes as f64 / 60.0
    );
    current_layer.use_text(summary, 11.0, Mm(left), Mm(y), &font);
    y -= 12.0;

    // header
    let header = "Date        Project              Dur    Bill  Description";
    current_layer.use_text(header, 9.0, Mm(left), Mm(y), &font_bold);
    y -= 6.0;

    for e in entries {
        if y < 20.0 {
            let (page, layer) = doc.add_page(Mm(210.0), Mm(297.0), "Layer 1");
            current_layer = doc.get_page(page).get_layer(layer);
            y = 280.0;
        }
        let proj = e
            .project_id
            .and_then(|id| projects.get(&id))
            .map(|p| p.name.as_str())
            .unwrap_or("-");
        let proj_short: String = proj.chars().take(18).collect();
        let desc_short: String = e.description.replace('\n', " ").chars().take(45).collect();
        let line = format!(
            "{:<10}  {:<18}  {:>6}  {:<4}  {}",
            e.date.format("%Y-%m-%d"),
            proj_short,
            crate::domain::format_minutes(e.duration_minutes),
            if e.billable { "Yes" } else { "No" },
            desc_short
        );
        current_layer.use_text(line, 8.5, Mm(left), Mm(y), &font);
        y -= 5.0;
    }

    doc.save(&mut BufWriter::new(File::create(path)?))?;
    Ok(())
}
