pub mod csv_export;
pub mod excel_export;
pub mod markdown_export;
pub mod pdf_export;

#[derive(Debug, Clone, Default)]
pub struct ExportOptions {
    pub title: String,
    pub company_name: String,
    pub user_name: String,
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
            company_name: String::new(),
            user_name: String::new(),
            period_text: String::new(),
            filter_text: String::new(),
            group_by_project: false,
            include_summary: true,
        }
    }

    /// A small header line for exports, combining company/user when set.
    pub fn prepared_by(&self) -> String {
        match (self.company_name.trim(), self.user_name.trim()) {
            ("", "") => String::new(),
            (c, "") => c.to_string(),
            ("", u) => format!("Prepared by {}", u),
            (c, u) => format!("{} — {}", c, u),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prepared_by_combines_company_and_user() {
        let mut o = ExportOptions::basic();
        assert_eq!(o.prepared_by(), "");
        o.company_name = "Acme".into();
        assert_eq!(o.prepared_by(), "Acme");
        o.user_name = "Ada".into();
        assert_eq!(o.prepared_by(), "Acme — Ada");
        o.company_name.clear();
        assert_eq!(o.prepared_by(), "Prepared by Ada");
    }
}
