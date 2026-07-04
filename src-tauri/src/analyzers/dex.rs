use crate::parser::ApkReader;
use crate::parser::dex::DexParser;
use crate::models::dex::*;
use std::collections::HashMap;

pub struct DexAnalyzer;

impl super::Analyzer for DexAnalyzer {
    type Output = DexAnalysis;

    fn name(&self) -> &'static str {
        "dex"
    }

    fn analyze(&self, apk: &mut ApkReader) -> Result<Self::Output, String> {
        let file_names = apk.file_names();
        let dex_names: Vec<String> = file_names.into_iter()
            .filter(|f| f.starts_with("classes") && f.ends_with(".dex"))
            .collect();

        let mut dex_files: Vec<DexFile> = Vec::new();
        let mut all_packages: HashMap<String, PackageInfo> = HashMap::new();
        let mut total_classes = 0;
        let mut total_methods = 0;
        let mut total_fields = 0;
        let mut total_size = 0u64;

        for dex_name in &dex_names {
            if let Ok(data) = apk.read_file(dex_name) {
                if let Ok(stats) = DexParser::parse(&data) {
                    let dex_file = DexFile {
                        name: dex_name.clone(),
                        size: stats.file_size,
                        class_count: stats.class_count,
                        method_count: stats.method_count,
                        field_count: stats.field_count,
                    };

                    total_classes += stats.class_count;
                    total_methods += stats.method_count;
                    total_fields += stats.field_count;
                    total_size += stats.file_size;

                    // Merge packages
                    for (pkg_name, pkg_info) in stats.packages {
                        let entry = all_packages.entry(pkg_name).or_insert_with(|| PackageInfo {
                            name: String::new(),
                            class_count: 0,
                            method_count: 0,
                            field_count: 0,
                        });
                        entry.name = pkg_info.name;
                        entry.class_count += pkg_info.class_count;
                        entry.method_count += pkg_info.method_count;
                        entry.field_count += pkg_info.field_count;
                    }

                    dex_files.push(dex_file);
                }
            }
        }

        // Convert packages to sorted vector
        let mut packages: Vec<PackageInfo> = all_packages.into_values().collect();
        packages.sort_by(|a, b| b.class_count.cmp(&a.class_count));

        let largest_packages: Vec<PackageInfo> = packages.iter()
            .take(20)
            .cloned()
            .collect();

        Ok(DexAnalysis {
            dex_files,
            summary: DexSummary {
                total_dex_files: dex_names.len(),
                total_classes,
                total_methods,
                total_fields,
                total_size,
            },
            packages,
            largest_packages,
            largest_classes: Vec::new(), // Would need deeper DEX parsing
        })
    }
}
