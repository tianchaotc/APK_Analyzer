use std::os::raw::{c_char, c_int, c_void};
use std::ptr;

use log::{error, info, trace, warn};

use crate::parser::axml;
use crate::parser::dex::DexParser;
use crate::parser::ApkReader;

use super::abi::{self, HostApi, ERR_GENERIC, ERR_INVALID_ARG, ERR_NOT_FOUND, OK};

/// 宿主上下文。传递给插件的 HostApi.ctx 实际指向此结构。
/// 借用 ApkReader 和当前分析结果（用于 get_analysis）。
///
/// 注意：apk 字段使用裸指针 `*mut ApkReader`，因为 HostApi 通过 `*const c_void`
/// 传递给插件，但 read_apk_file 需要 `&mut ApkReader`。调用者保证 apk 在
/// HostContext 存活期间有效且没有别名借用冲突。
pub struct HostContext<'a> {
    pub apk: *mut ApkReader,
    /// 当前已完成的分析结果（JSON 字符串，按 key 索引）
    /// 在 analyze_apk 流程中，每完成一个内置分析器就插入对应 key。
    pub analysis_json: &'a std::collections::HashMap<&'static str, String>,
}

/// 构造一个 HostApi vtable，其 ctx 指向给定的 HostContext。
///
/// SAFETY: 调用者必须保证在插件使用 HostApi 期间，HostContext 保持存活，
/// 且 apk 指针在此期间没有其他可变借用。
pub fn build_host_api(ctx: &HostContext) -> HostApi {
    HostApi {
        abi_version: abi::ABI_VERSION,
        ctx: ctx as *const HostContext as *const c_void,
        read_apk_file: host_read_apk_file,
        list_apk_files: host_list_apk_files,
        parse_axml: host_parse_axml,
        parse_dex: host_parse_dex,
        get_analysis: host_get_analysis,
        log: host_log,
        free_host: host_free_host,
    }
}

// ============ HostApi 函数实现 ============

unsafe fn from_cstr_ptr(ptr: *const c_char, len: usize) -> Result<String, c_int> {
    if ptr.is_null() {
        return Err(ERR_INVALID_ARG);
    }
    let bytes = std::slice::from_raw_parts(ptr as *const u8, len);
    std::str::from_utf8(bytes)
        .map(|s| s.to_string())
        .map_err(|_| ERR_INVALID_ARG)
}

/// 分配 len 字节的 c_char 缓冲区并写入 bytes，返回指针。
/// 失败时返回 null（调用者应返回 ERR_ALLOC）。
unsafe fn alloc_cstring(bytes: &[u8]) -> *mut c_char {
    let layout = match std::alloc::Layout::array::<c_char>(bytes.len()) {
        Ok(l) => l,
        Err(_) => return ptr::null_mut(),
    };
    let ptr = std::alloc::alloc(layout) as *mut c_char;
    if ptr.is_null() {
        return ptr::null_mut();
    }
    std::ptr::copy_nonoverlapping(bytes.as_ptr() as *const c_char, ptr, bytes.len());
    ptr
}

unsafe fn alloc_bytes(bytes: &[u8]) -> *mut u8 {
    let layout = match std::alloc::Layout::array::<u8>(bytes.len()) {
        Ok(l) => l,
        Err(_) => return ptr::null_mut(),
    };
    let ptr = std::alloc::alloc(layout);
    if ptr.is_null() {
        return ptr::null_mut();
    }
    std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, bytes.len());
    ptr
}

