use crate::models::dex::*;
use crate::parser::dex::DexParser;
use crate::parser::ApkReader;
use std::collections::HashMap;

pub struct DexAnalyzer;

impl super::Analyzer for DexAnalyzer {
    type Output = DexAnalysis;

    fn name(&self) -> &'static str {
        "dex"
    }

    fn analyze(&self, apk: &mut ApkReader) -> Result<Self::Output, String> {
        let file_names = apk.file_names();
        let dex_names: Vec<String> = file_names
            .into_iter()
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

        let largest_packages: Vec<PackageInfo> = packages.iter().take(20).cloned().collect();

        let total_dex_files = dex_files.len();

        Ok(DexAnalysis {
            dex_files,
            summary: DexSummary {
                total_dex_files,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analyzers::Analyzer;
    use crate::parser::ApkReader;
    use std::fs::File;
    use std::io::Write;
    use zip::write::SimpleFileOptions;

    fn minimal_dex() -> Vec<u8> {
        let mut data = vec![0u8; 112];
        data[0..8].copy_from_slice(b"dex\n035\0");
        data[32..36].copy_from_slice(&(112u32).to_le_bytes());
        data[36..40].copy_from_slice(&(112u32).to_le_bytes());
        data
    }

    #[test]
    fn analyze_counts_only_dex_files_that_parse_successfully() {
        let path =
            std::env::temp_dir().join(format!("apk-analyzer-dex-test-{}.apk", std::process::id()));
        let file = File::create(&path).expect("test APK should be created");
        let mut zip = zip::ZipWriter::new(file);
        let options = SimpleFileOptions::default();
        zip.start_file("classes.dex", options)
            .expect("valid dex entry should start");
        zip.write_all(&minimal_dex())
            .expect("valid dex entry should write");
        zip.start_file("classes2.dex", options)
            .expect("invalid dex entry should start");
        zip.write_all(b"not a dex")
            .expect("invalid dex entry should write");
        zip.finish().expect("test APK should finish");

        let mut apk = ApkReader::open(&path.to_string_lossy()).expect("test APK should open");

        let analysis = DexAnalyzer
            .analyze(&mut apk)
            .expect("valid dex entry should analyze");

        assert_eq!(analysis.summary.total_dex_files, 1);
        assert_eq!(analysis.dex_files.len(), 1);

        std::fs::remove_file(path).expect("test APK should be removed");
    }
}
