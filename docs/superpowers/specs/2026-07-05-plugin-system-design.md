# Plugin System Design — APK Analyzer

**Date:** 2026-07-05
**Status:** Spec — awaiting user review
**Scope:** Single implementation plan

---

## 1. Goals & Non-Goals

### Goals
- 让第三方（或内置）扩展能向 APK Analyzer 添加**新的分析器**、**新的导出格式**、**新的命令动作**、**新的 UI 分析页**。
- 插件以**原生动态库**（`.dylib`/`.dll`/`.so`）形式分发，性能与内置分析器等价。
- 插件可**复用宿主已有的解析能力**（AXML、DEX、resources.arsc、签名块、证书），避免重新实现。
- 插件可**查询其他分析器的结果**（如插件做"权限 + 安全联合分析"）。
- 提供 `plugin-sdk` crate，让插件作者用纯 Rust trait 写插件，无需手写 C ABI。
- 提供 UI 管理面板，可启用/禁用插件、查看元数据、查看加载错误。

### Non-Goals
- 不支持非 Rust 语言插件（C ABI 虽语言中立，但 SDK 仅提供 Rust 封装）。
- 不实现插件市场/自动更新（本期仅本地扫描 + 手动安装）。
- 不支持插件间的依赖关系（每个插件独立加载）。
- 不支持插件运行时热卸载（卸载需重启应用）。
- 不支持插件执行任意 JS（UI 用声明式 schema 渲染，无 JS 执行面）。

---

## 2. Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                        Host Process                          │
│  ┌────────────────────┐    ┌──────────────────────────────┐ │
│  │  Rust Core (src-tauri) │  │  React Frontend              │ │
│  │                       │  │                              │ │
│  │  ┌─────────────────┐  │  │  ┌────────────────────────┐ │ │
│  │  │ Built-in        │  │  │  │ Built-in pages         │ │ │
│  │  │ analyzers (10)  │  │  │  │ (Overview, ...)        │ │ │
│  │  └─────────────────┘  │  │  └────────────────────────┘ │ │
│  │  ┌─────────────────┐  │  │  ┌────────────────────────┐ │ │
│  │  │ PluginManager   │──┼──┼─▶│ PluginPage (dynamic)   │ │ │
│  │  │  - discover     │  │  │  │  - schema renderer     │ │ │
│  │  │  - load .dylib  │  │  │  │  - management panel    │ │ │
│  │  │  - register     │  │  │  └────────────────────────┘ │ │
│  │  └─────────────────┘  │  └──────────────────────────────┘ │
│  │  ┌─────────────────┐  │                                   │
│  │  │ HostApi vtable  │  │                                   │
│  │  │  - read_file    │  │                                   │
│  │  │  - parse_axml   │  │                                   │
│  │  │  - get_analysis │  │                                   │
│  │  │  - log          │  │                                   │
│  │  └─────────────────┘  │                                   │
│  └────────────────────┘                                   │
└─────────────────────────────────────────────────────────────┘
        │ libloading
        ▼
┌─────────────────────────────────────────────────────────────┐
│  Plugin .dylib (loaded into host process)                   │
│  ┌────────────────────┐  ┌────────────────────────────────┐ │
│  │ plugin_analyze()   │  │ plugin_ui_schema()             │ │
│  │ plugin_metadata()  │  │ plugin_export() (optional)     │ │
│  │ plugin_command()   │  │ plugin_free()                  │ │
│  └────────────────────┘  └────────────────────────────────┘ │
│  调用 HostApi vtable 复用宿主能力                              │
└─────────────────────────────────────────────────────────────┘
```

**关键边界：**
- 插件运行在**宿主进程内**（非独立进程），通过 `libloading` 加载。
- 所有跨边界数据用 **C ABI + `#[repr(C)]`** 传递，避免 Rust vtable 不稳定。
- 字符串/字节数组用**显式长度**的双指针 (`*const c_char, usize`)，避免依赖 null terminator。
- 所有堆分配的返回值由**插件侧负责释放**（`plugin_free`），宿主不跨边界 free。

---

## 3. Component Design

### 3.1 ABI Surface (`src-tauri/src/plugin/abi.rs`)

定义两套 vtable：宿主提供给插件的 `HostApi`，插件导出的 `PluginVTable`。

