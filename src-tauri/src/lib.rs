pub mod analyzers;
pub mod commands;
pub mod export;
pub mod models;
pub mod parser;
pub mod plugin;
pub mod utils;

use once_cell::sync::Lazy;
use std::sync::Mutex;

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

    // 初始化插件管理器（扫描插件目录、加载动态库）
    plugin::manager::init_global();

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
            commands::list_plugins,
            commands::set_plugin_enabled,
            commands::get_plugins_dir,
            commands::plugin_command,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
