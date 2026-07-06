use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use libloading::Library;
use log::{error, info, warn};

use super::abi::{PluginVTable, ABI_VERSION, ENTRY_SYMBOL};
use super::manifest::PluginManifest;

/// 已加载的插件实例
pub struct LoadedPlugin {
    pub manifest: PluginManifest,
    /// vtable 指针。加载失败时为 null。加载成功时指向插件库内的静态 vtable，
    /// 生命周期与 `_lib` 库句柄一致。
    pub vtable: SendPtr<PluginVTable>,
    pub enabled: bool,
    /// 加载错误（动态库加载失败、ABI 不匹配等），加载成功时为 None
    pub load_error: Option<String>,
    /// 动态库文件路径（用于错误提示）
    pub lib_path: PathBuf,
    /// 库句柄，保持生命周期直到 LoadedPlugin 被 drop
    _lib: Option<Library>,
}

/// 裸指针包装器，实现 Send + Sync。
///
/// `*const T` 默认不实现 Send/Sync，但 PluginVTable 来自插件库的静态符号，
/// 在加载后是只读的、可在任意线程读取。Library 句柄也是线程安全的（libloading 内部用 dlopen）。
/// 用此包装器让 PluginManager 可放入 static Mutex。
#[derive(Debug, Clone, Copy)]
pub struct SendPtr<T>(*const T);

unsafe impl<T> Send for SendPtr<T> {}
unsafe impl<T> Sync for SendPtr<T> {}

impl<T> SendPtr<T> {
    pub const fn null() -> Self {
        Self(std::ptr::null())
    }
    pub fn new(ptr: *const T) -> Self {
        Self(ptr)
    }
    pub fn as_ptr(&self) -> *const T {
        self.0
    }
    pub fn is_null(&self) -> bool {
        self.0.is_null()
    }
    /// 获取引用（指针非空时）
    ///
    /// SAFETY: 调用者必须保证指针有效且指向的内存未被释放
    /// （库句柄在 LoadedPlugin._lib 中保活，保证 vtable 静态符号有效）。
    pub unsafe fn as_ref(&self) -> Option<&T> {
        if self.0.is_null() {
            None
        } else {
            Some(&*self.0)
        }
    }
}

impl LoadedPlugin {
    /// 创建一个加载失败的占位条目，保留路径和错误信息便于 UI 展示
    fn failed(lib_path: PathBuf, manifest: PluginManifest, err: String) -> Self {
        Self {
            manifest,
            vtable: SendPtr::null(),
            enabled: false,
            load_error: Some(err),
            lib_path,
            _lib: None,
        }
    }

    /// 获取 vtable 引用（加载成功时）
    ///
    /// 返回 `&'static PluginVTable`，因为 vtable 是插件库内的静态符号。
    /// SAFETY: 库句柄在 `_lib` 字段中保活，保证 vtable 在 LoadedPlugin 生命周期内有效。
    pub fn vtable(&self) -> Option<&'static PluginVTable> {
        let ptr = self.vtable.as_ptr();
        if ptr.is_null() {
            None
        } else {
            // SAFETY: vtable 来自插件导出的静态符号
            Some(unsafe { &*ptr })
        }
    }
}

/// 插件管理器。负责发现、加载、注册插件，以及持久化启用状态。
pub struct PluginManager {
    pub plugins: Vec<LoadedPlugin>,
    /// 持久化到 plugins.toml 的启用状态：plugin_id -> enabled
    enabled_state: HashMap<String, bool>,
    config_path: PathBuf,
}

impl PluginManager {
    /// 创建并扫描插件目录。
    /// 插件目录：~/.apk-analyzer/plugins/（macOS/Linux），%APPDATA%/apk-analyzer/plugins/（Windows）
    pub fn new() -> Self {
        let plugins_dir = plugins_dir();
        let config_path = plugins_dir.join("plugins.toml");
        let mut mgr = Self {
            plugins: Vec::new(),
            enabled_state: HashMap::new(),
            config_path,
        };
        mgr.load_config();
        mgr.scan(&plugins_dir);
        mgr
    }

    /// 扫描插件目录，加载每个子目录中的动态库 + manifest.json
    fn scan(&mut self, plugins_dir: &Path) {
        if !plugins_dir.exists() {
            info!("Plugin directory does not exist, creating: {:?}", plugins_dir);
            let _ = std::fs::create_dir_all(plugins_dir);
            return;
        }

        let entries = match std::fs::read_dir(plugins_dir) {
            Ok(e) => e,
            Err(e) => {
                error!("Failed to read plugin dir {:?}: {}", plugins_dir, e);
                return;
            }
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            self.load_plugin_dir(&path);
        }
    }

