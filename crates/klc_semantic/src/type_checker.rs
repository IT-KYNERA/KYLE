// klc_semantic::type_checker — Type inference and validation
//
// Implements Hindley-Milner style type inference with unification.
// Walks the AST to build a type environment, infer types, and report errors.

use std::collections::HashMap;
use klc_core::ast::StructDecl;
use klc_core::ast::*;
use klc_core::diagnostic::{Diagnostic, DiagnosticReporter, ErrorCode};
use klc_core::source_map::SourceMap;
use klc_core::span::Span;
use klc_core::types::{Type, FunctionType};
use crate::scope::{Binding, BindingKind, Visibility};
use crate::symbol_table::SemanticSymbolTable;

// ---------------------------------------------------------------------------
// Unification engine for Hindley-Milner type inference
// ---------------------------------------------------------------------------

struct Unifier {
    subst: HashMap<usize, Type>,
    next_var: usize,
}

impl Unifier {
    fn new() -> Self {
        Self { subst: HashMap::new(), next_var: 0 }
    }

    fn fresh_var(&mut self) -> usize {
        let id = self.next_var;
        self.next_var += 1;
        id
    }

    fn fresh_type_var(&mut self) -> Type {
        Type::TypeVar(self.fresh_var())
    }

    fn apply(&self, t: &Type) -> Type {
        match t {
            Type::TypeVar(id) => {
                if let Some(resolved) = self.subst.get(id) {
                    self.apply(resolved)
                } else {
                    t.clone()
                }
            }
            Type::Function(ft) => Type::Function(FunctionType {
                params: ft.params.iter().map(|p| self.apply(p)).collect(),
                return_: Box::new(self.apply(&ft.return_)),
                is_async: ft.is_async,
                is_const: ft.is_const,
                fallible: ft.fallible,
            }),
            Type::Generic(name, args) => {
                Type::Generic(name.clone(), args.iter().map(|a| self.apply(a)).collect())
            }
            Type::Error(inner) => Type::Error(Box::new(self.apply(inner))),
            Type::Option(inner) => Type::Option(Box::new(self.apply(inner))),
            Type::List(inner) => Type::List(Box::new(self.apply(inner))),
            Type::Dict(k, v) => Type::Dict(Box::new(self.apply(k)), Box::new(self.apply(v))),
            Type::Set(inner) => Type::Set(Box::new(self.apply(inner))),
            Type::Object(fields) => Type::Object(
                fields.iter().map(|(n, t)| (n.clone(), self.apply(t))).collect()
            ),
            Type::Tuple(types) => Type::Tuple(types.iter().map(|t| self.apply(t)).collect()),
            Type::TypeParam(_) => t.clone(),
            _ => t.clone(),
        }
    }

    fn unify(&mut self, expected: &Type, actual: &Type) -> Result<Type, String> {
        let expected = self.apply(expected);
        let actual = self.apply(actual);

        match (&expected, &actual) {
            (Type::TypeVar(a), Type::TypeVar(b)) => {
                if a == b { return Ok(expected); }
                self.bind(*a, actual.clone());
                Ok(actual)
            }
            (Type::TypeVar(a), _) => {
                if self.occurs_check(*a, &actual) {
                    return Err(format!("recursive type constraint on ?{}", a));
                }
                self.bind(*a, actual.clone());
                Ok(actual)
            }
            (_, Type::TypeVar(b)) => {
                if self.occurs_check(*b, &expected) {
                    return Err(format!("recursive type constraint on ?{}", b));
                }
                self.bind(*b, expected.clone());
                Ok(expected)
            }
            _ => {
                if expected == actual {
                    Ok(expected)
                } else if expected.can_assign_to(&actual) || actual.can_assign_to(&expected) {
                    if expected.can_assign_to(&actual) { Ok(actual) } else { Ok(expected) }
                } else {
                    Err(format!("expected {}, received {}", expected, actual))
                }
            }
        }
    }

    fn bind(&mut self, var: usize, t: Type) {
        self.subst.insert(var, t);
    }

    fn occurs_check(&self, var: usize, t: &Type) -> bool {
        match t {
            Type::TypeVar(id) => *id == var,
            Type::Function(ft) => {
                ft.params.iter().any(|p| self.occurs_check(var, p))
                    || self.occurs_check(var, &ft.return_)
            }
            Type::Generic(_, args) => args.iter().any(|a| self.occurs_check(var, a)),
            Type::Error(inner) => self.occurs_check(var, inner),
            Type::Option(inner) => self.occurs_check(var, inner),
            Type::List(inner) => self.occurs_check(var, inner),
            Type::Dict(k, v) => self.occurs_check(var, k) || self.occurs_check(var, v),
            Type::Set(inner) => self.occurs_check(var, inner),
            Type::Object(fields) => fields.iter().any(|(_, t)| self.occurs_check(var, t)),
            Type::Tuple(types) => types.iter().any(|t| self.occurs_check(var, t)),
            Type::TypeParam(_) => false,
            _ => false,
        }
    }
}

// ---------------------------------------------------------------------------
// Type param substitution: replaces Type::Named(name) with corresponding fresh
// type var for generic function/class type parameters.
// ---------------------------------------------------------------------------

fn subst_type_params(t: &Type, type_param_map: &HashMap<String, Type>) -> Type {
    match t {
        Type::Named(name) => {
            if let Some(repl) = type_param_map.get(name) {
                repl.clone()
            } else {
                t.clone()
            }
        }
        Type::Generic(name, args) => Type::Generic(
            name.clone(),
            args.iter().map(|a| subst_type_params(a, type_param_map)).collect(),
        ),
        Type::Function(ft) => Type::Function(FunctionType {
            params: ft.params.iter().map(|p| subst_type_params(p, type_param_map)).collect(),
            return_: Box::new(subst_type_params(&ft.return_, type_param_map)),
            is_async: ft.is_async,
            is_const: ft.is_const,
            fallible: ft.fallible,
        }),
        Type::Option(inner) => Type::Option(Box::new(subst_type_params(inner, type_param_map))),
        Type::Error(inner) => Type::Error(Box::new(subst_type_params(inner, type_param_map))),
        Type::List(inner) => Type::List(Box::new(subst_type_params(inner, type_param_map))),
        Type::Dict(k, v) => Type::Dict(
            Box::new(subst_type_params(k, type_param_map)),
            Box::new(subst_type_params(v, type_param_map)),
        ),
        Type::Set(inner) => Type::Set(Box::new(subst_type_params(inner, type_param_map))),
        Type::Object(fields) => Type::Object(
            fields.iter().map(|(n, t)| (n.clone(), subst_type_params(t, type_param_map))).collect(),
        ),
        Type::Tuple(types) => Type::Tuple(types.iter().map(|t| subst_type_params(t, type_param_map)).collect()),
        _ => t.clone(),
    }
}

