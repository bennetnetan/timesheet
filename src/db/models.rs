use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Project {
    pub id: i64,
    pub name: String,
    pub client: Option<String>,
    pub rate: Option<f64>,
    pub color: Option<String>,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewProject {
    pub name: String,
    pub client: Option<String>,
    pub rate: Option<f64>,
    pub color: Option<String>,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Entry {
    pub id: i64,
    pub project_id: Option<i64>,
    pub date: NaiveDate,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub duration_minutes: i64,
    pub description: String,
    pub billable: bool,
    pub tags: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewEntry {
    pub project_id: Option<i64>,
    pub date: NaiveDate,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub duration_minutes: i64,
    pub description: String,
    pub billable: bool,
    pub tags: String,
}

#[derive(Debug, Clone, Default)]
pub struct EntryFilter {
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
    pub project_id: Option<i64>,
    pub billable: Option<bool>,
}
