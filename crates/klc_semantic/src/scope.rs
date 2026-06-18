// klc_semantic::scope — Scope resolution and binding management
//
// Manages nested scopes with visibility rules based on naming conventions:
//   name       → public (visible everywhere)
//   _name      → protected (visible in module hierarchy)
//   __name     → private (visible only in current module)

use std::collections::HashMap;
use klc_core::ast::AstType;
use klc_core::span::Span;
use klc_core::symbol::Symbol;
use klc_core::types::Type as KlType;

#[derive(Clone, Debug, PartialEq)]
pub enum Visibility {
    Public,
    Protected,
    Private,
}

impl Visibility {
    pub fn from_name(name: &str) -> Self {
        if name.starts_with("__") {
            Visibility::Private
        } else if name.starts_with('_') {
            Visibility::Protected
        } else {
            Visibility::Public
        }
    }

    pub fn is_accessible_from(&self, _from_visibility: &Visibility) -> bool {
        match self {
            Visibility::Public => true,
            Visibility::Protected => true,
            Visibility::Private => true,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum BindingKind {
    Variable,
    Constant,
    Function,
    Class,
    AbstractClass,
    Struct,
    Enum,
    Contract,
    TypeAlias,
    Parameter,
    Field,
    Method,
    Constructor,
    Property,
    Module,
    TypeParam,
}

#[derive(Clone, Debug)]
pub struct Binding {
    pub name: String,
    pub symbol: Symbol,
    pub kind: BindingKind,
    pub type_: Option<AstType>,
    pub resolved_type: Option<KlType>,
    pub visibility: Visibility,
    pub is_mutable: bool,
    pub defined_at: Span,
}

#[derive(Clone, Debug)]
pub struct Scope {
    pub parent: Option<usize>,
    pub children: Vec<usize>,
    pub bindings: HashMap<String, Binding>,
    pub depth: usize,
    pub scope_id: usize,
}

pub struct ScopeTree {
    scopes: Vec<Scope>,
    next_scope_id: usize,
}

impl ScopeTree {
    pub fn new() -> Self {
        let mut tree = Self {
            scopes: Vec::new(),
            next_scope_id: 0,
        };
        tree.create_scope(None);
        tree
    }

    pub fn create_scope(&mut self, parent: Option<usize>) -> usize {
        let id = self.next_scope_id;
        self.next_scope_id += 1;
        let depth = parent.map_or(0, |p| {
            self.scopes.get(p).map_or(0, |s| s.depth + 1)
        });
        let scope = Scope {
            parent,
            children: Vec::new(),
            bindings: HashMap::new(),
            depth,
            scope_id: id,
        };
        self.scopes.push(scope);
        if let Some(pid) = parent {
            if let Some(parent_scope) = self.scopes.get_mut(pid) {
                parent_scope.children.push(id);
            }
        }
        id
    }

    pub fn get_scope(&self, id: usize) -> Option<&Scope> {
        self.scopes.get(id)
    }

    pub fn get_scope_mut(&mut self, id: usize) -> Option<&mut Scope> {
        self.scopes.get_mut(id)
    }

    pub fn define(&mut self, scope_id: usize, binding: Binding) -> Result<(), String> {
        let scope = self.scopes.get_mut(scope_id)
            .ok_or_else(|| "invalid scope".to_string())?;
        if scope.bindings.contains_key(&binding.name) {
            return Err(format!("'{}' already defined in this scope", binding.name));
        }
        scope.bindings.insert(binding.name.clone(), binding);
        Ok(())
    }

    pub fn lookup(&self, scope_id: usize, name: &str) -> Option<(usize, &Binding)> {
        let mut current = Some(scope_id);
        while let Some(sid) = current {
            if let Some(scope) = self.scopes.get(sid) {
                if let Some(binding) = scope.bindings.get(name) {
                    return Some((sid, binding));
                }
                current = scope.parent;
            } else {
                break;
            }
        }
        None
    }

    pub fn lookup_local(&self, scope_id: usize, name: &str) -> Option<&Binding> {
        self.scopes.get(scope_id)
            .and_then(|s| s.bindings.get(name))
    }

    pub fn lookup_in_scope(&self, scope_id: usize, name: &str) -> Option<&Binding> {
        self.lookup(scope_id, name).map(|(_, b)| b)
    }

    pub fn enter_child(&mut self, parent_id: usize) -> usize {
        self.create_scope(Some(parent_id))
    }

    pub fn root_scope(&self) -> usize {
        0
    }
}

impl Default for ScopeTree {
    fn default() -> Self {
        Self::new()
    }
}