    fn load_plugin_dir(&mut self, dir: &Path) {
        let manifest_path = dir.join("manifest.json");
        let lib_path = dir.join(plugin_lib_filename());

        // 读取 manifest.json（必需）
        let manifest: PluginManifest = match std::fs::read_to_string(&manifest_path) {
            Ok(content) => match serde_json::from_str(&content) {
                Ok(m) => m,
                Err(e) => {
                    let err = format!("Invalid manifest.json: {}", e);
                    warn!("Plugin {:?}: {}", dir, err);
                    self.plugins.push(LoadedPlugin::failed(
                        lib_path,
                        PluginManifest::default(),
                        err,
                    ));
                    return;
                }
            },
            Err(e) => {
                let err = format!("Cannot read manifest.json: {}", e);
                warn!("Plugin {:?}: {}", dir, err);
                self.plugins.push(LoadedPlugin::failed(
                    lib_path,
                    PluginManifest::default(),
                    err,
                ));
                return;
            }
        };

        if manifest.id.is_empty() {
            let err = "manifest.json missing 'id'".to_string();
            warn!("Plugin {:?}: {}", dir, err);
            self.plugins.push(LoadedPlugin::failed(
                lib_path,
                manifest,
                err,
            ));
            return;
        }

        // 加载动态库
        if !lib_path.exists() {
            let err = format!(
                "Dynamic library not found: {}",
                lib_path.display()
            );
            warn!("Plugin {}: {}", manifest.id, err);
            self.plugins.push(LoadedPlugin::failed(lib_path, manifest, err));
            return;
        }

        let lib = unsafe {
            match Library::new(&lib_path) {
                Ok(l) => l,
                Err(e) => {
                    let err = format!("Failed to load library: {}", e);
                    error!("Plugin {}: {}", manifest.id, err);
                    self.plugins.push(LoadedPlugin::failed(lib_path, manifest, err));
                    return;
                }
            }
        };

        // 获取 vtable 入口函数并调用。
        // 插件 SDK 导出的是 `apk_analyzer_plugin_vtable() -> *const PluginVTable`，
        // 不是静态指针变量；这里必须先取函数符号再调用。
        let vtable_ptr: *const PluginVTable = unsafe {
            match lib.get::<extern "C" fn() -> *const PluginVTable>(ENTRY_SYMBOL.as_bytes()) {
                Ok(entry) => entry(),
                Err(e) => {
                    let err = format!("Missing symbol '{}': {}", ENTRY_SYMBOL, e);
                    error!("Plugin {}: {}", manifest.id, err);
                    self.plugins.push(LoadedPlugin::failed(lib_path, manifest, err));
                    return;
                }
            }
        };

        if vtable_ptr.is_null() {
            let err = "vtable pointer is null".to_string();
            error!("Plugin {}: {}", manifest.id, err);
            self.plugins.push(LoadedPlugin::failed(lib_path, manifest, err));
            return;
        }

        // SAFETY: vtable_ptr 来自插件导出的 &'static，插件保证其生命周期与库一致。
        // 库句柄保存在 _lib 字段中，确保 vtable 在 LoadedPlugin 生命周期内有效。
        let vtable: &'static PluginVTable = unsafe { &*vtable_ptr };

        // 校验 ABI 版本
        if vtable.abi_version != ABI_VERSION {
            let err = format!(
                "ABI version mismatch: plugin={}, host={}",
                vtable.abi_version, ABI_VERSION
            );
            error!("Plugin {}: {}", manifest.id, err);
            self.plugins.push(LoadedPlugin::failed(lib_path, manifest, err));
            return;
        }

        let enabled = self.enabled_state.get(&manifest.id).copied().unwrap_or(true);

        info!(
            "Loaded plugin: {} v{} ({}) — enabled={}",
            manifest.id,
            manifest.version,
            manifest.name,
            enabled
        );

        // 将库句柄转移到 LoadedPlugin，保持 vtable 有效
        let loaded = LoadedPlugin {
            manifest,
            vtable: SendPtr::new(vtable_ptr),
            enabled,
            load_error: None,
            lib_path,
            _lib: Some(lib),
        };
        self.plugins.push(loaded);
    }

