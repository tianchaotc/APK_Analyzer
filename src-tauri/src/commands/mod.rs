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
    let mut analysis_json = std::collections::HashMap::new();
    let mut plugin_results = Vec::new();

    let _ = emit_progress(&window, "Overview", "Analyzing app overview...", 10);
    if CANCEL_FLAG.load(Ordering::SeqCst) { return Err("Analysis cancelled".to_string()); }
    let overview = overview::OverviewAnalyzer.analyze(&mut apk)?;
    insert_analysis_json(&mut analysis_json, "overview", &overview);
    plugin_results.extend(run_plugins_for_stage(
        &window,
        &path,
        &analysis_json,
        crate::plugin::manifest::AnalyzerStage::AfterOverview,
        12,
    )?);

    let _ = emit_progress(&window, "Manifest", "Parsing AndroidManifest.xml...", 20);
    if CANCEL_FLAG.load(Ordering::SeqCst) { return Err("Analysis cancelled".to_string()); }
    let manifest = manifest::ManifestAnalyzer.analyze(&mut apk)?;
    insert_analysis_json(&mut analysis_json, "manifest", &manifest);
    plugin_results.extend(run_plugins_for_stage(
        &window,
        &path,
        &analysis_json,
        crate::plugin::manifest::AnalyzerStage::AfterManifest,
        22,
    )?);

    let _ = emit_progress(&window, "Permissions", "Analyzing permissions...", 30);
    if CANCEL_FLAG.load(Ordering::SeqCst) { return Err("Analysis cancelled".to_string()); }
    let permissions = permissions::PermissionAnalyzer.analyze(&mut apk)?;
    insert_analysis_json(&mut analysis_json, "permissions", &permissions);

    let _ = emit_progress(&window, "Components", "Analyzing components...", 40);
    if CANCEL_FLAG.load(Ordering::SeqCst) { return Err("Analysis cancelled".to_string()); }
    let components = components::ComponentAnalyzer.analyze(&mut apk)?;
    insert_analysis_json(&mut analysis_json, "components", &components);

    let _ = emit_progress(&window, "Resources", "Analyzing resources...", 55);
    if CANCEL_FLAG.load(Ordering::SeqCst) { return Err("Analysis cancelled".to_string()); }
    let resources = resources::ResourceAnalyzer.analyze(&mut apk)?;
    insert_analysis_json(&mut analysis_json, "resources", &resources);

    let _ = emit_progress(&window, "Native Libraries", "Analyzing native libraries...", 65);
    if CANCEL_FLAG.load(Ordering::SeqCst) { return Err("Analysis cancelled".to_string()); }
    let native_libs = native_libs::NativeLibAnalyzer.analyze(&mut apk)?;
    insert_analysis_json(&mut analysis_json, "native_libs", &native_libs);

    let _ = emit_progress(&window, "DEX", "Analyzing DEX files...", 75);
    if CANCEL_FLAG.load(Ordering::SeqCst) { return Err("Analysis cancelled".to_string()); }
    let dex = dex::DexAnalyzer.analyze(&mut apk)?;
    insert_analysis_json(&mut analysis_json, "dex", &dex);

    let _ = emit_progress(&window, "Certificate", "Analyzing certificates...", 85);
    if CANCEL_FLAG.load(Ordering::SeqCst) { return Err("Analysis cancelled".to_string()); }
    let certificate = certificate::CertificateAnalyzer.analyze(&mut apk)?;
    insert_analysis_json(&mut analysis_json, "certificate", &certificate);

    let _ = emit_progress(&window, "Security", "Running security checks...", 95);
    if CANCEL_FLAG.load(Ordering::SeqCst) { return Err("Analysis cancelled".to_string()); }
    let security = security::SecurityAnalyzer.analyze(&mut apk)?;
    insert_analysis_json(&mut analysis_json, "security", &security);
    plugin_results.extend(run_plugins_for_stage(
        &window,
        &path,
        &analysis_json,
        crate::plugin::manifest::AnalyzerStage::AfterSecurity,
        96,
    )?);

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
        plugins: plugin_results,
    };

    // Generate AI summary
    let _ = emit_progress(&window, "AI Summary", "Generating AI summary...", 98);
    let mut apk_for_ai = ApkReader::open(&path)?;
    let ai_summary = ai_analyzer::generate_summary(&mut apk_for_ai, &analysis);
    insert_analysis_json(&mut analysis_json, "ai_summary", &ai_summary);
    analysis.ai_summary = Some(ai_summary);

    analysis.plugins.extend(run_plugins_for_stage(
        &window,
        &path,
        &analysis_json,
        crate::plugin::manifest::AnalyzerStage::Final,
        99,
    )?);

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

