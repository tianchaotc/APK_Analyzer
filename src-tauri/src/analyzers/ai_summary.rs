use crate::models::ai_summary::*;
use crate::models::analysis::ApkAnalysis;
use crate::parser::ApkReader;
use crate::utils::tech_detector;

pub struct AISummaryAnalyzer;

/// The AI Summary analyzer needs the full analysis result, so it's not a standard Analyzer.
/// It's called separately after all other analyzers complete.
pub fn generate_summary(apk: &mut ApkReader, analysis: &ApkAnalysis) -> AISummary {
    let file_names = apk.file_names();

    // Detect tech stack
    let tech_stack = tech_detector::detect(&file_names, &analysis.manifest, &analysis.dex);

    // Guess app type from permissions and components
    let app_type = guess_app_type(&analysis.permissions.permissions, &analysis.manifest);

    // Generate overview text
    let overview = format!(
        "{} (package: {}) version {} (code {}) targets Android {} (min {}). \
        The APK is {:.1} MB with {} DEX files containing {} classes, {} methods. \
        It uses {} native libraries across {} ABIs. Security score: {}/100.",
        analysis.overview.app_name,
        analysis.overview.package_name,
        analysis.overview.version_name,
        analysis.overview.version_code,
        analysis.overview.target_sdk,
        analysis.overview.min_sdk,
        analysis.overview.apk_size as f64 / 1_048_576.0,
        analysis.dex.summary.total_dex_files,
        analysis.dex.summary.total_classes,
        analysis.dex.summary.total_methods,
        analysis.native_libs.summary.total,
        analysis.native_libs.summary.abis.len(),
        analysis.security.score,
    );

    // Architecture guess
    let architecture_guess = guess_architecture(&tech_stack, &analysis.components.stats);

    // Potential risks
    let potential_risks = collect_risks(analysis);

    // Performance suggestions
    let performance_suggestions = collect_performance_suggestions(analysis);

    // Packaging suggestions
    let packaging_suggestions = collect_packaging_suggestions(analysis);

    // Permission review
    let permission_review = generate_permission_review(&analysis.permissions);

    AISummary {
        overview,
        app_type,
        tech_stack,
        architecture_guess,
        potential_risks,
        performance_suggestions,
        packaging_suggestions,
        permission_review,
    }
}

fn guess_app_type(
    permissions: &[crate::models::permissions::PermissionInfo],
    manifest: &crate::models::manifest::ManifestInfo,
) -> String {
    let perm_names: Vec<&str> = permissions.iter().map(|p| p.name.as_str()).collect();

    if perm_names
        .iter()
        .any(|p| p.contains("BILLING") || p.contains("PURCHASE"))
    {
        return "E-commerce / Shopping app with in-app purchases".to_string();
    }
    if perm_names.iter().any(|p| p.contains("LOCATION"))
        && perm_names.iter().any(|p| p.contains("CAMERA"))
    {
        return "Social / LBS app (location + camera features)".to_string();
    }
    if perm_names.iter().any(|p| p.contains("RECORD_AUDIO")) {
        return "Communication / Media app (audio recording)".to_string();
    }
    if perm_names
        .iter()
        .any(|p| p.contains("SMS") || p.contains("CALL_PHONE"))
    {
        return "Communication app (SMS/Phone features)".to_string();
    }
    if perm_names
        .iter()
        .any(|p| p.contains("WAKE_LOCK") && p.contains("VIBRATE"))
    {
        return "Utility / Productivity app".to_string();
    }
    if manifest.providers.len() > 3 {
        return "Data-rich app with content sharing".to_string();
    }
    if manifest.services.len() > 10 {
        return "Background-service heavy app (possibly IM, sync, or media)".to_string();
    }
    "General purpose application".to_string()
}

fn guess_architecture(
    tech_stack: &[TechStackEntry],
    stats: &crate::models::components::ComponentStats,
) -> String {
    let mut parts = Vec::new();

    for ts in tech_stack {
        parts.push(format!("{} ({})", ts.name, ts.confidence));
    }

    if stats.services > 15 {
        parts.push("Service-oriented architecture".to_string());
    }
    if stats.providers > 5 {
        parts.push("Content provider based data sharing".to_string());
    }
    if stats.receivers > 10 {
        parts.push("Event-driven with broadcast receivers".to_string());
    }

    if parts.is_empty() {
        "Standard Android MVC/MVP architecture".to_string()
    } else {
        parts.join("; ")
    }
}

