// APK Analyzer Plugin SDK
//
// 让插件作者用纯 Rust trait 写插件，无需手写 C ABI。
// 插件实现 `Plugin` trait，然后用 `export_plugin!` 宏导出入口符号。

pub mod abi;
pub mod host;

pub use abi::{HostApi, PluginVTable, ABI_VERSION};
pub use host::{Host, HostError, LogLevel, Metadata};

use std::ffi::CString;
use std::os::raw::{c_char, c_void};

/// 插件实现此 trait。
///
/// `analyze` 是必需方法；`export` 和 `command` 可选（默认返回 `Unsupported`）。
/// 实现者用 `export_plugin!` 宏导出 C ABI 入口。
///
/// 实现类型必须是 `new()` 可构造的（无参关联函数），由宏在静态上下文中调用。
pub trait Plugin: Send + Sync + 'static {
    /// 插件元数据
    fn metadata(&self) -> Metadata;

    /// 执行分析，返回 JSON 结果（任意结构，由 UI schema 描述如何渲染）。
    fn analyze(&self, host: &dyn Host, apk_path: &str) -> Result<serde_json::Value, HostError>;

    /// UI schema（声明式视图描述）。
    /// 返回的 JSON 结构由宿主的 PluginPage 渲染器解释。
    fn ui_schema(&self) -> serde_json::Value;

    /// 可选：导出报告。fmt: "pdf"/"sarif"/"custom"；data: 完整分析的 JSON。
    fn export(
        &self,
        _host: &dyn Host,
        _fmt: &str,
        _data: &serde_json::Value,
    ) -> Result<Vec<u8>, HostError> {
        Err(HostError::unsupported("export"))
    }

    /// 可选：执行命令动作。
    fn command(
        &self,
        _host: &dyn Host,
        _cmd: &str,
        _args: &serde_json::Value,
    ) -> Result<serde_json::Value, HostError> {
        Err(HostError::unsupported("command"))
    }
}

// ============ Host 调用辅助 ============

/// 把宿主返回的 C 字符串 (ptr, len) 转成 `Vec<u8>` 并释放宿主分配。
///
/// SAFETY: ptr 必须是 HostApi 函数返回的有效指针，len 是对应长度。
/// 调用后宿主分配已被 free_host 释放，ptr 不再有效。
unsafe fn take_host_string(
    host: &HostApi,
    ptr: *const c_char,
    len: usize,
) -> Result<Vec<u8>, HostError> {
    if ptr.is_null() {
        return Ok(Vec::new());
    }
    let slice = std::slice::from_raw_parts(ptr as *const u8, len);
    let bytes = slice.to_vec();
    (host.free_host)(ptr as *mut c_void, len);
    Ok(bytes)
}

/// 把宿主返回的字节缓冲区 (ptr, len) 转成 `Vec<u8>` 并释放宿主分配。
unsafe fn take_host_bytes(
    host: &HostApi,
    ptr: *const u8,
    len: usize,
) -> Result<Vec<u8>, HostError> {
    if ptr.is_null() {
        return Ok(Vec::new());
    }
    let slice = std::slice::from_raw_parts(ptr, len);
    let bytes = slice.to_vec();
    (host.free_host)(ptr as *mut c_void, len);
    Ok(bytes)
}

/// 把宿主返回的 JSON 字符串解析为 `serde_json::Value`。
unsafe fn take_host_json(
    host: &HostApi,
    ptr: *const c_char,
    len: usize,
) -> Result<serde_json::Value, HostError> {
    let bytes = take_host_string(host, ptr, len)?;
    if bytes.is_empty() {
        return Ok(serde_json::Value::Null);
    }
    serde_json::from_slice(&bytes).map_err(|e| HostError::host(format!("JSON parse: {}", e)))
}

// ============ HostApi 包装器 ============

/// 把 `&HostApi` 包装成 `dyn Host` 供插件使用。
///
/// 这是 SDK 内部类型，插件作者无需关心，由 `export_plugin!` 宏自动构造。
pub struct HostApiWrapper<'a> {
    api: &'a HostApi,
}

impl<'a> HostApiWrapper<'a> {
    pub fn new(api: &'a HostApi) -> Self {
        Self { api }
    }
}

