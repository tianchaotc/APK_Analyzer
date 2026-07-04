use crate::models::analysis::ApkAnalysis;
use std::fs;
use std::io::Write;

pub fn export_json(analysis: &ApkAnalysis, path: &str) -> Result<String, String> {
    let json = serde_json::to_string_pretty(analysis)
        .map_err(|e| format!("Failed to serialize: {}", e))?;
    fs::write(path, json)
        .map_err(|e| format!("Failed to write file: {}", e))?;
    Ok(path.to_string())
}

pub fn export_markdown(analysis: &ApkAnalysis, path: &str) -> Result<String, String> {
    let md = generate_markdown(analysis);
    fs::write(path, md)
        .map_err(|e| format!("Failed to write file: {}", e))?;
    Ok(path.to_string())
}

pub fn export_html(analysis: &ApkAnalysis, path: &str) -> Result<String, String> {
    let html = generate_html(analysis);
    fs::write(path, html)
        .map_err(|e| format!("Failed to write file: {}", e))?;
    Ok(path.to_string())
}

pub fn export_csv(analysis: &ApkAnalysis, path: &str) -> Result<String, String> {
    let csv = generate_csv(analysis);
    let mut file = fs::File::create(path)
        .map_err(|e| format!("Failed to create file: {}", e))?;
    file.write_all(csv.as_bytes())
        .map_err(|e| format!("Failed to write: {}", e))?;
    Ok(path.to_string())
}

fn generate_markdown(a: &ApkAnalysis) -> String {
    let mut md = String::new();

    md.push_str(&format!("# APK Analysis Report\n\n"));
    md.push_str(&format!("**File:** {}\n\n", a.file_name));
    md.push_str(&format!("**Size:** {:.2} MB\n\n", a.file_size as f64 / 1_048_576.0));
    md.push_str(&format!("**Analyzed:** {}\n\n", a.analyzed_at));
    md.push_str("---\n\n");

    // Overview
    md.push_str("## Overview\n\n");
    md.push_str(&format!("- **App Name:** {}\n", a.overview.app_name));
    md.push_str(&format!("- **Package:** {}\n", a.overview.package_name));
    md.push_str(&format!("- **Version:** {} (code: {})\n", a.overview.version_name, a.overview.version_code));
    md.push_str(&format!("- **Min SDK:** {}\n", a.overview.min_sdk));
    md.push_str(&format!("- **Target SDK:** {}\n", a.overview.target_sdk));
    md.push_str(&format!("- **ABIs:** {}\n", a.overview.abis.join(", ")));
    md.push_str(&format!("- **Languages:** {}\n", a.overview.languages.join(", ")));
    md.push_str(&format!("- **Debuggable:** {}\n", a.overview.debuggable));
    md.push_str(&format!("- **Allow Backup:** {}\n", a.overview.allow_backup));
    md.push_str(&format!("- **Cleartext Traffic:** {}\n\n", a.overview.uses_cleartext_traffic));

    // Permissions
    md.push_str("## Permissions\n\n");
    md.push_str(&format!("Total: {} (Normal: {}, Dangerous: {}, Signature: {}, Special: {})\n\n",
        a.permissions.summary.total, a.permissions.summary.normal,
        a.permissions.summary.dangerous, a.permissions.summary.signature,
        a.permissions.summary.special));
    if !a.permissions.permissions.is_empty() {
        md.push_str("| Permission | Level | Risk | Category |\n");
        md.push_str("|------------|-------|------|----------|\n");
        for p in &a.permissions.permissions {
            md.push_str(&format!("| {} | {} | {} | {} |\n", p.name, p.protection_level, p.risk_level, p.category));
        }
        md.push_str("\n");
    }

    // Components
    md.push_str("## Components\n\n");
    md.push_str(&format!("- Activities: {}\n", a.components.stats.activities));
    md.push_str(&format!("- Services: {}\n", a.components.stats.services));
    md.push_str(&format!("- Receivers: {}\n", a.components.stats.receivers));
    md.push_str(&format!("- Providers: {}\n", a.components.stats.providers));
    md.push_str(&format!("- Exported: {}\n\n", a.components.stats.exported));

    if !a.components.exported_components.is_empty() {
        md.push_str("### Exported Components\n\n");
        for ec in &a.components.exported_components {
            md.push_str(&format!("- **{}** ({}) - Permission: {}\n", ec.name, ec.component_type,
                ec.permission.as_ref().unwrap_or(&"none".to_string())));
        }
        md.push_str("\n");
    }

    // Native Libraries
    if a.native_libs.summary.total > 0 {
        md.push_str("## Native Libraries\n\n");
        md.push_str(&format!("Total: {} ({} ABIs)\n\n", a.native_libs.summary.total, a.native_libs.summary.abis.len()));
        for group in &a.native_libs.by_abi {
            md.push_str(&format!("### {} ({} files, {:.2} MB)\n\n", group.abi, group.count, group.total_size as f64 / 1_048_576.0));
            for lib in &group.libraries {
                md.push_str(&format!("- {} ({:.2} KB)\n", lib.file_name, lib.size as f64 / 1024.0));
            }
            md.push_str("\n");
        }
    }

    // DEX
    md.push_str("## DEX Analysis\n\n");
    md.push_str(&format!("- DEX files: {}\n", a.dex.summary.total_dex_files));
    md.push_str(&format!("- Total classes: {}\n", a.dex.summary.total_classes));
    md.push_str(&format!("- Total methods: {}\n", a.dex.summary.total_methods));
    md.push_str(&format!("- Total fields: {}\n\n", a.dex.summary.total_fields));

    // Certificate
    md.push_str("## Certificate\n\n");
    md.push_str(&format!("- Signature scheme: {}\n", a.certificate.signature_scheme));
    md.push_str(&format!("- Debug certificate: {}\n", a.certificate.is_debug_certificate));
    md.push_str(&format!("- Expired: {}\n\n", a.certificate.is_expired));
    for signer in &a.certificate.signers {
        md.push_str(&format!("### Signer\n\n"));
        md.push_str(&format!("- **Subject:** {}\n", signer.subject));
        md.push_str(&format!("- **Issuer:** {}\n", signer.issuer));
        md.push_str(&format!("- **SHA1:** {}\n", signer.sha1));
        md.push_str(&format!("- **SHA256:** {}\n", signer.sha256));
        md.push_str(&format!("- **MD5:** {}\n", signer.md5));
        md.push_str(&format!("- **Valid:** {} to {}\n\n", signer.not_before, signer.not_after));
    }

    // Security
    md.push_str(&format!("## Security Analysis\n\n"));
    md.push_str(&format!("**Score: {}/100**\n\n", a.security.score));
    if !a.security.issues.is_empty() {
        md.push_str("| Severity | Title | Recommendation |\n");
        md.push_str("|----------|-------|----------------|\n");
        for issue in &a.security.issues {
            md.push_str(&format!("| {} | {} | {} |\n", issue.severity, issue.title, issue.recommendation));
        }
        md.push_str("\n");
    }

    // AI Summary
    if let Some(ref ai) = a.ai_summary {
        md.push_str("## AI Summary\n\n");
        md.push_str(&format!("{}\n\n", ai.overview));
        md.push_str(&format!("**App Type:** {}\n\n", ai.app_type));
        md.push_str("### Technology Stack\n\n");
        for ts in &ai.tech_stack {
            md.push_str(&format!("- **{}** ({} confidence) - {}\n", ts.name, ts.confidence, ts.evidence.join("; ")));
        }
        md.push_str("\n");
        if !ai.potential_risks.is_empty() {
            md.push_str("### Potential Risks\n\n");
            for risk in &ai.potential_risks {
                md.push_str(&format!("- {}\n", risk));
            }
            md.push_str("\n");
        }
    }

    md
}

