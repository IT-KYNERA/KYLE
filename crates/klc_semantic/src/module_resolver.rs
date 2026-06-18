// klc_semantic::module_resolver — Module resolution for imports
//
// Resolves `import X` and `from X import Y` declarations by finding
// and parsing the corresponding .kl files.

use std::path::PathBuf;
use std::fs;
use std::collections::HashMap;
use klc_core::ast::{Decl, Program};
use klc_frontend::lexer::Lexer;
use klc_frontend::parser::Parser;

/// A resolved module with its declarations.
pub struct ResolvedModule {
    pub name: String,
    pub program: Program,
    pub path: PathBuf,
}

/// Resolves import declarations by finding and parsing .kl files.
pub struct ModuleResolver {
    /// Search paths for module resolution.
    pub search_paths: Vec<PathBuf>,
    /// Cache of already resolved modules (name -> module).
    pub cache: HashMap<String, ResolvedModule>,
}

impl ModuleResolver {
    pub fn new() -> Self {
        Self {
            search_paths: Vec::new(),
            cache: HashMap::new(),
        }
    }

    /// Add a directory to search for modules.
    pub fn add_search_path(&mut self, path: PathBuf) {
        if !self.search_paths.contains(&path) {
            self.search_paths.push(path);
        }
    }

    /// Resolve an `import X` declaration.
    /// Returns the parsed module and its declarations.
    pub fn resolve_import(&mut self, module_name: &str) -> Result<&ResolvedModule, String> {
        if self.cache.contains_key(module_name) {
            return Ok(self.cache.get(module_name).unwrap());
        }

        let path = self.find_module_file(module_name)?;
        let source = fs::read_to_string(&path)
            .map_err(|e| format!("cannot read module '{}': {}", module_name, e))?;

        let mut lexer = Lexer::new(&source);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let program = parser.parse()
            .map_err(|e| format!("parse error in module '{}': {}", module_name, e))?;

        let module = ResolvedModule {
            name: module_name.to_string(),
            program,
            path,
        };
        self.cache.insert(module_name.to_string(), module);
        Ok(self.cache.get(module_name).unwrap())
    }

    /// Find a module file on the search paths.
    fn find_module_file(&self, module_name: &str) -> Result<PathBuf, String> {
        let file_name = format!("{}.kl", module_name);
        for search_path in &self.search_paths {
            let candidate = search_path.join(&file_name);
            if candidate.exists() {
                return Ok(candidate);
            }
        }
        Err(format!("module '{}' not found in search paths", module_name))
    }

    /// Get declarations from a module (for `import X` — all decls, module-qualified).
    pub fn get_module_declarations(&mut self, module_name: &str) -> Result<Vec<Decl>, String> {
        let module = self.resolve_import(module_name)?;
        Ok(module.program.declarations.clone())
    }

    /// Get a specific declaration from a module (for `from X import Y`).
    pub fn get_imported_declaration(&mut self, module_name: &str, name: &str) -> Result<Decl, String> {
        let module = self.resolve_import(module_name)?;
        for decl in &module.program.declarations {
            let decl_name = match decl {
                Decl::Function(f) => &f.name,
                Decl::Variable(v) => &v.name,
                Decl::Constant(c) => &c.name,
                Decl::Class(c) => &c.name,
                Decl::AbstractClass(c) => &c.name,
                Decl::Struct(s) => &s.name,
                Decl::Enum(e) => &e.name,
                Decl::Contract(c) => &c.name,
                Decl::TypeAlias(t) => &t.name,
                Decl::Import(_) | Decl::FromImport(_) => continue,
            };
            if decl_name == name {
                return Ok(decl.clone());
            }
        }
        Err(format!("'{}' not found in module '{}'", name, module_name))
    }
}

impl Default for ModuleResolver {
    fn default() -> Self {
        Self::new()
    }
}
