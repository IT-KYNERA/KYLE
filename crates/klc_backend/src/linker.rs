use std::path::{Path, PathBuf};
use std::process::Command;

pub struct Linker;

impl Linker {
    pub fn new() -> Self {
        Self
    }

    fn linker_cmd() -> Command {
        if cfg!(target_os = "windows") {
            Command::new("link.exe")
        } else if cfg!(target_os = "macos") {
            Command::new("clang")
        } else {
            // Linux: try clang first, fallback to cc (gcc symlink)
            if Command::new("clang").arg("--version").output().is_ok() {
                Command::new("clang")
            } else {
                Command::new("cc")
            }
        }
    }

    pub fn link(&self, object_files: &[&Path], output: &Path, runtime_lib: Option<&Path>) -> Result<(), String> {
        if object_files.is_empty() {
            return Err("No object files to link".to_string());
        }

        let mut cmd = Self::linker_cmd();
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

        let mut cmd = Self::linker_cmd();
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

    /// Locate the klc_runtime static library.
    ///
    /// Search order:
    /// 1. Relative to the kl binary (installed: /usr/local/lib/kl/libklc_runtime.a)
    /// 2. Cargo workspace (debug/release)
    /// 3. Current working directory
    pub fn find_runtime_lib() -> Option<PathBuf> {
        let mut paths = Vec::new();

        // 1. Relative to the running binary
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                // /usr/local/bin/kl → /usr/local/lib/kl/libklc_runtime.a
                paths.push(exe_dir.join("../lib/kl/libklc_runtime.a"));
                // ~/.kl/bin/kl → ~/.kl/lib/libklc_runtime.a
                paths.push(exe_dir.join("../lib/libklc_runtime.a"));
                // Alongside binary
                paths.push(exe_dir.join("libklc_runtime.a"));
            }
        }

        // 2. Cargo workspace
        if let Some(root) = workspace_root() {
            paths.push(root.join("target").join("debug").join("libklc_runtime.a"));
            paths.push(root.join("target").join("release").join("libklc_runtime.a"));
        }

        // 3. Current working directory
        paths.push(PathBuf::from("./target/debug/libklc_runtime.a"));
        paths.push(PathBuf::from("./target/release/libklc_runtime.a"));
        paths.push(PathBuf::from("./libklc_runtime.a"));

        for p in &paths {
            if p.exists() {
                return Some(p.clone());
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