fn generate_html(a: &ApkAnalysis) -> String {
    let mut html = String::new();

    html.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n");
    html.push_str("<meta charset=\"UTF-8\">\n");
    html.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n");
    html.push_str(&format!("<title>APK Analysis - {}</title>\n", a.overview.app_name));
    html.push_str("<style>\n");
    html.push_str("body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif; margin: 40px; color: #333; line-height: 1.6; }\n");
    html.push_str("h1 { color: #1a1a1a; border-bottom: 2px solid #e0e0e0; padding-bottom: 10px; }\n");
    html.push_str("h2 { color: #2c3e50; margin-top: 30px; }\n");
    html.push_str("table { border-collapse: collapse; width: 100%; margin: 10px 0; }\n");
    html.push_str("th, td { border: 1px solid #ddd; padding: 8px 12px; text-align: left; }\n");
    html.push_str("th { background: #f5f5f5; font-weight: 600; }\n");
    html.push_str("tr:nth-child(even) { background: #fafafa; }\n");
    html.push_str(".score { font-size: 2em; font-weight: bold; }\n");
    html.push_str(".critical { color: #e74c3c; } .high { color: #e67e22; } .medium { color: #f39c12; } .low { color: #27ae60; }\n");
    html.push_str("</style>\n</head>\n<body>\n");

    html.push_str(&format!("<h1>APK Analysis Report</h1>\n"));
    html.push_str(&format!("<p><strong>File:</strong> {} | <strong>Size:</strong> {:.2} MB | <strong>Analyzed:</strong> {}</p>\n",
        a.file_name, a.file_size as f64 / 1_048_576.0, a.analyzed_at));

    // Overview
    html.push_str("<h2>Overview</h2>\n<table>\n");
    html.push_str(&format!("<tr><th>App Name</th><td>{}</td></tr>\n", a.overview.app_name));
    html.push_str(&format!("<tr><th>Package</th><td>{}</td></tr>\n", a.overview.package_name));
    html.push_str(&format!("<tr><th>Version</th><td>{} (code: {})</td></tr>\n", a.overview.version_name, a.overview.version_code));
    html.push_str(&format!("<tr><th>SDK</th><td>Min: {} / Target: {}</td></tr>\n", a.overview.min_sdk, a.overview.target_sdk));
    html.push_str(&format!("<tr><th>ABIs</th><td>{}</td></tr>\n", a.overview.abis.join(", ")));
    html.push_str("</table>\n");

    // Permissions
    html.push_str("<h2>Permissions</h2>\n");
    html.push_str(&format!("<p>Total: {} | Dangerous: {} | Normal: {}</p>\n",
        a.permissions.summary.total, a.permissions.summary.dangerous, a.permissions.summary.normal));
    if !a.permissions.permissions.is_empty() {
        html.push_str("<table><tr><th>Permission</th><th>Level</th><th>Risk</th></tr>\n");
        for p in &a.permissions.permissions {
            html.push_str(&format!("<tr><td>{}</td><td>{}</td><td class=\"{}\">{}</td></tr>\n",
                p.name, p.protection_level, p.risk_level, p.risk_level));
        }
        html.push_str("</table>\n");
    }

    // Components
    html.push_str("<h2>Components</h2>\n<table>\n");
    html.push_str(&format!("<tr><th>Activities</th><td>{}</td></tr>\n", a.components.stats.activities));
    html.push_str(&format!("<tr><th>Services</th><td>{}</td></tr>\n", a.components.stats.services));
    html.push_str(&format!("<tr><th>Receivers</th><td>{}</td></tr>\n", a.components.stats.receivers));
    html.push_str(&format!("<tr><th>Providers</th><td>{}</td></tr>\n", a.components.stats.providers));
    html.push_str(&format!("<tr><th>Exported</th><td>{}</td></tr>\n", a.components.stats.exported));
    html.push_str("</table>\n");

    // Security
    html.push_str("<h2>Security</h2>\n");
    html.push_str(&format!("<p class=\"score\">Score: {}/100</p>\n", a.security.score));
    if !a.security.issues.is_empty() {
        html.push_str("<table><tr><th>Severity</th><th>Issue</th><th>Recommendation</th></tr>\n");
        for issue in &a.security.issues {
            html.push_str(&format!("<tr><td class=\"{}\">{}</td><td>{}</td><td>{}</td></tr>\n",
                issue.severity, issue.severity, issue.title, issue.recommendation));
        }
        html.push_str("</table>\n");
    }

    html.push_str("</body>\n</html>\n");

    html
}

