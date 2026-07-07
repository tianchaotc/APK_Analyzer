pub mod apk;
pub mod axml;
pub mod dex;
pub mod resources;
pub mod signing;

use std::io::Read;
use zip::ZipArchive;

/// A reader for APK files (which are ZIP archives)
pub struct ApkReader {
    pub archive: ZipArchive<std::fs::File>,
    pub file_path: String,
    pub file_size: u64,
}

impl ApkReader {
    /// Open an APK file from disk
    pub fn open(path: &str) -> Result<Self, String> {
        let file = std::fs::File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
        let metadata = file
            .metadata()
            .map_err(|e| format!("Failed to read metadata: {}", e))?;
        let archive = ZipArchive::new(file)
            .map_err(|e| format!("Failed to read APK (not a valid ZIP): {}", e))?;
        Ok(Self {
            archive,
            file_path: path.to_string(),
            file_size: metadata.len(),
        })
    }

    /// Read a file from the APK by name
    pub fn read_file(&mut self, name: &str) -> Result<Vec<u8>, String> {
        let mut file = self
            .archive
            .by_name(name)
            .map_err(|e| format!("File '{}' not found in APK: {}", name, e))?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)
            .map_err(|e| format!("Failed to read '{}': {}", name, e))?;
        Ok(buf)
    }

    /// Try to read a file, return None if it doesn't exist
    pub fn try_read_file(&mut self, name: &str) -> Option<Vec<u8>> {
        self.read_file(name).ok()
    }

    /// Get list of all file names in the APK
    pub fn file_names(&self) -> Vec<String> {
        self.archive.file_names().map(|s| s.to_string()).collect()
    }

    /// Check if a file exists in the APK
    pub fn has_file(&self, name: &str) -> bool {
        self.archive.file_names().any(|f| f == name)
    }

    /// Iterate over all entries with their sizes
    pub fn entries(&mut self) -> Vec<(String, u64, u64)> {
        let mut entries = Vec::new();
        for i in 0..self.archive.len() {
            if let Ok(file) = self.archive.by_index(i) {
                entries.push((file.name().to_string(), file.size(), file.compressed_size()));
            }
        }
        entries
    }
}
