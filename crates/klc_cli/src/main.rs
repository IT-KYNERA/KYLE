use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

use klc_core::resolver::RegistryBackend;
use klc_tools::package::{
    Manifest, LockFile, find_project_root, main_source_path,
    cache,
    registry::RegistryClient,
};

fn bin_name() -> String {
    "kl".to_string()
}

fn exe_path(path: &Path) -> PathBuf {
    let ext = std::env::consts::EXE_EXTENSION;
    if ext.is_empty() { path.to_path_buf() } else { path.with_extension(ext) }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }

    let command = &args[1];

    match command.as_str() {
        "build" => cmd_build(&args),
        "run" => cmd_run(&args),
        "check" => cmd_check(&args),
        "parse" => cmd_parse(&args),
        "mir" => cmd_mir(&args),
        "test" => cmd_test(&args),
        "fmt" => cmd_fmt(&args),
        "new" | "init" => cmd_new(&args),
        "add" => cmd_add(&args),
        "remove" => cmd_remove(&args),
        "info" => cmd_info(&args),
        "publish" => cmd_publish(&args),
        "login" => cmd_login(&args),
        "update" => cmd_update(&args),
        "outdated" => cmd_outdated(&args),
        "uninstall" => cmd_uninstall(),
        "lsp" => cmd_lsp(&args),
        "completions" => cmd_completions(&args),
        "_complete" => cmd_complete(&args),
        "help" | "-h" => print_usage(),
        "--version" | "-v" => {
            println!("{} v{}", bin_name(), env!("CARGO_PKG_VERSION"));
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            print_usage();
            process::exit(1);
        }
    }
}

// ── Dependency Resolution ──

fn resolve_project_dependencies(project_root: &Path, manifest: &Manifest) -> Result<(), String> {
    if manifest.dependencies.is_empty() {
        return Ok(());
    }

    println!("Resolving dependencies...");

    let registry = RegistryClient::new();
    let graph = manifest.resolve_dependencies(&registry)?;

    if !graph.conflicts.is_empty() {
        for conflict in &graph.conflicts {
            eprintln!("Conflict: {}", conflict);
        }
        return Err("Dependency resolution failed due to conflicts".to_string());
    }

    cache::ensure_cache_dir()?;
    let source = "registry";

    for name in &graph.order {
        if let Some(version) = graph.version_str(name) {
            if !cache::is_cached(name, &version) {
                print!("  Downloading {} v{} ... ", name, version);
                let data = klc_tools::package::registry::download_package(name, &version)?;
                let dest = cache::package_cache_dir(name, &version);
                cache::extract_tarball(&data, &dest)?;
                println!("done");
            }
        }
    }

    let lock_path = project_root.join("kl.lock");
    let mut lock = LockFile::read(&lock_path).unwrap_or_default();
    lock.update_from_graph(&graph, source);
    lock.write(&lock_path)?;

    println!("Dependencies resolved ({} packages)", graph.packages.len());
    Ok(())
}

fn resolve_and_check(project_root: &Path) {
    if let Ok(manifest) = Manifest::find_in_directory(project_root) {
        if let Err(errors) = manifest.validate() {
            for err in &errors {
                eprintln!("Warning: kl.toml: {}", err);
            }
        }
        if !manifest.dependencies.is_empty() {
            if let Err(e) = resolve_project_dependencies(project_root, &manifest) {
                eprintln!("Dependency resolution error: {}", e);
                process::exit(1);
            }
        }
    }
}

// ── Command Implementations ──

fn cmd_build(args: &[String]) {
    let release = args.iter().any(|a| a == "--release");
    let file_arg = args.iter().skip(2).find(|a| !a.starts_with("--"));
    if file_arg.is_none() {
        // Project mode
        let project_root = match find_project_root(&std::env::current_dir().unwrap()) {
            Some(r) => r,
            None => { eprintln!("No kl.toml found"); process::exit(1); }
        };
        resolve_and_check(&project_root);

        if let Some(source_path) = main_source_path(&project_root) {
            let source = fs::read_to_string(&source_path).unwrap_or_else(|e| {
                eprintln!("Error reading {}: {}", source_path.display(), e);
                process::exit(1);
            });
            let file = source_path.to_string_lossy().to_string();
            let build_dir = project_root.join("target").join(if release { "release" } else { "debug" });
            let output = exe_path(&build_dir.join("main"));
            let _ = fs::create_dir_all(&build_dir);
            let build_result = if release {
                klc_driver::pipeline::Pipeline::build_source_with_artifacts_release(&source, &file, &output, &build_dir)
            } else {
                klc_driver::pipeline::Pipeline::build_source_with_artifacts(&source, &file, &output, &build_dir)
            };
            match build_result {
                Ok(()) => println!("Build complete: {}", output.display()),
                Err(e) => { eprintln!("Build error: {}", e); process::exit(1); }
            }
        } else {
            eprintln!("No src/main.kl found in project");
            process::exit(1);
        }
        return;
    }
    // Single-file mode
    let file = file_arg.unwrap();
    let file_idx = args.iter().position(|a| a == file).unwrap();
    let source = ensure_main(&load_source(args, file_idx));
    let file_stem = Path::new(file).file_stem().unwrap_or_default().to_string_lossy().to_string();
    let build_dir = Path::new("target").join(if release { "release" } else { "debug" });
    let output = exe_path(&build_dir.join(&file_stem));
    let _ = fs::create_dir_all(&build_dir);
    let build_result = if release {
        klc_driver::pipeline::Pipeline::build_source_with_artifacts_release(&source, file, &output, &build_dir)
    } else {
        klc_driver::pipeline::Pipeline::build_source_with_artifacts(&source, file, &output, &build_dir)
    };
    match build_result {
        Ok(()) => println!("Build complete: {}", output.display()),
        Err(e) => { eprintln!("Build error: {}", e); process::exit(1); }
    }
}

