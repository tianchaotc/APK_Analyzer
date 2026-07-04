use crate::parser::ApkReader;
use crate::models::security::*;
use crate::models::manifest::ManifestInfo;

pub struct SecurityAnalyzer;

impl super::Analyzer for SecurityAnalyzer {
    type Output = SecurityAnalysis;

    fn name(&self) -> &'static str {
        "security"
    }

    fn analyze(&self, apk: &mut ApkReader) -> Result<Self::Output, String> {
        let manifest_data = apk.read_file("AndroidManifest.xml")?;
        let element = crate::parser::axml::decode(&manifest_data)?;

        let mut issues: Vec<SecurityIssue> = Vec::new();
        let mut recommendations: Vec<String> = Vec::new();
        let mut score: u32 = 100;

        // Check debuggable
        if let Some(app) = element.find("application") {
            let debuggable = app.get_attr("android:debuggable")
                .or_else(|| app.get_attr("debuggable"))
                .map(|v| v == "true")
                .unwrap_or(false);
            if debuggable {
                issues.push(SecurityIssue {
                    severity: "critical".to_string(),
                    category: "Debug".to_string(),
                    title: "Debuggable enabled".to_string(),
                    description: "android:debuggable=true allows arbitrary code attachment and debugging. This should never be enabled in production.".to_string(),
                    recommendation: "Set android:debuggable=\"false\" in the manifest.".to_string(),
                });
                score -= 25;
            }

            // Check allowBackup
            let allow_backup = app.get_attr("android:allowBackup")
                .or_else(|| app.get_attr("allowBackup"))
                .map(|v| v != "false")
                .unwrap_or(true);
            if allow_backup {
                issues.push(SecurityIssue {
                    severity: "medium".to_string(),
                    category: "Data".to_string(),
                    title: "Backup enabled".to_string(),
                    description: "android:allowBackup=true allows app data to be backed up and restored via adb, potentially exposing sensitive data.".to_string(),
                    recommendation: "Set android:allowBackup=\"false\" if the app handles sensitive data.".to_string(),
                });
                score -= 10;
            }

            // Check cleartext traffic
            let cleartext = app.get_attr("android:usesCleartextTraffic")
                .or_else(|| app.get_attr("usesCleartextTraffic"))
                .map(|v| v == "true")
                .unwrap_or(false);
            if cleartext {
                issues.push(SecurityIssue {
                    severity: "high".to_string(),
                    category: "Network".to_string(),
                    title: "Cleartext traffic enabled".to_string(),
                    description: "android:usesCleartextTraffic=true allows HTTP (unencrypted) traffic, vulnerable to man-in-the-middle attacks.".to_string(),
                    recommendation: "Set android:usesCleartextTraffic=\"false\" and use HTTPS for all network communication.".to_string(),
                });
                score -= 15;
            }

            // Check exported components without permission
            for activity in app.find_all("activity") {
                let exported = activity.get_attr("android:exported")
                    .or_else(|| activity.get_attr("exported"))
                    .map(|v| v == "true")
                    .unwrap_or(false);
                let permission = activity.get_attr("android:permission")
                    .or_else(|| activity.get_attr("permission"));
                let has_intent_filter = activity.find("intent-filter").is_some();
                let name = activity.get_attr("android:name")
                    .or_else(|| activity.get_attr("name"))
                    .unwrap_or_default();

                if exported && permission.is_none() {
                    issues.push(SecurityIssue {
                        severity: "high".to_string(),
                        category: "Component".to_string(),
                        title: format!("Exported activity without permission: {}", name),
                        description: "This activity is exported but has no permission protection, allowing any app to launch it.".to_string(),
                        recommendation: "Set android:exported=\"false\" or add a custom permission.".to_string(),
                    });
                    score -= 5;
                }
            }

            for service in app.find_all("service") {
                let exported = service.get_attr("android:exported")
                    .or_else(|| service.get_attr("exported"))
                    .map(|v| v == "true")
                    .unwrap_or(false);
                let permission = service.get_attr("android:permission")
                    .or_else(|| service.get_attr("permission"));
                let name = service.get_attr("android:name")
                    .or_else(|| service.get_attr("name"))
                    .unwrap_or_default();

                if exported && permission.is_none() {
                    issues.push(SecurityIssue {
                        severity: "high".to_string(),
                        category: "Component".to_string(),
                        title: format!("Exported service without permission: {}", name),
                        description: "This service is exported but has no permission protection.".to_string(),
                        recommendation: "Set android:exported=\"false\" or add a custom permission.".to_string(),
                    });
                    score -= 5;
                }
            }

            for receiver in app.find_all("receiver") {
                let exported = receiver.get_attr("android:exported")
                    .or_else(|| receiver.get_attr("exported"))
                    .map(|v| v == "true")
                    .unwrap_or(false);
                let permission = receiver.get_attr("android:permission")
                    .or_else(|| receiver.get_attr("permission"));
                let name = receiver.get_attr("android:name")
                    .or_else(|| receiver.get_attr("name"))
                    .unwrap_or_default();

                if exported && permission.is_none() {
                    issues.push(SecurityIssue {
                        severity: "high".to_string(),
                        category: "Component".to_string(),
                        title: format!("Exported receiver without permission: {}", name),
                        description: "This broadcast receiver is exported but has no permission protection.".to_string(),
                        recommendation: "Set android:exported=\"false\" or add a custom permission.".to_string(),
                    });
                    score -= 5;
                }
            }

            for provider in app.find_all("provider") {
                let exported = provider.get_attr("android:exported")
                    .or_else(|| provider.get_attr("exported"))
                    .map(|v| v == "true")
                    .unwrap_or(false);
                let permission = provider.get_attr("android:permission")
                    .or_else(|| provider.get_attr("permission"));
                let name = provider.get_attr("android:name")
                    .or_else(|| provider.get_attr("name"))
                    .unwrap_or_default();

                if exported && permission.is_none() {
                    issues.push(SecurityIssue {
                        severity: "critical".to_string(),
                        category: "Component".to_string(),
                        title: format!("Exported provider without permission: {}", name),
                        description: "This content provider is exported without permission protection. Any app can read/write its data.".to_string(),
                        recommendation: "Set android:exported=\"false\" or add strict read/write permissions.".to_string(),
                    });
                    score -= 15;
                }
            }
        }

        // Check target SDK
        for child in element.find_all("uses-sdk") {
            if let Some(target) = child.get_attr("android:targetSdk")
                .or_else(|| child.get_attr("targetSdk"))
                .and_then(|v| v.parse::<u32>().ok())
            {
                if target < 30 {
                    issues.push(SecurityIssue {
                        severity: "medium".to_string(),
                        category: "SDK".to_string(),
                        title: format!("Low target SDK: {}", target),
                        description: format!("Targeting SDK {} misses security improvements in newer Android versions.", target),
                        recommendation: "Update targetSdkVersion to at least 33 (Android 13) to receive latest security enhancements.".to_string(),
                    });
                    score -= 5;
                }
            }
        }

        // Check high-risk permissions
        let high_risk_perms = [
            "android.permission.READ_SMS",
            "android.permission.SEND_SMS",
            "android.permission.RECEIVE_SMS",
            "android.permission.READ_CONTACTS",
            "android.permission.WRITE_CONTACTS",
            "android.permission.ACCESS_FINE_LOCATION",
            "android.permission.ACCESS_COARSE_LOCATION",
            "android.permission.RECORD_AUDIO",
            "android.permission.CAMERA",
            "android.permission.READ_EXTERNAL_STORAGE",
            "android.permission.WRITE_EXTERNAL_STORAGE",
            "android.permission.SYSTEM_ALERT_WINDOW",
            "android.permission.READ_PHONE_STATE",
            "android.permission.CALL_PHONE",
            "android.permission.READ_CALL_LOG",
            "android.permission.WRITE_CALL_LOG",
        ];

        for perm_elem in element.find_all("uses-permission") {
            if let Some(name) = perm_elem.get_attr("android:name").or_else(|| perm_elem.get_attr("name")) {
                if high_risk_perms.contains(&name.as_str()) {
                    issues.push(SecurityIssue {
                        severity: "info".to_string(),
                        category: "Permission".to_string(),
                        title: format!("High-risk permission: {}", name),
                        description: "This permission is classified as dangerous and requires user consent.".to_string(),
                        recommendation: "Ensure this permission is necessary and handle the data responsibly.".to_string(),
                    });
                }
            }
        }

        // Build recommendations
        for issue in &issues {
            if issue.severity == "critical" || issue.severity == "high" {
                recommendations.push(format!("[{}] {} - {}", issue.severity.to_uppercase(), issue.title, issue.recommendation));
            }
        }

        // Ensure score is within bounds
        score = score.min(100);

        Ok(SecurityAnalysis {
            score,
            issues,
            recommendations,
        })
    }
}
