// klc_semantic::symbol_table — Semantic symbol table for module-level resolution
//
// Manages top-level declarations, imports, and module structure.
// Works together with ScopeTree for per-scope bindings.

use std::collections::HashMap;
use klc_core::ast::{Decl, Import, FromImport, FunctionDecl, ClassDecl, StructDecl, EnumDecl, ContractDecl, VariableDecl, ConstantDecl, TypeAlias, Program};
use klc_core::span::Span;
use klc_core::symbol::Symbol;
use klc_core::types::Type;
use crate::scope::{ScopeTree, Binding, BindingKind, Visibility};

pub struct ModuleSymbol {
    pub name: Symbol,
    pub declarations: Vec<Decl>,
    pub imports: Vec<Import>,
    pub from_imports: Vec<FromImport>,
    pub scope_id: usize,
}

pub struct SemanticSymbolTable {
    pub modules: HashMap<Symbol, ModuleSymbol>,
    pub scope_tree: ScopeTree,
    pub current_scope: usize,
    pub current_module: Option<Symbol>,
}

impl SemanticSymbolTable {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
            scope_tree: ScopeTree::new(),
            current_scope: 0,
            current_module: None,
        }
    }

    pub fn build_from_program(&mut self, program: &Program) -> Result<(), String> {
        self.current_scope = self.scope_tree.root_scope();
        self.register_builtins()?;
        for decl in &program.declarations {
            self.register_declaration(decl)?;
        }
        Ok(())
    }

    fn register_builtins(&mut self) -> Result<(), String> {
        let builtins: Vec<(&str, Type)> = vec![
            ("print", Type::Void),
            ("println", Type::Void),
            ("str", Type::Str),
            ("len", Type::I32),
            ("int", Type::I32),
            ("float", Type::F64),
            ("bool", Type::Bool),
            ("input", Type::Str),
            ("range", Type::List(Box::new(Type::I32))),
            ("contains", Type::Bool),
            ("to_upper", Type::Str),
            ("to_lower", Type::Str),
            ("trim", Type::Str),
            ("replace", Type::Str),
            ("open", Type::I32),
            ("read_str", Type::Str),
            ("write_str", Type::I32),
            ("close", Type::I32),
            ("sleep", Type::Void),
            ("now", Type::I64),
            ("char_at", Type::Char),
            ("is_digit", Type::Bool),
            ("is_alpha", Type::Bool),
            ("is_alnum", Type::Bool),
            ("is_whitespace", Type::Bool),
            ("is_upper", Type::Bool),
            ("is_lower", Type::Bool),
            ("ord", Type::I32),
            ("list_new", Type::I64),
            ("list_push", Type::Void),
            ("list_get", Type::I64),
            ("list_set", Type::Void),
            ("list_len", Type::I64),
            ("substr", Type::Str),
        ];
        for (name, return_type) in builtins {
            let binding = Binding {
                name: name.to_string(),
                symbol: 0,
                kind: BindingKind::Function,
                type_: None,
                resolved_type: Some(return_type),
                visibility: Visibility::Public,
                is_mutable: false,
                defined_at: Span::dummy(),
            };
            self.scope_tree.define(self.current_scope, binding)?;
        }
        Ok(())
    }

    fn register_declaration(&mut self, decl: &Decl) -> Result<(), String> {
        match decl {
            Decl::Import(_i) => {
                Ok(())
            }
            Decl::FromImport(_fi) => {
                Ok(())
            }
            Decl::Variable(v) => {
                self.register_variable(v)
            }
            Decl::Constant(c) => {
                self.register_constant(c)
            }
            Decl::Function(f) => {
                self.register_function(f)
            }
            Decl::Class(c) => {
                self.register_class(c)
            }
            Decl::AbstractClass(c) => {
                self.register_abstract_class(c)
            }
            Decl::Struct(s) => {
                self.register_struct(s)
            }
            Decl::Enum(e) => {
                self.register_enum(e)
            }
            Decl::Contract(c) => {
                self.register_contract(c)
            }
            Decl::TypeAlias(t) => {
                self.register_type_alias(t)
            }
        }
    }

    fn register_variable(&mut self, v: &VariableDecl) -> Result<(), String> {
        let binding = Binding {
            name: v.name.clone(),
            symbol: 0,
            kind: BindingKind::Variable,
            type_: v.type_.clone(),
            resolved_type: None,
            visibility: Visibility::from_name(&v.name),
            is_mutable: v.is_mutable,
            defined_at: v.span,
        };
        self.scope_tree.define(self.current_scope, binding)
    }

    fn register_constant(&mut self, c: &ConstantDecl) -> Result<(), String> {
        let binding = Binding {
            name: c.name.clone(),
            symbol: 0,
            kind: BindingKind::Constant,
            type_: None,
            resolved_type: None,
            visibility: Visibility::Public,
            is_mutable: false,
            defined_at: c.span,
        };
        self.scope_tree.define(self.current_scope, binding)
    }

    fn register_function(&mut self, f: &FunctionDecl) -> Result<(), String> {
        let binding = Binding {
            name: f.name.clone(),
            symbol: 0,
            kind: BindingKind::Function,
            type_: f.return_type.clone(),
            resolved_type: None,
            visibility: Visibility::from_name(&f.name),
            is_mutable: false,
            defined_at: f.span,
        };
        self.scope_tree.define(self.current_scope, binding)
    }

    fn register_class(&mut self, c: &ClassDecl) -> Result<(), String> {
        let binding = Binding {
            name: c.name.clone(),
            symbol: 0,
            kind: BindingKind::Class,
            type_: None,
            resolved_type: None,
            visibility: Visibility::from_name(&c.name),
            is_mutable: false,
            defined_at: c.span,
        };
        self.scope_tree.define(self.current_scope, binding)
    }

    fn register_abstract_class(&mut self, c: &klc_core::ast::AbstractClassDecl) -> Result<(), String> {
        let binding = Binding {
            name: c.name.clone(),
            symbol: 0,
            kind: BindingKind::AbstractClass,
            type_: None,
            resolved_type: None,
            visibility: Visibility::from_name(&c.name),
            is_mutable: false,
            defined_at: c.span,
        };
        self.scope_tree.define(self.current_scope, binding)
    }

    fn register_struct(&mut self, s: &StructDecl) -> Result<(), String> {
        let binding = Binding {
            name: s.name.clone(),
            symbol: 0,
            kind: BindingKind::Struct,
            type_: None,
            resolved_type: None,
            visibility: Visibility::from_name(&s.name),
            is_mutable: false,
            defined_at: s.span,
        };
        self.scope_tree.define(self.current_scope, binding)
    }

    fn register_enum(&mut self, e: &EnumDecl) -> Result<(), String> {
        let binding = Binding {
            name: e.name.clone(),
            symbol: 0,
            kind: BindingKind::Enum,
            type_: None,
            resolved_type: None,
            visibility: Visibility::from_name(&e.name),
            is_mutable: false,
            defined_at: e.span,
        };
        self.scope_tree.define(self.current_scope, binding)
    }

    fn register_contract(&mut self, c: &ContractDecl) -> Result<(), String> {
        let binding = Binding {
            name: c.name.clone(),
            symbol: 0,
            kind: BindingKind::Contract,
            type_: None,
            resolved_type: None,
            visibility: Visibility::from_name(&c.name),
            is_mutable: false,
            defined_at: c.span,
        };
        self.scope_tree.define(self.current_scope, binding)
    }

    fn register_type_alias(&mut self, t: &TypeAlias) -> Result<(), String> {
        let binding = Binding {
            name: t.name.clone(),
            symbol: 0,
            kind: BindingKind::TypeAlias,
            type_: Some(t.type_.clone()),
            resolved_type: None,
            visibility: Visibility::from_name(&t.name),
            is_mutable: false,
            defined_at: t.span,
        };
        self.scope_tree.define(self.current_scope, binding)
    }

    pub fn enter_scope(&mut self) -> usize {
        let new_scope = self.scope_tree.enter_child(self.current_scope);
        self.current_scope = new_scope;
        new_scope
    }

    pub fn leave_scope(&mut self) {
        if let Some(scope) = self.scope_tree.get_scope(self.current_scope) {
            if let Some(parent) = scope.parent {
                self.current_scope = parent;
            }
        }
    }

    pub fn lookup(&self, name: &str) -> Option<crate::scope::Binding> {
        self.scope_tree.lookup(self.current_scope, name)
            .map(|(_, b)| b.clone())
    }

    pub fn lookup_local(&self, name: &str) -> Option<crate::scope::Binding> {
        self.scope_tree.lookup_local(self.current_scope, name)
            .cloned()
    }

    pub fn lookup_local_mut(&mut self, name: &str) -> Option<&mut crate::scope::Binding> {
        let scope = self.scope_tree.get_scope_mut(self.current_scope)?;
        scope.bindings.get_mut(name)
    }

    pub fn define(&mut self, binding: Binding) -> Result<(), String> {
        self.scope_tree.define(self.current_scope, binding)
    }
}

impl Default for SemanticSymbolTable {
    fn default() -> Self {
        Self::new()
    }
}
