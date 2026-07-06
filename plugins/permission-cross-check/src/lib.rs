// 示例插件：Permission Cross-Check
//
// 演示如何使用 plugin-sdk 编写一个原生分析器插件。
//
// 功能：
//   1. 通过 host.get_analysis("permissions") 查询内置 PermissionsAnalyzer 的结果
//   2. 按风险等级与类别聚合
//   3. 输出声明式 UI schema（stat_grid / chart_bar / table / markdown）
//
// 安装：
//   cargo build --release
//   mkdir -p ~/.apk-analyzer/plugins/permission-cross-check
//   cp manifest.json ~/.apk-analyzer/plugins/permission-cross-check/
//   cp target/release/libperm_xcheck_plugin.dylib ~/.apk-analyzer/plugins/permission-cross-check/plugin.dylib
//   (Linux: .so, Windows: plugin.dll → libperm_xcheck_plugin.dll)

use apk_analyzer_plugin_sdk::{
    export_plugin, Host, HostError, LogLevel, Metadata, Plugin,
};
use serde::Deserialize;
use serde_json::json;

pub struct PermXCheck;

impl PermXCheck {
    pub const fn new() -> Self {
        Self
    }
}

impl Plugin for PermXCheck {
    fn metadata(&self) -> Metadata {
        Metadata {
            id: "com.apkanalyzer.perm-xcheck".to_string(),
            name: "Permission Cross-Check".to_string(),
            version: "0.1.0".to_string(),
            author: "APK Analyzer Sample".to_string(),
            description: "Cross-checks declared permissions by risk level and category, surfacing dangerous grants and unknown protection levels.".to_string(),
        }
    }

    fn analyze(&self, host: &dyn Host, _apk_path: &str) -> Result<serde_json::Value, HostError> {
        host.log(LogLevel::Info, "Permission Cross-Check starting");

        let perms_value = host.get_analysis("permissions").ok_or_else(|| {
            HostError::host("permissions analysis not available; check analyzer_stage")
        })?;

        let perms: PermissionAnalysis = serde_json::from_value(perms_value).map_err(|e| {
            HostError::host(format!("failed to parse permissions analysis: {}", e))
        })?;

        // 聚合：按类别统计
        let mut by_category: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
        for p in &perms.permissions {
            *by_category.entry(p.category.clone()).or_insert(0) += 1;
        }

        // 危险权限清单
        let dangerous: Vec<&PermissionInfo> = perms
            .permissions
            .iter()
            .filter(|p| p.risk_level.eq_ignore_ascii_case("high") || p.risk_level.eq_ignore_ascii_case("critical"))
            .collect();

        // 未知风险等级的权限
        let unknown: Vec<&PermissionInfo> = perms
            .permissions
            .iter()
            .filter(|p| p.risk_level.eq_ignore_ascii_case("unknown") || p.risk_level.is_empty())
            .collect();

        // 生成叙述性摘要
        let summary = build_summary(&perms, &dangerous, &unknown);

        // 构造 chart_bar 数据（按类别计数）
        let category_bars: Vec<serde_json::Value> = by_category
            .iter()
            .map(|(k, v)| json!({ "category": k, "count": v }))
            .collect();

        // 构造 dangerous 表格行
        let dangerous_rows: Vec<serde_json::Value> = dangerous
            .iter()
            .map(|p| {
                json!({
                    "name": p.name,
                    "category": p.category,
                    "risk_level": p.risk_level,
                    "protection_level": p.protection_level,
                    "recommended_usage": p.recommended_usage,
                })
            })
            .collect();

        // 构造 unknown 表格行
        let unknown_rows: Vec<serde_json::Value> = unknown
            .iter()
            .map(|p| {
                json!({
                    "name": p.name,
                    "protection_level": p.protection_level,
                    "category": p.category,
                })
            })
            .collect();

        let data = json!({
            "summary": summary,
            "stats": {
                "total": perms.summary.total,
                "dangerous": perms.summary.dangerous,
                "signature": perms.summary.signature,
                "unknown": perms.summary.unknown,
            },
            "by_category": category_bars,
            "dangerous_permissions": dangerous_rows,
            "unknown_permissions": unknown_rows,
        });

        host.log(
            LogLevel::Info,
            &format!("Permission Cross-Check done: {} total / {} dangerous", perms.summary.total, perms.summary.dangerous),
        );

        Ok(data)
    }