fn cmd_run(args: &[String]) {
    let release = args.iter().any(|a| a == "--release");
    let file_arg = args.iter().skip(2).find(|a| !a.starts_with("--"));
    if file_arg.is_none() {
        // Project mode
        let project_root = match find_project_root(&std::env::current_dir().unwrap()) {
            Some(r) => r,
            None => { eprintln!("No kl.toml found"); process::exit(1); }
        };
        resolve_and_check(&project_root);

        if let Some(source_path) = main_source_path(&project_root) {
            let source = fs::read_to_string(&source_path).unwrap_or_else(|e| {
                eprintln!("Error reading {}: {}", source_path.display(), e);
                process::exit(1);
            });
            let file = source_path.to_string_lossy().to_string();
            let build_dir = project_root.join("target").join(if release { "release" } else { "debug" });
            let output = exe_path(&build_dir.join("main"));
            let _ = fs::create_dir_all(&build_dir);
            let run_result = if release {
                klc_driver::pipeline::Pipeline::build_source_with_artifacts_release(&source, &file, &output, &build_dir)
            } else {
                klc_driver::pipeline::Pipeline::build_source_with_artifacts(&source, &file, &output, &build_dir)
            };
            match run_result {
                Ok(()) => {
                    let status = process::Command::new(&output)
                        .args(args.iter().skip(2))
                        .status()
                        .expect("Failed to execute binary");
                    if !status.success() {
                        process::exit(status.code().unwrap_or(1));
                    }
                }
                Err(e) => { eprintln!("Run error: {}", e); process::exit(1); }
            }
        } else {
            eprintln!("No src/main.kl found in project");
            process::exit(1);
        }
        return;
    }
    // Single-file mode
    let file = file_arg.unwrap();
    let file_idx = args.iter().position(|a| a == file).unwrap();
    let source = ensure_main(&load_source(args, file_idx));
    let file_stem = Path::new(file).file_stem().unwrap_or_default().to_string_lossy().to_string();
    let build_dir = Path::new("target").join(if release { "release" } else { "debug" });
    let output = exe_path(&build_dir.join(&file_stem));
    let _ = fs::create_dir_all(&build_dir);
    let run_result = if release {
        klc_driver::pipeline::Pipeline::build_source_with_artifacts_release(&source, file, &output, &build_dir)
    } else {
        klc_driver::pipeline::Pipeline::build_source_with_artifacts(&source, file, &output, &build_dir)
    };
    match run_result {
        Ok(()) => {
            let status = process::Command::new(&output)
                .args(args.iter().skip(3))
                .status()
                .expect("Failed to execute binary");
            if !status.success() {
                process::exit(status.code().unwrap_or(1));
            }
        }
        Err(e) => { eprintln!("Run error: {}", e); process::exit(1); }
    }
}

fn cmd_check(args: &[String]) {
    if args.len() < 3 || args[2].starts_with("--") {
        // Project mode
        if let Some(project_root) = find_project_root(&std::env::current_dir().unwrap()) {
            resolve_and_check(&project_root);

            if let Some(source_path) = main_source_path(&project_root) {
                let source = fs::read_to_string(&source_path).unwrap_or_else(|e| {
                    eprintln!("Error reading {}: {}", source_path.display(), e);
                    process::exit(1);
                });
                let file = source_path.to_string_lossy().to_string();
                match klc_driver::pipeline::Pipeline::check_source(&source, &file) {
                    Ok(output) => {
                        if output.analyzer.has_errors() {
                            output.analyzer.emit_diagnostics();
                            process::exit(1);
                        } else {
                            println!("No errors found.");
                        }
                    }
                    Err(e) => { eprintln!("Check error: {}", e); process::exit(1); }
                }
            } else {
                eprintln!("No src/main.kl found in project");
                process::exit(1);
            }
        } else {
            eprintln!("Usage: {} check <file.kl>", bin_name());
            process::exit(1);
        }
        return;
    }
    // Single-file mode
    let source = load_source(args, 2);
    let file = &args[2];
    match klc_driver::pipeline::Pipeline::check_source(&source, file) {
        Ok(output) => {
            if output.analyzer.has_errors() {
                output.analyzer.emit_diagnostics();
                process::exit(1);
            } else {
                println!("No errors found.");
            }
        }
        Err(e) => { eprintln!("Check error: {}", e); process::exit(1); }
    }
}

fn cmd_parse(args: &[String]) {
    let source = load_source(args, 2);
    let file = &args[2];
    match klc_driver::pipeline::Pipeline::parse_source(&source) {
        Ok(output) => {
            println!("--- AST for {} ---", file);
            println!("{}", output.program);
        }
        Err(e) => { eprintln!("Parse error: {}", e); process::exit(1); }
    }
}

fn cmd_mir(args: &[String]) {
    let source = load_source(args, 2);
    let file = &args[2];
    match klc_driver::pipeline::Pipeline::mir_source(&source, file) {
        Ok(output) => {
            if output.analyzer.has_errors() {
                output.analyzer.emit_diagnostics();
                process::exit(1);
            }
            println!("{}", output.module);
        }
        Err(e) => { eprintln!("MIR error: {}", e); process::exit(1); }
    }
}

