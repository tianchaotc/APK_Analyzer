use crate::parser::ApkReader;
use crate::models::native_libs::*;
use std::collections::HashMap;

pub struct NativeLibAnalyzer;

impl super::Analyzer for NativeLibAnalyzer {
    type Output = NativeLibAnalysis;

    fn name(&self) -> &'static str {
        "native_libs"
    }

    fn analyze(&self, apk: &mut ApkReader) -> Result<Self::Output, String> {
        let entries = apk.entries();
        let mut libraries: Vec<NativeLib> = Vec::new();
        let mut by_abi_map: HashMap<String, Vec<NativeLib>> = HashMap::new();

        for (path, size, compressed) in &entries {
            if !path.starts_with("lib/") || !path.ends_with(".so") {
                continue;
            }

            // Extract ABI from path: lib/<abi>/<filename>.so
            let parts: Vec<&str> = path.splitn(3, '/').collect();
            if parts.len() < 3 {
                continue;
            }
            let abi = parts[1].to_string();
            let file_name = parts[2].to_string();

            let architecture = abi_to_architecture(&abi);
            let compression = if *compressed < *size && *size > 0 {
                format!("{:.0}%", (*compressed as f64 / *size as f64) * 100.0)
            } else {
                "none".to_string()
            };

            let lib = NativeLib {
                file_name: file_name.clone(),
                path: path.clone(),
                abi: abi.clone(),
                architecture,
                size: *size,
                compressed_size: *compressed,
                compression,
                export_symbols: Vec::new(), // Would need ELF parsing for symbols
            };

            by_abi_map.entry(abi).or_default().push(lib.clone());
            libraries.push(lib);
        }

        // Sort libraries by size
        libraries.sort_by(|a, b| b.size.cmp(&a.size));

        // Build ABI groups
        let mut by_abi: Vec<AbiGroup> = by_abi_map.into_iter()
            .map(|(abi, mut libs)| {
                libs.sort_by(|a, b| b.size.cmp(&a.size));
                let total_size: u64 = libs.iter().map(|l| l.size).sum();
                AbiGroup {
                    abi: abi.clone(),
                    count: libs.len(),
                    total_size,
                    libraries: libs,
                }
            })
            .collect();
        by_abi.sort_by(|a, b| b.total_size.cmp(&a.total_size));

        let total_size: u64 = libraries.iter().map(|l| l.size).sum();
        let lib_count = libraries.len();
        let abis: Vec<String> = by_abi.iter().map(|g| g.abi.clone()).collect();

        Ok(NativeLibAnalysis {
            libraries,
            by_abi,
            summary: NativeLibSummary {
                total: lib_count,
                total_size,
                abis,
            },
        })
    }
}

fn abi_to_architecture(abi: &str) -> String {
    match abi {
        "armeabi-v7a" => "ARMv7 (32-bit)".to_string(),
        "arm64-v8a" => "ARMv8 (64-bit, AArch64)".to_string(),
        "x86" => "x86 (32-bit)".to_string(),
        "x86_64" => "x86-64 (64-bit)".to_string(),
        "armeabi" => "ARMv5/ARMv6 (legacy)".to_string(),
        "mips" => "MIPS (deprecated)".to_string(),
        "mips64" => "MIPS64 (deprecated)".to_string(),
        "riscv64" => "RISC-V (64-bit)".to_string(),
        _ => format!("Unknown ({})", abi),
    }
}
