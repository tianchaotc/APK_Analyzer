use crate::parser::ApkReader;
use crate::models::resources::*;
use std::collections::HashMap;

pub struct ResourceAnalyzer;

impl super::Analyzer for ResourceAnalyzer {
    type Output = ResourceAnalysis;

    fn name(&self) -> &'static str {
        "resources"
    }

    fn analyze(&self, apk: &mut ApkReader) -> Result<Self::Output, String> {
        let entries = apk.entries();
        let mut by_type: HashMap<String, Vec<ResourceEntry>> = HashMap::new();
        let mut total_size: u64 = 0;
        let mut total_count: usize = 0;

        let resource_prefixes = [
            "drawable", "layout", "xml", "font", "anim", "raw",
            "values", "mipmap", "color", "menu",
        ];

        let image_extensions = ["png", "jpg", "jpeg", "gif", "webp", "bmp", "svg"];

        for (path, size, compressed) in &entries {
            if !path.starts_with("res/") {
                continue;
            }

            let resource_type = classify_resource(path, &resource_prefixes);
            let compression = if *compressed < *size && *size > 0 {
                format!("{:.0}%", (*compressed as f64 / *size as f64) * 100.0)
            } else {
                "none".to_string()
            };

            let entry = ResourceEntry {
                name: path.rsplit('/').next().unwrap_or(path).to_string(),
                path: path.clone(),
                size: *size,
                resource_type: resource_type.clone(),
                compression: Some(compression),
            };

            by_type.entry(resource_type.clone())
                .or_default()
                .push(entry);

            total_size += size;
            total_count += 1;
        }

        // Build type groups
        let mut type_groups: Vec<ResourceTypeGroup> = by_type.into_iter()
            .map(|(type_name, mut entries)| {
                entries.sort_by(|a, b| b.size.cmp(&a.size));
                let total: u64 = entries.iter().map(|e| e.size).sum();
                ResourceTypeGroup {
                    type_name,
                    count: entries.len(),
                    total_size: total,
                    entries,
                }
            })
            .collect();
        type_groups.sort_by(|a, b| b.total_size.cmp(&a.total_size));

        // Largest resources (top 50)
        let mut all_resources: Vec<ResourceEntry> = type_groups.iter()
            .flat_map(|g| g.entries.iter().cloned())
            .collect();
        all_resources.sort_by(|a, b| b.size.cmp(&a.size));
        let largest_resources: Vec<ResourceEntry> = all_resources.iter()
            .take(50)
            .cloned()
            .collect();

        // Find duplicates (same file name, different configs)
        let duplicate_resources = find_duplicates(&all_resources);

        // Image stats
        let image_stats = build_image_stats(&entries, &image_extensions);

        Ok(ResourceAnalysis {
            summary: ResourceSummary {
                total: total_count,
                total_size,
                types: type_groups.len(),
            },
            by_type: type_groups,
            largest_resources,
            duplicate_resources,
            image_stats,
        })
    }
}

fn classify_resource(path: &str, prefixes: &[&str]) -> String {
    let trimmed = path.strip_prefix("res/").unwrap_or(path);
    let first_segment = trimmed.split('/').next().unwrap_or(trimmed);

    for prefix in prefixes {
        if first_segment == *prefix || first_segment.starts_with(&format!("{}-", prefix)) {
            return prefix.to_string();
        }
    }
    "other".to_string()
}

fn find_duplicates(resources: &[ResourceEntry]) -> Vec<DuplicateResource> {
    let mut name_map: HashMap<String, Vec<&ResourceEntry>> = HashMap::new();

    for r in resources {
        name_map.entry(r.name.clone()).or_default().push(r);
    }

    let mut duplicates: Vec<DuplicateResource> = name_map.into_iter()
        .filter(|(_, entries)| entries.len() > 1)
        .map(|(name, entries)| {
            let total_size: u64 = entries.iter().map(|e| e.size).sum();
            DuplicateResource {
                name,
                paths: entries.iter().map(|e| e.path.clone()).collect(),
                total_size,
            }
        })
        .collect();

    duplicates.sort_by(|a, b| b.total_size.cmp(&a.total_size));
    duplicates.into_iter().take(50).collect()
}

fn build_image_stats(entries: &[(String, u64, u64)], image_exts: &[&str]) -> ImageStats {
    let mut by_format: HashMap<String, (usize, u64)> = HashMap::new();
    let mut image_entries: Vec<ResourceEntry> = Vec::new();

    for (path, size, _compressed) in entries {
        if !path.starts_with("res/") {
            continue;
        }
        let ext = path.rsplit('.').next().unwrap_or("").to_lowercase();
        if !image_exts.contains(&ext.as_str()) {
            continue;
        }

        let format = ext.to_uppercase();
        let (count, total) = by_format.entry(format).or_insert((0usize, 0u64));
        *count += 1;
        *total += size;

        image_entries.push(ResourceEntry {
            name: path.rsplit('/').next().unwrap_or(path).to_string(),
            path: path.clone(),
            size: *size,
            resource_type: "image".to_string(),
            compression: None,
        });
    }

    image_entries.sort_by(|a, b| b.size.cmp(&a.size));
    let total_images: usize = by_format.values().map(|(c, _)| c).sum();
    let total_size: u64 = by_format.values().map(|(_, s)| s).sum();

    let mut by_format_vec: Vec<FormatStat> = by_format.into_iter()
        .map(|(format, (count, total_size))| FormatStat { format, count, total_size })
        .collect();
    by_format_vec.sort_by(|a, b| b.total_size.cmp(&a.total_size));

    ImageStats {
        total_images,
        total_size,
        by_format: by_format_vec,
        largest_images: image_entries.into_iter().take(20).collect(),
    }
}
