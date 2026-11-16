use crate::db::Database;
use crate::db::models::{Entry, EntryFilter, Project};
use crate::settings::{self, Settings};
use crate::timer_persist::{self, PersistedTimer};
use anyhow::Result;
use chrono::Timelike;
use eframe::egui;
use std::collections::HashSet;
use std::time::{Duration, Instant};

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Tab {
    Dashboard,
    Entries,
    Projects,
    Reports,
    Export,
    Settings,
}

#[derive(Clone)]
pub enum ConfirmAction {
    DeleteEntry(i64),
    DeleteEntries(Vec<i64>),
    DeleteProject(i64),
    RestoreBackup,
}

pub struct ConfirmState {
    pub title: String,
    pub message: String,
    pub action: ConfirmAction,
    pub backup_path: Option<std::path::PathBuf>,
}

#[derive(Clone)]
pub struct TimerState {
    pub running_since: Instant,
    pub date: chrono::NaiveDate,
    pub project_id: Option<i64>,
    pub start_time: String,
    pub description: String,
    pub billable: bool,
    pub tags: String,
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
    // bulk/inline helpers
    pub selected_entry_ids: HashSet<i64>,
    pub bulk_tags: String,
    pub bulk_billable: bool,
    pub focus_new_entry: bool,
    pub inline_edit: Option<InlineEditState>,
    // timer
    pub timer: Option<TimerState>,
    // settings & confirmations
    pub settings: Settings,
    pub confirm: Option<ConfirmState>,
}

#[derive(Clone)]
pub struct InlineEditState {
    pub id: i64,
    pub date: chrono::NaiveDate,
    pub project_idx: Option<usize>,
    pub start: String,
    pub end: String,
    pub duration: String,
    pub description: String,
    pub billable: bool,
    pub tags: String,
}