fn cmd_fmt(args: &[String]) {
    let check = args.iter().any(|a| a == "--check");

    // Determine project root from cwd or explicit path
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    // Try loading format config from kl.toml (from either cwd or the first explicit path)
    let config = {
        let root = if args.len() > 2 && !args[2].starts_with("--") {
            let first = Path::new(&args[2]);
            if first.is_file() {
                first.parent().and_then(|p| klc_tools::package::find_project_root(p))
            } else if first.is_dir() {
                klc_tools::package::find_project_root(first)
            } else {
                klc_tools::package::find_project_root(&cwd)
            }
        } else {
            klc_tools::package::find_project_root(&cwd)
        };
        if let Some(root) = root {
            let manifest_path = root.join("kl.toml");
            if manifest_path.exists() {
                if let Ok(toml_str) = fs::read_to_string(&manifest_path) {
                    if let Ok(manifest) = klc_tools::package::Manifest::from_str(&toml_str) {
                        manifest.format
                    } else {
                        klc_tools::package::FormatConfig::default()
                    }
                } else {
                    klc_tools::package::FormatConfig::default()
                }
            } else {
                klc_tools::package::FormatConfig::default()
            }
        } else {
            klc_tools::package::FormatConfig::default()
        }
    };
    let formatter = klc_tools::formatter::Formatter::with_config(config);
    let mut any_fail = false;

    // Collect files to format
    let files: Vec<PathBuf> = if args.len() > 2 && !args[2].starts_with("--") {
        // Explicit path(s) given
        args[2..].iter()
            .take_while(|a| !a.starts_with("--"))
            .flat_map(|p| {
                let path = Path::new(p);
                if path.is_dir() {
                    // Format all .kl files in directory
                    fs::read_dir(path).ok().into_iter().flat_map(|rd| {
                        rd.flatten().filter_map(|e| {
                            let p = e.path();
                            if p.extension().map_or(false, |ext| ext == "kl") {
                                Some(p)
                            } else {
                                None
                            }
                        }).collect::<Vec<_>>()
                    }).collect::<Vec<_>>()
                } else {
                    vec![path.to_path_buf()]
                }
            }).collect()
    } else {
        // No path — try project mode (format all in src/ and tests/)
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        if let Some(root) = klc_tools::package::find_project_root(&cwd) {
            let src_dir = root.join("src");
            let tests_dir = root.join("tests");
            let mut files = Vec::new();
            if src_dir.is_dir() {
                if let Ok(rd) = fs::read_dir(&src_dir) {
                    for entry in rd.flatten() {
                        let p = entry.path();
                        if p.extension().map_or(false, |ext| ext == "kl") {
                            files.push(p);
                        }
                    }
                }
            }
            if tests_dir.is_dir() {
                if let Ok(rd) = fs::read_dir(&tests_dir) {
                    for entry in rd.flatten() {
                        let p = entry.path();
                        if p.extension().map_or(false, |ext| ext == "kl") {
                            files.push(p);
                        }
                    }
                }
            }
            files
        } else {
            eprintln!("Error: no project found and no file specified");
            eprintln!("Usage: {} fmt [<file.kl> | <dir/>]", bin_name());
            process::exit(1);
        }
    };

    if files.is_empty() {
        eprintln!("No .kl files found to format");
        process::exit(1);
    }

    for path in &files {
        let source = match fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Error reading '{}': {}", path.display(), e);
                any_fail = true;
                continue;
            }
        };
        match formatter.format(&source) {
            Ok(formatted) => {
                if check {
                    if source != formatted {
                        eprintln!("{}: would reformat", path.display());
                        any_fail = true;
                    }
                } else {
                    if source != formatted {
                        fs::write(path, &formatted).unwrap_or_else(|e| {
                            eprintln!("Error writing '{}': {}", path.display(), e);
                            any_fail = true;
                        });
                        println!("Formatted: {}", path.display());
                    }
                }
            }
            Err(e) => {
                eprintln!("{}: format error: {}", path.display(), e);
                any_fail = true;
            }
        }
    }

    if any_fail {
        process::exit(1);
    }
}

