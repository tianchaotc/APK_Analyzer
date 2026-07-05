// Plugin system: native dynamic library based extension mechanism.
//
// 插件以 .dylib/.dll/.so 形式分发，通过 libloading 加载到宿主进程。
// 跨边界用 C ABI + #[repr(C)] vtable，避免 Rust vtable 不稳定。

pub mod abi;
pub mod host;
pub mod manager;
pub mod manifest;

pub use abi::{HostApi, PluginVTable, ABI_VERSION, ENTRY_SYMBOL};
pub use manager::{LoadedPlugin, PluginManager};
pub use manifest::{AnalyzerStage, Capability, PluginManifest};
