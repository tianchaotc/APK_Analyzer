use tauri::{Emitter, State};
use crate::models::analysis::*;
use crate::parser::ApkReader;
use crate::analyzers::*;
use crate::analyzers::ai_summary as ai_analyzer;
use crate::export;
use crate::utils::recent_files;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use once_cell::sync::Lazy;

static CANCEL_FLAG: Lazy<Arc<AtomicBool>> = Lazy::new(|| Arc::new(AtomicBool::new(false)));

/// Open and validate an APK file
#[tauri::command]
pub fn open_apk(path: String) -> Result<ApkFileInfo, String> {
    let file_name = std::path::Path::new(&path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown.apk")
        .to_string();

    let metadata = std::fs::metadata(&path)
        .map_err(|e| format!("Cannot access file: {}", e))?;

    if !path.to_lowercase().ends_with(".apk") {
        return Err("File must be an APK".to_string());
    }

    // Quick validate: try to open as ZIP
    let _ = ApkReader::open(&path)?;

    Ok(ApkFileInfo {
        path: path.clone(),
        name: file_name,
        size: metadata.len(),
    })
}

#[derive(serde::Serialize)]
pub struct ApkFileInfo {
    pub path: String,
    pub name: String,
    pub size: u64,
}

/// Run full APK analysis
#[tauri::command]
pub async fn analyze_apk(
    window: tauri::WebviewWindow,
    state: State<'_, crate::AppState>,
    path: String,
) -> Result<ApkAnalysis, String> {
    CANCEL_FLAG.store(false, Ordering::SeqCst);

    let file_name = std::path::Path::new(&path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown.apk")
        .to_string();

    let file_size = std::fs::metadata(&path)
        .map_err(|e| format!("Cannot access file: {}", e))?
        .len();

    // Open APK
    let _ = emit_progress(&window, "Opening APK", &format!("Opening {}", file_name), 5);
    let mut apk = ApkReader::open(&path)?;

    // Run analyzers sequentially with progress updates
    let stages: &[(&str, u8)] = &[
        ("overview", 10), ("manifest", 20), ("permissions", 30),
        ("components", 40), ("resources", 55), ("native_libs", 65),
        ("dex", 75), ("certificate", 85), ("security", 95),
    ];

    let _ = emit_progress(&window, "Overview", "Analyzing app overview...", 10);
    if CANCEL_FLAG.load(Ordering::SeqCst) { return Err("Analysis cancelled".to_string()); }
    let overview = overview::OverviewAnalyzer.analyze(&mut apk)?;

    let _ = emit_progress(&window, "Manifest", "Parsing AndroidManifest.xml...", 20);
    if CANCEL_FLAG.load(Ordering::SeqCst) { return Err("Analysis cancelled".to_string()); }
    let manifest = manifest::ManifestAnalyzer.analyze(&mut apk)?;

    let _ = emit_progress(&window, "Permissions", "Analyzing permissions...", 30);
    if CANCEL_FLAG.load(Ordering::SeqCst) { return Err("Analysis cancelled".to_string()); }
    let permissions = permissions::PermissionAnalyzer.analyze(&mut apk)?;

    let _ = emit_progress(&window, "Components", "Analyzing components...", 40);
    if CANCEL_FLAG.load(Ordering::SeqCst) { return Err("Analysis cancelled".to_string()); }
    let components = components::ComponentAnalyzer.analyze(&mut apk)?;

    let _ = emit_progress(&window, "Resources", "Analyzing resources...", 55);
    if CANCEL_FLAG.load(Ordering::SeqCst) { return Err("Analysis cancelled".to_string()); }
    let resources = resources::ResourceAnalyzer.analyze(&mut apk)?;

    let _ = emit_progress(&window, "Native Libraries", "Analyzing native libraries...", 65);
    if CANCEL_FLAG.load(Ordering::SeqCst) { return Err("Analysis cancelled".to_string()); }
    let native_libs = native_libs::NativeLibAnalyzer.analyze(&mut apk)?;

    let _ = emit_progress(&window, "DEX", "Analyzing DEX files...", 75);
    if CANCEL_FLAG.load(Ordering::SeqCst) { return Err("Analysis cancelled".to_string()); }
    let dex = dex::DexAnalyzer.analyze(&mut apk)?;

    let _ = emit_progress(&window, "Certificate", "Analyzing certificates...", 85);
    if CANCEL_FLAG.load(Ordering::SeqCst) { return Err("Analysis cancelled".to_string()); }
    let certificate = certificate::CertificateAnalyzer.analyze(&mut apk)?;

    let _ = emit_progress(&window, "Security", "Running security checks...", 95);
    if CANCEL_FLAG.load(Ordering::SeqCst) { return Err("Analysis cancelled".to_string()); }
    let security = security::SecurityAnalyzer.analyze(&mut apk)?;

    // Build complete analysis
    let mut analysis = ApkAnalysis {
        file_path: path.clone(),
        file_name: file_name.clone(),
        file_size,
        analyzed_at: chrono::Utc::now().to_rfc3339(),
        overview,
        manifest,
        permissions,
        components,
        resources,
        native_libs,
        dex,
        certificate,
        security,
        ai_summary: None,
    };

    // Generate AI summary
    let _ = emit_progress(&window, "AI Summary", "Generating AI summary...", 98);
    let mut apk_for_ai = ApkReader::open(&path)?;
    let ai_summary = ai_analyzer::generate_summary(&mut apk_for_ai, &analysis);
    analysis.ai_summary = Some(ai_summary);

    let _ = emit_progress(&window, "Complete", "Analysis complete!", 100);

    // Save to recent files
    recent_files::add(&path, &file_name, file_size);

    // Store in state
    *state.current_analysis.lock().unwrap() = Some(analysis.clone());

    Ok(analysis)
}

/// Get the current analysis from state
#[tauri::command]
pub fn get_analysis(state: State<'_, crate::AppState>) -> Option<ApkAnalysis> {
    state.current_analysis.lock().unwrap().clone()
}

/// Global search across analysis results
#[tauri::command]
pub fn search_global(
    state: State<'_, crate::AppState>,
    query: String,
) -> Result<Vec<SearchResult>, String> {
    let analysis = state.current_analysis.lock().unwrap();
    let analysis = analysis.as_ref().ok_or("No analysis available")?;
    let query_lower = query.to_lowercase();
    let mut results = Vec::new();

    // Search manifest
    for activity in &analysis.manifest.activities {
        if activity.name.to_lowercase().contains(&query_lower) {
            results.push(SearchResult {
                category: "Manifest > Activities".to_string(),
                title: activity.name.clone(),
                detail: format!("Exported: {}, Intent filters: {}", activity.exported, activity.intent_filters.len()),
            });
        }
    }
    for service in &analysis.manifest.services {
        if service.name.to_lowercase().contains(&query_lower) {
            results.push(SearchResult {
                category: "Manifest > Services".to_string(),
                title: service.name.clone(),
                detail: format!("Exported: {}", service.exported),
            });
        }
    }
    for receiver in &analysis.manifest.receivers {
        if receiver.name.to_lowercase().contains(&query_lower) {
            results.push(SearchResult {
                category: "Manifest > Receivers".to_string(),
                title: receiver.name.clone(),
                detail: format!("Exported: {}", receiver.exported),
            });
        }
    }
    for provider in &analysis.manifest.providers {
        if provider.name.to_lowercase().contains(&query_lower) {
            results.push(SearchResult {
                category: "Manifest > Providers".to_string(),
                title: provider.name.clone(),
                detail: format!("Exported: {}", provider.exported),
            });
        }
    }

    // Search permissions
    for perm in &analysis.permissions.permissions {
        if perm.name.to_lowercase().contains(&query_lower)
            || perm.category.to_lowercase().contains(&query_lower)
        {
            results.push(SearchResult {
                category: "Permissions".to_string(),
                title: perm.name.clone(),
                detail: format!("Level: {}, Risk: {}", perm.protection_level, perm.risk_level),
            });
        }
    }

    // Search native libs
    for lib in &analysis.native_libs.libraries {
        if lib.file_name.to_lowercase().contains(&query_lower)
            || lib.abi.to_lowercase().contains(&query_lower)
        {
            results.push(SearchResult {
                category: format!("Native Libraries > {}", lib.abi),
                title: lib.file_name.clone(),
                detail: format!("Size: {} bytes", lib.size),
            });
        }
    }

    // Search DEX packages
    for pkg in &analysis.dex.packages {
        if pkg.name.to_lowercase().contains(&query_lower) {
            results.push(SearchResult {
                category: "DEX > Packages".to_string(),
                title: pkg.name.clone(),
                detail: format!("Classes: {}, Methods: {}", pkg.class_count, pkg.method_count),
            });
        }
    }

    // Search resources
    for res in &analysis.resources.largest_resources {
        if res.name.to_lowercase().contains(&query_lower)
            || res.path.to_lowercase().contains(&query_lower)
        {
            results.push(SearchResult {
                category: format!("Resources > {}", res.resource_type),
                title: res.name.clone(),
                detail: format!("Path: {}, Size: {} bytes", res.path, res.size),
            });
        }
    }

    // Search certificate
    for signer in &analysis.certificate.signers {
        if signer.subject.to_lowercase().contains(&query_lower)
            || signer.issuer.to_lowercase().contains(&query_lower)
        {
            results.push(SearchResult {
                category: "Certificate".to_string(),
                title: signer.subject.clone(),
                detail: format!("Issuer: {}", signer.issuer),
            });
        }
    }

    Ok(results)
}

#[derive(serde::Serialize)]
pub struct SearchResult {
    pub category: String,
    pub title: String,
    pub detail: String,
}

/// Export analysis report
#[tauri::command]
pub fn export_report(
    state: State<'_, crate::AppState>,
    format: String,
    output_path: String,
) -> Result<String, String> {
    let analysis = state.current_analysis.lock().unwrap();
    let analysis = analysis.as_ref().ok_or("No analysis available")?;

    match format.as_str() {
        "json" => export::export_json(analysis, &output_path),
        "markdown" => export::export_markdown(analysis, &output_path),
        "html" => export::export_html(analysis, &output_path),
        "csv" => export::export_csv(analysis, &output_path),
        _ => Err(format!("Unknown format: {}", format)),
    }
}

/// Get recent files
#[tauri::command]
pub fn get_recent_files() -> Vec<recent_files::RecentFile> {
    recent_files::load()
}

/// Add a recent file
#[tauri::command]
pub fn add_recent_file(path: String, name: String, size: u64) -> Vec<recent_files::RecentFile> {
    recent_files::add(&path, &name, size)
}

/// Clear recent files
#[tauri::command]
pub fn clear_recent_files() {
    recent_files::clear();
}

/// Cancel ongoing analysis
#[tauri::command]
pub fn cancel_analysis() {
    CANCEL_FLAG.store(true, Ordering::SeqCst);
}

fn emit_progress(window: &tauri::WebviewWindow, stage: &str, message: &str, percent: u8) -> Result<(), String> {
    window.emit("analysis-progress", ProgressUpdate {
        stage: stage.to_string(),
        message: message.to_string(),
        percent,
    }).map_err(|e| format!("Failed to emit progress: {}", e))
}
