use crate::models::components::*;
use crate::models::manifest::ManifestInfo;
use crate::parser::ApkReader;

pub struct ComponentAnalyzer;

impl super::Analyzer for ComponentAnalyzer {
    type Output = ComponentAnalysis;

    fn name(&self) -> &'static str {
        "components"
    }

    fn analyze(&self, apk: &mut ApkReader) -> Result<Self::Output, String> {
        // Reuse manifest analyzer
        let manifest_analyzer = super::manifest::ManifestAnalyzer;
        let manifest: ManifestInfo = manifest_analyzer.analyze(apk)?;

        let stats = ComponentStats {
            activities: manifest.activities.len(),
            services: manifest.services.len(),
            receivers: manifest.receivers.len(),
            providers: manifest.providers.len(),
            exported: count_exported(&manifest),
            with_intent_filters: count_with_intent_filters(&manifest),
        };

        let mut exported_components = Vec::new();

        for c in &manifest.activities {
            if c.exported {
                exported_components.push(ExportedComponent {
                    name: c.name.clone(),
                    component_type: "Activity".to_string(),
                    permission: c.permission.clone(),
                    has_intent_filter: !c.intent_filters.is_empty(),
                });
            }
        }
        for c in &manifest.services {
            if c.exported {
                exported_components.push(ExportedComponent {
                    name: c.name.clone(),
                    component_type: "Service".to_string(),
                    permission: c.permission.clone(),
                    has_intent_filter: !c.intent_filters.is_empty(),
                });
            }
        }
        for c in &manifest.receivers {
            if c.exported {
                exported_components.push(ExportedComponent {
                    name: c.name.clone(),
                    component_type: "Receiver".to_string(),
                    permission: c.permission.clone(),
                    has_intent_filter: !c.intent_filters.is_empty(),
                });
            }
        }
        for c in &manifest.providers {
            if c.exported {
                exported_components.push(ExportedComponent {
                    name: c.name.clone(),
                    component_type: "Provider".to_string(),
                    permission: c.permission.clone(),
                    has_intent_filter: !c.intent_filters.is_empty(),
                });
            }
        }

        Ok(ComponentAnalysis {
            stats,
            activities: manifest.activities,
            services: manifest.services,
            receivers: manifest.receivers,
            providers: manifest.providers,
            exported_components,
        })
    }
}

fn count_exported(manifest: &ManifestInfo) -> usize {
    manifest.activities.iter().filter(|c| c.exported).count()
        + manifest.services.iter().filter(|c| c.exported).count()
        + manifest.receivers.iter().filter(|c| c.exported).count()
        + manifest.providers.iter().filter(|c| c.exported).count()
}

fn count_with_intent_filters(manifest: &ManifestInfo) -> usize {
    manifest
        .activities
        .iter()
        .filter(|c| !c.intent_filters.is_empty())
        .count()
        + manifest
            .services
            .iter()
            .filter(|c| !c.intent_filters.is_empty())
            .count()
        + manifest
            .receivers
            .iter()
            .filter(|c| !c.intent_filters.is_empty())
            .count()
        + manifest
            .providers
            .iter()
            .filter(|c| !c.intent_filters.is_empty())
            .count()
}