fn cmd_new(args: &[String]) {
    if args.len() < 3 {
        eprintln!("Error: missing project name");
        eprintln!("Usage: {} new <project>", bin_name());
        process::exit(1);
    }
    let project_name_arg = &args[2];
    let project_dir = Path::new(project_name_arg);
    let project_name = project_dir.file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    if project_dir.exists() {
        eprintln!("Error: directory '{}' already exists", project_name);
        process::exit(1);
    }
    for dir in &["src", "tests"] {
        fs::create_dir_all(project_dir.join(dir)).unwrap_or_else(|e| {
            eprintln!("Error creating project: {}", e);
            process::exit(1);
        });
    }

    let main_kl = format!(
        "# {} — entry point\nfrom lib import greet\n\nfn main() i32:\n    println(greet(\"World\"))\n    0\n",
        project_name
    );
    fs::write(project_dir.join("src").join("main.kl"), &main_kl).unwrap_or_else(|e| {
        eprintln!("Error writing src/main.kl: {}", e);
        process::exit(1);
    });

    let lib_kl = format!(
        "# {} — library module\nfn greet(name: str) str:\n    \"Hello, \" + name + \"!\"\n",
        project_name
    );
    fs::write(project_dir.join("src").join("lib.kl"), &lib_kl).unwrap_or_else(|e| {
        eprintln!("Error writing src/lib.kl: {}", e);
        process::exit(1);
    });

    let test_kl = format!(
        "# Tests for {}\nfrom lib import greet\n\nfn test_greet() i32:\n    result := greet(\"World\")\n    assert(result == \"Hello, World!\")\n    println(\"test_greet PASS\")\n    0\n",
        project_name
    );
    fs::write(project_dir.join("tests").join("test_main.kl"), &test_kl).unwrap_or_else(|e| {
        eprintln!("Error writing tests/test_main.kl: {}", e);
        process::exit(1);
    });

    let manifest = format!(
        "[project]\nname = \"{}\"\nversion = \"0.1.0\"\nedition = \"2024\"\nauthors = [\"You <you@example.com>\"]\nlicense = \"MIT\"\ndescription = \"A Kyle programming language project\"\nmain = \"src/main.kl\"\n\n[compiler]\noptimization = \"O2\"\ntarget = \"native\"\n\n[dependencies]\n",
        project_name
    );
    fs::write(project_dir.join("kl.toml"), &manifest).unwrap_or_else(|e| {
        eprintln!("Error writing kl.toml: {}", e);
        process::exit(1);
    });

    let gitignore = "target/\n*.klc-build/\nkl.lock\n.vscode/\n";
    fs::write(project_dir.join(".gitignore"), gitignore).unwrap_or_else(|e| {
        eprintln!("Error writing .gitignore: {}", e);
        process::exit(1);
    });

    let readme = format!(
        "# {}\n\n{}\n\n## Usage\n\n```console\n# Build and run\n{bname} build\n{bname} run\n\n# Run tests\n{bname} test\n\n# Release build\n{bname} build --release\n```\n\n## Project Structure\n\n```\n├── src/\n│   ├── main.kl       # Entry point\n│   └── lib.kl        # Library module\n├── tests/\n│   └── test_main.kl  # Tests\n├── kl.toml           # Project manifest\n└── README.md\n```\n",
        project_name, project_name, bname = bin_name()
    );
    fs::write(project_dir.join("README.md"), &readme).unwrap_or_else(|e| {
        eprintln!("Error writing README.md: {}", e);
        process::exit(1);
    });

    let vscode_settings = format!(
        r#"{{"kl.klcPath":"{}","files.associations":{{"*.kl":"kl"}}}}"#,
        bin_name()
    );
    let vscode_dir = project_dir.join(".vscode");
    fs::create_dir_all(&vscode_dir).unwrap_or_else(|e| {
        eprintln!("Error creating .vscode: {}", e);
        process::exit(1);
    });
    fs::write(vscode_dir.join("settings.json"), vscode_settings).unwrap_or_else(|e| {
        eprintln!("Error writing .vscode/settings.json: {}", e);
        process::exit(1);
    });

    println!("✅ Created project '{}'", project_name);
    println!("   ├── src/main.kl        — entry point");
    println!("   ├── src/lib.kl         — library module");
    println!("   ├── tests/             — tests");
    println!("   ├── kl.toml            — manifest");
    println!("   ├── README.md          — project docs");
    println!("   └── .vscode/           — VS Code settings");
    println!();
    println!("   cd {} && {} run", project_name, bin_name());
}

