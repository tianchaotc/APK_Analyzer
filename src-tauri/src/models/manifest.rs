use serde::{Deserialize, Serialize};

/// Parsed AndroidManifest.xml content
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ManifestInfo {
    pub package: String,
    pub version_code: String,
    pub version_name: String,
    pub min_sdk: u32,
    pub target_sdk: u32,
    pub compile_sdk: u32,
    pub debuggable: bool,
    pub allow_backup: bool,
    pub extract_native_libs: bool,
    pub uses_cleartext_traffic: bool,
    pub instant_app: bool,
    pub activities: Vec<Component>,
    pub services: Vec<Component>,
    pub receivers: Vec<Component>,
    pub providers: Vec<Component>,
    pub permissions_declared: Vec<String>,
    pub uses_features: Vec<Feature>,
    pub queries: Vec<Query>,
    pub meta_data: Vec<MetaData>,
    pub launch_activity: Option<String>,
    pub raw_xml: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Component {
    pub name: String,
    pub exported: bool,
    pub enabled: bool,
    pub permission: Option<String>,
    pub process: Option<String>,
    pub intent_filters: Vec<IntentFilter>,
    pub meta_data: Vec<MetaData>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IntentFilter {
    pub actions: Vec<String>,
    pub categories: Vec<String>,
    pub data_schemes: Vec<String>,
    pub data_hosts: Vec<String>,
    pub data_paths: Vec<String>,
    pub data_mime_types: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Feature {
    pub name: String,
    pub required: bool,
    pub version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Query {
    pub package: Option<String>,
    pub intent: Option<IntentFilter>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MetaData {
    pub name: String,
    pub value: Option<String>,
    pub resource: Option<String>,
}
