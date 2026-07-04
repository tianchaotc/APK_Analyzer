use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SecurityAnalysis {
    pub score: u32,
    pub issues: Vec<SecurityIssue>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SecurityIssue {
    pub severity: String,
    pub category: String,
    pub title: String,
    pub description: String,
    pub recommendation: String,
}