fn cmd_lsp(_args: &[String]) {
    match klc_tools::lsp::LanguageServer::new() {
        Ok(mut server) => {
            if let Err(e) = server.run() {
                eprintln!("LSP error: {}", e);
                process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Failed to start LSP server: {}", e);
            process::exit(1);
        }
    }
}

fn cmd_completions(args: &[String]) {
    let name = bin_name();
    let shell = if args.len() > 2 { &args[2] } else { "bash" };
    match shell {
        "bash" => print!("{}", bash_completions(&name)),
        "zsh" => print!("{}", zsh_completions(&name)),
        "fish" => print!("{}", fish_completions(&name)),
        "powershell" | "pwsh" => print!("{}", powershell_completions(&name)),
        _ => {
            eprintln!("Unknown shell '{}'. Supported: bash, zsh, fish, powershell", shell);
            process::exit(1);
        }
    }
}

/// Hidden command `kl _complete <cmd> <prefix>` — outputs completion candidates
/// for dynamic completion (e.g., package names for `kl add`).
fn cmd_complete(args: &[String]) {
    let sub = args.get(2).map(|s| s.as_str()).unwrap_or("");
    let prefix = args.get(3).map(|s| s.as_str()).unwrap_or("");
    match sub {
        "add" => {
            // Suggest cached packages matching prefix
            if let Ok(packages) = klc_tools::package::cache::list_cached_packages() {
                for (name, _version) in &packages {
                    if name.starts_with(prefix) {
                        println!("{}", name);
                    }
                }
            }
        }
        _ => {}
    }
}

fn bash_completions(name: &str) -> String {
    format!(
        "_{name}() {{
    local cur prev words cword
    _init_completion || return
    local cmds=\"build run check test parse mir fmt new add remove info publish login update outdated lsp completions help\"
    if [[ $cword -eq 1 ]]; then
        COMPREPLY=($(compgen -W \"$cmds\" -- \"$cur\"))
    elif [[ $cword -ge 2 ]]; then
        case \"${{prev}}\" in
            build|run|check|test|parse|mir|fmt)
                COMPREPLY=($(compgen -f -X '!*.kl' -- \"$cur\"))
                ;;
            add)
                local packages=$({name} _complete add \"$cur\" 2>/dev/null)
                COMPREPLY=($(compgen -W \"$packages\" -- \"$cur\"))
                ;;
            *) ;;
        esac
    fi
}} &&
complete -F _{name} {name}
"
    )
}

fn zsh_completions(name: &str) -> String {
    format!(
        "#compdef {name}
local -a cmds
cmds=(
    'build:Compile project or file'
    'run:Compile and execute'
    'check:Type-check without codegen'
    'parse:Parse and dump AST'
    'mir:Parse and dump MIR'
    'fmt:Format source code'
    'new:Create new project'
    'add:Add dependency'
    'remove:Remove dependency'
    'info:Show project info'
    'publish:Publish package to registry'
    'login:Login to package registry'
    'update:Update lock file'
    'outdated:List outdated dependencies'
    'lsp:Start LSP server'
    'completions:Generate shell completions'
    'help:Show help'
)
_describe -t commands '{name}' cmds && ret=0

# Dynamic completion for `kl add` — suggest cached packages
_{{name}}_add() {{
  local -a packages
  packages=(${{(f)\"$({name} _complete add \\\"$words[CURRENT]\\\" 2>/dev/null)\"}})
  _describe -t packages 'package' packages
}}
compdef _{{name}}_add {name}
"
    )
}

fn fish_completions(name: &str) -> String {
    format!(
        "complete -c {name} -f -a build -d \"Compile project or file\"
complete -c {name} -f -a run -d \"Compile and execute\"
complete -c {name} -f -a check -d \"Type-check without codegen\"
complete -c {name} -f -a parse -d \"Parse and dump AST\"
complete -c {name} -f -a mir -d \"Parse and dump MIR\"
complete -c {name} -f -a fmt -d \"Format source code\"
complete -c {name} -f -a new -d \"Create new project\"
complete -c {name} -f -a add -d \"Add dependency\"
complete -c {name} -f -a remove -d \"Remove dependency\"
complete -c {name} -f -a info -d \"Show project info\"
complete -c {name} -f -a publish -d \"Publish package to registry\"
complete -c {name} -f -a login -d \"Login to package registry\"
complete -c {name} -f -a update -d \"Update lock file\"
complete -c {name} -f -a outdated -d \"List outdated dependencies\"
complete -c {name} -f -a lsp -d \"Start LSP server\"
complete -c {name} -f -a completions -d \"Generate shell completions\"
complete -c {name} -f -a help -d \"Show help\"
# Dynamic completion for `kl add` — suggest cached packages
complete -c {name} -f -n \"__fish_seen_subcommand_from add\" -a \"({name} _complete add (commandline -ct) 2>/dev/null)\" -d \"Cached package\"
"
    )
}

fn powershell_completions(name: &str) -> String {
    format!(
        "Register-ArgumentCompleter -Native -CommandName '{name}' -ScriptBlock {{
    param($wordToComplete, $commandAst, $cursorPosition)
    $commands = @(
        'build', 'run', 'check', 'test', 'parse', 'mir', 'fmt',
        'new', 'add', 'remove', 'info', 'publish', 'login',
        'update', 'outdated', 'lsp', 'completions', 'help'
    )
    $cmd = $commandAst.CommandElements[1].Value
    if ($commandAst.CommandElements.Count -eq 2) {{
        $commands | Where-Object {{ $_ -like \"$wordToComplete*\" }}
    }} elseif ($commandAst.CommandElements.Count -eq 3 -and @('build','run','check','test','parse','mir','fmt') -contains $cmd) {{
        Get-ChildItem -Filter *.kl | Where-Object {{ $_ -like \"$wordToComplete*\" }} | ForEach-Object {{ $_.Name }}
    }} elseif ($commandAst.CommandElements.Count -eq 3 -and $cmd -eq 'add') {{
        # Suggest cached packages
        $home = if ($env:KL_HOME) {{ \"$env:KL_HOME/cache\" }} else {{ \"$env:USERPROFILE\\.kl\\cache\" }}
        if (Test-Path $home) {{
            Get-ChildItem $home -Directory | ForEach-Object {{ $_.Name -replace '-[0-9]+\\\\.[0-9]+\\\\.[0-9]+$', '' }} | Sort-Object -Unique | Where-Object {{ $_ -like \"$wordToComplete*\" }}
        }}
    }}
}}
"
    )
}

fn print_usage() {
    let name = bin_name();
    eprintln!("{} v{} — Kyle Programming Language Compiler", name, env!("CARGO_PKG_VERSION"));
    eprintln!();
    eprintln!("Project commands (run from a project directory with kl.toml):");
    eprintln!("  {name} build [--release]   Compile project to native binary");
    eprintln!("  {name} run   [--release]   Compile and execute project");
    eprintln!("  {name} test                Run project tests");
    eprintln!("  {name} info                Show project info");
    eprintln!("  {name} add   <dep>[@<ver>] Add dependency");
    eprintln!("  {name} remove <dep>        Remove dependency");
    eprintln!("  {name} update              Update lock file to latest compatible versions");
    eprintln!("  {name} outdated            List outdated dependencies");
    eprintln!("  {name} publish             Publish project to registry");
    eprintln!("  {name} login               Login to package registry");
    eprintln!();
    eprintln!("File commands:");
    eprintln!("  {name} build <file.kl>     Compile single file");
    eprintln!("  {name} run   <file.kl>     Compile and run single file");
    eprintln!("  {name} check <file.kl>     Type-check without codegen");
    eprintln!("  {name} parse <file.kl>     Parse and dump AST");
    eprintln!("  {name} mir   <file.kl>     Parse and dump MIR");
    eprintln!("  {name} fmt   [file/dir]    Format source files in project or specific file/dir");
    eprintln!("  {name} fmt   [file.kl] --check  Check formatting (CI mode)");
    eprintln!();
    eprintln!("Project creation:");
    eprintln!("  {name} new   <project>     Create new KL project");
    eprintln!();
    eprintln!("Tools:");
    eprintln!("  {name} lsp                 Start LSP server (stdio)");
    eprintln!("  {name} completions <shell>  Generate shell completions (bash, zsh, fish, powershell)");
    eprintln!("  {name} help                Show this help");
}

fn cmd_test(args: &[String]) {
    let release = args.iter().any(|a| a == "--release");
    let project_root = match find_project_root(&std::env::current_dir().unwrap()) {
        Some(r) => r,
        None => { eprintln!("No kl.toml found"); process::exit(1); }
    };
    resolve_and_check(&project_root);

    let test_files = klc_tools::package::test_source_paths(&project_root);
    if test_files.is_empty() {
        println!("No test files found in tests/");
        println!("All checks passed.");
        return;
    }

    let mut total = 0u32;
    let mut passed = 0u32;
    let mut failed: Vec<String> = Vec::new();

    let build_dir = project_root.join("target").join(if release { "release" } else { "debug" });
    let _ = fs::create_dir_all(&build_dir);

    for test_file_path in &test_files {
        let source = fs::read_to_string(test_file_path).unwrap_or_else(|e| {
            eprintln!("Error reading {}: {}", test_file_path.display(), e);
            process::exit(1);
        });
        let file_name = test_file_path.file_stem().unwrap_or_default().to_string_lossy().to_string();

        // Parse to find #[test] functions
        let parsed = match klc_driver::pipeline::Pipeline::parse_source(&source) {
            Ok(p) => p,
            Err(e) => { eprintln!("Parse error in {}: {}", test_file_path.display(), e); process::exit(1); }
        };

        // Collect #[test] functions
        let test_fns: Vec<&klc_core::ast::FunctionDecl> = parsed.program.declarations.iter()
            .filter_map(|d| {
                if let klc_core::ast::Decl::Function(f) = d {
                    if f.is_test { Some(f) } else { None }
                } else {
                    None
                }
            })
            .collect();

        if test_fns.is_empty() {
            println!("  {}: no #[test] functions", test_file_path.file_name().unwrap_or_default().to_string_lossy());
            continue;
        }

        for test_fn in &test_fns {
            total += 1;
            let test_name = format!("{}::{}", file_name, test_fn.name);

            // Validate test function signature
            if !test_fn.params.is_empty() {
                eprintln!("  FAIL {}: test function must have no parameters", test_name);
                failed.push(test_name);
                continue;
            }

            // Generate wrapper: original source + test main
            let wrapper = format!(
                "{}\nfn main() i32:\n    {}()\n    println(\"  PASS {}\")\n    0\n",
                source, test_fn.name, test_name
            );

            print!("  test {} ... ", test_name);
            let output = exe_path(&build_dir.join(format!("test_{}", test_fn.name)));
            let build_result = if release {
                klc_driver::pipeline::Pipeline::build_source_with_artifacts_release(&wrapper, test_file_path.to_str().unwrap(), &output, &build_dir)
            } else {
                klc_driver::pipeline::Pipeline::build_source_with_artifacts(&wrapper, test_file_path.to_str().unwrap(), &output, &build_dir)
            };

            match build_result {
                Ok(()) => {
                    let run_result = std::process::Command::new(&output)
                        .output()
                        .map_err(|e| format!("failed to execute test binary: {}", e));

                    match run_result {
                        Ok(run_output) => {
                            if run_output.status.success() {
                                passed += 1;
                                let stdout = String::from_utf8_lossy(&run_output.stdout);
                                println!("OK");
                                print!("{}", stdout);
                            } else {
                                let stdout = String::from_utf8_lossy(&run_output.stdout);
                                let stderr = String::from_utf8_lossy(&run_output.stderr);
                                println!("FAILED");
                                if !stderr.is_empty() {
                                    eprintln!("{}", stderr);
                                }
                                if !stdout.is_empty() {
                                    println!("{}", stdout);
                                }
                                failed.push(test_name);
                                // Clean up binary
                                let _ = fs::remove_file(&output);
                            }
                        }
                        Err(e) => {
                            println!("ERROR");
                            eprintln!("  {}", e);
                            failed.push(test_name);
                        }
                    }
                }
                Err(e) => {
                    println!("BUILD FAILED");
                    eprintln!("  {}", e);
                    failed.push(test_name);
                }
            }
            // Clean up binary
            let _ = fs::remove_file(&output);
        }
    }

    println!("\nTest results: {}/{} passed", passed, total);
    if !failed.is_empty() {
        eprintln!("Failed tests:");
        for f in &failed {
            eprintln!("  - {}", f);
        }
        process::exit(1);
    }
}

fn cmd_add(args: &[String]) {
    if args.len() < 3 {
        eprintln!("Usage: {} add <dependency>[@<version>]", bin_name());
        process::exit(1);
    }
    let dep_str = &args[2];
    let (name, version) = if let Some(at_pos) = dep_str.find('@') {
        (&dep_str[..at_pos], &dep_str[at_pos + 1..])
    } else {
        (dep_str.as_str(), "*")
    };

    let cwd = std::env::current_dir().unwrap();
    match Manifest::find_in_cwd() {
        Ok(mut manifest) => {
            manifest.add_dependency(name, version);
            if let Err(e) = manifest.save_to_dir(&cwd) {
                eprintln!("Error saving manifest: {}", e);
                process::exit(1);
            }
            println!("Added dependency '{}' version '{}'", name, version);

            if let Some(project_root) = find_project_root(&cwd) {
                if let Err(e) = resolve_project_dependencies(&project_root, &manifest) {
                    eprintln!("Warning: could not resolve dependencies: {}", e);
                    eprintln!("Run 'kl update' to resolve later");
                }
            }
        }
        Err(e) => { eprintln!("Error: {}", e); process::exit(1); }
    }
}

fn cmd_remove(args: &[String]) {
    if args.len() < 3 {
        eprintln!("Usage: {} remove <dependency>", bin_name());
        process::exit(1);
    }
    let name = &args[2];
    match Manifest::find_in_cwd() {
        Ok(mut manifest) => {
            if manifest.remove_dependency(name) {
                if let Err(e) = manifest.save_to_dir(&std::env::current_dir().unwrap()) {
                    eprintln!("Error saving manifest: {}", e);
                    process::exit(1);
                }
                println!("Removed dependency '{}'", name);
            } else {
                eprintln!("Dependency '{}' not found", name);
                process::exit(1);
            }
        }
        Err(e) => { eprintln!("Error: {}", e); process::exit(1); }
    }
}

fn cmd_info(args: &[String]) {
    let dir = if args.len() > 2 {
        PathBuf::from(&args[2])
    } else {
        std::env::current_dir().unwrap()
    };
    match Manifest::find_in_directory(&dir) {
        Ok(manifest) => {
            println!("Project: {}", manifest.project_name());
            println!("Version: {}", manifest.project_version());
            println!("Edition: {}", manifest.project_edition());
            let authors = manifest.project_authors();
            if !authors.is_empty() {
                println!("Authors: {}", authors.join(", "));
            }
            println!("License: {}", manifest.project_license());
            let desc = manifest.project_description();
            if !desc.is_empty() {
                println!("Description: {}", desc);
            }
            let main = manifest.project_main();
            if !main.is_empty() {
                println!("Main: {}", main);
            }
            println!();
            println!("Compiler: {} ({})", manifest.compiler.target, manifest.compiler.optimization);
            println!();
            if manifest.dependencies.is_empty() {
                println!("Dependencies: (none)");
            } else {
                println!("Dependencies:");
                for line in manifest.dependency_summary() {
                    println!("{}", line);
                }
            }

            let lock_path = dir.join("kl.lock");
            if let Ok(lock) = LockFile::read(&lock_path) {
                if !lock.packages.is_empty() {
                    println!();
                    println!("Resolved packages (kl.lock):");
                    for pkg in &lock.packages {
                        let cached = if cache::is_cached(&pkg.name, &pkg.version) {
                            "cached"
                        } else {
                            "not cached"
                        };
                        println!("  {} v{} [{}]", pkg.name, pkg.version, cached);
                    }
                }
            }
        }
        Err(e) => { eprintln!("Error: {}", e); process::exit(1); }
    }
}

// ── New Commands ──

fn cmd_publish(_args: &[String]) {
    let cwd = std::env::current_dir().unwrap();
    let manifest = match Manifest::find_in_cwd() {
        Ok(m) => m,
        Err(e) => { eprintln!("{}", e); process::exit(1); }
    };

    if let Err(errors) = manifest.validate() {
        for err in &errors {
            eprintln!("Validation error: {}", err);
        }
        process::exit(1);
    }

    let name = manifest.project_name().to_string();
    let version = manifest.project_version().to_string();

    println!("Publishing '{}' v{} ...", name, version);

    let tarball_data = match create_package_tarball(&cwd) {
        Ok(data) => data,
        Err(e) => { eprintln!("Failed to create package: {}", e); process::exit(1); }
    };

    match upload_package(&name, &version, &tarball_data) {
        Ok(()) => println!("✅ Published '{}' v{}", name, version),
        Err(e) => {
            eprintln!("Failed to publish: {}", e);
            eprintln!();
            eprintln!("To publish, set the KL_REGISTRY environment variable or ensure");
            eprintln!("the registry server is running. For local testing:");
            eprintln!("  export KL_REGISTRY=http://localhost:8080/v1");
            process::exit(1);
        }
    }
}

fn create_package_tarball(project_dir: &Path) -> Result<Vec<u8>, String> {
    let mut buf: Vec<u8> = Vec::new();
    let encoder = flate2::write::GzEncoder::new(&mut buf, flate2::Compression::default());
    let mut tar = tar::Builder::new(encoder);

    let manifest_path = project_dir.join("kl.toml");
    let manifest_content = fs::read_to_string(&manifest_path)
        .map_err(|e| format!("Failed to read kl.toml: {}", e))?;
    let mut header = tar::Header::new_ustar();
    header.set_path("kl.toml").map_err(|e| format!("tar header error: {}", e))?;
    header.set_size(manifest_content.len() as u64);
    header.set_mode(0o644);
    tar.append(&header, manifest_content.as_bytes())
        .map_err(|e| format!("tar append error: {}", e))?;

    let src_dir = project_dir.join("src");
    if src_dir.exists() {
        tar.append_dir_all("src", &src_dir)
            .map_err(|e| format!("Failed to add src/: {}", e))?;
    }

    let readme_path = project_dir.join("README.md");
    if readme_path.exists() {
        tar.append_path_with_name(&readme_path, "README.md")
            .map_err(|e| format!("Failed to add README.md: {}", e))?;
    }

    let encoder = tar.into_inner()
        .map_err(|e| format!("tar finalize error: {}", e))?;
    encoder.finish()
        .map_err(|e| format!("gzip finish error: {}", e))?;

    Ok(buf)
}

fn upload_package(name: &str, version: &str, data: &[u8]) -> Result<(), String> {
    let registry_url = std::env::var("KL_REGISTRY")
        .unwrap_or_else(|_| "https://registry.kyle-lang.org/v1".to_string());
    let url = format!("{}/packages/{}/{}/upload", registry_url, name, version);

    let resp = ureq::put(&url)
        .header("Content-Type", "application/gzip")
        .send(data)
        .map_err(|e| format!("Upload failed: {}", e))?;

    if resp.status() == 200 || resp.status() == 201 {
        Ok(())
    } else {
        Err(format!("Registry returned {} (expected 200/201)", resp.status()))
    }
}

fn cmd_login(_args: &[String]) {
    let registry_url = std::env::var("KL_REGISTRY")
        .unwrap_or_else(|_| "https://registry.kyle-lang.org/v1".to_string());

    println!("Login to {}", registry_url);
    println!("Enter your API key (or press Enter to skip):");

    let mut api_key = String::new();
    std::io::stdin().read_line(&mut api_key).unwrap_or_default();
    let api_key = api_key.trim();

    if api_key.is_empty() {
        println!("Skipped login.");
        return;
    }

    let verify_url = format!("{}/auth/verify", registry_url);
    match ureq::get(&verify_url)
        .header("Authorization", &format!("Bearer {}", api_key))
        .call()
    {
        Ok(resp) if resp.status() == 200 => {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            let config_dir = PathBuf::from(&home).join(".kl");
            fs::create_dir_all(&config_dir).unwrap_or_else(|e| {
                eprintln!("Warning: could not create {}: {}", config_dir.display(), e);
            });

            let config_path = config_dir.join("config.toml");
            let config_content = format!("registry = \"{}\"\napi_key = \"{}\"\n", registry_url, api_key);
            fs::write(&config_path, &config_content).unwrap_or_else(|e| {
                eprintln!("Warning: could not write {}: {}", config_path.display(), e);
            });

            println!("✅ Logged in successfully.");
            println!("   API key saved to {}", config_path.display());
        }
        Ok(resp) => {
            eprintln!("Login failed: registry returned {}", resp.status());
            process::exit(1);
        }
        Err(e) => {
            eprintln!("Could not contact registry: {}", e);
            eprintln!("API key not saved. You can set KL_API_KEY environment variable instead.");
            process::exit(1);
        }
    }
}

fn cmd_update(_args: &[String]) {
    let cwd = std::env::current_dir().unwrap();
    let project_root = match find_project_root(&cwd) {
        Some(r) => r,
        None => { eprintln!("No kl.toml found"); process::exit(1); }
    };
    let manifest = match Manifest::find_in_directory(&project_root) {
        Ok(m) => m,
        Err(e) => { eprintln!("{}", e); process::exit(1); }
    };

    println!("Updating dependencies...");
    match resolve_project_dependencies(&project_root, &manifest) {
        Ok(()) => println!("✅ Lock file updated."),
        Err(e) => { eprintln!("Update failed: {}", e); process::exit(1); }
    }
}

fn cmd_outdated(_args: &[String]) {
    let cwd = std::env::current_dir().unwrap();
    let project_root = match find_project_root(&cwd) {
        Some(r) => r,
        None => { eprintln!("No kl.toml found"); process::exit(1); }
    };
    let manifest = match Manifest::find_in_directory(&project_root) {
        Ok(m) => m,
        Err(e) => { eprintln!("{}", e); process::exit(1); }
    };
    let lock_path = project_root.join("kl.lock");
    let lock = LockFile::read(&lock_path).unwrap_or_default();

    if manifest.dependencies.is_empty() {
        println!("No dependencies.");
        return;
    }

    let registry = RegistryClient::new();
    println!("Checking for outdated dependencies...");
    println!();

    let mut any_outdated = false;
    for (dep_name, dep_req_str) in &manifest.dependencies {
        let current_version = lock.get_version(dep_name).unwrap_or("—");

        match registry.get_versions(dep_name) {
            Ok(versions) => {
                let latest = versions.last();
                let latest_str = latest.map(|v| format!("{}", v)).unwrap_or_else(|| "?".to_string());

                let is_outdated = match latest {
                    Some(latest_v) => {
                        if let Ok(current_v) = klc_core::semver::parse_version(current_version) {
                            latest_v > &current_v
                        } else {
                            false
                        }
                    }
                    None => false,
                };

                if is_outdated {
                    println!("  ⚠ {}  {}  →  {}  ({})", dep_name, current_version, latest_str, dep_req_str);
                    any_outdated = true;
                } else {
                    println!("  ✓ {}  {}  (latest)", dep_name, current_version);
                }
            }
            Err(_) => {
                println!("  ? {}  {}  (registry unavailable)", dep_name, current_version);
            }
        }
    }

    if !any_outdated {
        println!();
        println!("All dependencies are up to date.");
    } else {
        println!();
        println!("Run 'kl update' to update the lock file.");
    }
}

fn cmd_uninstall() {
    let home = std::env::var("HOME").unwrap_or_default();
    let targets = [
        "/usr/local/bin/kl",
        "/usr/local/lib/kl/libklc_runtime.a",
        &format!("{}/.kl/bin/kl", home),
        &format!("{}/.kl/lib/kl/libklc_runtime.a", home),
        &format!("{}/.kl/lib/libklc_runtime.a", home),
    ];
    let mut uninstalled = false;
    for target in &targets {
        if Path::new(target).exists() {
            let _ = fs::remove_file(target);
            println!("Removed {}", target);
            uninstalled = true;
        }
    }
    if uninstalled {
        println!("kl uninstalled.");
    } else {
        println!("kl is not installed.");
    }
}

fn load_source(args: &[String], index: usize) -> String {
    if args.len() <= index {
        eprintln!("Error: missing file argument");
        print_usage();
        process::exit(1);
    }
    let path = &args[index];
    match fs::read_to_string(path) {
        Ok(source) => source,
        Err(e) => {
            eprintln!("Error reading '{}': {}", path, e);
            process::exit(1);
        }
    }
}

fn ensure_main(source: &str) -> String {
    if source.contains("fn main(") || source.contains("fn main ") {
        source.to_string()
    } else {
        let body: String = source.lines()
            .map(|l| {
                let t = l.trim();
                if t.is_empty() || t.starts_with('#') {
                    l.to_string()
                } else {
                    format!("    {}", l)
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        format!("fn main() i32:\n{}\n    0", body)
    }
}
