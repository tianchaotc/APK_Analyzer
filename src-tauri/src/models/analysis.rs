use serde::{Deserialize, Serialize};

/// Top-level APK analysis result containing all analyzer outputs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApkAnalysis {
    pub file_path: String,
    pub file_name: String,
    pub file_size: u64,
    pub analyzed_at: String,
    pub overview: super::overview::OverviewInfo,
    pub manifest: super::manifest::ManifestInfo,
    pub permissions: super::permissions::PermissionAnalysis,
    pub components: super::components::ComponentAnalysis,
    pub resources: super::resources::ResourceAnalysis,
    pub native_libs: super::native_libs::NativeLibAnalysis,
    pub dex: super::dex::DexAnalysis,
    pub certificate: super::certificate::CertificateAnalysis,
    pub security: super::security::SecurityAnalysis,
    pub ai_summary: Option<super::ai_summary::AISummary>,
    /// 插件分析结果（按加载顺序）
    #[serde(default)]
    pub plugins: Vec<PluginResult>,
}

/// 单个插件的分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginResult {
    pub plugin_id: String,
    pub plugin_name: String,
    /// 插件返回的任意 JSON 数据，由 ui_schema 描述如何渲染
    pub data: serde_json::Value,
    /// UI schema JSON（声明式视图描述）
    pub ui_schema: serde_json::Value,
    /// 分析错误（如有），UI 显示错误状态
    pub error: Option<String>,
    /// 分析耗时（毫秒）
    pub duration_ms: u64,
}

/// Progress update sent to frontend during analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressUpdate {
    pub stage: String,
    pub message: String,
    pub percent: u8,
}
