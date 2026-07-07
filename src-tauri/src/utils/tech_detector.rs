use crate::models::ai_summary::*;
use crate::models::dex::DexAnalysis;
use crate::models::manifest::ManifestInfo;

/// Detect technology stack from file names, manifest, and DEX content
pub fn detect(
    file_names: &[String],
    manifest: &ManifestInfo,
    _dex: &DexAnalysis,
) -> Vec<TechStackEntry> {
    let mut results = Vec::new();

    // === Flutter ===
    let has_flutter = file_names.iter().any(|f| f == "libflutter.so")
        || file_names.iter().any(|f| f.contains("flutter_assets"))
        || file_names
            .iter()
            .any(|f| f.starts_with("lib/arm64") && f.contains("libflutter"));
    if has_flutter {
        results.push(TechStackEntry {
            name: "Flutter".to_string(),
            confidence: "high".to_string(),
            evidence: vec![
                "libflutter.so native library found".to_string(),
                "Flutter engine binary detected".to_string(),
            ],
        });
    }

    // === React Native ===
    let has_rn = file_names
        .iter()
        .any(|f| f == "assets/index.android.bundle")
        || file_names.iter().any(|f| f.contains("react_native"))
        || file_names.iter().any(|f| f == "lib/hermes.so")
        || file_names.iter().any(|f| f.contains("libreactnativejni"));
    if has_rn {
        let mut evidence = vec!["JavaScript bundle detected".to_string()];
        if file_names.iter().any(|f| f == "lib/hermes.so") {
            evidence.push("Hermes JS engine detected".to_string());
        }
        results.push(TechStackEntry {
            name: "React Native".to_string(),
            confidence: "high".to_string(),
            evidence,
        });
    }

    // === Unity ===
    let has_unity = file_names.iter().any(|f| f.contains("libunity.so"))
        || file_names
            .iter()
            .any(|f| f.contains("libmain.so") && f.contains("unity"))
        || file_names.iter().any(|f| f.contains("assets/bin/Data"))
        || file_names
            .iter()
            .any(|f| f == "assets/bin/Data/Managed/Mono.dll");
    if has_unity {
        results.push(TechStackEntry {
            name: "Unity".to_string(),
            confidence: "high".to_string(),
            evidence: vec![
                "Unity engine native library detected".to_string(),
                "Unity asset structure found".to_string(),
            ],
        });
    }

    // === Unreal Engine ===
    let has_unreal = file_names
        .iter()
        .any(|f| f.contains("libUE4") || f.contains("libUnreal"))
        || file_names.iter().any(|f| f.contains("UE4Game"));
    if has_unreal {
        results.push(TechStackEntry {
            name: "Unreal Engine".to_string(),
            confidence: "high".to_string(),
            evidence: vec!["Unreal Engine native library detected".to_string()],
        });
    }

    // === Cordova / PhoneGap ===
    let has_cordova = file_names.iter().any(|f| f == "assets/www/cordova.js")
        || file_names.iter().any(|f| f.contains("cordova"));
    if has_cordova {
        results.push(TechStackEntry {
            name: "Cordova / PhoneGap".to_string(),
            confidence: "high".to_string(),
            evidence: vec!["Cordova JavaScript files detected".to_string()],
        });
    }

    // === Xamarin ===
    let has_xamarin = file_names.iter().any(|f| f.contains("libmonodroid"))
        || file_names.iter().any(|f| f.contains("Xamarin"))
        || file_names.iter().any(|f| f.contains("libxamarin"));
    if has_xamarin {
        results.push(TechStackEntry {
            name: "Xamarin / .NET MAUI".to_string(),
            confidence: "high".to_string(),
            evidence: vec!["Mono runtime / Xamarin libraries detected".to_string()],
        });
    }

    // === Kotlin ===
    // Check for Kotlin-specific classes in DEX or kotlin-related files
    let has_kotlin = file_names.iter().any(|f| f.contains("kotlin"))
        || _dex.packages.iter().any(|p| p.name.contains("kotlin"));
    if has_kotlin {
        results.push(TechStackEntry {
            name: "Kotlin".to_string(),
            confidence: "high".to_string(),
            evidence: vec!["Kotlin standard library classes found in DEX".to_string()],
        });
    }

    // === Jetpack Compose ===
    let has_compose = _dex
        .packages
        .iter()
        .any(|p| p.name.contains("androidx.compose") || p.name.contains("androidx.compose.ui"));
    if has_compose {
        results.push(TechStackEntry {
            name: "Jetpack Compose".to_string(),
            confidence: "high".to_string(),
            evidence: vec!["androidx.compose packages found in DEX".to_string()],
        });
    }

    // === AndroidX / Jetpack ===
    let has_androidx = _dex
        .packages
        .iter()
        .any(|p| p.name.starts_with("androidx."));
    if has_androidx {
        results.push(TechStackEntry {
            name: "AndroidX / Jetpack Libraries".to_string(),
            confidence: "high".to_string(),
            evidence: vec!["androidx.* packages found in DEX".to_string()],
        });
    }

    // === OkHttp / Retrofit ===
    let has_okhttp = _dex
        .packages
        .iter()
        .any(|p| p.name.contains("okhttp3") || p.name.contains("okhttp"));
    let has_retrofit = _dex
        .packages
        .iter()
        .any(|p| p.name.contains("retrofit2") || p.name.contains("retrofit"));
    if has_okhttp || has_retrofit {
        let mut evidence = Vec::new();
        if has_okhttp {
            evidence.push("OkHttp HTTP client found".to_string());
        }
        if has_retrofit {
            evidence.push("Retrofit REST client found".to_string());
        }
        results.push(TechStackEntry {
            name: "OkHttp / Retrofit".to_string(),
            confidence: "high".to_string(),
            evidence,
        });
    }

    // === Glide / Picasso / Coil ===
    let has_glide = _dex
        .packages
        .iter()
        .any(|p| p.name.contains("com.bumptech.glide"));
    let has_picasso = _dex
        .packages
        .iter()
        .any(|p| p.name.contains("com.squareup.picasso"));
    let has_coil = _dex.packages.iter().any(|p| p.name.contains("coil"));
    if has_glide {
        results.push(TechStackEntry {
            name: "Glide".to_string(),
            confidence: "high".to_string(),
            evidence: vec!["Glide image loading library found".to_string()],
        });
    }
    if has_picasso {
        results.push(TechStackEntry {
            name: "Picasso".to_string(),
            confidence: "high".to_string(),
            evidence: vec!["Picasso image loading library found".to_string()],
        });
    }
    if has_coil {
        results.push(TechStackEntry {
            name: "Coil".to_string(),
            confidence: "high".to_string(),
            evidence: vec!["Coil image loading library found".to_string()],
        });
    }

    // === Room ===
    let has_room = _dex
        .packages
        .iter()
        .any(|p| p.name.contains("androidx.room"));
    if has_room {
        results.push(TechStackEntry {
            name: "Room (SQLite ORM)".to_string(),
            confidence: "high".to_string(),
            evidence: vec!["Room database library found".to_string()],
        });
    }

    // === Firebase ===
    let has_firebase = file_names.iter().any(|f| f.contains("firebase"))
        || _dex
            .packages
            .iter()
            .any(|p| p.name.contains("com.google.firebase"));
    if has_firebase {
        results.push(TechStackEntry {
            name: "Firebase".to_string(),
            confidence: "high".to_string(),
            evidence: vec!["Firebase SDK detected".to_string()],
        });
    }

    // === Google Play Services ===
    let has_gms = _dex
        .packages
        .iter()
        .any(|p| p.name.contains("com.google.android.gms"));
    if has_gms {
        results.push(TechStackEntry {
            name: "Google Play Services".to_string(),
            confidence: "high".to_string(),
            evidence: vec!["Google Play Services SDK detected".to_string()],
        });
    }

    // === WebView-based (Hybrid) ===
    let has_webview = manifest
        .activities
        .iter()
        .any(|a| a.name.contains("WebView"))
        || file_names
            .iter()
            .any(|f| f.starts_with("assets/www/") && f.ends_with(".html"));
    if has_webview && !has_cordova && !has_rn {
        results.push(TechStackEntry {
            name: "WebView (Hybrid)".to_string(),
            confidence: "medium".to_string(),
            evidence: vec!["HTML assets or WebView components detected".to_string()],
        });
    }

    // === Coroutines ===
    let has_coroutines = _dex
        .packages
        .iter()
        .any(|p| p.name.contains("kotlinx.coroutines"));
    if has_coroutines {
        results.push(TechStackEntry {
            name: "Kotlin Coroutines".to_string(),
            confidence: "high".to_string(),
            evidence: vec!["kotlinx.coroutines package found".to_string()],
        });
    }

    // === RxJava ===
    let has_rxjava = _dex
        .packages
        .iter()
        .any(|p| p.name.contains("io.reactivex"));
    if has_rxjava {
        results.push(TechStackEntry {
            name: "RxJava".to_string(),
            confidence: "high".to_string(),
            evidence: vec!["RxJava reactive library found".to_string()],
        });
    }

    // === Native (C/C++) ===
    let has_native = file_names.iter().any(|f| {
        f.starts_with("lib/")
            && f.ends_with(".so")
            && !f.contains("libflutter")
            && !f.contains("libreact")
            && !f.contains("libunity")
    });
    if has_native {
        results.push(TechStackEntry {
            name: "C/C++ (Native)".to_string(),
            confidence: "high".to_string(),
            evidence: vec!["Native .so libraries detected".to_string()],
        });
    }

    // === Java (default) ===
    if results.is_empty() || !has_kotlin {
        results.push(TechStackEntry {
            name: "Java".to_string(),
            confidence: if !has_kotlin { "medium" } else { "low" }.to_string(),
            evidence: vec!["Android app using Java (no Kotlin detected)".to_string()],
        });
    }

    results
}
