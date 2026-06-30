use klc_core::semver::{parse_version, SemanticVersion};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub edition: String,
    #[serde(default)]
    pub authors: Vec<String>,
    #[serde(default)]
    pub license: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub main: String,
    #[serde(default)]
    pub compiler: CompilerConfig,
    #[serde(default)]
    pub dependencies: HashMap<String, String>,
    #[serde(default, rename = "dev-dependencies")]
    pub dev_dependencies: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilerConfig {
    #[serde(default = "default_optimization")]
    pub optimization: String,
    #[serde(default = "default_target")]
    pub target: String,
    #[serde(default)]
    pub debug: bool,
}

fn default_optimization() -> String { "O2".to_string() }
fn default_target() -> String { "native".to_string() }

impl Default for CompilerConfig {
    fn default() -> Self {
        Self {
            optimization: default_optimization(),
            target: default_target(),
            debug: false,
        }
    }
}

impl Manifest {
    pub fn read(path: &Path) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
        toml::from_str(&content)
            .map_err(|e| format!("Failed to parse {}: {}", path.display(), e))
    }

    pub fn write(&self, path: &Path) -> Result<(), String> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize manifest: {}", e))?;
        fs::write(path, &content)
            .map_err(|e| format!("Failed to write {}: {}", path.display(), e))
    }

    pub fn find_in_directory(dir: &Path) -> Result<Self, String> {
        let manifest_path = dir.join("kl.toml");
        if !manifest_path.exists() {
            return Err(format!("No kl.toml found in {}", dir.display()));
        }
        Self::read(&manifest_path)
    }

    pub fn find_in_cwd() -> Result<Self, String> {
        Self::find_in_directory(&std::env::current_dir().map_err(|e| format!("{}", e))?)
    }

    pub fn add_dependency(&mut self, name: &str, version: &str) {
        self.dependencies.insert(name.to_string(), version.to_string());
    }

    pub fn remove_dependency(&mut self, name: &str) -> bool {
        self.dependencies.remove(name).is_some()
    }

    pub fn save_to_dir(&self, dir: &Path) -> Result<(), String> {
        self.write(&dir.join("kl.toml"))
    }

    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors: Vec<String> = Vec::new();

        if self.name.is_empty() {
            errors.push("project name is required".into());
        } else if !self.name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            errors.push(format!("invalid project name '{}': only letters, numbers, _, - allowed", self.name));
        }

        if self.version.is_empty() {
            errors.push("project version is required".into());
        } else if let Err(e) = parse_version(&self.version) {
            errors.push(format!("invalid project version '{}': {}", self.version, e));
        }

        if !self.edition.is_empty() && self.edition != "2024" {
            errors.push(format!("unsupported edition '{}' (supported: 2024)", self.edition));
        }

        for (dep_name, dep_ver) in &self.dependencies {
            if dep_name.is_empty() {
                errors.push("dependency name cannot be empty".into());
            }
            if let Err(e) = klc_core::semver::parse_requirement(dep_ver) {
                errors.push(format!("invalid version requirement for '{}': {} ({})", dep_name, e, dep_ver));
            }
        }

        for (dep_name, dep_ver) in &self.dev_dependencies {
            if dep_name.is_empty() {
                errors.push("dev-dependency name cannot be empty".into());
            }
            if let Err(e) = klc_core::semver::parse_requirement(dep_ver) {
                errors.push(format!("invalid version requirement for dev-dep '{}': {} ({})", dep_name, e, dep_ver));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub fn parsed_version(&self) -> Result<SemanticVersion, String> {
        parse_version(&self.version)
    }
}
