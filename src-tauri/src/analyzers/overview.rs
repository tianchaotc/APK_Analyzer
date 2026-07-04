use crate::parser::ApkReader;
use crate::models::overview::*;
use crate::utils::apk_utils;

pub struct OverviewAnalyzer;

impl super::Analyzer for OverviewAnalyzer {
    type Output = OverviewInfo;

    fn name(&self) -> &'static str {
        "overview"
    }

    fn analyze(&self, apk: &mut ApkReader) -> Result<Self::Output, String> {
        let file_size = apk.file_size;

        // Parse manifest for basic info
        let manifest_data = apk.read_file("AndroidManifest.xml")?;
        let element = crate::parser::axml::decode(&manifest_data)?;

        let app_name = apk_utils::get_app_name(apk, &element);
        let package_name = element.get_attr("package").unwrap_or_default();
        let version_name = element.get_attr("android:versionName")
            .or_else(|| element.get_attr("versionName"))
            .unwrap_or_default();
        let version_code = element.get_attr("android:versionCode")
            .or_else(|| element.get_attr("versionCode"))
            .unwrap_or_default();

        let (min_sdk, target_sdk, compile_sdk) = apk_utils::get_sdk_versions(&element);

        // Parse application attributes
        let (debuggable, allow_backup, extract_native_libs, uses_cleartext_traffic, instant_app) =
            if let Some(app) = element.find("application") {
                (
                    app.get_attr("android:debuggable").or_else(|| app.get_attr("debuggable"))
                        .map(|v| v == "true").unwrap_or(false),
                    app.get_attr("android:allowBackup").or_else(|| app.get_attr("allowBackup"))
                        .map(|v| v != "false").unwrap_or(true),
                    app.get_attr("android:extractNativeLibs").or_else(|| app.get_attr("extractNativeLibs"))
                        .map(|v| v == "true").unwrap_or(false),
                    app.get_attr("android:usesCleartextTraffic").or_else(|| app.get_attr("usesCleartextTraffic"))
                        .map(|v| v == "true").unwrap_or(false),
                    app.get_attr("android:isInstantApp").or_else(|| app.get_attr("isInstantApp"))
                        .map(|v| v == "true").unwrap_or(false),
                )
            } else {
                (false, true, false, false, false)
            };

        // Collect ABIs from native libs
        let abis = apk_utils::collect_abis(&apk.file_names());

        // Collect densities from resources
        let densities = apk_utils::collect_densities(&apk.file_names());

        // Collect languages from res/values-*/
        let languages = apk_utils::collect_languages(&apk.file_names());

        // Check for split APK
        let split_apk = apk.has_file("resources.pb") || apk_utils::is_split_apk(&apk.file_names());

        // Estimate install size (APK uncompressed size)
        let estimated_install_size = apk.entries().iter()
            .map(|(_, size, _)| size)
            .sum::<u64>();

        // Get app icon
        let app_icon_base64 = apk_utils::get_app_icon_base64(apk, &element);

        // Signature version - will be filled by certificate analyzer
        let signature_version = String::new();

        Ok(OverviewInfo {
            app_name,
            package_name,
            version_name,
            version_code,
            min_sdk: min_sdk.to_string(),
            target_sdk: target_sdk.to_string(),
            compile_sdk: compile_sdk.to_string(),
            apk_size: file_size,
            estimated_install_size,
            abis,
            languages,
            densities,
            signature_version,
            debuggable,
            allow_backup,
            extract_native_libs,
            uses_cleartext_traffic,
            instant_app,
            split_apk,
            bundle_info: None,
            app_icon_base64,
        })
    }
}