fn generate_csv(a: &ApkAnalysis) -> String {
    let mut csv = String::new();

    // Overview section
    csv.push_str("Section,Field,Value\n");
    csv.push_str(&format!("Overview,App Name,{}\n", a.overview.app_name));
    csv.push_str(&format!("Overview,Package,{}\n", a.overview.package_name));
    csv.push_str(&format!("Overview,Version,{}\n", a.overview.version_name));
    csv.push_str(&format!("Overview,Version Code,{}\n", a.overview.version_code));
    csv.push_str(&format!("Overview,Min SDK,{}\n", a.overview.min_sdk));
    csv.push_str(&format!("Overview,Target SDK,{}\n", a.overview.target_sdk));
    csv.push_str(&format!("Overview,APK Size,{}\n", a.file_size));
    csv.push_str(&format!("Overview,ABIs,{}\n", a.overview.abis.join("; ")));
    csv.push_str(&format!("Overview,Debuggable,{}\n", a.overview.debuggable));
    csv.push_str("\n");

    // Permissions
    csv.push_str("\nPermission,Protection Level,Risk Level,Category\n");
    for p in &a.permissions.permissions {
        csv.push_str(&format!("{},{},{},{}\n", p.name, p.protection_level, p.risk_level, p.category));
    }

    // Components
    csv.push_str("\nComponent,Name,Exported,Permission\n");
    for c in &a.manifest.activities {
        csv.push_str(&format!("Activity,{},{},{}\n", c.name, c.exported, c.permission.as_deref().unwrap_or("")));
    }
    for c in &a.manifest.services {
        csv.push_str(&format!("Service,{},{},{}\n", c.name, c.exported, c.permission.as_deref().unwrap_or("")));
    }
    for c in &a.manifest.receivers {
        csv.push_str(&format!("Receiver,{},{},{}\n", c.name, c.exported, c.permission.as_deref().unwrap_or("")));
    }
    for c in &a.manifest.providers {
        csv.push_str(&format!("Provider,{},{},{}\n", c.name, c.exported, c.permission.as_deref().unwrap_or("")));
    }

    // Security
    csv.push_str("\nSecurity Issue,Severity,Category,Recommendation\n");
    for issue in &a.security.issues {
        csv.push_str(&format!("{},{},{},{}\n", issue.title, issue.severity, issue.category, issue.recommendation));
    }

    csv
}