// ============ 插件集成 ============

fn insert_analysis_json<T: serde::Serialize>(
    map: &mut std::collections::HashMap<&'static str, String>,
    key: &'static str,
    value: &T,
) {
    if let Ok(s) = serde_json::to_string(value) {
        map.insert(key, s);
    }
}

fn analyzer_stage_label(stage: &crate::plugin::manifest::AnalyzerStage) -> &'static str {
    match stage {
        crate::plugin::manifest::AnalyzerStage::AfterOverview => "AfterOverview",
        crate::plugin::manifest::AnalyzerStage::AfterManifest => "AfterManifest",
        crate::plugin::manifest::AnalyzerStage::AfterSecurity => "AfterSecurity",
        crate::plugin::manifest::AnalyzerStage::Final => "Final",
    }
}

/// 运行指定阶段的已启用插件分析器。
/// 单个插件 panic 或出错不会中断其他插件。
fn run_plugins_for_stage(
    window: &tauri::WebviewWindow,
    apk_path: &str,
    analysis_json: &std::collections::HashMap<&'static str, String>,
    stage: crate::plugin::manifest::AnalyzerStage,
    progress_percent: u8,
) -> Result<Vec<crate::models::analysis::PluginResult>, String> {
    use crate::plugin;
    use std::ffi::CString;
    use std::panic::{catch_unwind, AssertUnwindSafe};

    let stage_for_filter = stage.clone();
    let plugins = plugin::manager::with_manager(|m| {
        m.enabled_with_capability(plugin::manifest::Capability::Analyzer)
            .into_iter()
            .filter(|p| {
                p.manifest
                    .analyzer_stage
                    .clone()
                    .unwrap_or(plugin::manifest::AnalyzerStage::Final)
                    == stage_for_filter
            })
            .filter_map(|p| {
                let ptr = p.vtable.as_ptr();
                if ptr.is_null() {
                    None
                } else {
                    // 提取 manifest.ui_tab 的 label/icon/order 供前端侧边栏使用
                    let (ui_tab_label, ui_tab_icon, ui_tab_order) = match &p.manifest.ui_tab {
                        Some(t) => (Some(t.label.clone()), Some(t.icon.clone()), Some(t.order)),
                        None => (None, None, None),
                    };
                    Some((
                        p.manifest.id.clone(),
                        p.manifest.name.clone(),
                        ptr,
                        ui_tab_label,
                        ui_tab_icon,
                        ui_tab_order,
                    ))
                }
            })
            .collect::<Vec<_>>()
    });

    if plugins.is_empty() {
        return Ok(Vec::new());
    }

    let stage_label = analyzer_stage_label(&stage);
    let _ = emit_progress(
        window,
        "Plugins",
        &format!("Running {} plugin(s) at {}...", plugins.len(), stage_label),
        progress_percent,
    );

    let mut apk = ApkReader::open(apk_path)?;
    let mut results = Vec::with_capacity(plugins.len());

    let apk_path_c = CString::new(apk_path).map_err(|_| "apk_path contains nul".to_string())?;
    let apk_path_bytes = apk_path_c.as_bytes();

    for (plugin_id, plugin_name, vtable_ptr, ui_tab_label, ui_tab_icon, ui_tab_order) in plugins {
        if vtable_ptr.is_null() {
            results.push(crate::models::analysis::PluginResult {
                plugin_id: plugin_id.clone(),
                plugin_name,
                data: serde_json::Value::Null,
                ui_schema: serde_json::Value::Null,
                error: Some("vtable is null (load failed)".to_string()),
                duration_ms: 0,
                ui_tab_label,
                ui_tab_icon,
                ui_tab_order,
            });
            continue;
        }

        // SAFETY: vtable_ptr 来自 PluginManager 的 LoadedPlugin，库句柄在 PluginManager 中保活。
        // PluginManager 是 static，库在整个进程生命周期内有效。
        let vtable: &plugin::PluginVTable = unsafe { &*vtable_ptr };

        let host_ctx = plugin::host::HostContext {
            apk: &mut apk as *mut _,
            analysis_json: &analysis_json,
        };
        let host_api = plugin::host::build_host_api(&host_ctx);

        let start = std::time::Instant::now();

        // 用 catch_unwind 防止插件 panic 导致宿主崩溃
        let analyze_result = catch_unwind(AssertUnwindSafe(|| {
            let mut out: *mut std::os::raw::c_char = std::ptr::null_mut();
            let mut out_len: usize = 0;
            let rc = (vtable.analyze)(
                &host_api as *const _,
                apk_path_bytes.as_ptr() as *const std::os::raw::c_char,
                apk_path_bytes.len(),
                &mut out,
                &mut out_len,
            );
            (rc, out, out_len)
        }));

        let duration_ms = start.elapsed().as_millis() as u64;

        let result = match analyze_result {
            Ok((0, out, out_len)) => {
                // 成功：解析 JSON
                let data = if out.is_null() || out_len == 0 {
                    serde_json::Value::Null
                } else {
                    let bytes = unsafe {
                        std::slice::from_raw_parts(out as *const u8, out_len)
                    };
                    match serde_json::from_slice(bytes) {
                        Ok(v) => v,
                        Err(e) => serde_json::json!({ "_error": format!("invalid JSON: {}", e) }),
                    }
                };
                // 释放插件分配的缓冲区
                if !out.is_null() && out_len > 0 {
                    (vtable.free)(out as *mut std::ffi::c_void, out_len);
                }

                // 获取 UI schema
                let ui_schema = catch_unwind(AssertUnwindSafe(|| {
                    let mut schema_out: *mut std::os::raw::c_char = std::ptr::null_mut();
                    let mut schema_len: usize = 0;
                    let rc = (vtable.ui_schema)(&mut schema_out, &mut schema_len);
                    (rc, schema_out, schema_len)
                }));
                let ui_schema = match ui_schema {
                    Ok((0, schema_out, schema_len)) if !schema_out.is_null() && schema_len > 0 => {
                        let bytes = unsafe {
                            std::slice::from_raw_parts(schema_out as *const u8, schema_len)
                        };
                        let v = serde_json::from_slice(bytes).unwrap_or(serde_json::Value::Null);
                        (vtable.free)(schema_out as *mut std::ffi::c_void, schema_len);
                        v
                    }
                    _ => serde_json::Value::Null,
                };

                crate::models::analysis::PluginResult {
                    plugin_id,
                    plugin_name,
                    data,
                    ui_schema,
                    error: None,
                    duration_ms,
                    ui_tab_label,
                    ui_tab_icon,
                    ui_tab_order,
                }
            }
            Ok((rc, out, out_len)) => {
                // 插件返回错误码
                if !out.is_null() && out_len > 0 {
                    (vtable.free)(out as *mut std::ffi::c_void, out_len);
                }
                crate::models::analysis::PluginResult {
                    plugin_id,
                    plugin_name,
                    data: serde_json::Value::Null,
                    ui_schema: serde_json::Value::Null,
                    error: Some(format!("plugin analyze failed (rc={})", rc)),
                    duration_ms,
                    ui_tab_label,
                    ui_tab_icon,
                    ui_tab_order,
                }
            }
            Err(_) => {
                log::error!("[plugin] {} panicked during analyze", plugin_id);
                crate::models::analysis::PluginResult {
                    plugin_id,
                    plugin_name,
                    data: serde_json::Value::Null,
                    ui_schema: serde_json::Value::Null,
                    error: Some("plugin panicked during analyze".to_string()),
                    duration_ms,
                    ui_tab_label,
                    ui_tab_icon,
                    ui_tab_order,
                }
            }
        };
        results.push(result);
    }

    Ok(results)
}

