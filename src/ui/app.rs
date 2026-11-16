use crate::db::Database;
use crate::db::models::{EntryFilter, Project};
use anyhow::Result;
use eframe::egui;

#[derive(PartialEq, Eq)]
pub enum Tab {
    Dashboard,
    Entries,
    Projects,
    Export,
}

pub struct TimesheetApp {
    pub db: Database,
    pub tab: Tab,
    pub status: String,
    pub projects_cache: Vec<Project>,
    pub filter: EntryFilter,
    // entry form state
    pub entry_date: chrono::NaiveDate,
    pub entry_project_idx: Option<usize>,
    pub entry_start: String,
    pub entry_end: String,
    pub entry_duration: String,
    pub entry_description: String,
    pub entry_billable: bool,
    pub entry_tags: String,
    pub editing_entry_id: Option<i64>,
    // project form state
    pub proj_name: String,
    pub proj_client: String,
    pub proj_rate: String,
    pub proj_active: bool,
    pub editing_project_id: Option<i64>,
}

impl TimesheetApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Result<Self> {
        let db = Database::init()?;
        let projects_cache = db.list_projects(true).unwrap_or_default();
        let today = chrono::Local::now().date_naive();
        Ok(Self {
            db,
            tab: Tab::Dashboard,
            status: "Ready".to_string(),
            projects_cache,
            filter: EntryFilter::default(),
            entry_date: today,
            entry_project_idx: None,
            entry_start: String::new(),
            entry_end: String::new(),
            entry_duration: String::new(),
            entry_description: String::new(),
            entry_billable: true,
            entry_tags: String::new(),
            editing_entry_id: None,
            proj_name: String::new(),
            proj_client: String::new(),
            proj_rate: String::new(),
            proj_active: true,
            editing_project_id: None,
        })
    }

    pub fn refresh_projects(&mut self) {
        if let Ok(p) = self.db.list_projects(true) {
            self.projects_cache = p;
        }
    }

    pub fn selected_project_id(&self) -> Option<i64> {
        self.entry_project_idx
            .and_then(|i| self.projects_cache.get(i))
            .map(|p| p.id)
    }

    pub fn clear_entry_form(&mut self) {
        self.entry_date = chrono::Local::now().date_naive();
        self.entry_project_idx = None;
        self.entry_start.clear();
        self.entry_end.clear();
        self.entry_duration.clear();
        self.entry_description.clear();
        self.entry_billable = true;
        self.entry_tags.clear();
        self.editing_entry_id = None;
    }

    pub fn clear_project_form(&mut self) {
        self.proj_name.clear();
        self.proj_client.clear();
        self.proj_rate.clear();
        self.proj_active = true;
        self.editing_project_id = None;
    }
}

impl eframe::App for TimesheetApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.selectable_value(&mut self.tab, Tab::Dashboard, "Dashboard");
                ui.selectable_value(&mut self.tab, Tab::Entries, "Entries");
                ui.selectable_value(&mut self.tab, Tab::Projects, "Projects");
                ui.selectable_value(&mut self.tab, Tab::Export, "Export");
                ui.separator();
                if let Ok(path) = Database::default_path() {
                    ui.label(format!("DB: {}", path.display()));
                }
            });
        });

        egui::TopBottomPanel::bottom("bottom").show(ctx, |ui| {
            ui.label(&self.status);
        });

        egui::CentralPanel::default().show(ctx, |ui| match self.tab {
            Tab::Dashboard => crate::ui::dashboard::show(self, ui),
            Tab::Entries => crate::ui::entries::show(self, ui),
            Tab::Projects => crate::ui::projects::show(self, ui),
            Tab::Export => crate::ui::export_view::show(self, ui),
        });
    }
}
