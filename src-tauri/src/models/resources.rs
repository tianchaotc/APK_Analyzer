use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResourceAnalysis {
    pub summary: ResourceSummary,
    pub by_type: Vec<ResourceTypeGroup>,
    pub largest_resources: Vec<ResourceEntry>,
    pub duplicate_resources: Vec<DuplicateResource>,
    pub image_stats: ImageStats,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResourceSummary {
    pub total: usize,
    pub total_size: u64,
    pub types: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResourceTypeGroup {
    pub type_name: String,
    pub count: usize,
    pub total_size: u64,
    pub entries: Vec<ResourceEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResourceEntry {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub resource_type: String,
    pub compression: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DuplicateResource {
    pub name: String,
    pub paths: Vec<String>,
    pub total_size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ImageStats {
    pub total_images: usize,
    pub total_size: u64,
    pub by_format: Vec<FormatStat>,
    pub largest_images: Vec<ResourceEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FormatStat {
    pub format: String,
    pub count: usize,
    pub total_size: u64,
}
