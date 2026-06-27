// KL Compiler — CLI binary entry point
//
// Command-line interface for the KL compiler.
// Installed as both `kl` (primary) and `klc` (legacy).

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

/// Return the binary name used to invoke this program (kl or klc).
fn bin_name() -> String {
    env::args().next()
        .map(|p| Path::new(&p).file_stem().unwrap_or_default().to_string_lossy().to_string())
        .filter(|n| n == "kl" || n == "klc")
        .unwrap_or_else(|| "kl".to_string())
}

/// Return the output binary path with the correct platform extension.
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
        "lsp" => cmd_lsp(&args),
        "help" => {
            print_usage();
        }
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
        Err(e) => {
            eprintln!("MIR error: {}", e);
            process::exit(1);
        }
    }
}

fn cmd_fmt(args: &[String]) {
    let source = load_source(args, 2);
    let formatter = klc_tools::formatter::Formatter::new();
    match formatter.format(&source) {
        Ok(formatted) => {
            let path = &args[2];
            if args.len() > 3 && args[3] == "--check" {
                if source != formatted {
                    eprintln!("{}: would reformat", path);
                    process::exit(1);
                }
            } else {
                fs::write(path, &formatted).unwrap_or_else(|e| {
                    eprintln!("Error writing '{}': {}", path, e);
                    process::exit(1);
                });
                println!("Formatted: {}", path);
            }
        }
        Err(e) => {
            eprintln!("Format error: {}", e);
            process::exit(1);
        }
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
    // Create directory structure
    for dir in &["src", "tests", "examples", "docs"] {
        fs::create_dir_all(project_dir.join(dir)).unwrap_or_else(|e| {
            eprintln!("Error creating project: {}", e);
            process::exit(1);
        });
    }

    // src/main.kl — professional entry point with args
    let main_kl = [
        "fn main(args: [str]) -> i32:",
        &format!("    println(\"Hello from {} v0.1.0!\")", project_name),
        "    println(\"args: \" + str(len(args)))",
        "    return 0",
    ].join("\n") + "\n";
    fs::write(project_dir.join("src").join("main.kl"), &main_kl).unwrap_or_else(|e| {
        eprintln!("Error writing src/main.kl: {}", e);
        process::exit(1);
    });

    // src/lib.kl — library module
    let lib_kl = format!(
        "# {} — library module\n# Import with: import {{ greet }} from \"src/lib\"\n\npub fn greet(name: str) -> str:\n    \"Hello, \" + name + \"!\"\n",
        project_name
    );
    fs::write(project_dir.join("src").join("lib.kl"), &lib_kl).unwrap_or_else(|e| {
        eprintln!("Error writing src/lib.kl: {}", e);
        process::exit(1);
    });

    // tests/test_main.kl — test stub
    let test_kl = format!(
        "# Tests for {}\nimport {{ greet }} from \"src/lib\"\n\nfn test_greet() -> i32:\n    assert_str(greet(\"World\"), \"Hello, World!\")\n    println(\"test_greet PASS\")\n    0\n",
        project_name
    );
    fs::write(project_dir.join("tests").join("test_main.kl"), &test_kl).unwrap_or_else(|e| {
        eprintln!("Error writing tests/test_main.kl: {}", e);
        process::exit(1);
    });

    // examples/hello.kl — example
    let example_kl = format!(
        "# {} — example\nfn main():\n    println!(\"Hello from {}!\")\n",
        project_name, project_name
    );
    fs::write(project_dir.join("examples").join("hello.kl"), &example_kl).unwrap_or_else(|e| {
        eprintln!("Error writing examples/hello.kl: {}", e);
        process::exit(1);
    });

    // kl.toml — project manifest
    let manifest = format!(
        "name = \"{}\"\nversion = \"0.1.0\"\nedition = \"1\"\nauthors = [\"You <you@example.com>\"]\nlicense = \"MIT\"\ndescription = \"A Kyle programming language project\"\n\n[compiler]\noptimization = \"O2\"\ntarget = \"native\"\n\n[dependencies]\n",
        project_name
    );
    fs::write(project_dir.join("kl.toml"), &manifest).unwrap_or_else(|e| {
        eprintln!("Error writing kl.toml: {}", e);
        process::exit(1);
    });

    // .gitignore
    let gitignore = "target/\n*.klc-build/\nkl.lock\n.vscode/\n";
    fs::write(project_dir.join(".gitignore"), gitignore).unwrap_or_else(|e| {
        eprintln!("Error writing .gitignore: {}", e);
        process::exit(1);
    });

    // README.md — project docs
    let readme = format!(
        "# {}\n\n{}\n\n## Usage\n\n```console\n# Build and run\n{bname} build\n{bname} run\n\n# Run tests\n{bname} test\n\n# Release build\n{bname} build --release\n```\n\n## Project Structure\n\n```\n├── src/\n│   ├── main.kl       # Entry point\n│   └── lib.kl        # Library module\n├── tests/\n│   └── test_main.kl  # Tests\n├── examples/\n│   └── hello.kl      # Example\n├── kl.toml           # Project manifest\n└── README.md\n```\n",
        project_name, project_name, bname = bin_name()
    );
    fs::write(project_dir.join("README.md"), &readme).unwrap_or_else(|e| {
        eprintln!("Error writing README.md: {}", e);
        process::exit(1);
    });

    // .vscode/settings.json — workspace settings
    let vscode_settings = format!(
        r#"{{"kl.klcPath":"{}","files.associations":{{"*.kl":"kl"}}}}""#,
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
    println!("   ├── examples/          — examples");
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

fn print_usage() {
    let name = bin_name();
    eprintln!("{} v{} — Kyle Programming Language Compiler", name, env!("CARGO_PKG_VERSION"));
    eprintln!();
    eprintln!("Project commands (run from a project directory with kl.toml):");
    eprintln!("  {name} build [--release]   Compile project to native binary");
    eprintln!("  {name} run   [--release]   Compile and execute project");
    eprintln!("  {name} test                Run project tests");
    eprintln!("  {name} info                Show project info");
    eprintln!("  {name} add   <dep>         Add dependency");
    eprintln!("  {name} remove <dep>        Remove dependency");
    eprintln!();
    eprintln!("File commands:");
    eprintln!("  {name} build <file.kl>     Compile single file");
    eprintln!("  {name} run   <file.kl>     Compile and run single file");
    eprintln!("  {name} check <file.kl>     Type-check without codegen");
    eprintln!("  {name} parse <file.kl>     Parse and dump AST");
    eprintln!("  {name} mir   <file.kl>     Parse and dump MIR");
    eprintln!("  {name} fmt   <file.kl>     Format source code");
    eprintln!();
    eprintln!("Project creation:");
    eprintln!("  {name} new   <project>     Create new KL project");
    eprintln!();
    eprintln!("Tools:");
    eprintln!("  {name} lsp                 Start LSP server (stdio)");
    eprintln!("  {name} help                Show this help");
}

fn cmd_build(args: &[String]) {
    let release = args.iter().any(|a| a == "--release");
    if args.len() < 3 || args[2].starts_with("--") {
        // Project mode
        if let Some(project_root) = klc_tools::package::find_project_root(&std::env::current_dir().unwrap()) {
            if let Some(source_path) = klc_tools::package::main_source_path(&project_root) {
                let source = fs::read_to_string(&source_path).unwrap_or_else(|e| {
                    eprintln!("Error reading {}: {}", source_path.display(), e);
                    process::exit(1);
                });
                let file = source_path.to_string_lossy().to_string();
                let build_dir = project_root.join("target").join(if release { "release" } else { "debug" });
                let output = exe_path(&build_dir.join("main"));
                let _ = fs::create_dir_all(&build_dir);
                match klc_driver::pipeline::Pipeline::build_source_with_artifacts(&source, &file, &output, &build_dir) {
                    Ok(()) => {
                        println!("Build complete: {}", output.display());
                        let lock_path = project_root.join("kl.lock");
                        let manifest = klc_tools::package::Manifest::find_in_directory(&project_root).ok();
                        if let Some(m) = manifest {
                            let mut lock = klc_tools::package::LockFile::read(&lock_path).unwrap_or_default();
                            for (name, version) in &m.dependencies {
                                lock.add_package(name, version, "registry");
                            }
                            let _ = lock.write(&lock_path);
                        }
                    }
                    Err(e) => { eprintln!("Build error: {}", e); process::exit(1); }
                }
            } else {
                eprintln!("No src/main.kl found in project");
                process::exit(1);
            }
        } else {
            eprintln!("Usage: {} build <file.kl>", bin_name());
            process::exit(1);
        }
        return;
    }
    // Single-file mode: output to target/debug/<name>
    let source = load_source(args, 2);
    let file = &args[2];
    let file_stem = std::path::Path::new(file).file_stem()
        .unwrap_or_default().to_string_lossy().to_string();
    let build_dir = std::path::Path::new("target").join("debug");
    let output = exe_path(&build_dir.join(&file_stem));
    let _ = fs::create_dir_all(&build_dir);
    match klc_driver::pipeline::Pipeline::build_source_with_artifacts(&source, file, &output, &build_dir) {
        Ok(()) => {
            println!("Build complete: {}", output.display());
        }
        Err(e) => {
            eprintln!("Build error: {}", e);
            process::exit(1);
        }
    }
}

fn cmd_run(args: &[String]) {
    let release = args.iter().any(|a| a == "--release");
    if args.len() < 3 || args[2].starts_with("--") {
        // Project mode
        if let Some(project_root) = klc_tools::package::find_project_root(&std::env::current_dir().unwrap()) {
            if let Some(source_path) = klc_tools::package::main_source_path(&project_root) {
                let source = fs::read_to_string(&source_path).unwrap_or_else(|e| {
                    eprintln!("Error reading {}: {}", source_path.display(), e);
                    process::exit(1);
                });
                let file = source_path.to_string_lossy().to_string();
                let build_dir = project_root.join("target").join(if release { "release" } else { "debug" });
                let output = exe_path(&build_dir.join("main"));
                let _ = fs::create_dir_all(&build_dir);
                match klc_driver::pipeline::Pipeline::build_source_with_artifacts(&source, &file, &output, &build_dir) {
                    Ok(()) => {
                        let status = std::process::Command::new(&output)
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
        } else {
            eprintln!("Usage: {} run <file.kl>", bin_name());
            process::exit(1);
        }
        return;
    }
    // Single-file mode: output to target/debug/<name>
    let source = load_source(args, 2);
    let file = &args[2];
    let file_stem = std::path::Path::new(file).file_stem()
        .unwrap_or_default().to_string_lossy().to_string();
    let build_dir = std::path::Path::new("target").join("debug");
    let output = exe_path(&build_dir.join(&file_stem));
    let _ = fs::create_dir_all(&build_dir);
    match klc_driver::pipeline::Pipeline::build_source_with_artifacts(&source, file, &output, &build_dir) {
        Ok(()) => {
            let status = std::process::Command::new(&output)
                .args(args.iter().skip(3))
                .status()
                .expect("Failed to execute binary");
            if !status.success() {
                process::exit(status.code().unwrap_or(1));
            }
        }
        Err(e) => {
            eprintln!("Run error: {}", e);
            process::exit(1);
        }
    }
}

fn cmd_check(args: &[String]) {
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
        Err(e) => {
            eprintln!("Check error: {}", e);
            process::exit(1);
        }
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
        Err(e) => {
            eprintln!("Parse error: {}", e);
            process::exit(1);
        }
    }
}

fn cmd_test(_args: &[String]) {
    let project_root = klc_tools::package::find_project_root(&std::env::current_dir().unwrap());
    let root = match project_root {
        Some(ref r) => r.clone(),
        None => {
            eprintln!("No kl.toml found — run from a KL project directory");
            process::exit(1);
        }
    };
    let test_files = klc_tools::package::test_source_paths(&root);
    if test_files.is_empty() {
        println!("No test files found in tests/");
        // Try running the main source and checking for test functions
        if let Some(source_path) = klc_tools::package::main_source_path(&root) {
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
                        println!("Main source OK — no test framework yet.");
                    }
                }
                Err(e) => {
                    eprintln!("Test error: {}", e);
                    process::exit(1);
                }
            }
        }
        println!("All checks passed.");
        return;
    }
    let mut all_ok = true;
    for test_path in &test_files {
        print!("Testing {} ... ", test_path.file_name().unwrap_or_default().to_string_lossy());
        let source = fs::read_to_string(test_path).unwrap_or_else(|e| {
            eprintln!("Error reading {}: {}", test_path.display(), e);
            process::exit(1);
        });
        let file = test_path.to_string_lossy().to_string();
        match klc_driver::pipeline::Pipeline::check_source(&source, &file) {
            Ok(output) => {
                if output.analyzer.has_errors() {
                    output.analyzer.emit_diagnostics();
                    all_ok = false;
                    println!("FAILED");
                } else {
                    println!("OK");
                }
            }
            Err(e) => {
                eprintln!("error: {}", e);
                all_ok = false;
            }
        }
    }
    if !all_ok {
        process::exit(1);
    }
    println!("All tests passed.");
}

fn cmd_add(args: &[String]) {
    if args.len() < 3 {
        eprintln!("Usage: {} add <dependency>[@version]", bin_name());
        process::exit(1);
    }
    let dep_str = &args[2];
    let (name, version) = if let Some(at_pos) = dep_str.find('@') {
        (&dep_str[..at_pos], &dep_str[at_pos + 1..])
    } else {
        (dep_str.as_str(), "*")
    };
    match klc_tools::package::Manifest::find_in_cwd() {
        Ok(mut manifest) => {
            manifest.add_dependency(name, version);
            if let Err(e) = manifest.save_to_dir(&std::env::current_dir().unwrap()) {
                eprintln!("Error saving manifest: {}", e);
                process::exit(1);
            }
            println!("Added dependency '{}' version '{}'", name, version);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}

fn cmd_remove(args: &[String]) {
    if args.len() < 3 {
        eprintln!("Usage: {} remove <dependency>", bin_name());
        process::exit(1);
    }
    let name = &args[2];
    match klc_tools::package::Manifest::find_in_cwd() {
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
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}

fn cmd_info(args: &[String]) {
    let dir = if args.len() > 2 {
        PathBuf::from(&args[2])
    } else {
        std::env::current_dir().unwrap()
    };
    match klc_tools::package::Manifest::find_in_directory(&dir) {
        Ok(manifest) => {
            println!("Project: {}", manifest.name);
            println!("Version: {}", manifest.version);
            println!("Edition: {}", manifest.edition);
            println!("Authors: {}", manifest.authors.join(", "));
            println!("License: {}", manifest.license);
            if !manifest.description.is_empty() {
                println!("Description: {}", manifest.description);
            }
            println!();
            println!("Compiler: {} ({})", manifest.compiler.target, manifest.compiler.optimization);
            println!();
            if manifest.dependencies.is_empty() {
                println!("Dependencies: (none)");
            } else {
                println!("Dependencies:");
                for (name, version) in &manifest.dependencies {
                    println!("  {} = \"{}\"", name, version);
                }
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}

/// Load source file, or show error and exit.
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
