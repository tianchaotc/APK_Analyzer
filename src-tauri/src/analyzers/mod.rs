pub mod manifest;
pub mod overview;
pub mod permissions;
pub mod components;
pub mod resources;
pub mod native_libs;
pub mod dex;
pub mod certificate;
pub mod security;
pub mod ai_summary;

use crate::parser::ApkReader;

/// Trait that every analyzer must implement.
/// Each analyzer is independent and produces its own result.
pub trait Analyzer {
    type Output: serde::Serialize + serde::de::DeserializeOwned + Send + 'static;

    /// Name of this analyzer stage
    fn name(&self) -> &'static str;

    /// Run the analysis
    fn analyze(&self, apk: &mut ApkReader) -> Result<Self::Output, String>;
}
