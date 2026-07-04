use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AISummary {
    pub overview: String,
    pub app_type: String,
    pub tech_stack: Vec<TechStackEntry>,
    pub architecture_guess: String,
    pub potential_risks: Vec<String>,
    pub performance_suggestions: Vec<String>,
    pub packaging_suggestions: Vec<String>,
    pub permission_review: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TechStackEntry {
    pub name: String,
    pub confidence: String,
    pub evidence: Vec<String>,
}
