use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NativeLibAnalysis {
    pub libraries: Vec<NativeLib>,
    pub by_abi: Vec<AbiGroup>,
    pub summary: NativeLibSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NativeLib {
    pub file_name: String,
    pub path: String,
    pub abi: String,
    pub architecture: String,
    pub size: u64,
    pub compressed_size: u64,
    pub compression: String,
    pub export_symbols: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AbiGroup {
    pub abi: String,
    pub count: usize,
    pub total_size: u64,
    pub libraries: Vec<NativeLib>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NativeLibSummary {
    pub total: usize,
    pub total_size: u64,
    pub abis: Vec<String>,
}
