use crate::parser::ApkReader;
use crate::models::permissions::*;
use crate::utils::permission_db;

pub struct PermissionAnalyzer;

impl super::Analyzer for PermissionAnalyzer {
    type Output = PermissionAnalysis;

    fn name(&self) -> &'static str {
        "permissions"
    }

    fn analyze(&self, apk: &mut ApkReader) -> Result<Self::Output, String> {
        let manifest_data = apk.read_file("AndroidManifest.xml")?;
        let element = crate::parser::axml::decode(&manifest_data)?;

        let mut permissions: Vec<PermissionInfo> = Vec::new();
        let mut summary = PermissionSummary::default();

        for perm_elem in element.find_all("uses-permission") {
            let name = perm_elem.get_attr("android:name")
                .or_else(|| perm_elem.get_attr("name"))
                .unwrap_or_default();
            if name.is_empty() {
                continue;
            }

            let db_entry = permission_db::lookup(&name);
            summary.total += 1;

            match db_entry.protection_level.as_str() {
                "normal" => summary.normal += 1,
                "dangerous" => summary.dangerous += 1,
                "signature" => summary.signature += 1,
                "special" => summary.special += 1,
                _ => summary.unknown += 1,
            }

            permissions.push(PermissionInfo {
                name,
                protection_level: db_entry.protection_level,
                description: db_entry.description,
                risk_level: db_entry.risk_level,
                recommended_usage: db_entry.recommended_usage,
                category: db_entry.category,
            });
        }

        // Sort: dangerous first, then special, signature, normal, unknown
        permissions.sort_by(|a, b| {
            let order = |level: &str| -> u8 {
                match level {
                    "dangerous" => 0,
                    "special" => 1,
                    "signature" => 2,
                    "normal" => 3,
                    _ => 4,
                }
            };
            order(&a.protection_level).cmp(&order(&b.protection_level))
        });

        Ok(PermissionAnalysis { permissions, summary })
    }
}
