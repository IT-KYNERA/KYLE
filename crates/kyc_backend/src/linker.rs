use std::path::{Path, PathBuf};
use std::process::Command;

pub struct Linker;

impl Linker {
    pub fn new() -> Self {
        Self
    }

    fn linker_cmd() -> Command {
        if cfg!(target_os = "windows") {
            // Use clang as linker driver (it invokes lld/link internally)
            // clang is always available as part of LLVM distribution
            if Command::new("clang").arg("--version").output().is_ok() {
                Command::new("clang")
            } else if Command::new("lld-link.exe").arg("--version").output().is_ok() {
                Command::new("lld-link.exe")
            } else {
                Command::new("link.exe")
            }
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

    pub fn link(&self, object_files: &[&Path], output: &Path, runtime_lib: Option<&Path>, release: bool, links: &[String]) -> Result<(), String> {
        if object_files.is_empty() {
            return Err("No object files to link".to_string());
        }

        let mut cmd = Self::linker_cmd();
        cmd.arg("-o").arg(output);

        if release {
            cmd.arg("-O3");
            cmd.arg("-flto");  // GCC Link-Time Optimization
        }

        for obj in object_files {
            cmd.arg(obj);
        }

        if let Some(runtime) = runtime_lib {
            cmd.arg(runtime);
        }

        for link in links {
            if link.starts_with("-framework ") {
                let framework_name = link.trim_start_matches("-framework ");
                cmd.arg("-framework");
                cmd.arg(framework_name);
            } else if link.starts_with("-L") {
                // Library search path
                cmd.arg(link);
            } else {
                cmd.arg(format!("-l{}", link));
            }
        }

        if cfg!(target_os = "windows") {
            // Windows system libraries needed by Rust std
            cmd.arg("-lkernel32");
            cmd.arg("-lws2_32");
            cmd.arg("-lbcrypt");
            cmd.arg("-luserenv");
            cmd.arg("-lntdll");
            cmd.arg("-ladvapi32");
            cmd.arg("-lcfgmgr32");
            cmd.arg("-lshlwapi");
            cmd.arg("-liphlpapi");
        }

        if release {
            cmd.arg("-lm");
        }

        // On macOS, link CoreFoundation (needed by chrono/iana-time-zone)
        // Check at runtime (not compile-time) so cross-compiled binary works
        if std::env::consts::OS == "macos" {
            // Detect deployment target to match the compiled runtime .o files
            let ver = std::env::var("MACOSX_DEPLOYMENT_TARGET").unwrap_or_else(|_| {
                std::process::Command::new("sw_vers")
                    .arg("-productVersion")
                    .output()
                    .ok()
                    .and_then(|o| if o.status.success() {
                        let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
                        // Take major.minor only (e.g. "26.5.1" → "26.5")
                        let parts: Vec<&str> = s.split('.').collect();
                        if parts.len() >= 2 {
                            Some(format!("{}.{}", parts[0], parts[1]))
                        } else { None }
                    } else { None })
                    .unwrap_or_else(|| "26.0".to_string())
            });
            if !ver.is_empty() {
                cmd.arg(format!("-mmacosx-version-min={}", ver));
            }
            cmd.arg("-framework").arg("CoreFoundation");
            // Common Homebrew library paths
            let homebrew_paths = [
                "/opt/homebrew/opt/libpq/lib",
                "/opt/homebrew/lib",
                "/usr/local/lib",
            ];
            for p in &homebrew_paths {
                if std::path::Path::new(p).exists() {
                    cmd.arg(format!("-L{}", p));
                    cmd.arg(format!("-Wl,-rpath,{}", p));
                }
            }
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

    /// Locate the kyc_runtime static library.
    ///
    /// Search order:
    /// 1. Relative to the kl binary (installed: /usr/local/lib/kl/libkyc_runtime.a)
    /// 2. Cargo workspace (debug/release)
    /// 3. Current working directory
    pub fn find_runtime_lib() -> Option<PathBuf> {
        let runtime_lib = if cfg!(target_os = "windows") {
            "kyc_runtime.lib"
        } else {
            "libkyc_runtime.a"
        };

        let mut paths = Vec::new();

        // 1. Relative to the running binary
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                // /usr/local/bin/kl → /usr/local/lib/kl/libkyc_runtime.a
                paths.push(exe_dir.join("../lib/kl").join(runtime_lib));
                // ~/.ky/bin/kl → ~/.ky/lib/libkyc_runtime.a
                paths.push(exe_dir.join("../lib").join(runtime_lib));
                // Alongside binary
                paths.push(exe_dir.join(runtime_lib));
            }
        }

        // 2. Cargo workspace
        if let Some(root) = workspace_root() {
            paths.push(root.join("target").join("debug").join(runtime_lib));
            paths.push(root.join("target").join("release").join(runtime_lib));
        }

        // 3. Current working directory
        paths.push(PathBuf::from("./target/debug").join(runtime_lib));
        paths.push(PathBuf::from("./target/release").join(runtime_lib));
        paths.push(PathBuf::from(runtime_lib));

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
