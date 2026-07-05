use serde::{Deserialize, Serialize};

/// 插件能力。插件可声明多种能力（如同时是 analyzer + ui）。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    Analyzer,
    Export,
    Command,
    Ui,
}

/// 分析器在分析流程中的执行位置。
/// 决定插件何时运行以及能查询到哪些内置分析器结果。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AnalyzerStage {
    /// 在 overview 之后运行（仅能查询 overview）
    AfterOverview,
    /// 在 manifest 之后运行（可查询 overview + manifest）
    AfterManifest,
    /// 在 security 之后运行（可查询所有内置分析器结果）
    AfterSecurity,
    /// 所有内置分析器之后运行（含 ai_summary）
    Final,
}

/// UI 标签页定义。插件声明它在侧边栏的入口。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiTabDef {
    /// 标签页 ID，唯一，如 "cert_deep"
    pub id: String,
    /// 显示名，如 "Cert Deep"
    pub label: String,
    /// 图标名（lucide-react 图标名），如 "ShieldCheck"
    pub icon: String,
    /// 在侧边栏的位置（数字越小越靠前，内置页面在 1-10）
    pub order: u32,
}

/// 插件元数据。同时存在于磁盘 manifest.json 和插件 vtable.metadata()。
/// 磁盘 manifest.json 优先（便于在不重新编译插件的情况下禁用能力）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    /// 唯一 ID，反向域名格式，如 "com.example.certdeep"
    pub id: String,
    /// 显示名
    pub name: String,
    /// 语义化版本号
    pub version: String,
    pub author: String,
    pub description: String,
    /// 声明的能力列表
    pub capabilities: Vec<Capability>,
    /// 分析器执行位置（仅有 Capability::Analyzer 时有效）
    #[serde(default)]
    pub analyzer_stage: Option<AnalyzerStage>,
    /// UI 标签页定义（仅有 Capability::Ui 时有效）
    #[serde(default)]
    pub ui_tab: Option<UiTabDef>,
}

impl Default for PluginManifest {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            version: "0.0.0".to_string(),
            author: String::new(),
            description: String::new(),
            capabilities: Vec::new(),
            analyzer_stage: None,
            ui_tab: None,
        }
    }
}
