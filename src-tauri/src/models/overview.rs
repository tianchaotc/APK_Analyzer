use serde::{Deserialize, Serialize};

/// Overview information shown on the Overview page
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OverviewInfo {
    pub app_name: String,
    pub package_name: String,
    pub version_name: String,
    pub version_code: String,
    pub min_sdk: String,
    pub target_sdk: String,
    pub compile_sdk: String,
    pub apk_size: u64,
    pub estimated_install_size: u64,
    pub abis: Vec<String>,
    pub languages: Vec<String>,
    pub densities: Vec<String>,
    pub signature_version: String,
    pub debuggable: bool,
    pub allow_backup: bool,
    pub extract_native_libs: bool,
    pub uses_cleartext_traffic: bool,
    pub instant_app: bool,
    pub split_apk: bool,
    pub bundle_info: Option<String>,
    pub app_icon_base64: Option<String>,
}