```rust
use std::os::raw::{c_char, c_int, c_void};

/// 宿主提供给插件的能力（vtable，函数指针表）
#[repr(C)]
pub struct HostApi {
    pub abi_version: u32,
    pub ctx: *const c_void,  // 宿主内部上下文，插件不透明

    // —— 文件与 APK 访问 ——
    /// 读取 APK 内的文件，返回堆分配的字节缓冲区
    pub read_apk_file: extern "C" fn(ctx: *const c_void, path: *const c_char, path_len: usize,
                                     out: *mut *mut u8, out_len: *mut usize) -> c_int,
    /// 列出 APK 内所有文件名（返回 JSON 数组字符串，需 plugin_free）
    pub list_apk_files: extern "C" fn(ctx: *const c_void,
                                      out: *mut *mut c_char, out_len: *mut usize) -> c_int,

    // —— 复用宿主解析器 ——
    /// 解析二进制 AndroidManifest.xml，返回 JSON（AxmlElement 树序列化）
    pub parse_axml: extern "C" fn(ctx: *const c_void, bytes: *const u8, len: usize,
                                  out: *mut *mut c_char, out_len: *mut usize) -> c_int,
    /// 解析 DEX 文件，返回 JSON（class/method/field 统计 + 包层级）
    pub parse_dex: extern "C" fn(ctx: *const c_void, bytes: *const u8, len: usize,
                                 out: *mut *mut c_char, out_len: *mut usize) -> c_int,

    // —— 查询其他分析器结果 ——
    /// 按 key 查询已完成的内置分析器结果（"overview"/"manifest"/"permissions"/...）
    /// 返回 JSON 字符串，找不到返回空
    pub get_analysis: extern "C" fn(ctx: *const c_void, key: *const c_char, key_len: usize,
                                    out: *mut *mut c_char, out_len: *mut usize) -> c_int,

    // —— 日志 ——
    /// level: 0=trace 1=debug 2=info 3=warn 4=error
    pub log: extern "C" fn(ctx: *const c_void, level: c_int, msg: *const c_char, msg_len: usize),
}

/// 插件导出的函数指针表
#[repr(C)]
pub struct PluginVTable {
    pub abi_version: u32,

    /// 返回 manifest JSON（元数据：name, version, author, description, capabilities）
    pub metadata: extern "C" fn() -> *const c_char,  // 静态字符串，无需 free

    /// 执行分析。返回 JSON 结果（schema + data），需 plugin_free
    /// host: HostApi 指针；apk_path: APK 路径
    pub analyze: extern "C" fn(host: *const HostApi, apk_path: *const c_char, apk_path_len: usize,
                               out: *mut *mut c_char, out_len: *mut usize) -> c_int,

    /// 返回 UI schema JSON（声明式视图描述），需 plugin_free
    pub ui_schema: extern "C" fn(out: *mut *mut c_char, out_len: *mut usize) -> c_int,

    /// 可选：导出报告。fmt: "pdf"/"sarif"/"custom"；返回字节缓冲区
    pub export: Option<extern "C" fn(host: *const HostApi, fmt: *const c_char, fmt_len: usize,
                                     data: *const c_char, data_len: usize,
                                     out: *mut *mut u8, out_len: *mut usize) -> c_int>,

    /// 可选：执行命令动作。cmd: 命令名；args: JSON 参数；返回 JSON 结果
    pub command: Option<extern "C" fn(host: *const HostApi, cmd: *const c_char, cmd_len: usize,
                                      args: *const c_char, args_len: usize,
                                      out: *mut *mut c_char, out_len: *mut usize) -> c_int>,

    /// 释放插件分配的缓冲区（字符串或字节）
    pub free: extern "C" fn(ptr: *mut c_void, len: usize),
}

/// 插件必须导出的入口符号名
pub const ENTRY_SYMBOL: &str = "apk_analyzer_plugin_vtable";

/// 当前 ABI 版本，不匹配则拒绝加载
pub const ABI_VERSION: u32 = 1;
```

**返回码约定：** `0 = 成功`，`-1 = 通用错误`，`-2 = 参数无效`，`-3 = 内存分配失败`。错误详情通过 `log` 写出。

### 3.2 Plugin Manager (`src-tauri/src/plugin/manager.rs`)

负责发现、加载、注册、生命周期管理。

