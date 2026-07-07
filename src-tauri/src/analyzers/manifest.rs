use crate::models::manifest::*;
use crate::parser::{axml, ApkReader};

pub struct ManifestAnalyzer;

impl super::Analyzer for ManifestAnalyzer {
    type Output = ManifestInfo;

    fn name(&self) -> &'static str {
        "manifest"
    }

    fn analyze(&self, apk: &mut ApkReader) -> Result<Self::Output, String> {
        let manifest_data = apk.read_file("AndroidManifest.xml")?;
        let element = axml::decode(&manifest_data)?;
        let raw_xml = axml::to_xml(&element);

        let mut info = ManifestInfo {
            raw_xml,
            ..Default::default()
        };

        // Parse root <manifest> element
        info.package = element.get_attr("package").unwrap_or_default();
        info.version_code = element
            .get_attr("android:versionCode")
            .or_else(|| element.get_attr("versionCode"))
            .unwrap_or_default();
        info.version_name = element
            .get_attr("android:versionName")
            .or_else(|| element.get_attr("versionName"))
            .unwrap_or_default();

        // Parse uses-sdk
        for child in element.find_all("uses-sdk") {
            if let Some(v) = child
                .get_attr("android:minSdk")
                .or_else(|| child.get_attr("minSdk"))
            {
                info.min_sdk = v.parse().unwrap_or(0);
            }
            if let Some(v) = child
                .get_attr("android:targetSdk")
                .or_else(|| child.get_attr("targetSdk"))
            {
                info.target_sdk = v.parse().unwrap_or(0);
            }
            if let Some(v) = child
                .get_attr("android:compileSdk")
                .or_else(|| child.get_attr("compileSdk"))
            {
                info.compile_sdk = v.parse().unwrap_or(0);
            }
        }

        // Parse application attributes
        if let Some(app) = element.find("application") {
            info.debuggable = app
                .get_attr("android:debuggable")
                .or_else(|| app.get_attr("debuggable"))
                .map(|v| v == "true")
                .unwrap_or(false);
            info.allow_backup = app
                .get_attr("android:allowBackup")
                .or_else(|| app.get_attr("allowBackup"))
                .map(|v| v != "false")
                .unwrap_or(true);
            info.extract_native_libs = app
                .get_attr("android:extractNativeLibs")
                .or_else(|| app.get_attr("extractNativeLibs"))
                .map(|v| v == "true")
                .unwrap_or(false);
            info.uses_cleartext_traffic = app
                .get_attr("android:usesCleartextTraffic")
                .or_else(|| app.get_attr("usesCleartextTraffic"))
                .map(|v| v == "true")
                .unwrap_or(false);
            info.instant_app = app
                .get_attr("android:isInstantApp")
                .or_else(|| app.get_attr("isInstantApp"))
                .map(|v| v == "true")
                .unwrap_or(false);

            // Parse components
            info.activities = app
                .find_all("activity")
                .map(|elem| parse_component_with_kind(elem, ComponentKind::Activity))
                .collect();
            info.services = app
                .find_all("service")
                .map(|elem| parse_component_with_kind(elem, ComponentKind::Service))
                .collect();
            info.receivers = app
                .find_all("receiver")
                .map(|elem| parse_component_with_kind(elem, ComponentKind::Receiver))
                .collect();
            info.providers = app
                .find_all("provider")
                .map(|elem| parse_component_with_kind(elem, ComponentKind::Provider))
                .collect();

            // Find launch activity (with MAIN + LAUNCHER intent filter)
            for activity in &info.activities {
                for filter in &activity.intent_filters {
                    if filter
                        .actions
                        .iter()
                        .any(|a| a == "android.intent.action.MAIN")
                        && filter
                            .categories
                            .iter()
                            .any(|c| c == "android.intent.category.LAUNCHER")
                    {
                        info.launch_activity = Some(activity.name.clone());
                        break;
                    }
                }
                if info.launch_activity.is_some() {
                    break;
                }
            }

            // Parse meta-data
            info.meta_data = app
                .find_all("meta-data")
                .map(|md| MetaData {
                    name: md
                        .get_attr("android:name")
                        .or_else(|| md.get_attr("name"))
                        .unwrap_or_default(),
                    value: md
                        .get_attr("android:value")
                        .or_else(|| md.get_attr("value")),
                    resource: md
                        .get_attr("android:resource")
                        .or_else(|| md.get_attr("resource")),
                })
                .collect();
        }

        // Parse permissions declared
        for child in element.find_all("uses-permission") {
            if let Some(name) = child
                .get_attr("android:name")
                .or_else(|| child.get_attr("name"))
            {
                info.permissions_declared.push(name);
            }
        }

        // Parse features
        for child in element.find_all("uses-feature") {
            info.uses_features.push(Feature {
                name: child
                    .get_attr("android:name")
                    .or_else(|| child.get_attr("name"))
                    .unwrap_or_default(),
                required: child
                    .get_attr("android:required")
                    .or_else(|| child.get_attr("required"))
                    .map(|v| v != "false")
                    .unwrap_or(true),
                version: child
                    .get_attr("android:version")
                    .or_else(|| child.get_attr("version"))
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(0),
            });
        }

        // Parse queries
        for child in element.find_all("queries") {
            for pkg in child.find_all("package") {
                info.queries.push(Query {
                    package: pkg
                        .get_attr("android:name")
                        .or_else(|| pkg.get_attr("name")),
                    intent: None,
                });
            }
            for intent in child.find_all("intent") {
                let filter = parse_intent_filter(intent);
                info.queries.push(Query {
                    package: None,
                    intent: Some(filter),
                });
            }
        }

        Ok(info)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentKind {
    Activity,
    Service,
    Receiver,
    Provider,
}

pub fn infer_exported(elem: &axml::AxmlElement, kind: ComponentKind) -> bool {
    if let Some(value) = elem
        .get_attr("android:exported")
        .or_else(|| elem.get_attr("exported"))
    {
        return value == "true";
    }

    match kind {
        ComponentKind::Activity | ComponentKind::Service | ComponentKind::Receiver => {
            elem.find("intent-filter").is_some()
        }
        ComponentKind::Provider => false,
    }
}

fn parse_component_with_kind(elem: &axml::AxmlElement, kind: ComponentKind) -> Component {
    let intent_filters: Vec<IntentFilter> = elem
        .find_all("intent-filter")
        .map(parse_intent_filter)
        .collect();

    let meta_data = elem
        .find_all("meta-data")
        .map(|md| MetaData {
            name: md
                .get_attr("android:name")
                .or_else(|| md.get_attr("name"))
                .unwrap_or_default(),
            value: md
                .get_attr("android:value")
                .or_else(|| md.get_attr("value")),
            resource: md
                .get_attr("android:resource")
                .or_else(|| md.get_attr("resource")),
        })
        .collect();

    Component {
        name: elem
            .get_attr("android:name")
            .or_else(|| elem.get_attr("name"))
            .unwrap_or_default(),
        exported: infer_exported(elem, kind),
        enabled: elem
            .get_attr("android:enabled")
            .or_else(|| elem.get_attr("enabled"))
            .map(|v| v != "false")
            .unwrap_or(true),
        permission: elem
            .get_attr("android:permission")
            .or_else(|| elem.get_attr("permission")),
        process: elem
            .get_attr("android:process")
            .or_else(|| elem.get_attr("process")),
        intent_filters,
        meta_data,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn element(
        name: &str,
        attrs: &[(&str, &str)],
        children: Vec<axml::AxmlElement>,
    ) -> axml::AxmlElement {
        axml::AxmlElement {
            name: name.to_string(),
            namespace: None,
            attributes: attrs
                .iter()
                .map(|(name, value)| axml::AxmlAttribute {
                    name: (*name).to_string(),
                    namespace: None,
                    value: (*value).to_string(),
                    raw_value: Some((*value).to_string()),
                    typed_type: 0,
                })
                .collect(),
            children,
            text: None,
        }
    }

    #[test]
    fn parse_component_infers_activity_exported_when_intent_filter_is_present() {
        let activity = element(
            "activity",
            &[("android:name", ".MainActivity")],
            vec![element("intent-filter", &[], Vec::new())],
        );

        let component = parse_component_with_kind(&activity, ComponentKind::Activity);

        assert!(component.exported);
    }

    #[test]
    fn parse_component_respects_explicit_false_over_intent_filter() {
        let service = element(
            "service",
            &[
                ("android:name", ".SyncService"),
                ("android:exported", "false"),
            ],
            vec![element("intent-filter", &[], Vec::new())],
        );

        let component = parse_component_with_kind(&service, ComponentKind::Service);

        assert!(!component.exported);
    }

    #[test]
    fn parse_component_keeps_provider_unexported_when_exported_is_missing() {
        let provider = element(
            "provider",
            &[("android:name", ".DataProvider")],
            vec![element("intent-filter", &[], Vec::new())],
        );

        let component = parse_component_with_kind(&provider, ComponentKind::Provider);

        assert!(!component.exported);
    }

    #[test]
    fn parse_component_respects_explicit_provider_true() {
        let provider = element(
            "provider",
            &[
                ("android:name", ".DataProvider"),
                ("android:exported", "true"),
            ],
            Vec::new(),
        );

        let component = parse_component_with_kind(&provider, ComponentKind::Provider);

        assert!(component.exported);
    }
}

fn parse_intent_filter(elem: &axml::AxmlElement) -> IntentFilter {
    let mut filter = IntentFilter::default();

    for action in elem.find_all("action") {
        if let Some(name) = action
            .get_attr("android:name")
            .or_else(|| action.get_attr("name"))
        {
            filter.actions.push(name);
        }
    }

    for cat in elem.find_all("category") {
        if let Some(name) = cat
            .get_attr("android:name")
            .or_else(|| cat.get_attr("name"))
        {
            filter.categories.push(name);
        }
    }

    for data in elem.find_all("data") {
        if let Some(s) = data
            .get_attr("android:scheme")
            .or_else(|| data.get_attr("scheme"))
        {
            filter.data_schemes.push(s);
        }
        if let Some(h) = data
            .get_attr("android:host")
            .or_else(|| data.get_attr("host"))
        {
            filter.data_hosts.push(h);
        }
        if let Some(p) = data
            .get_attr("android:path")
            .or_else(|| data.get_attr("path"))
        {
            filter.data_paths.push(p);
        }
        if let Some(m) = data
            .get_attr("android:mimeType")
            .or_else(|| data.get_attr("mimeType"))
        {
            filter.data_mime_types.push(m);
        }
    }

    filter
}

// Extension methods for AxmlElement
impl axml::AxmlElement {
    pub fn get_attr(&self, name: &str) -> Option<String> {
        self.attributes
            .iter()
            .find(|a| a.name == name)
            .map(|a| a.value.clone())
    }

    pub fn find(&self, tag: &str) -> Option<&axml::AxmlElement> {
        self.children.iter().find(|c| c.name == tag)
    }

    pub fn find_all<'a>(
        &'a self,
        tag: &'a str,
    ) -> impl Iterator<Item = &'a axml::AxmlElement> + 'a {
        self.children.iter().filter(move |c| c.name == tag)
    }
}