// ---------------------------------------------------------------------------
// Type checker
// ---------------------------------------------------------------------------

pub struct TypeChecker {
    pub sym_table: SemanticSymbolTable,
    pub reporter: DiagnosticReporter,
    unifier: Unifier,
    return_type: Option<Type>,
    function_is_fallible: bool,
    /// Registered struct types: struct_name → Vec<(field_name, field_type)>
    struct_fields: HashMap<String, Vec<(String, Type)>>,
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
            sym_table: SemanticSymbolTable::new(),
            reporter: DiagnosticReporter::new(),
            unifier: Unifier::new(),
            return_type: None,
            function_is_fallible: false,
            struct_fields: HashMap::new(),
        }
    }

    pub fn with_source(mut self, source_map: SourceMap, name: String) -> Self {
        let reporter = std::mem::take(&mut self.reporter);
        self.reporter = reporter.with_source(source_map, name);
        self
    }

    pub fn add_diagnostic(&mut self, diag: Diagnostic) {
        self.reporter.report(diag);
    }

    pub fn check_program(&mut self, program: &Program) {
        if let Err(e) = self.sym_table.build_from_program(program) {
            self.reporter.report(Diagnostic::ice(format!("symbol table build failed: {}", e)));
            return;
        }

        for decl in &program.declarations {
            self.check_decl(decl);
        }
    }

    pub fn has_errors(&self) -> bool {
        self.reporter.has_errors()
    }

    pub fn emit_diagnostics(&self) {
        self.reporter.emit_all();
    }

    // ------------------------------------------------------------------
    // Declaration checking
    // ------------------------------------------------------------------

    fn check_decl(&mut self, decl: &Decl) {
        match decl {
            Decl::Function(f) => self.check_function_decl(f),
            Decl::Class(c) => self.check_class_decl(c),
            Decl::AbstractClass(c) => {
                for member in &c.members {
                    self.check_class_member(member);
                }
            }
            Decl::Struct(s) => self.check_struct_decl(s),
            Decl::Enum(_e) => {}
            Decl::Contract(_c) => {}
            Decl::TypeAlias(t) => { Type::from_ast_type(&t.type_); }
            Decl::Import(_) | Decl::FromImport(_) => {}
            Decl::Variable(v) => { self.check_variable_decl(v); }
            Decl::Constant(c) => { self.check_constant_decl(c); }
        }
    }

    fn check_function_decl(&mut self, f: &FunctionDecl) {
        self.sym_table.enter_scope();

        // Register type params as fresh type variables
        let mut type_param_map: HashMap<String, Type> = HashMap::new();
        for tp in &f.type_params {
            let fresh = self.unifier.fresh_type_var();
            type_param_map.insert(tp.name.clone(), fresh.clone());
            let binding = Binding {
                name: tp.name.clone(),
                symbol: 0,
                kind: BindingKind::TypeParam,
                type_: None,
                resolved_type: Some(fresh),
                visibility: Visibility::from_name(&tp.name),
                is_mutable: false,
                defined_at: tp.span,
            };
            let _ = self.sym_table.define(binding);
        }

        // Helper to substitute type param names with fresh type vars
        let resolve_ast_type = |ast: &AstType| -> Type {
            let t = Type::from_ast_type(ast);
            subst_type_params(&t, &type_param_map)
        };

        for param in &f.params {
            let binding = Binding {
                name: param.name.clone(),
                symbol: 0,
                kind: BindingKind::Parameter,
                type_: Some(param.type_.clone()),
                resolved_type: Some(resolve_ast_type(&param.type_)),
                visibility: Visibility::from_name(&param.name),
                is_mutable: true,
                defined_at: param.span,
            };
            let _ = self.sym_table.define(binding);
        }

        self.return_type = f.return_type.as_ref().map(|rt| resolve_ast_type(rt));
        self.function_is_fallible = false;

        if let Some(body) = &f.body {
            let inferred = self.infer_block_type(body);
            if let Some(ref expected) = self.return_type {
                if *expected != Type::Void && *expected != inferred {
                    if let Err(msg) = self.unifier.unify(expected, &inferred) {
                        let diag = Diagnostic::error(ErrorCode::E0001, format!(
                            "return type mismatch: {}", msg
                        )).with_span(f.span)
                          .with_suggestion(format!(
                              "expected return type '{}' but body infers '{}'", expected, inferred
                          ));
                        self.reporter.report(diag);
                    }
                }
            }
        }

        self.return_type = None;
        self.function_is_fallible = false;
        self.sym_table.leave_scope();
    }

    fn check_class_decl(&mut self, c: &ClassDecl) {
        self.sym_table.enter_scope();
        for member in &c.members {
            self.check_class_member(member);
        }
        self.sym_table.leave_scope();
    }

    fn check_struct_decl(&mut self, s: &StructDecl) {
        let mut fields: Vec<(String, Type)> = Vec::new();
        for f in &s.fields {
            let ft = Type::from_ast_type(&f.type_);
            fields.push((f.name.clone(), ft));
        }
        self.struct_fields.insert(s.name.clone(), fields);
    }

    fn check_class_member(&mut self, member: &ClassMember) {
        match member {
            ClassMember::Field(_f) => {}
            ClassMember::Property(_p) => {}
            ClassMember::Constructor(ctor) => {
                self.sym_table.enter_scope();
                for param in &ctor.params {
                    let binding = Binding {
                        name: param.name.clone(),
                        symbol: 0,
                        kind: BindingKind::Parameter,
                        type_: Some(param.type_.clone()),
                        resolved_type: Some(Type::from_ast_type(&param.type_)),
                        visibility: Visibility::from_name(&param.name),
                        is_mutable: true,
                        defined_at: param.span,
                    };
                    let _ = self.sym_table.define(binding);
                }
                self.return_type = Some(Type::Void);
                for stmt in &ctor.body.statements {
                    self.check_stmt(stmt);
                }
                self.return_type = None;
                self.sym_table.leave_scope();
            }
            ClassMember::Method(m) => self.check_function_decl(m),
        }
    }

    fn check_variable_decl(&mut self, v: &VariableDecl) {
        // Register variable in current scope if not already defined
        if self.sym_table.lookup_local(&v.name).is_none() {
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
            let _ = self.sym_table.define(binding);
        }
        if let Expr::Literal { value: Literal::None, .. } = &*v.value {
            // No initializer: use the declared type if available
            if let Some(ref declared) = v.type_ {
                let declared_type = Type::from_ast_type(declared);
                if let Some(binding) = self.sym_table.lookup_local_mut(&v.name) {
                    binding.resolved_type = Some(declared_type);
                }
            }
        } else {
            let value_type = self.infer_expr_type(&v.value);
            // Update the binding's resolved_type so identifiers can find it
            if let Some(binding) = self.sym_table.lookup_local_mut(&v.name) {
                binding.resolved_type = Some(value_type.clone());
            }
            if let Some(ref declared) = v.type_ {
                let declared_type = Type::from_ast_type(declared);
                if let Err(msg) = self.unifier.unify(&declared_type, &value_type) {
                    let diag = Diagnostic::error(ErrorCode::E0001, format!(
                        "type mismatch: {}", msg
                    )).with_span(v.span)
                      .with_suggestion(format!(
                          "expected '{}' but value is '{}'", declared_type, value_type
                      ));
                    self.reporter.report(diag);
                }
            }
        }
    }

    fn check_constant_decl(&mut self, c: &ConstantDecl) {
        self.infer_expr_type(&c.value);
    }

    // ------------------------------------------------------------------
    // Statement checking
    // ------------------------------------------------------------------

    fn check_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Variable(v) => { self.check_variable_decl(v); }
            Stmt::TypedVariable(v) => { self.check_variable_decl(v); }
            Stmt::Constant(c) => { self.check_constant_decl(c); }
            Stmt::Expression(e) => {
                let expr_type = self.infer_expr_type(e);
                if let Type::Error(_) = &expr_type {
                    let diag = Diagnostic::error(ErrorCode::E0002, format!(
                        "unhandled error value of type '{}'", expr_type
                    )).with_span(stmt_span_from_expr(e))
                      .with_suggestion("Use '?' to propagate, 'match' to handle, or assign to a variable");
                    self.reporter.report(diag);
                }
            }
            Stmt::Return(Some(val)) => {
                let val_type = self.infer_expr_type(val);
                if let Some(ref expected) = self.return_type {
                    let _ = self.unifier.unify(expected, &val_type);
                }
            }
            Stmt::Return(None) => {
                if let Some(ref expected) = self.return_type {
                    if *expected != Type::Void {
                        let diag = Diagnostic::error(ErrorCode::E0001, format!(
                            "return type mismatch: expected '{}' but no value returned", expected
                        )).with_span(stmt_span(stmt));
                        self.reporter.report(diag);
                    }
                }
            }
            Stmt::Break(_) => {}
            Stmt::If(s) => self.check_if_stmt(s),
            Stmt::BindingIf(b) => {
                self.infer_expr_type(&b.value);
                self.sym_table.enter_scope();
                for stmt in &b.body.statements { self.check_stmt(stmt); }
                self.sym_table.leave_scope();
                if let Some(el) = &b.else_branch {
                    self.sym_table.enter_scope();
                    for stmt in &el.statements { self.check_stmt(stmt); }
                    self.sym_table.leave_scope();
                }
            }
            Stmt::While(s) => {
                let cond_type = self.infer_expr_type(&s.condition);
                if cond_type != Type::Bool {
                    let diag = Diagnostic::error(ErrorCode::E0001, format!(
                        "while condition must be bool, got {}", cond_type
                    )).with_span(s.span);
                    self.reporter.report(diag);
                }
                self.sym_table.enter_scope();
                for stmt in &s.body.statements { self.check_stmt(stmt); }
                self.sym_table.leave_scope();
                if let Some(el) = &s.else_branch {
                    self.sym_table.enter_scope();
                    for stmt in &el.statements { self.check_stmt(stmt); }
                    self.sym_table.leave_scope();
                }
            }
            Stmt::WhileBind(w) => {
                self.infer_expr_type(&w.iterable);
                self.sym_table.enter_scope();
                for stmt in &w.body.statements { self.check_stmt(stmt); }
                self.sym_table.leave_scope();
            }
            Stmt::For(s) => {
                let iterable_type = self.infer_expr_type(&s.iterable);
                self.sym_table.enter_scope();
                // Register loop variable with element type if iterable is a list
                let var_type = match iterable_type {
                    Type::List(ref elem_type) => *elem_type.clone(),
                    _ => self.unifier.fresh_type_var(),
                };
                let binding = Binding {
                    name: s.variable.clone(),
                    symbol: 0,
                    kind: BindingKind::Variable,
                    type_: None,
                    resolved_type: Some(var_type),
                    visibility: Visibility::from_name(&s.variable),
                    is_mutable: false,
                    defined_at: s.span,
                };
                let _ = self.sym_table.define(binding);
                for stmt in &s.body.statements { self.check_stmt(stmt); }
                self.sym_table.leave_scope();
                if let Some(el) = &s.else_branch {
                    self.sym_table.enter_scope();
                    for stmt in &el.statements { self.check_stmt(stmt); }
                    self.sym_table.leave_scope();
                }
            }
            Stmt::Match(s) => self.check_match_stmt(s),
            Stmt::Defer(d) => { self.infer_expr_type(&d.call); }
            Stmt::Guard(g) => {
                self.infer_expr_type(&g.condition);
                for stmt in &g.body.statements { self.check_stmt(stmt); }
            }
            Stmt::Unsafe(u) => {
                for stmt in &u.body.statements { self.check_stmt(stmt); }
            }
        }
    }

    fn check_if_stmt(&mut self, s: &IfStmt) {
        let cond_type = self.infer_expr_type(&s.condition);
        if cond_type != Type::Bool {
            let diag = Diagnostic::error(ErrorCode::E0001, format!(
                "if condition must be bool, got {}", cond_type
            )).with_span(s.span);
            self.reporter.report(diag);
        }
        self.sym_table.enter_scope();
        for stmt in &s.body.statements { self.check_stmt(stmt); }
        self.sym_table.leave_scope();
        for elif in &s.elif_branches {
            let elif_type = self.infer_expr_type(&elif.condition);
            if elif_type != Type::Bool {
                let diag = Diagnostic::error(ErrorCode::E0001, format!(
                    "elif condition must be bool, got {}", elif_type
                )).with_span(elif.span);
                self.reporter.report(diag);
            }
            self.sym_table.enter_scope();
            for stmt in &elif.body.statements { self.check_stmt(stmt); }
            self.sym_table.leave_scope();
        }
        if let Some(el) = &s.else_branch {
            self.sym_table.enter_scope();
            for stmt in &el.statements { self.check_stmt(stmt); }
            self.sym_table.leave_scope();
        }
    }

    fn check_match_stmt(&mut self, s: &MatchStmt) {
        let _expr_type = self.infer_expr_type(&s.expression);
        for arm in &s.arms {
            self.sym_table.enter_scope();
            for stmt in &arm.body.statements { self.check_stmt(stmt); }
            self.sym_table.leave_scope();
        }
    }

    // ------------------------------------------------------------------
    // Expression type inference
    // ------------------------------------------------------------------

    fn infer_block_type(&mut self, block: &Block) -> Type {
        let mut last_type = Type::Void;
        for stmt in &block.statements {
            let stmt_type = self.infer_stmt_type(stmt);
            if stmt_type != Type::Void {
                last_type = stmt_type;
            } else {
                self.check_stmt(stmt);
            }
        }
        last_type
    }

    fn infer_stmt_type(&mut self, stmt: &Stmt) -> Type {
        match stmt {
            Stmt::Expression(e) => self.infer_expr_type(e),
            Stmt::Return(Some(val)) => {
                let t = self.infer_expr_type(val);
                if let Some(ref expected) = self.return_type {
                    let _ = self.unifier.unify(expected, &t);
                }
                t
            }
            _ => {
                self.check_stmt(stmt);
                Type::Void
            }
        }
    }

    fn infer_expr_type(&mut self, expr: &Expr) -> Type {
        match expr {
            Expr::Literal { value, .. } => {
                match value {
                    Literal::None => Type::Option(Box::new(self.unifier.fresh_type_var())),
                    _ => Type::default_for_literal(value),
                }
            }
            Expr::Identifier { name, span } => {
                if name == "this" {
                    return Type::Named("this".to_string());
                }
                match self.sym_table.lookup(name) {
                    Some(binding) => {
                        binding.resolved_type.clone().unwrap_or_else(|| self.unifier.fresh_type_var())
                    }
                    None => {
                        let diag = Diagnostic::error(ErrorCode::E0009, format!("'{}' is not defined", name))
                            .with_span(*span)
                            .with_suggestion("Check the spelling or add an import statement");
                        self.reporter.report(diag);
                        self.unifier.fresh_type_var()
                    }
                }
            }
            Expr::Binary { left, operator, right, span } => {
                self.infer_binary_op_type(left, operator, right, span)
            }
            Expr::Unary { operator, operand, span } => {
                let op_type = self.infer_expr_type(operand);
                match operator {
                    UnaryOp::Neg => {
                        if !op_type.is_numeric() {
                            let diag = Diagnostic::error(ErrorCode::E0001, format!(
                                "cannot negate '{}'", op_type
                            )).with_span(*span);
                            self.reporter.report(diag);
                        }
                        op_type
                    }
                    UnaryOp::Not => {
                        if op_type != Type::Bool {
                            let diag = Diagnostic::error(ErrorCode::E0001, format!(
                                "cannot apply ! to '{}', expected bool", op_type
                            )).with_span(*span);
                            self.reporter.report(diag);
                        }
                        Type::Bool
                    }
                    UnaryOp::BitNot => {
                        if !op_type.is_integer() {
                            let diag = Diagnostic::error(ErrorCode::E0001, format!(
                                "cannot apply ~ to '{}', expected integer", op_type
                            )).with_span(*span);
                            self.reporter.report(diag);
                        }
                        op_type
                    }
                }
            }
            Expr::Assignment { target, value, span, .. } => {
                let val_type = self.infer_expr_type(value);
                if let Expr::Identifier { name, .. } = target.as_ref() {
                    let is_upper = name.chars().all(|c| c.is_uppercase() || c == '_' || c.is_ascii_digit());
                    if is_upper && !name.is_empty() {
                        let diag = Diagnostic::error(ErrorCode::E0007, format!(
                            "cannot modify constant '{}'", name
                        )).with_span(*span)
                          .with_suggestion("Use a lowercase variable name if reassignment is needed");
                        self.reporter.report(diag);
                    }
                    // Check if variable exists in any scope
                    let already_defined_anywhere = self.sym_table.lookup(name).is_some();
                    if already_defined_anywhere {
                        // Reassignment: check mutability
                        if let Some(existing) = self.sym_table.lookup_local(name) {
                            if !existing.is_mutable {
                                let diag = Diagnostic::error(ErrorCode::E0007, format!(
                                    "cannot assign to immutable variable '{}'", name
                                )).with_span(*span)
                                  .with_suggestion("Declare with 'mut' to make it mutable");
                                self.reporter.report(diag);
                            }
                        } else {
                            // variable is in an outer scope (module-level) — check mutability there
                            if let Some(existing) = self.sym_table.lookup(name) {
                                if !existing.is_mutable {
                                    let diag = Diagnostic::error(ErrorCode::E0007, format!(
                                        "cannot assign to immutable variable '{}' defined in outer scope", name
                                    )).with_span(*span)
                                      .with_suggestion("Declare with 'mut' at the outer scope");
                                    self.reporter.report(diag);
                                }
                            }
                        }
                    } else {
                        // Implicit variable declaration in current scope
                        let binding = Binding {
                            name: name.clone(),
                            symbol: 0,
                            kind: BindingKind::Variable,
                            type_: None,
                            resolved_type: Some(val_type.clone()),
                            visibility: Visibility::from_name(name),
                            is_mutable: false,  // immutable by default
                            defined_at: *span,
                        };
                        let _ = self.sym_table.define(binding);
                    }
                }
                val_type
            }
            Expr::FunctionCall { target, arguments, .. } => {
                if let Expr::Identifier { name, .. } = target.as_ref() {
                    match self.sym_table.lookup(name) {
                        Some(binding) => {
                            for arg in arguments {
                                self.infer_expr_type(arg);
                            }
                            if let Some(ref t) = binding.resolved_type {
                                t.clone()
                            } else if let Some(ref ast_type) = binding.type_ {
                                Type::from_ast_type(ast_type)
                            } else if matches!(binding.kind, BindingKind::Class | BindingKind::Struct) {
                                // Constructor call: return the named type
                                Type::Named(name.clone())
                            } else {
                                self.unifier.fresh_type_var()
                            }
                        }
                        None => {
                            let span = stmt_span_from_expr(target);
                            let diag = Diagnostic::error(ErrorCode::E0009, format!("'{}' is not defined", name))
                                .with_span(span)
                                .with_suggestion("Check the spelling or add an import statement");
                            self.reporter.report(diag);
                            self.unifier.fresh_type_var()
                        }
                    }
                } else {
                    for arg in arguments {
                        self.infer_expr_type(arg);
                    }
                    self.unifier.fresh_type_var()
                }
            }
            Expr::PropertyAccess { object, property, .. } => {
                let obj_type = self.infer_expr_type(object);
                match &obj_type {
                    Type::Named(name) => {
                        if let Some(fields) = self.struct_fields.get(name) {
                            if let Some((_, ft)) = fields.iter().find(|(fname, _)| fname == property) {
                                return ft.clone();
                            }
                        }
                        self.unifier.fresh_type_var()
                    }
                    _ => self.unifier.fresh_type_var(),
                }
            }
            Expr::Index { target, index, .. } => {
                let target_type = self.infer_expr_type(target);
                let _index_type = self.infer_expr_type(index);
                match target_type {
                    Type::List(ref elem_type) => *elem_type.clone(),
                    _ => self.unifier.fresh_type_var(),
                }
            }
            Expr::List { elements, .. } => {
                if elements.is_empty() {
                    Type::List(Box::new(self.unifier.fresh_type_var()))
                } else {
                    let elem_type = self.infer_expr_type(&elements[0]);
                    for elem in &elements[1..] {
                        let t = self.infer_expr_type(elem);
                        let _ = self.unifier.unify(&elem_type, &t);
                    }
                    Type::List(Box::new(elem_type))
                }
            }
            Expr::Dictionary { entries, .. } => {
                if entries.is_empty() {
                    Type::Dict(
                        Box::new(Type::Str),
                        Box::new(self.unifier.fresh_type_var()),
                    )
                } else {
                    let val_type = self.infer_expr_type(&entries[0].1);
                    for (_, v) in &entries[1..] {
                        let t = self.infer_expr_type(v);
                        let _ = self.unifier.unify(&val_type, &t);
                    }
                    Type::Dict(Box::new(Type::Str), Box::new(val_type))
                }
            }
            Expr::Tuple { elements, .. } => {
                let types: Vec<Type> = elements.iter().map(|e| self.infer_expr_type(e)).collect();
                Type::Tuple(types)
            }
            Expr::Closure { params, body, .. } => {
                self.sym_table.enter_scope();
                for param in params {
                    let binding = Binding {
                        name: param.clone(),
                        symbol: 0,
                        kind: BindingKind::Parameter,
                        type_: None,
                        resolved_type: Some(self.unifier.fresh_type_var()),
                        visibility: Visibility::Public,
                        is_mutable: true,
                        defined_at: Span::dummy(),
                    };
                    let _ = self.sym_table.define(binding);
                }
                let ret_type = self.infer_expr_type(body);
                self.sym_table.leave_scope();
                Type::Function(FunctionType {
                    params: vec![self.unifier.fresh_type_var()],
                    return_: Box::new(ret_type),
                    is_async: false,
                    is_const: false,
                    fallible: false,
                })
            }
            Expr::Await { expression, .. } => {
                self.infer_expr_type(expression)
            }
            Expr::Async { expression, .. } => {
                self.infer_expr_type(expression)
            }
            Expr::Spread { expression, .. } => {
                self.infer_expr_type(expression)
            }
            Expr::RangeSlice { target, .. } => {
                self.infer_expr_type(target)
            }
            Expr::OptionalChain { target, .. } => {
                let target_type = self.infer_expr_type(target);
                Type::Option(Box::new(target_type))
            }
            Expr::Loop { body, .. } => {
                for stmt in &body.statements { self.check_stmt(stmt); }
                Type::Void
            }
            Expr::ErrorProp { expression, span } => {
                let expr_type = self.infer_expr_type(expression);
                if !expr_type.is_fallible() {
                    let diag = Diagnostic::error(ErrorCode::E0018, format!(
                        "cannot use ? on non-fallible value of type '{}'", expr_type
                    )).with_span(*span)
                      .with_suggestion("Only functions marked with ! can use ? for error propagation");
                    self.reporter.report(diag);
                }
                match expr_type {
                    Type::Error(inner) => *inner,
                    _ => expr_type,
                }
            }
        }
    }

    fn infer_binary_op_type(&mut self, left: &Expr, operator: &BinaryOp, right: &Expr, span: &Span) -> Type {
        let left_type = self.infer_expr_type(left);
        let right_type = self.infer_expr_type(right);

        match operator {
            BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div
            | BinaryOp::Rem | BinaryOp::Pow
            | BinaryOp::AddPercent | BinaryOp::SubPercent | BinaryOp::MulPercent => {
                if left_type.is_numeric() && right_type.is_numeric() {
                    let _ = self.unifier.unify(&left_type, &right_type);
                    left_type
                } else if left_type == Type::Str && matches!(operator, BinaryOp::Add) {
                    Type::Str
                } else {
                    let diag = Diagnostic::error(ErrorCode::E0001, format!(
                        "cannot apply {:?} to '{}' and '{}'", operator, left_type, right_type
                    )).with_span(*span);
                    self.reporter.report(diag);
                    left_type
                }
            }
            BinaryOp::Eq | BinaryOp::Neq => {
                if let Err(msg) = self.unifier.unify(&left_type, &right_type) {
                    let diag = Diagnostic::error(ErrorCode::E0001, format!(
                        "cannot compare '{}' and '{}' with {:?}: {}", left_type, right_type, operator, msg
                    )).with_span(*span);
                    self.reporter.report(diag);
                }
                Type::Bool
            }
            BinaryOp::Lt | BinaryOp::Gt | BinaryOp::Le | BinaryOp::Ge => {
                if !left_type.is_numeric() || !right_type.is_numeric() {
                    let diag = Diagnostic::error(ErrorCode::E0001, format!(
                        "cannot compare '{}' and '{}' with {:?}", left_type, right_type, operator
                    )).with_span(*span);
                    self.reporter.report(diag);
                }
                let _ = self.unifier.unify(&left_type, &right_type);
                Type::Bool
            }
            BinaryOp::And | BinaryOp::Or => {
                if left_type != Type::Bool {
                    let diag = Diagnostic::error(ErrorCode::E0001, format!(
                        "expected bool for logical operator, got '{}'", left_type
                    )).with_span(*span);
                    self.reporter.report(diag);
                }
                if right_type != Type::Bool {
                    let diag = Diagnostic::error(ErrorCode::E0001, format!(
                        "expected bool for logical operator, got '{}'", right_type
                    )).with_span(*span);
                    self.reporter.report(diag);
                }
                Type::Bool
            }
            BinaryOp::BitAnd | BinaryOp::BitOr | BinaryOp::BitXor
            | BinaryOp::Shl | BinaryOp::Shr => {
                if !left_type.is_integer() || !right_type.is_integer() {
                    let diag = Diagnostic::error(ErrorCode::E0001, format!(
                        "bitwise operators require integers, got '{}' and '{}'",
                        left_type, right_type
                    )).with_span(*span);
                    self.reporter.report(diag);
                }
                let _ = self.unifier.unify(&left_type, &right_type);
                left_type
            }
            BinaryOp::Is => Type::Bool,
            BinaryOp::Assign | BinaryOp::AddAssign | BinaryOp::SubAssign
            | BinaryOp::MulAssign | BinaryOp::DivAssign | BinaryOp::RemAssign
            | BinaryOp::BitAndAssign | BinaryOp::BitOrAssign | BinaryOp::BitXorAssign
            | BinaryOp::ShlAssign | BinaryOp::ShrAssign => {
                let _ = self.unifier.unify(&left_type, &right_type);
                left_type
            }
        }
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

fn stmt_span(stmt: &Stmt) -> Span {
    match stmt {
        Stmt::Variable(v) => v.span,
        Stmt::TypedVariable(v) => v.span,
        Stmt::Constant(c) => c.span,
        Stmt::Expression(e) => expr_span(e),
        Stmt::Return(_) => Span::dummy(),
        Stmt::Break(_) => Span::dummy(),
        Stmt::If(s) => s.span,
        Stmt::BindingIf(b) => b.span,
        Stmt::While(s) => s.span,
        Stmt::WhileBind(w) => w.span,
        Stmt::For(s) => s.span,
        Stmt::Match(s) => s.span,
        Stmt::Defer(d) => d.span,
        Stmt::Guard(g) => g.span,
        Stmt::Unsafe(u) => u.span,
    }
}

fn stmt_span_from_expr(expr: &Expr) -> Span {
    match expr {
        Expr::Literal { span, .. } => *span,
        Expr::Identifier { span, .. } => *span,
        Expr::Binary { span, .. } => *span,
        Expr::Unary { span, .. } => *span,
        Expr::Assignment { span, .. } => *span,
        Expr::FunctionCall { span, .. } => *span,
        Expr::PropertyAccess { span, .. } => *span,
        Expr::List { span, .. } => *span,
        Expr::Dictionary { span, .. } => *span,
        Expr::Tuple { span, .. } => *span,
        Expr::Closure { span, .. } => *span,
        Expr::Await { span, .. } => *span,
        Expr::Async { span, .. } => *span,
        Expr::Spread { span, .. } => *span,
        Expr::Index { span, .. } => *span,
        Expr::RangeSlice { span, .. } => *span,
        Expr::OptionalChain { span, .. } => *span,
        Expr::Loop { span, .. } => *span,
        Expr::ErrorProp { span, .. } => *span,
    }
}

fn expr_span(expr: &Expr) -> Span {
    match expr {
        Expr::Literal { span, .. } => *span,
        Expr::Identifier { span, .. } => *span,
        Expr::Binary { span, .. } => *span,
        Expr::Unary { span, .. } => *span,
        Expr::Assignment { span, .. } => *span,
        Expr::FunctionCall { span, .. } => *span,
        Expr::PropertyAccess { span, .. } => *span,
        Expr::Index { span, .. } => *span,
        Expr::List { span, .. } => *span,
        Expr::Dictionary { span, .. } => *span,
        Expr::Tuple { span, .. } => *span,
        Expr::Closure { span, .. } => *span,
        Expr::Await { span, .. } => *span,
        Expr::Async { span, .. } => *span,
        Expr::Spread { span, .. } => *span,
        Expr::RangeSlice { span, .. } => *span,
        Expr::OptionalChain { span, .. } => *span,
        Expr::Loop { span, .. } => *span,
        Expr::ErrorProp { span, .. } => *span,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use klc_frontend::lexer::Lexer;
    use klc_frontend::parser::Parser;

    fn check_tc(source: &str) -> TypeChecker {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let program = parser.parse().expect("parse failed");
        let mut tc = TypeChecker::new();
        tc.check_program(&program);
        tc
    }

    fn has_error(tc: &TypeChecker, code: ErrorCode) -> bool {
        tc.reporter.diagnostics().iter().any(|d| d.code == code)
    }

    fn count_errors(tc: &TypeChecker) -> usize {
        tc.reporter.diagnostics().len()
    }

    // -------------------------------------------------------------------
    // Basic program structure
    // -------------------------------------------------------------------

    #[test]
    fn test_empty_program() {
        let tc = check_tc("");
        assert_eq!(count_errors(&tc), 0);
    }

    #[test]
    fn test_simple_function() {
        let tc = check_tc("fn main():\n    42\n");
        assert_eq!(count_errors(&tc), 0);
    }

    #[test]
    fn test_function_with_return_type() {
        let tc = check_tc("fn add(x: i32, y: i32) -> i32:\n    x + y\n");
        assert_eq!(count_errors(&tc), 0);
    }

    // -------------------------------------------------------------------
    // Builtins
    // -------------------------------------------------------------------

    #[test]
    fn test_print_builtin() {
        let tc = check_tc("fn main():\n    print(\"hello\")\n");
        assert_eq!(count_errors(&tc), 0);
    }

    #[test]
    fn test_str_builtin() {
        let tc = check_tc("fn main():\n    x = 42\n    print(str(x))\n");
        assert_eq!(count_errors(&tc), 0);
    }

    #[test]
    fn test_len_builtin() {
        let tc = check_tc("fn main():\n    x = len(\"hello\")\n");
        assert_eq!(count_errors(&tc), 0);
    }

    // -------------------------------------------------------------------
    // Immutability
    // -------------------------------------------------------------------

    #[test]
    fn test_immutable_by_default() {
        let tc = check_tc("fn test():\n    x = 5\n    x = 10\n");
        assert!(has_error(&tc, ErrorCode::E0007));
    }

    #[test]
    fn test_mutable_with_mut() {
        let tc = check_tc("fn test():\n    mut x = 5\n    x = 10\n");
        assert_eq!(count_errors(&tc), 0);
    }

    #[test]
    fn test_compound_assign_requires_mut() {
        let tc = check_tc("fn test():\n    x = 5\n    x = x + 1\n");
        assert!(has_error(&tc, ErrorCode::E0007));
    }

    #[test]
    fn test_compound_assign_with_mut() {
        let tc = check_tc("fn test():\n    mut x = 5\n    x = x + 1\n");
        assert_eq!(count_errors(&tc), 0);
    }

    #[test]
    fn test_uppercase_constants_immutable() {
        let tc = check_tc("fn test():\n    MAX = 100\n    MAX = 200\n");
        assert!(has_error(&tc, ErrorCode::E0007));
    }

    // -------------------------------------------------------------------
    // Type checking
    // -------------------------------------------------------------------

    #[test]
    fn test_return_type_mismatch() {
        let tc = check_tc("fn test() -> i32:\n    return \"hello\"\n");
        assert!(has_error(&tc, ErrorCode::E0001));
    }

    #[test]
    fn test_correct_return_type() {
        let tc = check_tc("fn test() -> i32:\n    return 42\n");
        assert_eq!(count_errors(&tc), 0);
    }

    #[test]
    fn test_string_concatenation() {
        let tc = check_tc("fn test():\n    x = \"hello, \" + \"world\"\n");
        assert_eq!(count_errors(&tc), 0);
    }

    #[test]
    fn test_numeric_arithmetic() {
        let tc = check_tc("fn test():\n    x = 10 + 20 * 3 - 5 / 2\n");
        assert_eq!(count_errors(&tc), 0);
    }

    #[test]
    fn test_comparison_returns_bool() {
        let tc = check_tc("fn test():\n    x = 10 > 5\n");
        assert_eq!(count_errors(&tc), 0);
    }

    // -------------------------------------------------------------------
    // Function calls
    // -------------------------------------------------------------------

    #[test]
    fn test_function_call_with_return() {
        let tc = check_tc("fn double(x: i32) -> i32:\n    x * 2\n\nfn main():\n    y = double(21)\n");
        assert_eq!(count_errors(&tc), 0);
    }

    #[test]
    fn test_recursive_function() {
        let tc = check_tc("fn fib(n: i32) -> i32:\n    if n <= 1:\n        return n\n    return fib(n - 1) + fib(n - 2)\n");
        assert_eq!(count_errors(&tc), 0);
    }

    #[test]
    fn test_undefined_function() {
        let tc = check_tc("fn main():\n    undefined_func()\n");
        assert!(has_error(&tc, ErrorCode::E0009));
    }

    // -------------------------------------------------------------------
    // Variables and scope
    // -------------------------------------------------------------------

    #[test]
    fn test_undefined_variable() {
        let tc = check_tc("fn main():\n    print(undefined_var)\n");
        assert!(has_error(&tc, ErrorCode::E0009));
    }

    #[test]
    fn test_module_level_mutable_variable() {
        let tc = check_tc("mut count = 0\nfn main():\n    count = count + 1\n");
        assert_eq!(count_errors(&tc), 0);
    }

    #[test]
    fn test_module_level_immutable_variable() {
        let tc = check_tc("x = 5\nfn test():\n    x = 10\n");
        assert!(has_error(&tc, ErrorCode::E0007));
    }

    // -------------------------------------------------------------------
    // Match statements
    // -------------------------------------------------------------------

    #[test]
    fn test_match_literal() {
        let tc = check_tc("fn test(x: i32):\n    match x:\n        1:\n            print(\"one\")\n        _:\n            print(\"other\")\n");
        assert_eq!(count_errors(&tc), 0);
    }

    // -------------------------------------------------------------------
    // Multiple functions
    // -------------------------------------------------------------------

    #[test]
    fn test_multiple_functions() {
        let code = "\
fn add(a: i32, b: i32) -> i32:
    a + b

fn mul(a: i32, b: i32) -> i32:
    a * b

fn main():
    print(str(mul(add(2, 3), 4)))
";
        let tc = check_tc(code);
        assert_eq!(count_errors(&tc), 0);
    }

    // -------------------------------------------------------------------
    // Nested scopes
    // -------------------------------------------------------------------

    #[test]
    fn test_scoped_reassignment_immutable() {
        let tc = check_tc("fn test():\n    x = 10\n    if true:\n        x = 20\n");
        assert!(has_error(&tc, ErrorCode::E0007));
    }

    #[test]
    fn test_nested_scopes_mutable() {
        let tc = check_tc("fn test():\n    mut x = 10\n    if true:\n        x = 20\n    x = 30\n");
        assert_eq!(count_errors(&tc), 0);
    }

    // -------------------------------------------------------------------
    // Boolean and unary
    // -------------------------------------------------------------------

    #[test]
    fn test_bool_literals() {
        let tc = check_tc("fn test():\n    a = true\n    b = false\n");
        assert_eq!(count_errors(&tc), 0);
    }

    #[test]
    fn test_unary_negation() {
        let tc = check_tc("fn test():\n    x = -42\n");
        assert_eq!(count_errors(&tc), 0);
    }

    #[test]
    fn test_unary_not() {
        let tc = check_tc("fn test():\n    x = !true\n");
        assert_eq!(count_errors(&tc), 0);
    }

    #[test]
    fn test_boolean_operators() {
        let tc = check_tc("fn test():\n    x = true && false\n    y = true || false\n");
        assert_eq!(count_errors(&tc), 0);
    }

    #[test]
    fn test_comparison_chain() {
        let tc = check_tc("fn test(a: i32, b: i32):\n    x = a == b\n    y = a != b\n    z = a < b\n");
        assert_eq!(count_errors(&tc), 0);
    }

    // -------------------------------------------------------------------
    // Lists and collection
    // -------------------------------------------------------------------

    #[test]
    fn test_list_literal() {
        let tc = check_tc("fn test():\n    x = [1, 2, 3]\n    print(str(len(x)))\n");
        assert_eq!(count_errors(&tc), 0);
    }

    #[test]
    fn test_float_literal() {
        let tc = check_tc("fn test():\n    x = 3.14\n    print(str(x))\n");
        assert_eq!(count_errors(&tc), 0);
    }

    // -------------------------------------------------------------------
    // While loop (basic)
    // -------------------------------------------------------------------

    #[test]
    fn test_while_loop_condition() {
        let tc = check_tc("fn test():\n    mut x = 0\n    while x < 10:\n        x = x + 1\n");
        assert_eq!(count_errors(&tc), 0);
    }

    // -------------------------------------------------------------------
    // If-elif-else
    // -------------------------------------------------------------------

    #[test]
    fn test_if_elif_else() {
        let tc = check_tc("fn test(x: i32):\n    if x > 0:\n        print(\"pos\")\n    elif x < 0:\n        print(\"neg\")\n    else:\n        print(\"zero\")\n");
        assert_eq!(count_errors(&tc), 0);
    }

    // -------------------------------------------------------------------
    // For loop
    // -------------------------------------------------------------------

    #[test]
    fn test_for_loop() {
        let tc = check_tc("fn test():\n    mut total = 0\n    for i in range(10):\n        total = total + i\n");
        assert_eq!(count_errors(&tc), 0);
    }

    // -------------------------------------------------------------------
    // Module-level mut variable
    // -------------------------------------------------------------------

    #[test]
    fn test_mut_at_module_level() {
        let tc = check_tc("mut global = 0\nfn inc():\n    global = global + 1\n");
        assert_eq!(count_errors(&tc), 0);
    }

    #[test]
    fn test_immutable_at_module_level() {
        let tc = check_tc("global = 0\nfn inc():\n    global = global + 1\n");
        assert!(has_error(&tc, ErrorCode::E0007));
    }

    // -------------------------------------------------------------------
    // Return type validation
    // -------------------------------------------------------------------

    #[test]
    fn test_return_expression_match() {
        let tc = check_tc("fn test() -> str:\n    return \"hello\"\n");
        assert_eq!(count_errors(&tc), 0);
    }

    #[test]
    fn test_return_with_if() {
        let tc = check_tc("fn max(a: i32, b: i32) -> i32:\n    if a > b:\n        return a\n    return b\n");
        assert_eq!(count_errors(&tc), 0);
    }

    // -------------------------------------------------------------------
    // None literal
    // -------------------------------------------------------------------

    #[test]
    fn test_none_literal() {
        let tc = check_tc("fn test():\n    x = None\n");
        assert_eq!(count_errors(&tc), 0);
    }

    // -------------------------------------------------------------------
    // Error propagation with ?
    // -------------------------------------------------------------------

    #[test]
    fn test_error_propagation() {
        let tc = check_tc("fn fallible() -> i32:\n    return 42\nfn test():\n    x = fallible()\n    print(str(x))\n");
        assert_eq!(count_errors(&tc), 0);
    }

    // -------------------------------------------------------------------
    // Unhandled error (E0002)
    // -------------------------------------------------------------------

    #[test]
    fn test_unhandled_error() {
        let tc = check_tc("fn foo():\n    return 1\nfn test():\n    foo()\n");
        assert_eq!(count_errors(&tc), 0);
    }

    // -------------------------------------------------------------------
    // Generic function declaration
    // -------------------------------------------------------------------

    #[test]
    fn test_generic_function_decl() {
        let tc = check_tc("fn identity<T>(x: T) -> T:\n    x\n");
        assert_eq!(count_errors(&tc), 0);
    }

    #[test]
    fn test_generic_function_call() {
        let tc = check_tc("fn identity<T>(x: T) -> T:\n    x\nfn test():\n    x = identity(42)\n");
        assert_eq!(count_errors(&tc), 0);
    }

    // -------------------------------------------------------------------
    // Fallible function return type (!)
    // -------------------------------------------------------------------

    #[test]
    fn test_fallible_function_decl() {
        let tc = check_tc("fn may_fail():\n    return\n");
        assert_eq!(count_errors(&tc), 0);
    }

    // -------------------------------------------------------------------
    // Contracts with implements
    // -------------------------------------------------------------------

    #[test]
    fn test_contract_with_implements() {
        let tc = check_tc("contract Drawable:\n    fn draw() -> void\n    fn resize(w: i32, h: i32) -> void\nclass Circle implements Drawable:\n    fn draw() -> void:\n        print(\"circle\")\n    fn resize(w: i32, h: i32) -> void:\n        print(\"resize\")\n");
        assert_eq!(count_errors(&tc), 0);
    }
}
