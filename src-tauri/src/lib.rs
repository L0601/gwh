mod db;

use chrono::{Datelike, Local, NaiveDate};
use db::{AppState, ContactPayload, StatsFilters, TaskFilters, TaskPayload};
use tauri::Manager;

#[tauri::command]
fn list_contacts(state: tauri::State<AppState>) -> Result<Vec<db::Contact>, String> {
    state.list_contacts().map_err(|err| err.to_string())
}

#[tauri::command]
fn create_contact(
    state: tauri::State<AppState>,
    payload: ContactPayload,
) -> Result<db::Contact, String> {
    state.create_contact(payload).map_err(|err| err.to_string())
}

#[tauri::command]
fn delete_contact(state: tauri::State<AppState>, id: i64) -> Result<(), String> {
    state.delete_contact(id).map_err(|err| err.to_string())
}

#[tauri::command]
fn delete_task(state: tauri::State<AppState>, id: i64) -> Result<(), String> {
    state.delete_task(id).map_err(|err| err.to_string())
}

#[tauri::command]
fn list_tasks(
    state: tauri::State<AppState>,
    filters: TaskFilters,
) -> Result<Vec<db::TaskItem>, String> {
    state.list_tasks(filters).map_err(|err| err.to_string())
}

#[tauri::command]
fn save_task(
    state: tauri::State<AppState>,
    payload: TaskPayload,
) -> Result<db::TaskItem, String> {
    state.save_task(payload).map_err(|err| err.to_string())
}

#[tauri::command]
fn stats_tasks(
    state: tauri::State<AppState>,
    filters: StatsFilters,
) -> Result<db::StatsPayload, String> {
    let normalized = filters.with_defaults(default_range_start(), default_range_end());
    state.stats_tasks(normalized).map_err(|err| err.to_string())
}

#[tauri::command]
fn import_tasks(state: tauri::State<AppState>, path: String) -> Result<usize, String> {
    state.import_tasks(&path).map_err(|err| err.to_string())
}

#[tauri::command]
fn export_tasks(state: tauri::State<AppState>, path: String) -> Result<usize, String> {
    state.export_tasks(&path).map_err(|err| err.to_string())
}

#[tauri::command]
fn export_template(state: tauri::State<AppState>, path: String) -> Result<usize, String> {
    state.export_template(&path).map_err(|err| err.to_string())
}

fn default_range_start() -> NaiveDate {
    let now = Local::now().date_naive();
    NaiveDate::from_ymd_opt(now.year(), now.month(), 1).unwrap_or(now)
}

fn default_range_end() -> NaiveDate {
    Local::now().date_naive()
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let data_dir = app
                .path()
                .app_data_dir()
                .map_err(|err| err.to_string())?;
            std::fs::create_dir_all(&data_dir).map_err(|err| err.to_string())?;
            let db_path = data_dir.join("editor-desk.sqlite");
            app.manage(AppState::new(&db_path).map_err(|err| err.to_string())?);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_contacts,
            create_contact,
            delete_contact,
            delete_task,
            list_tasks,
            save_task,
            stats_tasks,
            import_tasks,
            export_tasks,
            export_template
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
