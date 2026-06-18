use std::path::{Path, PathBuf};
use std::process::Command;

pub struct Linker;

impl Linker {
    pub fn new() -> Self {
        Self
    }

    pub fn link(&self, object_files: &[&Path], output: &Path, runtime_lib: Option<&Path>) -> Result<(), String> {
        if object_files.is_empty() {
            return Err("No object files to link".to_string());
        }

        let mut cmd = Command::new("clang");
        cmd.arg("-o").arg(output);

        for obj in object_files {
            cmd.arg(obj);
        }

        if let Some(runtime) = runtime_lib {
            cmd.arg(runtime);
        }

        let status = cmd.status().map_err(|e| format!("Linker failed: {}", e))?;

        if !status.success() {
            return Err("Linking failed".to_string());
        }

        Ok(())
    }

    pub fn link_shared(&self, object_files: &[&Path], output: &Path) -> Result<(), String> {
        if object_files.is_empty() {
            return Err("No object files to link".to_string());
        }

        let mut cmd = Command::new("clang");
        cmd.arg("-shared").arg("-o").arg(output);

        for obj in object_files {
            cmd.arg(obj);
        }

        let status = cmd.status().map_err(|e| format!("Linker failed: {}", e))?;

        if !status.success() {
            return Err("Linking failed".to_string());
        }

        Ok(())
    }

    pub fn verify_output(output: &Path) -> bool {
        output.exists()
    }

    /// Locate the klc_runtime static library in the workspace target directory.
    pub fn find_runtime_lib() -> Option<PathBuf> {
        let search_paths = [
            // Cargo workspace root relative to this crate
            workspace_root().map(|r| r.join("target").join("debug").join("libklc_runtime.a")),
            workspace_root().map(|r| r.join("target").join("release").join("libklc_runtime.a")),
            // Fallback: current dir
            Some(PathBuf::from("./target/debug/libklc_runtime.a")),
            Some(PathBuf::from("./target/release/libklc_runtime.a")),
        ];

        for path in search_paths.into_iter().flatten() {
            if path.exists() {
                return Some(path);
            }
        }
        None
    }
}

fn workspace_root() -> Option<PathBuf> {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for ancestor in manifest.ancestors() {
        if ancestor.join("Cargo.toml").exists() {
            // Check it's the workspace root (has [workspace])
            if let Ok(content) = std::fs::read_to_string(ancestor.join("Cargo.toml")) {
                if content.contains("[workspace]") {
                    return Some(ancestor.to_path_buf());
                }
            }
        }
    }
    None
}
