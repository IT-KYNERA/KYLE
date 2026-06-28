use std::path::{Path, PathBuf};
use klc_core::ast::Program;
use klc_core::source_map::SourceMap;
use klc_frontend::lexer::Lexer;
use klc_frontend::parser::Parser;
use klc_semantic::analyzer::SemanticAnalyzer;
use klc_semantic::module_resolver::ModuleResolver;
use klc_mir::mir::MirModule;
use klc_mir::lower::Lowerer;
use klc_mir::optimize::Optimizer;
use klc_mir::ownership::OwnershipPass;
use klc_backend::codegen::Codegen;
use klc_backend::linker::Linker;
use klc_tools::package::find_project_root;
use inkwell::context::Context;
use inkwell::targets::{FileType, InitializationConfig, Target, TargetMachine};
use inkwell::OptimizationLevel;
use std::io::Write;

#[derive(Default)]
pub struct Pipeline;

impl Pipeline {
    pub fn parse_source(source: &str) -> Result<ParsedOutput, String> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let program = parser.parse()?;
        Ok(ParsedOutput { program })
    }

    fn resolve_imports(program: &mut Program, file_name: &str) -> Result<(), String> {
        let base_dir = PathBuf::from(file_name)
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));

        let mut resolver = ModuleResolver::new();
        resolver.set_source_dir(base_dir.clone());
        resolver.add_search_path(base_dir.clone());

        if let Some(project_root) = find_project_root(&base_dir) {
            let src_dir = project_root.join("src");
            if src_dir.exists() && src_dir != base_dir {
                resolver.add_search_path(src_dir);
            }
        }

        if let Some(cwd) = std::env::current_dir().ok() {
            let std_path = cwd.join("std");
            if std_path.exists() {
                resolver.add_search_path(std_path);
            }
        }
        let local_std = base_dir.join("std");
        if local_std.exists() {
            resolver.add_search_path(local_std);
        }

        let mut import_decls: Vec<(usize, Vec<klc_core::ast::Decl>)> = Vec::new();
        let mut scope_registrations: Vec<(String, String)> = Vec::new();

        for (i, decl) in program.declarations.iter().enumerate() {
            match decl {
                klc_core::ast::Decl::Import(imp) => {
                    let module_decls = resolver.get_module_declarations(&imp.module_name, imp.relative)?;
                    let scope_name = imp.alias.clone().unwrap_or_else(|| imp.module_name.clone());
                    scope_registrations.push((scope_name, imp.module_name.clone()));
                    import_decls.push((i, module_decls));
                }
                klc_core::ast::Decl::FromImport(fi) => {
                    let decl = resolver.get_imported_declaration(&fi.module_name, &fi.imported_name, fi.relative)?;
                    scope_registrations.push((fi.imported_name.clone(), fi.module_name.clone()));
                    import_decls.push((i, vec![decl]));
                }
                _ => {}
            }
        }

        for (idx, decls) in import_decls.into_iter().rev() {
            let mut rest = program.declarations.split_off(idx + 1);
            program.declarations.extend(decls);
            program.declarations.append(&mut rest);
        }

        // Transitive import resolution: resolve imports within each cached
        // module so that parent class declarations become available.
        // For example, `from ~entities.employee import Employee` parses
        // entities/employee.kl which itself has `from ~base import BaseEntity`.
        // We need to resolve that inner import so BaseEntity's declaration
        // is available when lowering Employee's class fields.
        let cached_keys: Vec<String> = resolver.cache.keys().cloned().collect();
        for cache_key in &cached_keys {
            // Save resolver's source_dir and temporarily set it to the
            // cached module's directory for correct relative resolution.
            let old_source_dir = resolver.source_dir.clone();
            let module_dir = {
                if let Some(module) = resolver.cache.get(cache_key) {
                    module.path.parent().map(|p| p.to_path_buf())
                } else { None }
            };
            if let Some(ref dir) = module_dir {
                resolver.source_dir = Some(dir.clone());
            }

            // Collect FromImport info from this cached module
            let import_info: Vec<(usize, String, String, bool)> = {
                if let Some(module) = resolver.cache.get(cache_key) {
                    module.program.declarations.iter().enumerate()
                        .filter_map(|(i, d)| {
                            if let klc_core::ast::Decl::FromImport(fi) = d {
                                Some((i, fi.module_name.clone(), fi.imported_name.clone(), fi.relative))
                            } else { None }
                        })
                        .collect()
                } else { Vec::new() }
            };

            // Resolve each import
            let mut import_decls: Vec<(usize, Vec<klc_core::ast::Decl>)> = Vec::new();
            for (i, mod_name, imported_name, rel) in import_info {
                if let Ok(decl) = resolver.get_imported_declaration(&mod_name, &imported_name, rel) {
                    import_decls.push((i, vec![decl]));
                }
            }

            // Splice resolved declarations into the cached module
            for (idx, decls) in import_decls.into_iter().rev() {
                if let Some(module) = resolver.cache.get_mut(cache_key) {
                    let mut rest = module.program.declarations.split_off(idx + 1);
                    module.program.declarations.extend(decls);
                    module.program.declarations.append(&mut rest);
                }
            }

            resolver.source_dir = old_source_dir;
        }

        // Now pull any class declarations from cached modules that serve as
        // parent classes for classes already in program.declarations but whose
        // declarations aren't yet in the program.
        loop {
            let class_names: std::collections::HashSet<String> = program.declarations.iter()
                .filter_map(|d| {
                    if let klc_core::ast::Decl::Class(c) = d { Some(c.name.clone()) }
                    else { None }
                })
                .collect();
            let needed_parents: Vec<String> = program.declarations.iter()
                .filter_map(|d| {
                    if let klc_core::ast::Decl::Class(c) = d {
                        c.parent.as_ref().and_then(|p| {
                            if class_names.contains(p) { None } else { Some(p.clone()) }
                        })
                    } else { None }
                })
                .collect();

            if needed_parents.is_empty() { break; }

            let mut any_added = false;
            for cached in resolver.cache.values() {
                for cd in &cached.program.declarations {
                    if let klc_core::ast::Decl::Class(pc) = cd {
                        if needed_parents.contains(&pc.name) && !class_names.contains(&pc.name) {
                            program.declarations.push(cd.clone());
                            any_added = true;
                        }
                    }
                }
            }
            if !any_added { break; }
        }

        Ok(())
    }

    pub fn check_source(source: &str, file_name: &str) -> Result<CheckedOutput, String> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let mut program = parser.parse()?;

        Self::resolve_imports(&mut program, file_name)?;

        let mut source_map = SourceMap::new();
        let file_id = source_map.add(file_name.to_string(), source.to_string());

        let mut analyzer = SemanticAnalyzer::new()
            .with_source(source_map, file_name.to_string());
        analyzer.analyze(&program);

        Ok(CheckedOutput {
            program,
            analyzer,
            file_id,
        })
    }

    pub fn mir_source(source: &str, file_name: &str) -> Result<MirOutput, String> {
        let checked = Self::check_source(source, file_name)?;

        let lowerer = Lowerer::new();
        let mut module = lowerer.lower_program(&checked.program);

        let optimizer = Optimizer::new();
        optimizer.optimize(&mut module);

        let ownership = OwnershipPass::new();
        ownership.run(&mut module);

        Ok(MirOutput {
            module,
            analyzer: checked.analyzer,
        })
    }

    /// Build source: output binary goes to `output_path`, intermediary files
    /// (.o, .ll) go to `artifact_dir` (defaults to output_path's directory).
    pub fn build_source(source: &str, file_name: &str, output_path: &Path) -> Result<(), String> {
        let default_dir = output_path.parent().unwrap_or_else(|| Path::new("."));
        Self::build_source_with_artifacts(source, file_name, output_path, default_dir)
    }

    /// Build source with explicit artifact directory for .o / .ll files.
    pub fn build_source_with_artifacts(
        source: &str,
        file_name: &str,
        output_path: &Path,
        artifact_dir: &Path,
    ) -> Result<(), String> {
        let mir = Self::mir_source(source, file_name)?;

        if mir.analyzer.has_errors() {
            mir.analyzer.emit_diagnostics();
            return Err("Type-check errors".to_string());
        }

        let context = Context::create();
        let mut codegen = Codegen::new(&context, "kl_module");
        codegen.compile(&mir.module)?;

        // Ensure artifact directory exists
        std::fs::create_dir_all(artifact_dir)
            .map_err(|e| format!("Failed to create artifact dir: {}", e))?;

        let stem = output_path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("kl_out");

        // LLVM IR dump
        let ir_path = artifact_dir.join(format!("{}.ll", stem));
        let ir_str = codegen.module().print_to_string().to_string();
        if let Ok(mut f) = std::fs::File::create(&ir_path) {
            let _ = write!(f, "{}", ir_str);
        }

        // Object file
        let obj_path = artifact_dir.join(format!("{}.o", stem));
        emit_object(codegen.module(), &obj_path)?;

        // Link
        let linker = Linker::new();
        let runtime_lib = Linker::find_runtime_lib();
        linker.link(&[&obj_path], output_path, runtime_lib.as_deref())
            .map_err(|e| format!("Link error: {}", e))?;

        Ok(())
    }
}

/// Emit a native object file from an LLVM module using TargetMachine.
fn emit_object(module: &inkwell::module::Module, path: &Path) -> Result<(), String> {
    Target::initialize_all(&InitializationConfig::default());

    let triple = TargetMachine::get_default_triple();
    let triple_str = triple.as_str().to_str().unwrap_or("x86_64-unknown-linux-gnu");
    let target = Target::from_triple(&triple)
        .map_err(|e| format!("Failed to get target '{}': {}", triple_str, e))?;
    let target_machine = target.create_target_machine(
        &triple,
        "generic",
        "",
        OptimizationLevel::Default,
        inkwell::targets::RelocMode::PIC,
        inkwell::targets::CodeModel::Default,
    ).ok_or_else(|| "Failed to create target machine".to_string())?;

    target_machine.write_to_file(module, FileType::Object, path)
        .map_err(|e| format!("Failed to emit object file: {}", e))
}

pub struct ParsedOutput {
    pub program: Program,
}

pub struct CheckedOutput {
    pub program: Program,
    pub analyzer: SemanticAnalyzer,
    pub file_id: usize,
}

pub struct MirOutput {
    pub module: MirModule,
    pub analyzer: SemanticAnalyzer,
}