```rust
pub struct PluginManager {
    plugins: Vec<LoadedPlugin>,
    enabled: std::collections::HashMap<String, bool>,  // 持久化到 config
}

struct LoadedPlugin {
    manifest: PluginManifest,        // 解析后的元数据
    lib: libloading::Library,        // 动态库句柄
    vtable: PluginVTable,            // 函数指针表
    enabled: bool,
    load_error: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct PluginManifest {
    pub id: String,                  // 唯一 ID，如 "com.example.certdeep"
    pub name: String,                // 显示名 "Cert Deep Analyzer"
    pub version: String,             // "1.0.0"
    pub author: String,
    pub description: String,
    pub capabilities: Vec<Capability>,  // ["analyzer", "export", "command", "ui"]
    pub analyzer_stage: Option<AnalyzerStage>,  // 在分析流程中的位置
    pub ui_tab: Option<UiTabDef>,    // UI 标签页定义
}

pub enum Capability { Analyzer, Export, Command, Ui }
pub enum AnalyzerStage {
    AfterOverview,    // 在 overview 之后，可读取 overview 结果
    AfterManifest,
    AfterSecurity,    // 在 security 之后，可读取所有内置结果
    Final,            // 所有内置分析器之后
}
```

**加载流程：**
1. 扫描插件目录（macOS: `~/.apk-analyzer/plugins/`，Windows: `%APPDATA%/apk-analyzer/plugins/`）。
2. 每个插件是一个子目录，包含 `plugin.dylib`（或 `.dll`/`.so`）+ `manifest.json`。
3. 用 `libloading::Library::new` 加载动态库。
4. 用 `lib.get(ENTRY_SYMBOL)` 取 `PluginVTable`。
5. 校验 `vtable.abi_version == ABI_VERSION`，不匹配跳过并记录错误。
6. 调用 `vtable.metadata()`，与磁盘上的 `manifest.json` 合并（磁盘优先，便于禁用）。
7. 读用户配置 `plugins.toml`（启用/禁用状态、加载顺序）。
8. 注册到 `PluginManager.plugins`。

**卸载：** 本期不支持热卸载。`PluginManager::drop` 时释放所有 `Library`。

### 3.3 Plugin SDK (`plugin-sdk/`，独立 crate)

把 C ABI 封装成 Rust 友好的 trait，插件作者无需手写 `extern "C"`。

```rust
// plugin-sdk/src/lib.rs
pub use host::{Host, HostError};

/// 插件实现此 trait
pub trait Plugin: Send + Sync {
    fn metadata(&self) -> Metadata;
    fn analyze(&self, host: &dyn Host, apk_path: &str) -> Result<serde_json::Value, HostError>;
    fn ui_schema(&self) -> serde_json::Value;
    fn export(&self, _host: &dyn Host, _fmt: &str, _data: &serde_json::Value)
        -> Result<Vec<u8>, HostError> { Err(HostError::Unsupported) }
    fn command(&self, _host: &dyn Host, _cmd: &str, _args: &serde_json::Value)
        -> Result<serde_json::Value, HostError> { Err(HostError::Unsupported) }
}

/// 宿主能力接口
pub trait Host {
    fn read_apk_file(&self, path: &str) -> Result<Vec<u8>, HostError>;
    fn list_apk_files(&self) -> Result<Vec<String>, HostError>;
    fn parse_axml(&self, bytes: &[u8]) -> Result<serde_json::Value, HostError>;
    fn parse_dex(&self, bytes: &[u8]) -> Result<serde_json::Value, HostError>;
    fn get_analysis(&self, key: &str) -> Option<serde_json::Value>;
    fn log(&self, level: LogLevel, msg: &str);
}

/// 由插件 crate 调用，导出 C ABI 入口
#[macro_export]
macro_rules! export_plugin {
    ($plugin:expr) => {
        #[no_mangle]
        pub extern "C" fn apk_analyzer_plugin_vtable() -> *const $crate::abi::PluginVTable {
            // 构造 vtable，把 trait 方法适配到 C ABI
            // ...
        }
    };
}
```

**插件作者只需：**
```rust
// my-plugin/src/lib.rs
use plugin_sdk::{Plugin, Host, HostError, Metadata, export_plugin};

struct CertDeepAnalyzer;

impl Plugin for CertDeepAnalyzer {
    fn metadata(&self) -> Metadata { /* ... */ }
    fn analyze(&self, host: &dyn Host, apk_path: &str) -> Result<serde_json::Value, HostError> {
        let manifest_bytes = host.read_apk_file("AndroidManifest.xml")?;
        let axml = host.parse_axml(&manifest_bytes)?;
        // 复用宿主解析器，不重新实现
        let overview = host.get_analysis("overview").unwrap();
        // ... 自定义分析逻辑
        Ok(serde_json::json!({ "cert_chain_depth": 3, "issues": [...] }))
    }
    fn ui_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "title": "Cert Deep",
            "sections": [{ "type": "table", "data_key": "issues", "columns": [...] }]
        })
    }
}

export_plugin!(CertDeepAnalyzer);
```

