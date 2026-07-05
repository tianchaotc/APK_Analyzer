pub mod commands;
pub mod analyzers;
pub mod parser;
pub mod models;
pub mod export;
pub mod utils;
pub mod plugin;

use std::sync::Mutex;
use once_cell::sync::Lazy;

/// Global state: holds the current analysis result
pub struct AppState {
    pub current_analysis: Mutex<Option<models::ApkAnalysis>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            current_analysis: Mutex::new(None),
        }
    }
}

pub static STATE: Lazy<AppState> = Lazy::new(AppState::default);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::open_apk,
            commands::analyze_apk,
            commands::get_analysis,
            commands::search_global,
            commands::export_report,
            commands::get_recent_files,
            commands::add_recent_file,
            commands::clear_recent_files,
            commands::cancel_analysis,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
