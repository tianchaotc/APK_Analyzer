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
}

/// Progress update sent to frontend during analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressUpdate {
    pub stage: String,
    pub message: String,
    pub percent: u8,
}
