// 与宿主 src-tauri/src/plugin/abi.rs 完全一致的 C ABI 定义。
// 插件 crate 依赖此 crate 即可获得 ABI 类型，无需复制代码。

use std::os::raw::{c_char, c_int, c_void};

/// 宿主提供给插件的能力（vtable）
#[repr(C)]
pub struct HostApi {
    pub abi_version: u32,
    pub ctx: *const c_void,
    pub read_apk_file: extern "C" fn(
        ctx: *const c_void,
        path: *const c_char,
        path_len: usize,
        out: *mut *mut u8,
        out_len: *mut usize,
    ) -> c_int,
    pub list_apk_files: extern "C" fn(
        ctx: *const c_void,
        out: *mut *mut c_char,
        out_len: *mut usize,
    ) -> c_int,
    pub parse_axml: extern "C" fn(
        ctx: *const c_void,
        bytes: *const u8,
        len: usize,
        out: *mut *mut c_char,
        out_len: *mut usize,
    ) -> c_int,
    pub parse_dex: extern "C" fn(
        ctx: *const c_void,
        bytes: *const u8,
        len: usize,
        out: *mut *mut c_char,
        out_len: *mut usize,
    ) -> c_int,
    pub get_analysis: extern "C" fn(
        ctx: *const c_void,
        key: *const c_char,
        key_len: usize,
        out: *mut *mut c_char,
        out_len: *mut usize,
    ) -> c_int,
    pub log: extern "C" fn(
        ctx: *const c_void,
        level: c_int,
        msg: *const c_char,
        msg_len: usize,
    ),
    pub free_host: extern "C" fn(ptr: *mut c_void, len: usize),
}

/// 插件导出的函数指针表
#[repr(C)]
pub struct PluginVTable {
    pub abi_version: u32,
    pub metadata: extern "C" fn() -> *const c_char,
    pub analyze: extern "C" fn(
        host: *const HostApi,
        apk_path: *const c_char,
        apk_path_len: usize,
        out: *mut *mut c_char,
        out_len: *mut usize,
    ) -> c_int,
    pub ui_schema: extern "C" fn(
        out: *mut *mut c_char,
        out_len: *mut usize,
    ) -> c_int,
    pub export: Option<extern "C" fn(
        host: *const HostApi,
        fmt: *const c_char,
        fmt_len: usize,
        data: *const c_char,
        data_len: usize,
        out: *mut *mut u8,
        out_len: *mut usize,
    ) -> c_int>,
    pub command: Option<extern "C" fn(
        host: *const HostApi,
        cmd: *const c_char,
        cmd_len: usize,
        args: *const c_char,
        args_len: usize,
        out: *mut *mut c_char,
        out_len: *mut usize,
    ) -> c_int>,
    pub free: extern "C" fn(ptr: *mut c_void, len: usize),
}

pub const ABI_VERSION: u32 = 1;
