pub mod csv_export;
pub mod excel_export;
pub mod markdown_export;
pub mod pdf_export;

#[derive(Debug, Clone, Default)]
pub struct ExportOptions {
    pub title: String,
    pub period_text: String,
    pub filter_text: String,
    pub group_by_project: bool,
    pub include_summary: bool,
}

impl ExportOptions {
    #[allow(dead_code)]
    pub fn basic() -> Self {
        Self {
            title: "Timesheet Report".to_string(),
            period_text: String::new(),
            filter_text: String::new(),
            group_by_project: false,
            include_summary: true,
        }
    }
}

pub fn project_name<'a>(
    e: &crate::db::models::Entry,
    projects: &'a std::collections::HashMap<i64, crate::db::models::Project>,
) -> &'a str {
    e.project_id
        .and_then(|id| projects.get(&id))
        .map(|p| p.name.as_str())
        .unwrap_or("(No project)")
}

pub fn client_name<'a>(
    e: &crate::db::models::Entry,
    projects: &'a std::collections::HashMap<i64, crate::db::models::Project>,
) -> &'a str {
    e.project_id
        .and_then(|id| projects.get(&id))
        .and_then(|p| p.client.as_deref())
        .unwrap_or("")
}
