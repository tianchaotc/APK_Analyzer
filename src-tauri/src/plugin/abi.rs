use std::os::raw::{c_char, c_int, c_void};

/// 宿主提供给插件的能力（vtable，函数指针表）
///
/// 插件通过此表调用宿主能力：读取 APK 文件、复用解析器、查询分析结果、日志。
/// 所有跨边界数据用 (ptr, len) 双值传递，避免 null terminator 依赖。
/// 插件分配的缓冲区由插件自己的 `free` 函数释放，宿主不跨边界 free。
#[repr(C)]
pub struct HostApi {
    /// ABI 版本，插件校验此值是否匹配自身编译时的版本
    pub abi_version: u32,
    /// 宿主内部上下文（指向 HostContext），插件不透明，仅用于回传给宿主函数
    pub ctx: *const c_void,

    // —— 文件与 APK 访问 ——

    /// 读取 APK 内的文件，返回堆分配的字节缓冲区。
    /// 调用者负责用 `HostApi` 的释放函数或插件 `free` 释放（取决于分配方）。
    /// 返回码：0=成功，-1=文件不存在，-2=参数无效，-3=内存分配失败
    pub read_apk_file: extern "C" fn(
        ctx: *const c_void,
        path: *const c_char,
        path_len: usize,
        out: *mut *mut u8,
        out_len: *mut usize,
    ) -> c_int,

    /// 列出 APK 内所有文件名（返回 JSON 数组字符串 `["a","b"]`）。
    /// 缓冲区由宿主分配，插件必须用 `HostApi.free_host` 释放。
    pub list_apk_files: extern "C" fn(
        ctx: *const c_void,
        out: *mut *mut c_char,
        out_len: *mut usize,
    ) -> c_int,

    // —— 复用宿主解析器 ——

    /// 解析二进制 AndroidManifest.xml，返回 JSON（AxmlElement 树序列化）。
    /// 复用宿主已调试好的 AXML 解码器，插件无需重新实现。
    pub parse_axml: extern "C" fn(
        ctx: *const c_void,
        bytes: *const u8,
        len: usize,
        out: *mut *mut c_char,
        out_len: *mut usize,
    ) -> c_int,

    /// 解析 DEX 文件，返回 JSON（class/method/field 统计 + 包层级）。
    pub parse_dex: extern "C" fn(
        ctx: *const c_void,
        bytes: *const u8,
        len: usize,
        out: *mut *mut c_char,
        out_len: *mut usize,
    ) -> c_int,

    // —— 查询其他分析器结果 ——

    /// 按 key 查询已完成的内置分析器结果。
    /// key ∈ {"overview","manifest","permissions","components","resources",
    ///         "native_libs","dex","certificate","security","ai_summary"}
    /// 返回 JSON 字符串；找不到时返回 0 且 out 为空字符串。
    pub get_analysis: extern "C" fn(
        ctx: *const c_void,
        key: *const c_char,
        key_len: usize,
        out: *mut *mut c_char,
        out_len: *mut usize,
    ) -> c_int,

    // —— 日志 ——

    /// level: 0=trace 1=debug 2=info 3=warn 4=error
    pub log: extern "C" fn(
        ctx: *const c_void,
        level: c_int,
        msg: *const c_char,
        msg_len: usize,
    ),

    // —— 释放宿主分配的缓冲区 ——

    /// 释放宿主通过 read_apk_file/list_apk_files/parse_axml/parse_dex/get_analysis 返回的缓冲区。
    /// ptr 和 len 是返回时给出的值。ptr 为 null 时无操作。
    pub free_host: extern "C" fn(ptr: *mut c_void, len: usize),
}

/// 插件导出的函数指针表
///
/// 插件必须导出名为 `apk_analyzer_plugin_vtable` 的符号，返回此表的 `&'static` 引用。
/// 宿主加载时校验 `abi_version`，不匹配则拒绝加载。
#[repr(C)]
pub struct PluginVTable {
    /// 插件编译时的 ABI 版本，必须等于宿主的 `ABI_VERSION`
    pub abi_version: u32,

    /// 返回 manifest JSON 字符串（静态分配，无需 free）。
    /// JSON 结构对应 `PluginManifest`，但 capabilities/stage 等以磁盘 manifest.json 为准。
    pub metadata: extern "C" fn() -> *const c_char,

    /// 执行分析。返回 JSON 结果字符串，需用 `free` 释放。
    /// host: HostApi 指针；apk_path: APK 文件系统路径。
    /// 返回码：0=成功，-1=分析错误（错误详情通过 host.log 写出），-2=参数无效
    pub analyze: extern "C" fn(
        host: *const HostApi,
        apk_path: *const c_char,
        apk_path_len: usize,
        out: *mut *mut c_char,
        out_len: *mut usize,
    ) -> c_int,

    /// 返回 UI schema JSON（声明式视图描述），需用 `free` 释放。
    pub ui_schema: extern "C" fn(
        out: *mut *mut c_char,
        out_len: *mut usize,
    ) -> c_int,

    /// 可选：导出报告。fmt: "pdf"/"sarif"/"custom"；data: 已完成分析的 JSON；
    /// 返回字节缓冲区（如 PDF 二进制），需用 `free` 释放。
    pub export: Option<extern "C" fn(
        host: *const HostApi,
        fmt: *const c_char,
        fmt_len: usize,
        data: *const c_char,
        data_len: usize,
        out: *mut *mut u8,
        out_len: *mut usize,
    ) -> c_int>,

    /// 可选：执行命令动作。cmd: 命令名；args: JSON 参数；返回 JSON 结果，需用 `free` 释放。
    pub command: Option<extern "C" fn(
        host: *const HostApi,
        cmd: *const c_char,
        cmd_len: usize,
        args: *const c_char,
        args_len: usize,
        out: *mut *mut c_char,
        out_len: *mut usize,
    ) -> c_int>,

    /// 释放插件分配的缓冲区（analyze/ui_schema/export/command 返回的）。
    /// ptr 为 null 时无操作。
    pub free: extern "C" fn(ptr: *mut c_void, len: usize),
}

/// 插件必须导出的入口符号名
pub const ENTRY_SYMBOL: &str = "apk_analyzer_plugin_vtable";

/// 当前 ABI 版本。插件编译时的版本必须等于此值才能加载。
/// 修改 ABI 后必须递增此值。
pub const ABI_VERSION: u32 = 1;

// 返回码常量
pub const OK: c_int = 0;
pub const ERR_GENERIC: c_int = -1;
pub const ERR_INVALID_ARG: c_int = -2;
pub const ERR_ALLOC: c_int = -3;
pub const ERR_NOT_FOUND: c_int = -4;