impl<'a> Host for HostApiWrapper<'a> {
    fn read_apk_file(&self, path: &str) -> Result<Vec<u8>, HostError> {
        let path_c = CString::new(path).map_err(|_| HostError::invalid_arg("path contains nul"))?;
        let path_bytes = path_c.as_bytes();
        let mut out: *mut u8 = std::ptr::null_mut();
        let mut out_len: usize = 0;
        let rc = (self.api.read_apk_file)(
            self.api.ctx,
            path_bytes.as_ptr() as *const c_char,
            path_bytes.len(),
            &mut out,
            &mut out_len,
        );
        match rc {
            0 => unsafe { take_host_bytes(self.api, out, out_len) },
            -4 => Err(HostError::not_found(path)),
            code => Err(HostError::host(format!("read_apk_file rc={}", code))),
        }
    }

    fn list_apk_files(&self) -> Result<Vec<String>, HostError> {
        let mut out: *mut c_char = std::ptr::null_mut();
        let mut out_len: usize = 0;
        let rc = (self.api.list_apk_files)(self.api.ctx, &mut out, &mut out_len);
        if rc != 0 {
            return Err(HostError::host(format!("list_apk_files rc={}", rc)));
        }
        let bytes = unsafe { take_host_string(self.api, out, out_len)? };
        let names: Vec<String> = serde_json::from_slice(&bytes)
            .map_err(|e| HostError::host(format!("list_apk_files JSON: {}", e)))?;
        Ok(names)
    }

    fn parse_axml(&self, bytes: &[u8]) -> Result<serde_json::Value, HostError> {
        let mut out: *mut c_char = std::ptr::null_mut();
        let mut out_len: usize = 0;
        let rc = (self.api.parse_axml)(
            self.api.ctx,
            bytes.as_ptr(),
            bytes.len(),
            &mut out,
            &mut out_len,
        );
        if rc != 0 {
            return Err(HostError::host(format!("parse_axml rc={}", rc)));
        }
        unsafe { take_host_json(self.api, out, out_len) }
    }

    fn parse_dex(&self, bytes: &[u8]) -> Result<serde_json::Value, HostError> {
        let mut out: *mut c_char = std::ptr::null_mut();
        let mut out_len: usize = 0;
        let rc = (self.api.parse_dex)(
            self.api.ctx,
            bytes.as_ptr(),
            bytes.len(),
            &mut out,
            &mut out_len,
        );
        if rc != 0 {
            return Err(HostError::host(format!("parse_dex rc={}", rc)));
        }
        unsafe { take_host_json(self.api, out, out_len) }
    }

    fn get_analysis(&self, key: &str) -> Option<serde_json::Value> {
        let key_c = CString::new(key).ok()?;
        let key_bytes = key_c.as_bytes();
        let mut out: *mut c_char = std::ptr::null_mut();
        let mut out_len: usize = 0;
        let rc = (self.api.get_analysis)(
            self.api.ctx,
            key_bytes.as_ptr() as *const c_char,
            key_bytes.len(),
            &mut out,
            &mut out_len,
        );
        if rc != 0 {
            return None;
        }
        unsafe { take_host_json(self.api, out, out_len).ok() }
    }

    fn log(&self, level: LogLevel, msg: &str) {
        let msg_c = match CString::new(msg) {
            Ok(c) => c,
            Err(_) => return,
        };
        let msg_bytes = msg_c.as_bytes();
        (self.api.log)(
            self.api.ctx,
            level as i32,
            msg_bytes.as_ptr() as *const c_char,
            msg_bytes.len(),
        );
    }
}

// ============ 插件分配的缓冲区管理 ============

/// 把插件的 Vec<u8> 转成 (ptr, len)，调用者负责用 `plugin_free` 释放。
pub fn alloc_plugin_bytes(bytes: &[u8]) -> (*mut c_char, usize) {
    if bytes.is_empty() {
        // 返回非 null 的对齐 dangling 指针，避免 free 时误判
        return (std::ptr::NonNull::<c_char>::dangling().as_ptr(), 0);
    }
    let layout = match std::alloc::Layout::array::<c_char>(bytes.len()) {
        Ok(l) => l,
        Err(_) => return (std::ptr::null_mut(), 0),
    };
    let ptr = unsafe { std::alloc::alloc(layout) as *mut c_char };
    if ptr.is_null() {
        return (std::ptr::null_mut(), 0);
    }
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr() as *const c_char, ptr, bytes.len());
    }
    (ptr, bytes.len())
}

/// 释放插件分配的缓冲区。
///
/// SAFETY: ptr 必须是 `alloc_plugin_bytes` 返回的指针，len 必须匹配。
pub unsafe fn free_plugin_buffer(ptr: *mut c_void, len: usize) {
    if ptr.is_null() || len == 0 {
        return;
    }
    if let Ok(layout) = std::alloc::Layout::array::<c_char>(len) {
        std::alloc::dealloc(ptr as *mut u8, layout);
    }
}

