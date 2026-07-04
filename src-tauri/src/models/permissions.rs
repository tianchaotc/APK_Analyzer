use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PermissionAnalysis {
    pub permissions: Vec<PermissionInfo>,
    pub summary: PermissionSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PermissionInfo {
    pub name: String,
    pub protection_level: String,
    pub description: String,
    pub risk_level: String,
    pub recommended_usage: String,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PermissionSummary {
    pub total: usize,
    pub normal: usize,
    pub dangerous: usize,
    pub signature: usize,
    pub special: usize,
    pub unknown: usize,
}
