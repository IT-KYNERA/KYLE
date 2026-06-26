// KL Compiler — CLI binary entry point
//
// Command-line interface for the KL compiler.
// Usage:
//   klc build <file.kl>    Compile KL source to native binary
//   klc run   <file.kl>    Compile and execute
//   klc check <file.kl>    Type-check without code generation
//   klc parse <file.kl>    Parse KL source and dump AST
//   klc mir   <file.kl>    Parse and dump MIR
//   klc fmt   <file.kl>    Format KL source code
//   klc new   <project>    Create new KL project
//   klc test                Run tests
//   klc lsp                 Start language server
//   klc add     <dep>       Add dependency
//   klc remove  <dep>       Remove dependency
//   klc info                Show project info
//   klc help                Show this help

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

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
            println!("klc v{}", env!("CARGO_PKG_VERSION"));
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
        eprintln!("Usage: klc new <project>");
        process::exit(1);
    }
    let project_name = &args[2];
    let project_dir = Path::new(project_name);
    if project_dir.exists() {
        eprintln!("Error: directory '{}' already exists", project_name);
        process::exit(1);
    }
    fs::create_dir_all(project_dir.join("src")).unwrap_or_else(|e| {
        eprintln!("Error creating project: {}", e);
        process::exit(1);
    });
    let _ = fs::create_dir(project_dir.join("tests"));
    let main_kl = format!(
        "fn main() -> i32:\n    println(\"Hello, {}!\")\n    return 0\n",
        project_name
    );
    let manifest = format!(
        "name = \"{}\"\nversion = \"0.1.0\"\nedition = \"1\"\nauthors = []\nlicense = \"MIT\"\n\n[compiler]\noptimization = \"O2\"\ntarget = \"native\"\n\n[dependencies]\n",
        project_name
    );
    fs::write(project_dir.join("src").join("main.kl"), &main_kl).unwrap_or_else(|e| {
        eprintln!("Error writing main.kl: {}", e);
        process::exit(1);
    });
    fs::write(project_dir.join("kl.toml"), &manifest).unwrap_or_else(|e| {
        eprintln!("Error writing kl.toml: {}", e);
        process::exit(1);
    });
    println!("Created project '{}'", project_name);
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
    let name = env::args().next().unwrap_or_else(|| "klc".to_string());
    eprintln!("KL Compiler v{}", env!("CARGO_PKG_VERSION"));
    eprintln!();
    eprintln!("Usage:");
    eprintln!("  {name} build <file.kl>    Compile to native binary");
    eprintln!("  {name} run   <file.kl>    Compile and execute");
    eprintln!("  {name} check <file.kl>    Type-check without codegen");
    eprintln!("  {name} parse <file.kl>    Parse and dump AST");
    eprintln!("  {name} mir   <file.kl>    Parse and dump MIR");
    eprintln!("  {name} fmt   <file.kl>    Format KL source code");
    eprintln!("  {name} new   <project>    Create new KL project");
    eprintln!("  {name} add   <dep>        Add dependency");
    eprintln!("  {name} remove <dep>        Remove dependency");
    eprintln!("  {name} info               Show project info");
    eprintln!("  {name} lsp               Start LSP server (stdio)");
    eprintln!("  {name} test               Run tests");
    eprintln!("  {name} help               Show this help");
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
            eprintln!("Usage: klc build <file.kl>");
            process::exit(1);
        }
        return;
    }
    // Single-file mode
    let source = load_source(args, 2);
    let file = &args[2];
    let file_stem = std::path::Path::new(file).file_stem()
        .unwrap_or_default().to_string_lossy().to_string();
    let build_dir = std::path::Path::new(".klc-build");
    let output = exe_path(&build_dir.join(&file_stem));
    let _ = fs::create_dir_all(build_dir);
    match klc_driver::pipeline::Pipeline::build_source_with_artifacts(&source, file, &output, build_dir) {
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
    if args.len() < 3 {
        // Project mode
        if let Some(project_root) = klc_tools::package::find_project_root(&std::env::current_dir().unwrap()) {
            if let Some(source_path) = klc_tools::package::main_source_path(&project_root) {
                let source = fs::read_to_string(&source_path).unwrap_or_else(|e| {
                    eprintln!("Error reading {}: {}", source_path.display(), e);
                    process::exit(1);
                });
                let file = source_path.to_string_lossy().to_string();
                let build_dir = project_root.join("target").join("debug");
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
            eprintln!("Usage: klc run <file.kl>");
            process::exit(1);
        }
        return;
    }
    // Single-file mode
    let source = load_source(args, 2);
    let file = &args[2];
    let file_stem = std::path::Path::new(file).file_stem()
        .unwrap_or_default().to_string_lossy().to_string();
    let build_dir = std::path::Path::new(".klc-build");
    let output = exe_path(&build_dir.join(&file_stem));
    let _ = fs::create_dir_all(build_dir);
    match klc_driver::pipeline::Pipeline::build_source_with_artifacts(&source, file, &output, build_dir) {
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
        eprintln!("Usage: klc add <dependency>[@version]");
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
        eprintln!("Usage: klc remove <dependency>");
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