extern "C" fn host_read_apk_file(
    ctx: *const c_void,
    path: *const c_char,
    path_len: usize,
    out: *mut *mut u8,
    out_len: *mut usize,
) -> c_int {
    if ctx.is_null() || out.is_null() || out_len.is_null() {
        return ERR_INVALID_ARG;
    }
    let path = match unsafe { from_cstr_ptr(path, path_len) } {
        Ok(s) => s,
        Err(code) => return code,
    };
    let ctx = unsafe { &*(ctx as *const HostContext) };
    if ctx.apk.is_null() {
        return ERR_NOT_FOUND;
    }
    // SAFETY: 调用者保证 apk 指针有效且无别名借用
    let apk = unsafe { &mut *ctx.apk };
    match apk.read_file(&path) {
        Ok(bytes) => {
            let ptr = unsafe { alloc_bytes(&bytes) };
            if ptr.is_null() {
                return ERR_GENERIC;
            }
            unsafe {
                *out = ptr;
                *out_len = bytes.len();
            }
            OK
        }
        Err(_) => ERR_NOT_FOUND,
    }
}

extern "C" fn host_list_apk_files(
    ctx: *const c_void,
    out: *mut *mut c_char,
    out_len: *mut usize,
) -> c_int {
    if ctx.is_null() || out.is_null() || out_len.is_null() {
        return ERR_INVALID_ARG;
    }
    let ctx = unsafe { &*(ctx as *const HostContext) };
    if ctx.apk.is_null() {
        return ERR_NOT_FOUND;
    }
    // SAFETY: 同上
    let apk = unsafe { &*ctx.apk };
    let names = apk.file_names();
    let json = serde_json::to_string(&names).unwrap_or_else(|_| "[]".to_string());
    let bytes = json.into_bytes();
    let ptr = unsafe { alloc_cstring(&bytes) };
    if ptr.is_null() {
        return ERR_GENERIC;
    }
    unsafe {
        *out = ptr;
        *out_len = bytes.len();
    }
    OK
}

extern "C" fn host_parse_axml(
    _ctx: *const c_void,
    bytes: *const u8,
    len: usize,
    out: *mut *mut c_char,
    out_len: *mut usize,
) -> c_int {
    if bytes.is_null() || out.is_null() || out_len.is_null() {
        return ERR_INVALID_ARG;
    }
    let input = unsafe { std::slice::from_raw_parts(bytes, len) };
    let element = match axml::decode(input) {
        Ok(e) => e,
        Err(e) => {
            error!("[plugin] parse_axml error: {}", e);
            return ERR_GENERIC;
        }
    };
    let json = axml_element_to_json(&element);
    let bytes = json.into_bytes();
    let ptr = unsafe { alloc_cstring(&bytes) };
    if ptr.is_null() {
        return ERR_GENERIC;
    }
    unsafe {
        *out = ptr;
        *out_len = bytes.len();
    }
    OK
}

extern "C" fn host_parse_dex(
    _ctx: *const c_void,
    bytes: *const u8,
    len: usize,
    out: *mut *mut c_char,
    out_len: *mut usize,
) -> c_int {
    if bytes.is_null() || out.is_null() || out_len.is_null() {
        return ERR_INVALID_ARG;
    }
    let input = unsafe { std::slice::from_raw_parts(bytes, len) };
    // 复用宿主 DEX 分析器逻辑，输出 JSON
    match DexParser::parse(input) {
        Ok(dex_info) => {
            let json = dex_stats_to_json(&dex_info);
            let bytes = json.into_bytes();
            let ptr = unsafe { alloc_cstring(&bytes) };
            if ptr.is_null() {
                return ERR_GENERIC;
            }
            unsafe {
                *out = ptr;
                *out_len = bytes.len();
            }
            OK
        }
        Err(e) => {
            error!("[plugin] parse_dex error: {}", e);
            ERR_GENERIC
        }
    }
}

extern "C" fn host_get_analysis(
    ctx: *const c_void,
    key: *const c_char,
    key_len: usize,
    out: *mut *mut c_char,
    out_len: *mut usize,
) -> c_int {
    if ctx.is_null() || out.is_null() || out_len.is_null() {
        return ERR_INVALID_ARG;
    }
    let key = match unsafe { from_cstr_ptr(key, key_len) } {
        Ok(s) => s,
        Err(code) => return code,
    };
    let ctx = unsafe { &*(ctx as *const HostContext) };
    match ctx.analysis_json.get(key.as_str()) {
        Some(json) => {
            let bytes = json.as_bytes();
            let ptr = unsafe { alloc_cstring(bytes) };
            if ptr.is_null() {
                return ERR_GENERIC;
            }
            unsafe {
                *out = ptr;
                *out_len = bytes.len();
            }
            OK
        }
        None => {
            // 找不到时返回空字符串（不是错误）
            unsafe {
                *out = ptr::null_mut();
                *out_len = 0;
            }
            OK
        }
    }
}

