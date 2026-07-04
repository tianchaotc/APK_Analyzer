use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DexAnalysis {
    pub dex_files: Vec<DexFile>,
    pub summary: DexSummary,
    pub packages: Vec<PackageInfo>,
    pub largest_packages: Vec<PackageInfo>,
    pub largest_classes: Vec<ClassInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DexFile {
    pub name: String,
    pub size: u64,
    pub class_count: usize,
    pub method_count: usize,
    pub field_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DexSummary {
    pub total_dex_files: usize,
    pub total_classes: usize,
    pub total_methods: usize,
    pub total_fields: usize,
    pub total_size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PackageInfo {
    pub name: String,
    pub class_count: usize,
    pub method_count: usize,
    pub field_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClassInfo {
    pub name: String,
    pub method_count: usize,
    pub field_count: usize,
    pub dex_file: String,
}
