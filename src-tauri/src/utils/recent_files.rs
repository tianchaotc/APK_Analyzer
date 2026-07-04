use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentFile {
    pub path: String,
    pub name: String,
    pub size: u64,
    pub last_opened: String,
}

/// Get the path to the recent files storage
fn get_storage_path() -> PathBuf {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home)
        .join(".apk_analyzer")
        .join("recent.json")
}

/// Load recent files
pub fn load() -> Vec<RecentFile> {
    let path = get_storage_path();
    match fs::read_to_string(&path) {
        Ok(data) => serde_json::from_str(&data).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

/// Add or update a recent file
pub fn add(file_path: &str, file_name: &str, file_size: u64) -> Vec<RecentFile> {
    let mut recent = load();

    // Remove existing entry for the same path
    recent.retain(|r| r.path != file_path);

    // Add new entry at the top
    recent.insert(0, RecentFile {
        path: file_path.to_string(),
        name: file_name.to_string(),
        size: file_size,
        last_opened: chrono::Utc::now().to_rfc3339(),
    });

    // Keep only last 20 entries
    recent.truncate(20);

    // Save
    save(&recent);

    recent
}

/// Clear all recent files
pub fn clear() {
    let path = get_storage_path();
    let _ = fs::write(&path, "[]");
}

/// Save recent files
fn save(recent: &[RecentFile]) {
    let path = get_storage_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(data) = serde_json::to_string_pretty(recent) {
        let _ = fs::write(&path, data);
    }
}