fn collect_risks(analysis: &ApkAnalysis) -> Vec<String> {
    let mut risks = Vec::new();

    if analysis.overview.debuggable {
        risks.push("Debuggable flag is enabled - critical security risk in production".to_string());
    }
    if analysis.overview.uses_cleartext_traffic {
        risks.push("Cleartext HTTP traffic is allowed - vulnerable to MITM attacks".to_string());
    }
    if analysis.overview.allow_backup {
        risks.push("Backup is enabled - sensitive data may be extractable via adb".to_string());
    }
    if analysis.certificate.is_debug_certificate {
        risks.push(
            "Signed with a debug certificate - not suitable for production release".to_string(),
        );
    }
    if analysis.certificate.is_expired {
        risks.push("Certificate has expired - app may not install on newer devices".to_string());
    }

    let exported_count = analysis.components.stats.exported;
    if exported_count > 5 {
        risks.push(format!(
            "{} exported components - review each for security implications",
            exported_count
        ));
    }

    let dangerous_perms = analysis.permissions.summary.dangerous;
    if dangerous_perms > 10 {
        risks.push(format!(
            "{} dangerous permissions - ensure each is justified and necessary",
            dangerous_perms
        ));
    }

    if analysis.overview.target_sdk.parse::<u32>().unwrap_or(0) < 30 {
        risks.push(
            "Low target SDK version - missing security improvements from recent Android versions"
                .to_string(),
        );
    }

    if risks.is_empty() {
        risks.push("No significant security risks detected".to_string());
    }

    risks
}

fn collect_performance_suggestions(analysis: &ApkAnalysis) -> Vec<String> {
    let mut suggestions = Vec::new();

    // APK size analysis
    let apk_mb = analysis.overview.apk_size as f64 / 1_048_576.0;
    if apk_mb > 100.0 {
        suggestions.push(format!(
            "APK is {:.0}MB - consider using App Bundle (AAB) for size optimization",
            apk_mb
        ));
    } else if apk_mb > 50.0 {
        suggestions.push(format!(
            "APK is {:.0}MB - review large resources and consider asset optimization",
            apk_mb
        ));
    }

    // DEX analysis
    if analysis.dex.summary.total_methods > 65000 {
        suggestions
            .push("Method count exceeds 65K - ensure multidex is properly configured".to_string());
    }

    // Native libs
    if analysis.native_libs.summary.abis.len() > 3 {
        suggestions.push(
            "Multiple ABIs bundled - consider using App Bundle for per-ABI delivery".to_string(),
        );
    }

    // Resource analysis
    let total_res_mb = analysis.resources.summary.total_size as f64 / 1_048_576.0;
    if total_res_mb > 20.0 {
        suggestions.push(format!(
            "Resources total {:.0}MB - optimize images and remove unused resources",
            total_res_mb
        ));
    }

    if !analysis.resources.duplicate_resources.is_empty() {
        suggestions.push(format!(
            "{} duplicate resources detected - consolidate to save space",
            analysis.resources.duplicate_resources.len()
        ));
    }

    if suggestions.is_empty() {
        suggestions.push("No significant performance issues detected".to_string());
    }

    suggestions
}

fn collect_packaging_suggestions(analysis: &ApkAnalysis) -> Vec<String> {
    let mut suggestions = Vec::new();

    if !analysis.certificate.has_v2 {
        suggestions.push(
            "Use APK Signature Scheme v2 or v3 for faster install and better security".to_string(),
        );
    }

    if analysis.overview.extract_native_libs {
        suggestions.push(
            "Consider setting extractNativeLibs=\"false\" to reduce install size".to_string(),
        );
    }

    let total_libs = analysis.native_libs.summary.total;
    if total_libs > 0
        && analysis
            .native_libs
            .summary
            .abis
            .contains(&"armeabi-v7a".to_string())
    {
        suggestions.push("Consider dropping armeabi-v7a if targeting modern devices (arm64-v8a covers 99%+ of devices)".to_string());
    }

    if suggestions.is_empty() {
        suggestions.push("Packaging configuration looks good".to_string());
    }

    suggestions
}

fn generate_permission_review(perms: &crate::models::permissions::PermissionAnalysis) -> String {
    if perms.summary.total == 0 {
        return "No permissions requested.".to_string();
    }

    format!(
        "The app requests {} permissions: {} normal, {} dangerous, {} signature, {} special. \
        {} high-risk permissions require user consent. Review dangerous permissions for privacy compliance.",
        perms.summary.total,
        perms.summary.normal,
        perms.summary.dangerous,
        perms.summary.signature,
        perms.summary.special,
        perms.summary.dangerous,
    )
}
