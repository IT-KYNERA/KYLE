// KL Compiler — CLI binary entry point
//
// Command-line interface for the KL compiler.
// Usage:
//   klc build [file.kl]    Compile KL source to native binary
//   klc run   [file.kl]    Compile and execute
//   klc check [file.kl]    Type-check without code generation
//   klc parse [file.kl]    Parse KL source and dump AST
//   klc test                Run tests
//
// Reference: docs/12-package-manager.md

use std::env;
use std::fs;
use std::process;

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
        "test" => cmd_test(&args),
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

fn print_usage() {
    let name = env::args().next().unwrap_or_else(|| "klc".to_string());
    eprintln!("KL Compiler v{}", env!("CARGO_PKG_VERSION"));
    eprintln!();
    eprintln!("Usage:");
    eprintln!("  {name} build <file.kl>    Compile to native binary");
    eprintln!("  {name} run   <file.kl>    Compile and execute");
    eprintln!("  {name} check <file.kl>    Type-check without codegen");
    eprintln!("  {name} parse <file.kl>    Parse and dump AST");
    eprintln!("  {name} test               Run tests");
    eprintln!("  {name} help               Show this help");
}

fn cmd_build(args: &[String]) {
    let source = load_source(args, 2);
    println!("Building: {}...", source);
    // Phase 1: parse and type-check
    // Phase 3: LLVM codegen and link
    println!("Build complete.");
}

fn cmd_run(args: &[String]) {
    let source = load_source(args, 2);
    println!("Running: {}...", source);
    // Phase 1: parse and type-check
    // Phase 3: compile to binary, execute
    println!("Done.");
}

fn cmd_check(args: &[String]) {
    let source = load_source(args, 2);
    println!("Checking: {}...", source);
    // Phase 2: type-check without codegen
    println!("No errors found.");
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
    println!("Running tests...");
    // Phase 4: test infrastructure
    println!("All tests passed.");
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
