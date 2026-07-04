use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComponentAnalysis {
    pub stats: ComponentStats,
    pub activities: Vec<super::manifest::Component>,
    pub services: Vec<super::manifest::Component>,
    pub receivers: Vec<super::manifest::Component>,
    pub providers: Vec<super::manifest::Component>,
    pub exported_components: Vec<ExportedComponent>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComponentStats {
    pub activities: usize,
    pub services: usize,
    pub receivers: usize,
    pub providers: usize,
    pub exported: usize,
    pub with_intent_filters: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExportedComponent {
    pub name: String,
    pub component_type: String,
    pub permission: Option<String>,
    pub has_intent_filter: bool,
}