    fn ui_schema(&self) -> serde_json::Value {
        json!({
            "title": "Permission Cross-Check",
            "sections": [
                {
                    "type": "stat_grid",
                    "data_key": "stats",
                    "metrics": [
                        { "key": "total", "label": "Total" },
                        { "key": "dangerous", "label": "Dangerous" },
                        { "key": "signature", "label": "Signature" },
                        { "key": "unknown", "label": "Unknown" }
                    ]
                },
                {
                    "type": "chart_bar",
                    "data_key": "by_category",
                    "x_key": "category",
                    "y_key": "count"
                },
                {
                    "type": "table",
                    "data_key": "dangerous_permissions",
                    "columns": [
                        { "key": "name", "label": "Permission", "width": "30%" },
                        { "key": "category", "label": "Category", "width": "15%" },
                        { "key": "risk_level", "label": "Risk", "width": "10%" },
                        { "key": "protection_level", "label": "Protection", "width": "15%" },
                        { "key": "recommended_usage", "label": "Recommended Usage" }
                    ]
                },
                {
                    "type": "table",
                    "data_key": "unknown_permissions",
                    "columns": [
                        { "key": "name", "label": "Permission", "width": "40%" },
                        { "key": "category", "label": "Category", "width": "20%" },
                        { "key": "protection_level", "label": "Protection Level" }
                    ]
                },
                {
                    "type": "markdown",
                    "data_key": "summary"
                }
            ]
        })
    }
}

fn build_summary(
    perms: &PermissionAnalysis,
    dangerous: &[&PermissionInfo],
    unknown: &[&PermissionInfo],
) -> String {
    let mut lines = Vec::new();
    lines.push(format!("# Permission Cross-Check Report"));
    lines.push(String::new());
    lines.push(format!(
        "This APK declares **{}** permissions, of which **{}** are classified as dangerous and **{}** carry signature-level protection.",
        perms.summary.total, perms.summary.dangerous, perms.summary.signature
    ));
    lines.push(String::new());

    if !dangerous.is_empty() {
        lines.push(format!("## High-Risk Permissions ({})", dangerous.len()));
        for p in dangerous.iter().take(5) {
            lines.push(format!("- **{}** — {} ({})", p.name, p.category, p.risk_level));
        }
        if dangerous.len() > 5 {
            lines.push(format!("- ... and {} more", dangerous.len() - 5));
        }
        lines.push(String::new());
    }

    if !unknown.is_empty() {
        lines.push(format!("## Unknown Risk Permissions ({})", unknown.len()));
        lines.push(format!(
            "These permissions have no documented risk level and may require manual review:"
        ));
        for p in unknown.iter().take(5) {
            lines.push(format!("- {} (protection: {})", p.name, p.protection_level));
        }
        lines.push(String::new());
    }

    // 综合评估
    let risk_score = (perms.summary.dangerous as f64 / perms.summary.total.max(1) as f64) * 100.0;
    lines.push("## Overall Assessment".to_string());
    if risk_score > 50.0 {
        lines.push(format!(
            "⚠️ High risk density: {:.0}% of declared permissions are dangerous. Review whether all are strictly necessary.",
            risk_score
        ));
    } else if risk_score > 20.0 {
        lines.push(format!(
            "⚖️ Moderate risk density: {:.0}% of declared permissions are dangerous. Ensure each has a clear use case.",
            risk_score
        ));
    } else {
        lines.push(format!(
            "✅ Low risk density: {:.0}% of declared permissions are dangerous.",
            risk_score
        ));
    }

    lines.join("\n")
}

// ============ 内置分析模型镜像 ============
//
// 这些类型必须与宿主 models::permissions 中的字段保持一致。
// 这里独立定义避免依赖宿主 crate（保持插件独立性）。

#[derive(Debug, Deserialize)]
struct PermissionAnalysis {
    permissions: Vec<PermissionInfo>,
    summary: PermissionSummary,
}

#[derive(Debug, Deserialize)]
struct PermissionInfo {
    name: String,
    protection_level: String,
    #[serde(default)]
    description: String,
    risk_level: String,
    #[serde(default)]
    recommended_usage: String,
    category: String,
}

#[derive(Debug, Deserialize)]
struct PermissionSummary {
    total: usize,
    dangerous: usize,
    signature: usize,
    #[serde(default)]
    unknown: usize,
}

export_plugin!(PermXCheck);