    /// 读取 plugins.toml 持久化的启用状态
    fn load_config(&mut self) {
        let Ok(content) = std::fs::read_to_string(&self.config_path) else {
            // 配置不存在时视为空配置，使用默认（启用）
            return;
        };
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((key, val)) = line.split_once('=') {
                let key = key.trim().to_string();
                let val = val.trim();
                let enabled = val == "true" || val == "1";
                self.enabled_state.insert(key, enabled);
            }
        }
    }

    /// 持久化启用状态到 plugins.toml
    pub fn save_config(&self) {
        let mut content = String::from("# APK Analyzer plugin enabled state\n");
        for p in &self.plugins {
            content.push_str(&format!("{} = {}\n", p.manifest.id, p.enabled));
        }
        if let Err(e) = std::fs::create_dir_all(self.config_path.parent().unwrap_or(Path::new("."))) {
            error!("Failed to create config dir: {}", e);
            return;
        }
        if let Err(e) = std::fs::write(&self.config_path, content) {
            error!("Failed to write plugin config: {}", e);
        }
    }

    /// 设置插件启用状态并持久化
    pub fn set_enabled(&mut self, plugin_id: &str, enabled: bool) -> Result<(), String> {
        let p = self
            .plugins
            .iter_mut()
            .find(|p| p.manifest.id == plugin_id)
            .ok_or_else(|| format!("Plugin not found: {}", plugin_id))?;
        if p.load_error.is_some() {
            return Err(format!(
                "Plugin '{}' has load error: {}",
                plugin_id,
                p.load_error.as_ref().unwrap()
            ));
        }
        p.enabled = enabled;
        self.enabled_state.insert(plugin_id.to_string(), enabled);
        self.save_config();
        Ok(())
    }

    /// 获取所有已启用的、加载成功的、具备指定能力的插件
    pub fn enabled_with_capability(&self, cap: super::manifest::Capability) -> Vec<&LoadedPlugin> {
        self.plugins
            .iter()
            .filter(|p| p.enabled && p.load_error.is_none())
            .filter(|p| p.manifest.capabilities.contains(&cap))
            .collect()
    }

    /// 按 ID 获取插件
    pub fn get(&self, plugin_id: &str) -> Option<&LoadedPlugin> {
        self.plugins.iter().find(|p| p.manifest.id == plugin_id)
    }

    pub fn get_mut(&mut self, plugin_id: &str) -> Option<&mut LoadedPlugin> {
        self.plugins.iter_mut().find(|p| p.manifest.id == plugin_id)
    }

    /// 用于 UI 展示的简短摘要
    pub fn summary(&self) -> Vec<PluginSummary> {
        self.plugins
            .iter()
            .map(|p| PluginSummary {
                id: p.manifest.id.clone(),
                name: p.manifest.name.clone(),
                version: p.manifest.version.clone(),
                author: p.manifest.author.clone(),
                description: p.manifest.description.clone(),
                enabled: p.enabled,
                load_error: p.load_error.clone(),
                capabilities: p.manifest.capabilities.iter().map(|c| format!("{:?}", c).to_lowercase()).collect(),
            })
            .collect()
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct PluginSummary {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub enabled: bool,
    pub load_error: Option<String>,
    pub capabilities: Vec<String>,
}

/// 获取插件目录路径
pub fn plugins_dir() -> PathBuf {
    let base = dirs::config_dir()
        .or_else(dirs::home_dir)
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("apk-analyzer").join("plugins")
}

/// 当前平台的动态库文件名（不含路径）
fn plugin_lib_filename() -> &'static str {
    #[cfg(target_os = "macos")]
    return "plugin.dylib";
    #[cfg(target_os = "windows")]
    return "plugin.dll";
    #[cfg(target_os = "linux")]
    return "plugin.so";
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    return "plugin.so";
}

/// 全局 PluginManager 单例（用 once_cell + Mutex 保护）
static MANAGER: Mutex<Option<PluginManager>> = Mutex::new(None);

/// 初始化全局 PluginManager（启动时调用一次）
pub fn init_global() {
    let mut guard = MANAGER.lock().unwrap();
    if guard.is_none() {
        *guard = Some(PluginManager::new());
    }
}

/// 获取全局 PluginManager 的只读访问
pub fn with_manager<F, R>(f: F) -> R
where
    F: FnOnce(&PluginManager) -> R,
{
    let guard = MANAGER.lock().unwrap();
    f(guard.as_ref().expect("PluginManager not initialized; call init_global() first"))
}

/// 获取全局 PluginManager 的可变访问
pub fn with_manager_mut<F, R>(f: F) -> R
where
    F: FnOnce(&mut PluginManager) -> R,
{
    let mut guard = MANAGER.lock().unwrap();
    f(guard.as_mut().expect("PluginManager not initialized; call init_global() first"))
}