/// 列出所有已发现的插件（用于 UI 管理面板）
#[tauri::command]
pub fn list_plugins() -> Vec<crate::plugin::manager::PluginSummary> {
    crate::plugin::manager::with_manager(|m| m.summary())
}

/// 启用/禁用插件
#[tauri::command]
pub fn set_plugin_enabled(plugin_id: String, enabled: bool) -> Result<(), String> {
    crate::plugin::manager::with_manager_mut(|m| m.set_enabled(&plugin_id, enabled))
}

/// 获取插件目录路径（用于 UI "打开插件目录" 按钮）
#[tauri::command]
pub fn get_plugins_dir() -> String {
    crate::plugin::manager::plugins_dir().to_string_lossy().to_string()
}

/// 执行插件命令
#[tauri::command]
pub fn plugin_command(
    plugin_id: String,
    cmd: String,
    args: serde_json::Value,
) -> Result<serde_json::Value, String> {
    use std::ffi::CString;
    use std::panic::{catch_unwind, AssertUnwindSafe};

    let vtable_ptr = crate::plugin::manager::with_manager(|m| {
        m.get(&plugin_id).map(|p| p.vtable.as_ptr())
    }).ok_or_else(|| format!("Plugin not found: {}", plugin_id))?;

    if vtable_ptr.is_null() {
        return Err(format!("Plugin '{}' not properly loaded", plugin_id));
    }

    // SAFETY: 同 run_plugins
    let vtable: &crate::plugin::PluginVTable = unsafe { &*vtable_ptr };
    let command_fn = vtable.command.ok_or_else(|| format!("Plugin '{}' has no command capability", plugin_id))?;

    // 命令通常不需要 APK 访问，但 HostApi 仍需构造（约定）
    // 这里用空 analysis_json，apk 不提供
    let empty_map = std::collections::HashMap::new();
    // host_ctx 需要 apk，但命令场景下我们不传 apk：用 dangling 指针
    // 插件命令不应调用 read_apk_file，否则会段错误——这是约定
    let host_ctx = crate::plugin::host::HostContext {
        apk: std::ptr::null_mut(),
        analysis_json: &empty_map,
    };
    let host_api = crate::plugin::host::build_host_api(&host_ctx);

    let cmd_c = CString::new(cmd.clone()).map_err(|_| "cmd contains nul".to_string())?;
    let args_str = args.to_string();
    let args_c = CString::new(args_str).map_err(|_| "args contains nul".to_string())?;

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut out: *mut std::os::raw::c_char = std::ptr::null_mut();
        let mut out_len: usize = 0;
        let rc = command_fn(
            &host_api as *const _,
            cmd_c.as_bytes().as_ptr() as *const std::os::raw::c_char,
            cmd_c.as_bytes().len(),
            args_c.as_bytes().as_ptr() as *const std::os::raw::c_char,
            args_c.as_bytes().len(),
            &mut out,
            &mut out_len,
        );
        (rc, out, out_len)
    }));

    match result {
        Ok((0, out, out_len)) => {
            let value = if out.is_null() || out_len == 0 {
                serde_json::Value::Null
            } else {
                let bytes = unsafe { std::slice::from_raw_parts(out as *const u8, out_len) };
                let v = serde_json::from_slice(bytes).unwrap_or(serde_json::Value::Null);
                (vtable.free)(out as *mut std::ffi::c_void, out_len);
                v
            };
            Ok(value)
        }
        Ok((rc, out, out_len)) => {
            if !out.is_null() && out_len > 0 {
                (vtable.free)(out as *mut std::ffi::c_void, out_len);
            }
            Err(format!("plugin command failed (rc={})", rc))
        }
        Err(_) => Err(format!("plugin '{}' panicked during command", plugin_id)),
    }
}