### 3.4 UI Schema Renderer (`src/components/plugin/PluginPage.tsx`)

宿主前端根据插件返回的 schema JSON 动态渲染页面。

```typescript
// 插件返回的 UI schema
interface PluginUiSchema {
  title: string;
  sections: UiSection[];
}
type UiSection =
  | { type: 'table'; data_key: string; columns: { key: string; label: string; width?: number }[] }
  | { type: 'cards'; data_key: string; card: { title_key: string; body_key: string } }
  | { type: 'stat_grid'; data_key: string; metrics: { key: string; label: string; unit?: string }[] }
  | { type: 'markdown'; data_key: string }
  | { type: 'chart_bar'; data_key: string; x_key: string; y_key: string };
```

宿主实现一个 `PluginPage` 组件，接收 schema + data，渲染表格/卡片/统计网格/Markdown/柱状图。复用现有 `DataTable` 组件。

### 3.5 Management Panel (`src/components/plugin/PluginManagerPanel.tsx`)

UI 提供插件管理面板：
- 列出所有已发现的插件（名称、版本、作者、状态、能力）
- 启用/禁用开关
- 查看加载错误
- 打开插件目录按钮
- 重载按钮（重启应用后生效）

状态写入 `~/.apk-analyzer/plugins.toml`。

---

## 4. Data Flow

### 4.1 启动时加载插件

```
main() → PluginManager::new()
  → scan_plugins(plugins_dir)
  → for each plugin dir:
      → load manifest.json
      → libloading::Library::new(dylib_path)
      → lib.get(ENTRY_SYMBOL) → PluginVTable
      → check abi_version
      → read enabled state from plugins.toml
      → push to self.plugins
  → 注册到 AppState
```

### 4.2 分析时调用插件

```
commands::analyze_apk(path)
  → run built-in analyzers (overview, manifest, ...)
  → build HostApi vtable (ctx = &ApkReader + &ApkAnalysis so far)
  → for each enabled plugin with Capability::Analyzer:
      → stage 匹配（如 AfterSecurity 在 security 之后运行）
      → call vtable.analyze(host, apk_path) → JSON
      → parse JSON → 插入到 ApkAnalysis.plugins: HashMap<String, serde_json::Value>
      → emit progress event
  → return ApkAnalysis (含插件结果)
```

### 4.3 前端渲染插件页

```
React App ← ApkAnalysis
  → Sidebar 渲染插件 tab（来自 manifest.ui_tab）
  → PluginPage 接收 plugin result + ui_schema
  → PluginPageRenderer 按 section 类型渲染
```

### 4.4 插件命令调用

```
React → invoke('plugin_command', { plugin_id, cmd, args })
  → commands::plugin_command
  → PluginManager::get(plugin_id)
  → vtable.command(host, cmd, args_json) → JSON
  → return to frontend
```

---

## 5. Data Model Changes

### 5.1 `ApkAnalysis` 新增字段

```rust
// src-tauri/src/models/analysis.rs
pub struct ApkAnalysis {
    // ... 现有字段 ...
    pub plugins: Vec<PluginResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginResult {
    pub plugin_id: String,
    pub plugin_name: String,
    pub data: serde_json::Value,
    pub ui_schema: serde_json::Value,
    pub error: Option<String>,
    pub duration_ms: u64,
}
```

### 5.2 `AppState` 新增字段

```rust
pub struct AppState {
    pub current_analysis: Mutex<Option<models::ApkAnalysis>>,
    pub plugin_manager: Mutex<plugin::PluginManager>,  // 新增
}
```

---

## 6. Error Handling

| 错误场景 | 处理 |
|---------|------|
| 动态库加载失败（缺符号、架构不匹配） | 记录到 `load_error`，插件标记 disabled，不阻塞启动 |
| ABI 版本不匹配 | 同上，错误消息明确提示版本号 |
| `analyze` panic | 用 `std::panic::catch_unwind` 包裹调用，捕获后插件标记 errored，继续其他插件 |
| `analyze` 返回错误码 | 记录错误，`PluginResult.error` 填消息，UI 显示错误状态 |
| 返回的 JSON 解析失败 | 同上 |
| `export`/`command` 不支持 | 返回 `HostError::Unsupported`，UI 提示该插件不支持此操作 |
| 插件目录不存在 | 自动创建，无插件时静默 |
| `plugins.toml` 损坏 | 备份后重置为默认配置 |