extern "C" fn host_log(ctx: *const c_void, level: c_int, msg: *const c_char, msg_len: usize) {
    // ctx 在纯 log 调用时可能不被需要，但保留以匹配 ABI
    let _ = ctx;
    let msg = if msg.is_null() || msg_len == 0 {
        String::new()
    } else {
        unsafe { from_cstr_ptr(msg, msg_len).unwrap_or_default() }
    };
    match level {
        0 => trace!("[plugin] {}", msg),
        1 => info!("[plugin] {}", msg),
        3 => warn!("[plugin] {}", msg),
        4 => error!("[plugin] {}", msg),
        _ => info!("[plugin] {}", msg), // 2 和其他默认 info
    }
}

extern "C" fn host_free_host(ptr: *mut c_void, len: usize) {
    if ptr.is_null() || len == 0 {
        return;
    }
    // 尝试按 c_char 数组释放（list_apk_files/parse_axml/get_analysis 返回的）
    if let Ok(layout) = std::alloc::Layout::array::<c_char>(len) {
        unsafe { std::alloc::dealloc(ptr as *mut u8, layout) };
    }
}

// ============ AXML 序列化为 JSON ============

/// 将 AxmlElement 树序列化为 JSON 对象。
/// 不依赖 serde derive，避免修改 parser/axml.rs。
fn axml_element_to_json(elem: &axml::AxmlElement) -> String {
    let json = serde_json::json!({
        "name": elem.name,
        "namespace": elem.namespace,
        "text": elem.text,
        "attributes": elem.attributes.iter().map(|a| serde_json::json!({
            "name": a.name,
            "namespace": a.namespace,
            "value": a.value,
            "raw_value": a.raw_value,
            "typed_type": a.typed_type,
        })).collect::<Vec<_>>(),
        "children": elem.children.iter().map(axml_element_to_json_value).collect::<Vec<_>>(),
    });
    json.to_string()
}

fn axml_element_to_json_value(elem: &axml::AxmlElement) -> serde_json::Value {
    serde_json::json!({
        "name": elem.name,
        "namespace": elem.namespace,
        "text": elem.text,
        "attributes": elem.attributes.iter().map(|a| serde_json::json!({
            "name": a.name,
            "namespace": a.namespace,
            "value": a.value,
            "raw_value": a.raw_value,
            "typed_type": a.typed_type,
        })).collect::<Vec<_>>(),
        "children": elem.children.iter().map(axml_element_to_json_value).collect::<Vec<_>>(),
    })
}

// ============ DEX 序列化为 JSON ============

/// 将 DexStats 序列化为 JSON。
/// DexStats 没有派生 Serialize，这里手动构造，避免修改 parser/dex.rs。
fn dex_stats_to_json(stats: &crate::parser::dex::DexStats) -> String {
    let packages: Vec<serde_json::Value> = stats
        .packages
        .iter()
        .map(|(name, pkg)| {
            serde_json::json!({
                "name": name,
                "class_count": pkg.class_count,
                "method_count": pkg.method_count,
                "field_count": pkg.field_count,
            })
        })
        .collect();
    serde_json::json!({
        "file_size": stats.file_size,
        "class_count": stats.class_count,
        "method_count": stats.method_count,
        "field_count": stats.field_count,
        "string_count": stats.string_count,
        "type_count": stats.type_count,
        "proto_count": stats.proto_count,
        "class_names": stats.class_names,
        "packages": packages,
    })
    .to_string()
}
