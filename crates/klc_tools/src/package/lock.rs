use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockEntry {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub checksum: String,
    #[serde(default)]
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockFile {
    pub version: i32,
    #[serde(default)]
    pub packages: Vec<LockEntry>,
}

impl LockFile {
    pub fn read(path: &Path) -> Result<Self, String> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read lock file: {}", e))?;
        toml::from_str(&content)
            .map_err(|e| format!("Failed to parse lock file: {}", e))
    }

    pub fn write(&self, path: &Path) -> Result<(), String> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize lock file: {}", e))?;
        fs::write(path, &content)
            .map_err(|e| format!("Failed to write lock file: {}", e))
    }

    pub fn add_package(&mut self, name: &str, version: &str, source: &str) {
        // Remove existing entry for the same name
        self.packages.retain(|p| p.name != name);
        self.packages.push(LockEntry {
            name: name.to_string(),
            version: version.to_string(),
            checksum: String::new(),
            source: source.to_string(),
        });
    }
}

impl Default for LockFile {
    fn default() -> Self {
        Self {
            version: 1,
            packages: Vec::new(),
        }
    }
}
