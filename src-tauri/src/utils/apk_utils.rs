use crate::parser::axml;
use crate::parser::ApkReader;

/// Get app name from manifest label or resource lookup
pub fn get_app_name(_apk: &mut ApkReader, element: &axml::AxmlElement) -> String {
    let app = match element.find("application") {
        Some(a) => a,
        None => return element.get_attr("package").unwrap_or_default(),
    };

    let label = app
        .get_attr("android:label")
        .or_else(|| app.get_attr("label"))
        .unwrap_or_default();

    // If it's a resource reference (@string/app_name), try to resolve from resources.arsc
    // For now, if it starts with @, we can't easily resolve without full arsc parsing
    // Return the package name as fallback
    if label.is_empty() || label.starts_with('@') {
        element
            .get_attr("package")
            .unwrap_or_else(|| "Unknown".to_string())
    } else {
        label
    }
}

/// Get SDK versions from manifest
pub fn get_sdk_versions(element: &axml::AxmlElement) -> (u32, u32, u32) {
    let mut min_sdk = 0u32;
    let mut target_sdk = 0u32;
    let mut compile_sdk = 0u32;

    if let Some(uses_sdk) = element.find("uses-sdk") {
        min_sdk = uses_sdk
            .get_attr("android:minSdk")
            .or_else(|| uses_sdk.get_attr("minSdk"))
            .and_then(|v| v.parse().ok())
            .unwrap_or(1);
        target_sdk = uses_sdk
            .get_attr("android:targetSdk")
            .or_else(|| uses_sdk.get_attr("targetSdk"))
            .and_then(|v| v.parse().ok())
            .unwrap_or(min_sdk);
        compile_sdk = uses_sdk
            .get_attr("android:compileSdk")
            .or_else(|| uses_sdk.get_attr("compileSdk"))
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);
    }

    (min_sdk, target_sdk, compile_sdk)
}

/// Collect ABIs from lib/ directories
pub fn collect_abis(file_names: &[String]) -> Vec<String> {
    let mut abis: Vec<String> = file_names
        .iter()
        .filter(|f| f.starts_with("lib/"))
        .filter_map(|f| {
            let parts: Vec<&str> = f.splitn(3, '/').collect();
            if parts.len() >= 2 {
                Some(parts[1].to_string())
            } else {
                None
            }
        })
        .collect();
    abis.sort();
    abis.dedup();
    abis
}

/// Collect screen densities from res/ directories
pub fn collect_densities(file_names: &[String]) -> Vec<String> {
    let density_suffixes = [
        "ldpi", "mdpi", "hdpi", "xhdpi", "xxhdpi", "xxxhdpi", "nodpi", "tvdpi", "anydpi",
    ];

    let mut densities: Vec<String> = file_names
        .iter()
        .filter_map(|f| {
            if !f.starts_with("res/") {
                return None;
            }
            let parts: Vec<&str> = f.split('/').collect();
            if parts.len() < 2 {
                return None;
            }
            let dir = parts[1];
            for suffix in &density_suffixes {
                if dir == *suffix || dir.ends_with(&format!("-{}", suffix)) {
                    return Some(suffix.to_string());
                }
            }
            None
        })
        .collect();
    densities.sort();
    densities.dedup();
    densities
}

/// Collect languages from res/values-*/
pub fn collect_languages(file_names: &[String]) -> Vec<String> {
    let mut langs: Vec<String> = file_names
        .iter()
        .filter_map(|f| {
            if !f.starts_with("res/values-") {
                return None;
            }
            let rest = f.strip_prefix("res/values-").unwrap_or("");
            let dir = rest.split('/').next().unwrap_or("");
            // Language codes are like "en", "zh-rCN", "de"
            let lang = dir.split('-').next().unwrap_or("");
            if lang.len() == 2 || lang.len() == 3 {
                Some(lang.to_string())
            } else {
                None
            }
        })
        .collect();
    langs.sort();
    langs.dedup();
    langs
}

/// Check if this is a split APK
pub fn is_split_apk(file_names: &[String]) -> bool {
    file_names
        .iter()
        .any(|f| f.starts_with("assets/split_") || f.contains("config."))
}

/// Get app icon as base64
pub fn get_app_icon_base64(apk: &mut ApkReader, element: &axml::AxmlElement) -> Option<String> {
    let app = element.find("application")?;
    let _icon_ref = app
        .get_attr("android:icon")
        .or_else(|| app.get_attr("icon"))
        .or_else(|| app.get_attr("android:roundIcon"))
        .or_else(|| app.get_attr("roundIcon"))?;

    // If icon is a resource reference, try to find the actual file
    // For now, search for common icon paths
    let icon_paths = [
        "res/mipmap-xxxhdpi/ic_launcher.png",
        "res/mipmap-xxhdpi/ic_launcher.png",
        "res/mipmap-xhdpi/ic_launcher.png",
        "res/mipmap-hdpi/ic_launcher.png",
        "res/mipmap-mdpi/ic_launcher.png",
        "res/drawable-xxxhdpi/ic_launcher.png",
        "res/drawable-xxhdpi/ic_launcher.png",
        "res/drawable-xhdpi/ic_launcher.png",
        "res/drawable-hdpi/ic_launcher.png",
        "res/drawable/ic_launcher.png",
    ];

    for path in &icon_paths {
        if let Ok(data) = apk.read_file(path) {
            return Some(base64_encode(&data));
        }
    }

    // Try to find any icon-like file
    let file_names = apk.file_names();
    for name in &file_names {
        if (name.contains("ic_launcher") || name.contains("icon"))
            && (name.ends_with(".png") || name.ends_with(".webp"))
        {
            if let Ok(data) = apk.read_file(name) {
                return Some(base64_encode(&data));
            }
        }
    }

    None
}

fn base64_encode(data: &[u8]) -> String {
    // Simple base64 encoding without external dependency
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::with_capacity((data.len() + 2) / 3 * 4);

    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };

        let triple = (b0 << 16) | (b1 << 8) | b2;

        result.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
        result.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            result.push(CHARS[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(CHARS[(triple & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }

    result
}