impl TimesheetApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Result<Self> {
        let db = Database::init()?;
        let projects_cache = db.list_projects(true).unwrap_or_default();
        let today = chrono::Local::now().date_naive();
        let settings = settings::load();
        let mut app = Self {
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
            selected_entry_ids: HashSet::new(),
            bulk_tags: String::new(),
            bulk_billable: true,
            focus_new_entry: false,
            inline_edit: None,
            timer: None,
            settings,
            confirm: None,
        };
        app.load_persisted_timer();
        app.apply_theme(&_cc.egui_ctx);
        Ok(app)
    }

    pub fn apply_theme(&self, ctx: &egui::Context) {
        if self.settings.dark_theme {
            ctx.set_visuals(egui::Visuals::dark());
        } else {
            ctx.set_visuals(egui::Visuals::light());
        }
    }

    pub fn save_settings(&mut self) {
        if let Err(e) = settings::save(&self.settings) {
            self.status = format!("Failed to save settings: {e}");
        } else {
            self.status = "Settings saved".to_string();
        }
    }

    pub fn show_confirm_dialog(&mut self, ctx: &egui::Context) {
        if self.confirm.is_none() {
            return;
        }
        let mut confirmed = false;
        let mut cancelled = false;
        {
            let state = self.confirm.as_ref().unwrap();
            let title = state.title.clone();
            let message = state.message.clone();
            egui::Window::new(title)
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .show(ctx, |ui| {
                    ui.label(message);
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            cancelled = true;
                        }
                        if ui.button("Confirm").clicked() {
                            confirmed = true;
                        }
                    });
                });
        }
        if cancelled {
            self.confirm = None;
        }
        if confirmed {
            let taken = self.confirm.take();
            let restore_path = taken.as_ref().and_then(|c| c.backup_path.clone());
            let action = taken.map(|c| c.action);
            if let Some(action) = action {
                match action {
                    ConfirmAction::DeleteEntry(id) => match self.db.delete_entry(id) {
                        Ok(_) => {
                            self.selected_entry_ids.remove(&id);
                            if self.inline_edit.as_ref().map(|x| x.id) == Some(id) {
                                self.inline_edit = None;
                            }
                            self.status = "Entry deleted".to_string();
                        }
                        Err(e) => self.status = format!("Delete failed: {e}"),
                    },
                    ConfirmAction::DeleteEntries(ids) => {
                        let mut deleted = 0;
                        for id in ids {
                            if self.db.delete_entry(id).is_ok() {
                                deleted += 1;
                            }
                        }
                        self.selected_entry_ids.clear();
                        self.status = format!(
                            "Deleted {} selected entr{}",
                            deleted,
                            if deleted == 1 { "y" } else { "ies" }
                        );
                    }
                    ConfirmAction::DeleteProject(id) => match self.db.delete_project(id) {
                        Ok(_) => {
                            self.status = "Project deleted".to_string();
                            self.refresh_projects();
                        }
                        Err(e) => self.status = format!("Delete failed: {e}"),
                    },
                    ConfirmAction::RestoreBackup => {
                        if let Some(path) = restore_path {
                            match crate::backup::stage_restore(&path) {
                                Ok(target) => {
                                    self.status = format!(
                                        "Backup restored to {} — please restart the app",
                                        target.display()
                                    );
                                }
                                Err(e) => self.status = format!("Restore failed: {e}"),
                            }
                        }
                    }
                }
            }
        }
    }

    fn load_persisted_timer(&mut self) {
        if let Some(p) = timer_persist::load() {
            let now = timer_persist::now_unix_ms();
            let elapsed_ms = (now - p.running_since_unix_ms).max(0) as u64;
            self.timer = Some(TimerState {
                running_since: Instant::now() - Duration::from_millis(elapsed_ms),
                date: p.date,
                project_id: p.project_id,
                start_time: p.start_time,
                description: p.description,
                billable: p.billable,
                tags: p.tags,
            });
            self.status = "Resumed running timer from previous session".to_string();
        }
    }

    pub fn persist_timer(&self) {
        if let Some(t) = &self.timer {
            let elapsed_ms = t.running_since.elapsed().as_millis() as i64;
            let running_since_unix_ms = timer_persist::now_unix_ms() - elapsed_ms;
            let p = PersistedTimer {
                running_since_unix_ms,
                date: t.date,
                project_id: t.project_id,
                start_time: t.start_time.clone(),
                description: t.description.clone(),
                billable: t.billable,
                tags: t.tags.clone(),
            };
            let _ = timer_persist::save(&p);
        }
    }

    pub fn clear_persisted_timer(&self) {
        let _ = timer_persist::clear();
    }

    pub fn export_dialog(
        &self,
        filter_name: &str,
        ext: &str,
        default_name: &str,
    ) -> rfd::FileDialog {
        let mut dialog = rfd::FileDialog::new()
            .add_filter(filter_name, &[ext])
            .set_file_name(default_name);
        if !self.settings.default_export_dir.trim().is_empty() {
            dialog = dialog.set_directory(&self.settings.default_export_dir);
        }
        dialog
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

    pub fn effective_entry_duration_minutes(&mut self) -> Option<i64> {
        if let Some(mins) =
            crate::domain::duration_minutes_from_times(&self.entry_start, &self.entry_end)
        {
            self.entry_duration = crate::domain::format_minutes(mins);
            return Some(mins);
        }
        crate::domain::parse_duration_to_minutes(&self.entry_duration)
    }

    pub fn project_idx_for_id(&self, id: Option<i64>) -> Option<usize> {
        id.and_then(|pid| self.projects_cache.iter().position(|p| p.id == pid))
    }

    pub fn find_overlaps(&self, candidate: &Entry, exclude_id: Option<i64>) -> Result<Vec<Entry>> {
        let filter = EntryFilter {
            from: Some(candidate.date),
            to: Some(candidate.date),
            project_id: None,
            billable: None,
        };
        let entries = self.db.list_entries(&filter)?;
        let c_start = candidate
            .start_time
            .as_deref()
            .and_then(crate::domain::parse_time_hhmm);
        let c_end = candidate
            .end_time
            .as_deref()
            .and_then(crate::domain::parse_time_hhmm);
        let (Some(cs), Some(ce)) = (c_start, c_end) else {
            return Ok(vec![]);
        };
        let c0 = cs.hour() as i64 * 60 + cs.minute() as i64;
        let mut c1 = ce.hour() as i64 * 60 + ce.minute() as i64;
        if c1 <= c0 {
            c1 += 24 * 60;
        }
        let mut overlaps = Vec::new();
        for e in entries {
            if Some(e.id) == exclude_id {
                continue;
            }
            let es = e
                .start_time
                .as_deref()
                .and_then(crate::domain::parse_time_hhmm);
            let ee = e
                .end_time
                .as_deref()
                .and_then(crate::domain::parse_time_hhmm);
            let (Some(es), Some(ee)) = (es, ee) else {
                continue;
            };
            let e0 = es.hour() as i64 * 60 + es.minute() as i64;
            let mut e1 = ee.hour() as i64 * 60 + ee.minute() as i64;
            if e1 <= e0 {
                e1 += 24 * 60;
            }
            if c0 < e1 && e0 < c1 {
                overlaps.push(e);
            }
        }
        Ok(overlaps)
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
                ui.selectable_value(&mut self.tab, Tab::Reports, "Reports");
                ui.selectable_value(&mut self.tab, Tab::Export, "Export");
                ui.selectable_value(&mut self.tab, Tab::Settings, "Settings");
                ui.separator();
                ui.checkbox(&mut self.settings.dark_theme, "Dark");
                if ui.button("Save").clicked() {
                    self.save_settings();
                }
                ui.separator();
                if let Ok(path) = Database::default_path() {
                    ui.label(format!("DB: {}", path.display()));
                }
            });
        });

        egui::TopBottomPanel::bottom("bottom").show(ctx, |ui| {
            ui.label(&self.status);
        });

        self.apply_theme(ctx);

        egui::CentralPanel::default().show(ctx, |ui| match self.tab {
            Tab::Dashboard => crate::ui::dashboard::show(self, ui),
            Tab::Entries => crate::ui::entries::show(self, ui),
            Tab::Projects => crate::ui::projects::show(self, ui),
            Tab::Reports => crate::ui::reports::show(self, ui),
            Tab::Export => crate::ui::export_view::show(self, ui),
            Tab::Settings => crate::ui::settings_view::show(self, ui),
        });

        self.show_confirm_dialog(ctx);
    }
}