**关键安全约束：** 所有跨边界调用都用 `catch_unwind` 包裹，防止插件 panic 导致宿主崩溃。插件 panic 后被禁用，下次启动不加载。

---

## 7. Security Considerations

- **信任模型：** 插件以原生动态库形式运行在宿主进程内，拥有与宿主相同的权限。用户需自行信任插件来源（类似 VS Code 扩展、GIMP 插件）。UI 在启用插件时明确提示。
- **不执行任意 JS：** UI 用声明式 schema 渲染，插件无法注入脚本到 webview。
- **文件访问：** 插件通过 `HostApi.read_apk_file` 访问 APK 内容，不直接拿到文件系统句柄。但插件是原生代码，理论上可绕过——信任模型已涵盖。
- **路径校验：** `apk_path` 由宿主传入，插件不接收用户输入的任意路径。

---

## 8. Directory Layout

```
APK_Analyzer/
├── src-tauri/src/
│   ├── plugin/                    # 新增模块
│   │   ├── mod.rs                 # pub use + 模块导出
│   │   ├── abi.rs                 # HostApi, PluginVTable, ABI_VERSION
│   │   ├── manager.rs             # PluginManager, LoadedPlugin, scan/load
│   │   ├── host.rs                # HostApi 实现（桥接 ApkReader + ApkAnalysis）
│   │   └── manifest.rs            # PluginManifest, Capability, AnalyzerStage
│   └── ...                        # 现有代码
├── plugin-sdk/                    # 新增独立 crate
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs                 # Plugin trait, Host trait, export_plugin! 宏
│       ├── host.rs                # HostError, LogLevel
│       └── abi.rs                 # 与宿主共享的 ABI 类型（re-export）
├── src/
│   ├── components/plugin/         # 新增前端组件
│   │   ├── PluginPage.tsx         # schema 驱动的渲染器
│   │   ├── PluginManagerPanel.tsx # 管理面板
│   │   └── PluginTab.tsx          # 侧边栏 tab 包装
│   ├── pages/
│   │   └── PluginsPage.tsx        # /plugins 路由（管理面板入口）
│   └── ...
└── docs/
    └── superpowers/specs/
        └── 2026-07-05-plugin-system-design.md  # 本文档
```

---

## 9. Implementation Phases

实现计划会由 writing-plans skill 生成详细步骤，这里仅列阶段：

1. **ABI + Manager 骨架** — `plugin/` 模块，定义 ABI 类型，实现 `PluginManager::scan` + `load`，不接前端。
2. **HostApi 实现** — 桥接 `ApkReader` 和 `ApkAnalysis`，提供 `read_apk_file`/`parse_axml`/`get_analysis` 等。
3. **plugin-sdk crate** — `Plugin` trait + `Host` trait + `export_plugin!` 宏 + 示例插件。
4. **集成到 analyze_apk** — `ApkAnalysis.plugins` 字段，按 `AnalyzerStage` 调用插件。
5. **前端 PluginPage** — schema 渲染器（table/cards/stat_grid/markdown/chart_bar）。
6. **管理面板** — 列表/启用禁用/错误显示/打开目录。
7. **示例插件** — 内置一个示例插件（如 "Manifest Permissions Cross-check"）验证全链路。

---

## 10. Testing Strategy

- **ABI 单元测试：** 用 mock 插件 dylib（编译时生成的测试 fixture）验证 `PluginManager` 加载、版本检查、错误处理。
- **HostApi 单元测试：** 用真实 APK 文件测试 `read_apk_file`/`parse_axml`/`get_analysis` 返回正确 JSON。
- **集成测试：** 加载示例插件，跑完整 `analyze_apk` 流程，断言 `plugins` 字段非空且 schema 正确。
- **前端测试：** PluginPage renderer 用固定 schema snapshot 测试。
- **手动验收：** 在 `~/.apk-analyzer/plugins/` 放示例插件，启动应用，验证管理面板和插件页。

---

## 11. Open Questions

无。所有关键决策已在 brainstorming 阶段确认：
- 扩展能力：全栈
- 运行时：原生动态库
- 发现：扫描 + UI 管理
- UI：声明式 schema

---

## 12. Out of Scope (Future Work)

- 插件市场与自动更新
- 非 Rust 语言插件 SDK（C/Go/Zig）
- 插件间依赖声明
- 热卸载/热重载
- 插件沙箱化（如 WASM 后端作为安全模式）
- 插件性能 profiling 工具