// ============ export_plugin! 宏 ============

/// 导出插件入口符号。
///
/// 用法：
/// ```ignore
/// struct MyPlugin;
/// impl MyPlugin { pub const fn new() -> Self { Self } }
/// impl Plugin for MyPlugin { /* ... */ }
/// export_plugin!(MyPlugin);
/// ```
///
/// 插件类型必须有无参 `new()` 关联函数（const fn 也可），用于静态构造实例。
/// 宏展开为 `#[no_mangle]` 的 `apk_analyzer_plugin_vtable` 入口函数，返回 `&'static PluginVTable`。
#[macro_export]
macro_rules! export_plugin {
    ($plugin_ty:ty) => {
        static PLUGIN_INSTANCE: $plugin_ty = <$plugin_ty>::new();

        // metadata 字符串：第一次调用时 leak 到 'static，后续直接返回。
        static METADATA_LEAKED: std::sync::OnceLock<&'static [u8]> = std::sync::OnceLock::new();

        static VTABLE: $crate::abi::PluginVTable = $crate::abi::PluginVTable {
            abi_version: $crate::ABI_VERSION,
            metadata: plugin_metadata,
            analyze: plugin_analyze,
            ui_schema: plugin_ui_schema,
            export: None,
            command: None,
            free: plugin_free,
        };

        #[no_mangle]
        pub extern "C" fn apk_analyzer_plugin_vtable() -> *const $crate::abi::PluginVTable {
            &VTABLE as *const _
        }

        extern "C" fn plugin_metadata() -> *const std::os::raw::c_char {
            use $crate::Plugin;
            let leaked = METADATA_LEAKED.get_or_init(|| {
                let m = PLUGIN_INSTANCE.metadata();
                let json = serde_json::to_string(&m).unwrap_or_else(|_| "{}".to_string());
                // leak：每个插件只在首次调用时泄漏一次，整个进程生命周期持有
                // into_bytes -> Vec<u8> -> into_boxed_slice -> Box<[u8]>
                Box::leak(json.into_bytes().into_boxed_slice())
            });
            leaked.as_ptr() as *const _
        }

        extern "C" fn plugin_analyze(
            host: *const $crate::abi::HostApi,
            apk_path: *const std::os::raw::c_char,
            apk_path_len: usize,
            out: *mut *mut std::os::raw::c_char,
            out_len: *mut usize,
        ) -> std::os::raw::c_int {
            use $crate::{HostApiWrapper, Plugin};
            if host.is_null() || apk_path.is_null() || out.is_null() || out_len.is_null() {
                return -2;
            }
            let api = unsafe { &*host };
            let path_bytes = unsafe {
                std::slice::from_raw_parts(apk_path as *const u8, apk_path_len)
            };
            let apk_path = match std::str::from_utf8(path_bytes) {
                Ok(s) => s,
                Err(_) => return -2,
            };
            let host_wrapper = HostApiWrapper::new(api);
            match PLUGIN_INSTANCE.analyze(&host_wrapper, apk_path) {
                Ok(value) => {
                    let json = value.to_string();
                    let (ptr, len) = $crate::alloc_plugin_bytes(json.as_bytes());
                    if ptr.is_null() && !json.is_empty() {
                        return -3;
                    }
                    unsafe {
                        *out = ptr;
                        *out_len = len;
                    }
                    0
                }
                Err(e) => {
                    let msg = format!("{}", e);
                    host_wrapper.log($crate::LogLevel::Error, &msg);
                    -1
                }
            }
        }

        extern "C" fn plugin_ui_schema(
            out: *mut *mut std::os::raw::c_char,
            out_len: *mut usize,
        ) -> std::os::raw::c_int {
            use $crate::Plugin;
            let schema = PLUGIN_INSTANCE.ui_schema();
            let json = schema.to_string();
            let (ptr, len) = $crate::alloc_plugin_bytes(json.as_bytes());
            if ptr.is_null() && !json.is_empty() {
                return -3;
            }
            unsafe {
                *out = ptr;
                *out_len = len;
            }
            0
        }

        extern "C" fn plugin_free(ptr: *mut std::os::raw::c_void, len: usize) {
            unsafe { $crate::free_plugin_buffer(ptr, len) };
        }
    };
}
