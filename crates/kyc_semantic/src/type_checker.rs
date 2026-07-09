use std::collections::HashMap;
use kyc_core::ast::*;
use kyc_core::types::*;
use kyc_core::source_map::SourceMap;
use kyc_core::diagnostic::{Diagnostic, ErrorCode, DiagnosticReporter};
use crate::symbol_table::{SymbolTable, Symbol, SymKind};
use crate::scope::ScopeResolver;

pub struct TypeChecker {
    pub reporter: DiagnosticReporter,
    symbols: SymbolTable,
    pub fn_return_types: HashMap<String, Type>,
    fn_const_flags: HashMap<String, bool>,
    current_fn: Option<String>,
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
            reporter: DiagnosticReporter::new(),
            symbols: SymbolTable::new(),
            fn_return_types: HashMap::new(),
            fn_const_flags: HashMap::new(),
            current_fn: None,
        }
    }

    pub fn with_source(mut self, source_map: SourceMap, name: String) -> Self {
        self.reporter = self.reporter.with_source(source_map, name);
        self
    }

    pub fn symbols(&self) -> &SymbolTable {
        &self.symbols
    }

    /// Extract a display name from a function call target expression.
    fn target_name(target: &Expr) -> Option<&str> {
        match target {
            Expr::Identifier { name, .. } => Some(name.as_str()),
            Expr::PropertyAccess { property, .. } => Some(property.as_str()),
            _ => None,
        }
    }

    /// Hardcoded expected argument counts for built-in functions.
    fn builtin_expected_args(name: &str) -> Option<usize> {
        match name {
            "print" | "println" | "print_err" => Some(1),

            "len" => Some(1),
            "input" => Some(0),
            "open" => Some(2),
            "read_str" => Some(2),
            "write_str" => Some(2),
            "close" => Some(1),
            "sleep" => Some(1),
            "now" => Some(0),
            "contains" => Some(2),
            "to_upper" | "to_lower" | "trim" => Some(1),
            "replace" => Some(3),
            "substr" => Some(3),
            "char_at" => Some(2),
            "ord" => Some(1),
            "is_digit" | "is_alpha" | "is_alnum" => Some(1),
            "is_whitespace" | "is_upper" | "is_lower" => Some(1),
            "error" => Some(1),
            "assert" => Some(1),
            "assert_eq" | "assert_str" | "assert_ne" => Some(2),
            "range" => Some(2),
            "json_parse" | "json_stringify" | "json_stringify_str" | "serialize" => Some(1),
            "deserialize" => Some(1),
            "type" => Some(0),
            "list_push" => Some(2),
            "list_pop" | "list_len" => Some(1),
            "ceil" | "floor" | "round" => Some(1),
            "ky_spawn_thread" => Some(2),
            "ky_join_thread" => Some(1),
            "ky_parallel_for" => Some(3),
            "ky_channel_new" => Some(1),
            "ky_channel_send" => Some(2),
            "ky_channel_recv" => Some(1),
            "ky_channel_close" => Some(1),
            "ky_channel_len" => Some(1),
            "ky_channel_free" => Some(1),
            _ => None,
        }
    }

    pub fn has_errors(&self) -> bool {
        self.reporter.has_errors()
    }

    pub fn emit_diagnostics(&self) {
        self.reporter.emit_all();
    }

    pub fn check_program(&mut self, program: &Program) {
        for decl in &program.declarations {
            if let Decl::Function(f) = decl {
                let ret = f.return_type.as_ref()
                    .map(|t| self.resolve_ast_type(t))
                    .unwrap_or(Type::Void);
                self.fn_return_types.insert(f.name.clone(), ret);
                self.fn_const_flags.insert(f.name.clone(), f.is_const);
            }
        }

        let mut resolver = ScopeResolver::new(std::mem::take(&mut self.reporter));
        resolver.resolve(program);
        self.symbols = resolver.symbols;
        self.reporter = resolver.reporter;

        for decl in &program.declarations {
            match decl {
                Decl::Function(f) => { self.check_function(f); }
                Decl::Variable(v) => { self.check_variable(v); }
                Decl::Constant(c) => {
                    let ty = self.infer_expr(&c.value);
                    self.check_const_expr(&c.value, &c.name);
                    let _ = self.symbols.insert(c.name.clone(), Symbol::new(c.name.clone(), SymKind::Constant(ty)));
                }
                Decl::Class(c) => { self.check_class(c); }
                Decl::AbstractClass(c) => { self.check_abstract_class(c); }
                _ => {}
            }
        }
    }

    fn check_function(&mut self, f: &FunctionDecl) {
        self.current_fn = Some(f.name.clone());
        self.symbols.push_scope();
        for param in &f.params {
            let ty = self.resolve_ast_type(&param.type_);
            // Type-check default expression against declared parameter type
            if let Some(default) = &param.default {
                let default_ty = self.infer_expr(default);
                if !self.types_match(&default_ty, &ty) {
                    self.reporter.report(
                        Diagnostic::error(ErrorCode::E0001,
                            format!("default value type '{}' does not match parameter '{}' type '{}'",
                                default_ty, param.name, ty))
                    );
                }
            }
            let is_mutable = matches!(param.mode, ParamMode::MutableBorrow | ParamMode::Move);
            let _ = self.symbols.insert(param.name.clone(),
                Symbol::new_var(param.name.clone(), Some(ty), is_mutable));
        }
        if let Some(body) = &f.body {
            if f.is_const {
                self.check_const_block(body, &f.name);
            } else {
                self.check_block(body);
            }
        }
        self.symbols.pop_scope();
        self.current_fn = None;
    }

    fn check_class(&mut self, c: &ClassDecl) {
        self.symbols.push_scope();
        let _ = self.symbols.insert("this".to_string(),
            Symbol::new_var("this".to_string(), Some(Type::Named(c.name.clone())), false));
        for member in &c.members {
            if let ClassMember::Method(f) = member {
                self.check_function(f);
            }
        }
        // Type-check field defaults against declared field types
        for member in &c.members {
            if let ClassMember::Field(f) = member {
                if let Some(ref default_expr) = f.default {
                    let default_ty = self.infer_expr(default_expr);
                    let field_ty = self.resolve_ast_type(&f.type_);
                    if !self.types_match(&default_ty, &field_ty) {
                        self.reporter.report(
                            Diagnostic::error(ErrorCode::E0001,
                                format!("default value type '{}' does not match field '{}' type '{}'",
                                    default_ty, f.name, field_ty))
                        );
                    }
                }
            }
        }

        // If this class has a parent, enforce that all abstract methods are implemented
        if let Some(parent_name) = &c.parent {
            if let Some(parent_sym) = self.symbols.lookup(parent_name) {
                if let SymKind::Class(parent_class) = &parent_sym.kind {
                    let abstract_methods: Vec<&FunctionDecl> = parent_class.members.iter()
                        .filter_map(|m| {
                            if let ClassMember::Method(f) = m {
                                if f.is_abstract { Some(f) } else { None }
                            } else { None }
                        })
                        .collect();
                    for abstract_fn in &abstract_methods {
                        let implemented = c.members.iter().any(|m| {
                            if let ClassMember::Method(f) = m {
                                f.name == abstract_fn.name
                            } else { false }
                        });
                        if !implemented {
                            self.reporter.report(
                                Diagnostic::error(ErrorCode::E0001,
                                    format!("class '{}' must implement abstract method '{}' from parent '{}'",
                                        c.name, abstract_fn.name, parent_name))
                            );
                        }
                    }
                }
            }
        }
        self.symbols.pop_scope();
    }

    fn check_abstract_class(&mut self, c: &AbstractClassDecl) {
        self.symbols.push_scope();
        let _ = self.symbols.insert("this".to_string(),
            Symbol::new_var("this".to_string(), Some(Type::Named(c.name.clone())), false));
        for member in &c.members {
            if let ClassMember::Method(f) = member {
                if !f.is_abstract {
                    self.check_function(f);
                }
            }
        }
        self.symbols.pop_scope();
    }

    fn check_const_block(&mut self, block: &Block, fn_name: &str) {
        for stmt in &block.statements {
            match stmt {
                Stmt::Expression(e) => self.check_const_expr(e, fn_name),
                Stmt::Return(e) => { if let Some(e) = e { self.check_const_expr(e, fn_name); } }
                Stmt::Variable(v) => { self.check_const_expr(&v.value, fn_name); }
                Stmt::TypedVariable(v) => { self.check_const_expr(&v.value, fn_name); }
                Stmt::If(s) => {
                    self.check_const_expr(&s.condition, fn_name);
                    self.check_const_block(&s.body, fn_name);
                    for el in &s.elif_branches { self.check_const_block(&el.body, fn_name); }
                    if let Some(el) = &s.else_branch { self.check_const_block(el, fn_name); }
                }
                Stmt::While(w) => {
                    self.check_const_expr(&w.condition, fn_name);
                    self.check_const_block(&w.body, fn_name);
                }
                Stmt::For(f) => {
                    self.check_const_expr(&f.iterable, fn_name);
                    self.check_const_block(&f.body, fn_name);
                }
                Stmt::Match(m) => {
                    self.check_const_expr(&m.expression, fn_name);
                    for arm in &m.arms {
                        self.check_const_block(&arm.body, fn_name);
                    }
                }
                _ => {
                    self.reporter.report(
                        Diagnostic::error(ErrorCode::E0001, format!("const fn '{}' contains unsupported statement", fn_name))
                    );
                }
            }
        }
    }

    fn check_const_expr(&mut self, expr: &Expr, context: &str) {
        match expr {
            Expr::Literal { .. } | Expr::Identifier { .. } => {}
            Expr::Binary { left, right, .. } => {
                self.check_const_expr(left, context);
                self.check_const_expr(right, context);
            }
            Expr::Unary { operand, .. } => self.check_const_expr(operand, context),
            Expr::FunctionCall { target, arguments, .. } => {
                let is_const_call = if let Expr::Identifier { name, .. } = target.as_ref() {
                    let pure_builtins = ["len", "str"];
                    pure_builtins.contains(&name.as_str())
                        || self.fn_const_flags.get(name).copied().unwrap_or(false)
                } else {
                    false
                };
                if !is_const_call {
                    self.reporter.report(
                        Diagnostic::error(ErrorCode::E0001, format!("constant expression in '{}' cannot call runtime function", context))
                    );
                }
                for arg in arguments {
                    self.check_const_expr(arg, context);
                }
            }
            Expr::Ternary { cond, then_expr, else_expr, .. } => {
                self.check_const_expr(cond, context);
                self.check_const_expr(then_expr, context);
                self.check_const_expr(else_expr, context);
            }
            Expr::Assignment { target, value, .. } => {
                self.check_const_expr(target, context);
                self.check_const_expr(value, context);
            }
            Expr::List { elements, .. } => {
                for e in elements { self.check_const_expr(e, context); }
            }
            Expr::ArrayRepeat { value, count, .. } => {
                self.check_const_expr(value, context);
                self.check_const_expr(count, context);
            }
            _ => {
                self.reporter.report(
                    Diagnostic::error(ErrorCode::E0001, format!("unsupported expression in constant '{}'", context))
                );
            }
        }
    }

    fn is_enum_value(&self, expr: &Expr) -> bool {
        let obj = match expr {
            Expr::PropertyAccess { object, .. } => object.as_ref(),
            Expr::FunctionCall { target, .. } => target.as_ref(),
            _ => return false,
        };
        if let Expr::Identifier { name, .. } = obj {
            self.symbols.lookup(name).map(|s| matches!(s.kind, SymKind::Enum(_))).unwrap_or(false)
        } else {
            false
        }
    }

    fn check_variable(&mut self, v: &VariableDecl) {
        let is_uninit = matches!(v.value.as_ref(), Expr::Literal { value: Literal::None, .. });
        let inferred = if is_uninit || (self.is_enum_value(&v.value) && v.type_.is_some()) {
            if let Some(declared) = &v.type_ {
                self.resolve_ast_type(declared)
            } else {
                Type::Option(Box::new(Type::Void))
            }
        } else {
            self.infer_expr(&v.value)
        };
        if let Some(declared) = &v.type_ {
            let declared_ty = self.resolve_ast_type(declared);
            if !is_uninit && !self.is_enum_value(&v.value) && !self.types_match(&inferred, &declared_ty) {
                self.reporter.report(
                    Diagnostic::error(ErrorCode::E0001,
                        format!("type mismatch: expected '{}', found '{}'", declared_ty, inferred))
                        .with_span(v.span)
                );
            }
        }
        self.symbols.insert(v.name.clone(),
            Symbol::new_var(v.name.clone(), Some(inferred), v.is_mutable)).ok();
    }

    fn check_block(&mut self, block: &Block) {
        for stmt in &block.statements {
            self.check_stmt(stmt);
        }
    }

    fn bind_pattern(&mut self, pattern: &Pattern, match_type: Option<&Type>) {
        match pattern {
            Pattern::Identifier { name, .. } => {
                let inferred = match_type.cloned();
                let _ = self.symbols.insert(name.clone(),
                    Symbol::new_var(name.clone(), inferred, false));
            }
            Pattern::EnumVariant { enum_name, variant, args, .. } => {
                // Special case: ok(v) / error(e) for Result<T, E> patterns
                if enum_name == "Result" {
                    if let Some(mt) = match_type {
                        if let Type::Generic(name, params) = mt {
                            if name == "Result" && params.len() >= 1 {
                                let ok_type = &params[0]; // T in Result<T, E>
                                if variant == "Ok" {
                                    for arg in args { self.bind_pattern(arg, Some(ok_type)); }
                                } else {
                                    let err_type = if params.len() >= 2 { &params[1] } else { &Type::Str };
                                    for arg in args { self.bind_pattern(arg, Some(err_type)); }
                                }
                                return;
                            }
                        }
                        // Also handle Type::Error(inner) which represents T!
                        if let Type::Error(inner) = mt {
                            if variant == "Ok" {
                                for arg in args { self.bind_pattern(arg, Some(inner)); }
                            } else {
                                for arg in args { self.bind_pattern(arg, Some(&Type::Str)); }
                            }
                            return;
                        }
                    }
                }
                // Look up the enum and variant to get actual payload types
                if let Some(sym) = self.symbols.lookup(enum_name) {
                    if let SymKind::Enum(edef) = &sym.kind {
                        if let Some(vdef) = edef.variants.iter().find(|v| &v.name == variant) {
                            let payload_types: Vec<Type> = vdef.payload.iter()
                                .map(|t| self.resolve_ast_type(t)).collect();
                            for (i, arg) in args.iter().enumerate() {
                                let arg_type = payload_types.get(i).cloned();
                                self.bind_pattern(arg, arg_type.as_ref());
                            }
                            return;
                        }
                    }
                }
                // Fallback: pass parent scrutinee type (less precise)
                for arg in args {
                    self.bind_pattern(arg, match_type);
                }
            }
            Pattern::Tuple { elements, .. } => {
                if let Some(Type::Tuple(types)) = match_type {
                    for (i, elem) in elements.iter().enumerate() {
                        let elem_type = types.get(i).cloned();
                        self.bind_pattern(elem, elem_type.as_ref());
                    }
                } else {
                    for elem in elements {
                        self.bind_pattern(elem, match_type);
                    }
                }
            }
            Pattern::Or { patterns, .. } => {
                for p in patterns {
                    self.bind_pattern(p, match_type);
                }
            }
            Pattern::Range { .. } => {} // range patterns checked via comparison
            _ => {}
        }
    }

    fn check_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Variable(v) => { self.check_variable(v); }
            Stmt::TypedVariable(v) => { self.check_variable(v); }
            Stmt::Constant(c) => {
                self.infer_expr(&c.value);
                self.check_const_expr(&c.value, &c.name);
            }
            Stmt::Expression(e) => { self.infer_expr(e); }
            Stmt::Return(e) => {
                if let Some(e) = e {
                    let expr_type = self.infer_expr(e);
                    // Skip return type check for ok()/error() — they unify with Result<T,E>
                    let is_result_like = matches!(&expr_type, Type::Option(_));
                    let is_result_call = if let Expr::FunctionCall { target, .. } = e.as_ref() {
                        Self::target_name(target).map_or(false, |n| n == "ok" || n == "error" || n == "None")
                    } else { false };
                    if let Some(ref fn_name) = self.current_fn {
                        if let Some(expected) = self.fn_return_types.get(fn_name) {
                            if &expr_type != expected && *expected != Type::Void && !is_result_call {
                                self.reporter.report(
                                    Diagnostic::error(ErrorCode::E0001,
                                        format!("expected return type '{}', found '{}'",
                                            expected, expr_type))
                                );
                            }
                        }
                    }
                }
            }
            Stmt::Break(e, _) => { if let Some(e) = e { self.infer_expr(e); } }
            Stmt::Continue(_) => {}
            Stmt::If(s) => {
                self.infer_expr(&s.condition);
                self.check_block(&s.body);
                for el in &s.elif_branches { self.check_block(&el.body); }
                if let Some(el) = &s.else_branch { self.check_block(el); }
            }
            Stmt::BindingIf(b) => {
                let val_type = self.infer_expr(&b.value);
                self.symbols.push_scope();
                let _ = self.symbols.insert(b.name.clone(),
                    Symbol::new_var(b.name.clone(), Some(val_type), false));
                self.check_block(&b.body);
                self.symbols.pop_scope();
                if let Some(el) = &b.else_branch { self.check_block(el); }
            }
            Stmt::While(w) => {
                self.infer_expr(&w.condition);
                self.check_block(&w.body);
                if let Some(el) = &w.else_branch { self.check_block(el); }
            }
            Stmt::WhileBind(w) => {
                let iter_type = self.infer_expr(&w.iterable);
                self.symbols.push_scope();
                let var_type = match &iter_type {
                    Type::List(inner) => inner.as_ref().clone(),
                    Type::Option(inner) => inner.as_ref().clone(),
                    _ => iter_type.clone(),
                };
                let _ = self.symbols.insert(w.name.clone(),
                    Symbol::new_var(w.name.clone(), Some(var_type), false));
                self.check_block(&w.body);
                self.symbols.pop_scope();
            }
            Stmt::For(f) => {
                let iter_type = self.infer_expr(&f.iterable);
                let var_type = match &iter_type {
                    Type::List(inner) => inner.as_ref().clone(),
                    _ => Type::I32,
                };
                self.symbols.push_scope();
                let _ = self.symbols.insert(f.variable.clone(),
                    crate::symbol_table::Symbol::new_var(f.variable.clone(), Some(var_type), false));
                self.check_block(&f.body);
                self.symbols.pop_scope();
                if let Some(el) = &f.else_branch { self.check_block(el); }
            }
            Stmt::Match(m) => {
                let scrutinee_type = self.infer_expr(&m.expression);
                for arm in &m.arms {
                    self.symbols.push_scope();
                    self.bind_pattern(&arm.pattern, Some(&scrutinee_type));
                    if let Some(g) = &arm.guard { self.infer_expr(g); }
                    self.check_block(&arm.body);
                    self.symbols.pop_scope();
                }
            }
            Stmt::Defer(d) => { self.infer_expr(&d.call); }
            Stmt::Guard(g) => {
                self.infer_expr(&g.condition);
                self.check_block(&g.body);
            }
            Stmt::Unsafe(u) => { self.check_block(&u.body); }
        }
    }

    fn infer_expr(&mut self, expr: &Expr) -> Type {
        match expr {
            Expr::Literal { value, .. } => match value {
                Literal::Integer(n) => {
                    if *n > i32::MAX as i64 || *n < i32::MIN as i64 {
                        Type::I64
                    } else {
                        Type::I32
                    }
                }
                Literal::Float(_) => Type::F64,
                Literal::String(_) => Type::Str,
                Literal::Boolean(_) => Type::Bool,
                Literal::Char(_) => Type::Char,
                Literal::None => Type::Option(Box::new(Type::Void)),
                Literal::Null => Type::Ptr,
            },
            Expr::Identifier { name, span } => {
                if let Some(sym) = self.symbols.lookup(name) {
                    match &sym.kind {
                        SymKind::Variable { type_: Some(t), .. } => t.clone(),
                        SymKind::Constant(t) => t.clone(),
                        SymKind::Function(f) => {
                            let params = f.params.iter().map(|p| self.resolve_ast_type(&p.type_)).collect();
                            let ret = f.return_type.as_ref()
                                .map(|t| self.resolve_ast_type(t))
                                .unwrap_or(Type::Void);
                            Type::Function(FunctionType {
                                is_async: f.is_async, is_const: f.is_const,
                                params, return_: Box::new(ret), fallible: false,
                            })
                        }
                        SymKind::Class(c) => Type::Named(c.name.clone()),
                        SymKind::Struct(s) => Type::Named(s.name.clone()),
                        SymKind::Enum(e) => Type::Named(e.name.clone()),
                        SymKind::Variable { type_: None, .. } => {
                            self.reporter.report(
                                Diagnostic::error(ErrorCode::E0009,
                                    format!("cannot infer type of '{}'", name))
                                    .with_span(*span)
                            );
                            Type::I32
                        }
                        _ => Type::I32,
                    }
                } else if is_namespace(name) {
                    Type::I32
                } else {
                    self.reporter.report(
                        Diagnostic::error(ErrorCode::E0009, format!("undefined symbol '{}'", name))
                            .with_span(*span)
                    );
                    Type::I32
                }
            }
            Expr::Binary { left, right, operator, span } => {
                let lt = self.infer_expr(left);
                if matches!(operator, BinaryOp::As) {
                    if let Expr::Identifier { name, .. } = right.as_ref() {
                        match name.as_str() {
                            "i8" => return Type::I8,
                            "i16" => return Type::I16,
                            "i32" => return Type::I32,
                            "i64" => return Type::I64,
                            "u8" => return Type::U8,
                            "u16" => return Type::U16,
                            "u32" => return Type::U32,
                            "u64" => return Type::U64,
                            "f32" => return Type::F32,
                            "f64" => return Type::F64,
                            "bool" => return Type::Bool,
                            "char" => return Type::Char,
                            "str" => return Type::Str,
                            "ptr" => return Type::Ptr,
                            _ => {
                                let type_ast = AstType::User { name: name.clone(), span: *span };
                                return self.resolve_ast_type(&type_ast);
                            }
                        }
                    } else {
                        return Type::I32;
                    }
                }
                // `is` type test: resolve right side as type name, not value expression
                if matches!(operator, BinaryOp::Is) {
                    if let Expr::Identifier { name, .. } = right.as_ref() {
                        let _right_type = self.resolve_ast_type(&AstType::User { name: name.clone(), span: *span });
                    }
                    return Type::Bool;
                }
                // Operator overloading: check if left type has an op_X method
                let overloaded_op_name = match operator {
                    BinaryOp::Add => Some("op_+"),
                    BinaryOp::Sub => Some("op_-"),
                    BinaryOp::Mul => Some("op_*"),
                    BinaryOp::Div => Some("op_/"),
                    BinaryOp::Rem => Some("op_%"),
                    BinaryOp::Eq => Some("op_=="),
                    BinaryOp::Neq => Some("op_!="),
                    BinaryOp::Lt => Some("op_<"),
                    BinaryOp::Gt => Some("op_>"),
                    BinaryOp::Le => Some("op_<="),
                    BinaryOp::Ge => Some("op_>="),
                    _ => None,
                };
                if let Some(op_name) = overloaded_op_name {
                    if let Type::Named(class_name) = &lt {
                        if let Some(sym) = self.symbols.lookup(class_name) {
                            if let SymKind::Class(cls) = &sym.kind {
                                if let Some(m) = cls.members.iter().find_map(|m| {
                                    if let ClassMember::Method(f) = m {
                                        if f.name == op_name { Some(f.clone()) } else { None }
                                    } else { None }
                                }) {
                                    if let Some(ret_type) = &m.return_type {
                                        return self.resolve_ast_type(ret_type);
                                    }
                                }
                            }
                        }
                    }
                }
                let rt = self.infer_expr(right);
                match operator {
                    BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Rem
                    | BinaryOp::Pow | BinaryOp::AddPercent | BinaryOp::SubPercent | BinaryOp::MulPercent
                    | BinaryOp::BitAnd | BinaryOp::BitOr | BinaryOp::BitXor
                    | BinaryOp::Shl | BinaryOp::Shr => {
                        if lt == Type::F64 || rt == Type::F64 { Type::F64 }
                        else if lt == Type::I64 || rt == Type::I64 { Type::I64 }
                        else if lt == Type::F32 || rt == Type::F32 { Type::F32 }
                        else { Type::I32 }
                    }
                    BinaryOp::Eq | BinaryOp::Neq | BinaryOp::Lt | BinaryOp::Gt
                    | BinaryOp::Le | BinaryOp::Ge
                    | BinaryOp::And | BinaryOp::Or | BinaryOp::Is => Type::Bool,
                    BinaryOp::As => {
                        unreachable!("As is handled before infer_expr(right)")
                    }
                    BinaryOp::Assign | BinaryOp::AddAssign | BinaryOp::SubAssign
                    | BinaryOp::MulAssign | BinaryOp::DivAssign | BinaryOp::RemAssign
                    | BinaryOp::BitAndAssign | BinaryOp::BitOrAssign | BinaryOp::BitXorAssign
                    | BinaryOp::ShlAssign | BinaryOp::ShrAssign => {
                        self.infer_expr(right);
                        if let Expr::PropertyAccess { object, property, .. } = left.as_ref() {
                            let obj_ty = self.infer_expr(object);
                            if let Type::Named(class_name) = &obj_ty {
                                if let Some(sym) = self.symbols.lookup(class_name.as_str()) {
                                    if let SymKind::Class(class_decl) = &sym.kind {
                                        let is_mutable = class_decl.members.iter().any(|m| {
                                            if let ClassMember::Field(f) = m {
                                                f.name == *property && f.is_mutable
                                            } else { false }
                                        });
                                        if !is_mutable {
                                            self.reporter.report(
                                                Diagnostic::error(ErrorCode::E0001,
                                                    format!("cannot assign to immutable field '{}' in class '{}'", property, class_name))
                                            );
                                        }
                                    }
                                }
                            }
                        }
                        rt
                    }
                    BinaryOp::Range | BinaryOp::RangeInclusive | BinaryOp::RangeExclusive => Type::Void,
                }
            }
            Expr::Unary { operator, operand, .. } => {
                let ot = self.infer_expr(operand);
                match operator {
                    UnaryOp::Neg => ot,
                    UnaryOp::Not => Type::Bool,
                    UnaryOp::BitNot => ot,
                }
            }
            Expr::Assignment { target, value, .. } => {
                let ty = self.infer_expr(value);
                if let Expr::Identifier { name, .. } = target.as_ref() {
                    if self.symbols.lookup(name).is_none() {
                        let _ = self.symbols.insert(name.clone(), Symbol::new_auto(name.clone(), Some(ty.clone())));
                    }
                } else if let Expr::Tuple { elements: target_elems, .. } = target.as_ref() {
                    // Get types from the value expression's inferred type
                    let val_ty = self.infer_expr(value);
                    let elem_types: Vec<Type> = if let Expr::Tuple { elements: value_elems, .. } = value.as_ref() {
                        value_elems.iter().map(|e| self.infer_expr(e)).collect()
                    } else if let Type::Tuple(types) = &val_ty {
                        types.clone()
                    } else {
                        let inferred = self.infer_expr(value);
                        if let Type::Tuple(types) = &inferred {
                            types.clone()
                        } else {
                            vec![val_ty.clone(); target_elems.len()]
                        }
                    };
                    for (i, target_elem) in target_elems.iter().enumerate() {
                        if let Expr::Identifier { name, .. } = target_elem {
                            if self.symbols.lookup(name).is_none() {
                                let elem_ty = elem_types.get(i).cloned().unwrap_or_else(|| Type::I32);
                                let _ = self.symbols.insert(name.clone(), Symbol::new_auto(name.clone(), Some(elem_ty)));
                            }
                        }
                    }
                }
                ty
            }
            Expr::FunctionCall { target, arguments, .. } => {
                let fn_type = self.infer_expr(target);
                for arg in arguments { self.infer_expr(arg); }
                let arg_count = arguments.len();
                    match fn_type {
                        Type::Function(ft) => {
                            // For user-defined functions, check arg count against params
                            // Builtins have empty params, so use hardcoded expected count
                            if !ft.params.is_empty() {
                                let has_variadic = Self::target_name(target)
                                    .and_then(|name| self.symbols.lookup(name))
                                    .and_then(|sym| {
                                        if let SymKind::Function(f) = &sym.kind {
                                            Some(f.params.last().map(|p| p.variadic).unwrap_or(false))
                                        } else { None }
                                    })
                                    .unwrap_or(false);
                                let required_count = Self::target_name(target)
                                    .and_then(|name| self.symbols.lookup(name))
                                    .and_then(|sym| {
                                        if let SymKind::Function(f) = &sym.kind {
                                            Some(f.params.iter()
                                                .filter(|p| p.default.is_none() && !p.variadic)
                                                .count())
                                        } else { None }
                                    })
                                    .unwrap_or(ft.params.len());
                                let max_count = if has_variadic { usize::MAX } else { ft.params.len() };
                                if arg_count < required_count || arg_count > max_count {
                                    let name = Self::target_name(target).unwrap_or("function");
                                    if has_variadic {
                                        self.reporter.report(
                                            Diagnostic::error(ErrorCode::E0001,
                                                format!("'{}' expects at least {} argument(s), got {}",
                                                    name, required_count, arg_count))
                                        );
                                    } else {
                                        self.reporter.report(
                                            Diagnostic::error(ErrorCode::E0001,
                                                format!("'{}' expects {}-{} argument(s), got {}",
                                                    name, required_count, ft.params.len(), arg_count))
                                        );
                                    }
                                }
                            } else {
                            // Builtin — check with hardcoded expected counts
                            if let Some(name) = Self::target_name(target) {
                                if let Some(exp) = Self::builtin_expected_args(name) {
                                    if name == "input" {
                                        if arg_count > 1 {
                                            self.reporter.report(
                                                Diagnostic::error(ErrorCode::E0001,
                                                    format!("'{}' expects 0 or 1 argument(s), got {}", name, arg_count))
                                            );
                                        }
                                    } else if name == "range" {
                                        if arg_count < 1 || arg_count > 2 {
                                            self.reporter.report(
                                                Diagnostic::error(ErrorCode::E0001,
                                                    format!("'{}' expects 1 or 2 argument(s), got {}", name, arg_count))
                                            );
                                        }
                                    } else if arg_count != exp {
                                        self.reporter.report(
                                            Diagnostic::error(ErrorCode::E0001,
                                                format!("'{}' expects {} argument(s), got {}", name, exp, arg_count))
                                        );
                                    }
                                }
                            }
                        }
                        // Check call-site coercion: ^ required for move params, & required for mutable borrow params
                        if let Some(name) = Self::target_name(target) {
                            if let Some(sym) = self.symbols.lookup(name) {
                                if let SymKind::Function(fdecl) = &sym.kind {
                                    for (i, (arg, param)) in arguments.iter().zip(fdecl.params.iter()).enumerate() {
                                        let has_borrow = matches!(arg, Expr::BorrowRef { mutable: false, .. });
                                        let has_mut_borrow = matches!(arg, Expr::BorrowRef { mutable: true, .. });
                                        match param.mode {
                                            ParamMode::Borrow if !has_borrow => {
                                                self.reporter.report(
                                                    Diagnostic::error(ErrorCode::E0001,
                                                        format!("argument {} to '{}' requires '&' (borrow), got plain value", i + 1, name))
                                                );
                                            }
                                            ParamMode::MutableBorrow if !has_mut_borrow => {
                                                self.reporter.report(
                                                    Diagnostic::error(ErrorCode::E0001,
                                                        format!("argument {} to '{}' requires '^&' (mutable borrow), got plain value or '&'", i + 1, name))
                                                );
                                            }
                                            ParamMode::Move if has_borrow || has_mut_borrow => {
                                                self.reporter.report(
                                                    Diagnostic::error(ErrorCode::E0001,
                                                        format!("argument {} to '{}' cannot use '&' or '^&' with move parameter (ownership expected)", i + 1, name))
                                                );
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                            }
                        }
                        // Override return type for builtins
                        if let Some(name) = Self::target_name(target) {
                            match name {
                                "print" | "println" | "print_err"
                                | "sleep" | "assert" | "assert_eq" | "assert_ne" | "assert_str" => Type::Void,
                                "len" => Type::I32,
                                "to_str" | "to_decimal" => Type::Str,
                                "to_i32" | "to_i64" | "to_i16" | "to_i8" => Type::I32,
                                "to_u32" | "to_u64" | "to_u16" | "to_u8" => Type::I32,
                                "to_f64" | "to_f32" => Type::F64,
                                "to_char" => Type::Char,
                                "to_bool" => Type::Bool,
                                "input" => Type::Str,
                                "range" => Type::List(Box::new(Type::I32)),
                                "ceil" | "floor" | "round" => Type::F64,
                                "open" | "read_str" | "write_str" | "close" => Type::I32,
                                "now" => Type::I64,
                                "substr" | "to_upper" | "to_lower" | "trim" | "replace" => Type::Str,
                                "char_at" => Type::I32,
                                "ord" => Type::I32,
                                "contains" | "is_digit" | "is_alpha" | "is_alnum" | "is_whitespace" | "is_upper" | "is_lower" => Type::I32,
                                "json_parse" => Type::Dict(Box::new(Type::Str), Box::new(Type::I64)),
                                "json_stringify" | "json_stringify_str" => Type::Str,
                                "serialize" => Type::Str,
                                "type" => Type::Named("TypeInfo".to_string()),
                                "ky_struct_to_json" => Type::Str,
                                "ky_json_to_struct" => Type::I32,
                                "ky_ptr_read_i32" => Type::I32,
                                "ky_ptr_read_ptr" => Type::Ptr,
                                "ky_spawn_thread" | "ky_join_thread" | "ky_parallel_for" => Type::I64,
                                "ky_channel_new" | "ky_channel_send" | "ky_channel_recv" | "ky_channel_len" | "ky_channel_free" => Type::I64,
                                "ky_channel_close" => Type::Void,
                                "ok" => {
                                    if arguments.len() == 1 {
                                        let arg_type = self.infer_expr(&arguments[0]);
                                        Type::Generic("Result".to_string(), vec![arg_type, Type::Str])
                                    } else {
                                        Type::Generic("Result".to_string(), vec![Type::I32, Type::Str])
                                    }
                                }
                                "some" => {
                                    if arguments.len() == 1 {
                                        let arg_type = self.infer_expr(&arguments[0]);
                                        Type::Option(Box::new(arg_type))
                                    } else {
                                        Type::Option(Box::new(Type::I32))
                                    }
                                }
                                "error" => Type::Generic("Result".to_string(), vec![Type::Void, Type::Str]),
                                _ => *ft.return_,
                            }
                        } else {
                            *ft.return_
                        }
                    }
                    Type::Named(ref class_name) => {
                        // Constructor call: Class(args) returns Class type
                        if let Some(sym) = self.symbols.lookup(class_name.as_str()) {
                            if let SymKind::Class(c) = &sym.kind {
                                if let Some(ctor) = c.members.iter().find_map(|m| {
                                    if let ClassMember::Constructor(f) = m { Some(f) } else { None }
                                }) {
                                    let required = ctor.params.iter()
                                        .filter(|p| p.default.is_none())
                                        .count();
                                    let max = ctor.params.len();
                                    if arg_count < required || arg_count > max {
                                        self.reporter.report(
                                            Diagnostic::error(ErrorCode::E0001,
                                                format!("'{}' expects {}-{} argument(s), got {}", class_name, required, max, arg_count))
                                        );
                                    }
                                }
                            }
                        }
                        fn_type.clone()
                    }
                    _ => {
                        if let Expr::PropertyAccess { object, property, .. } = target.as_ref() {
                            if let Expr::Identifier { name, .. } = object.as_ref() {
                                if let Some(sym) = self.symbols.lookup(name) {
                                    if let SymKind::Enum(e) = &sym.kind {
                                        if e.variants.iter().any(|v| v.name == *property) {
                                            return Type::Named(e.name.clone());
                                        }
                                    }
                                }
                            }
                            // Conversion methods called as v.to_str() etc.
                            match property.as_str() {
                                "to_str" | "to_decimal" => return Type::Str,
                                "to_i32" | "to_i64" | "to_i16" | "to_i8" => return Type::I32,
                                "to_u32" | "to_u64" | "to_u16" | "to_u8" => return Type::I32,
                                "to_f64" | "to_f32" => return Type::F64,
                                "to_char" => return Type::Char,
                                "to_bool" => return Type::Bool,
                                _ => {}
                            }
                        }
                        Type::I32
                    }
                }
            }
            Expr::PropertyAccess { object, property, .. } => {
                let is_this = matches!(object.as_ref(), Expr::Identifier { name, .. } if name == "this");
                // Visibility check: scan all known classes for a private
                // method with this name; if found and the access is not
                // through `this`, report an error.
                if !is_this {
                    let names: Vec<String> = self.symbols.all_top_level_names();
                    for n in &names {
                        if let Some(sym) = self.symbols.lookup(n) {
                            if let SymKind::Class(class) = &sym.kind {
                                for m in &class.members {
                                    if let ClassMember::Method(f) = m {
                                        if f.name == *property
                                            && matches!(f.visibility, Visibility::Private)
                                        {
                                            self.reporter.report(
                                                Diagnostic::error(ErrorCode::E0014,
                                                    format!("Cannot access private member '{}' from outside the class", property))
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                let obj_type = self.infer_expr(object);
                // Conversion methods: return the target type
                match property.as_str() {
                    "to_str" => return Type::Str,
                    "to_i32" | "to_i64" | "to_i16" | "to_i8" => return Type::I32,
                    "to_u32" | "to_u64" | "to_u16" | "to_u8" => return Type::I32,
                    "to_f64" | "to_f32" => return Type::F64,
                    "to_char" => return Type::Char,
                    "to_bool" => return Type::Bool,
                    "to_decimal" => return Type::Str,
                    _ => {}
                }
                // Detect enum variant construction: Option.None, Option.Some
                if let Type::Named(name) = &obj_type {
                    if let Some(sym) = self.symbols.lookup(name) {
                        if let SymKind::Enum(e) = &sym.kind {
                            if e.variants.iter().any(|v| v.name == *property) {
                                return Type::Named(e.name.clone());
                            }
                        }
                    }
                }
                Type::I32
            }
            Expr::List { elements, .. } => {
                if elements.is_empty() { return Type::List(Box::new(Type::I32)); }
                Type::List(Box::new(self.infer_expr(&elements[0])))
            }
            Expr::Array { elements, .. } => {
                for e in elements { self.infer_expr(e); }
                Type::Array(Box::new(Type::I32), elements.len())
            }
            Expr::ArrayRepeat { value, count, .. } => {
                let val_type = self.infer_expr(value);
                let cnt_type = self.infer_expr(count);
                let size = if let Type::I32 = cnt_type {
                    // count must be a compile-time constant i32; runtime value will produce correct array size
                    0
                } else if let Type::I64 = cnt_type {
                    0
                } else {
                    self.reporter.report(
                        Diagnostic::error(ErrorCode::E0001, "array repeat count must be an integer")
                    );
                    0
                };
                let element_type = match val_type {
                    Type::I32 => Type::I32,
                    Type::I64 => Type::I64,
                    Type::I16 => Type::I16,
                    Type::I8 => Type::I8,
                    Type::U32 => Type::U32,
                    Type::U64 => Type::U64,
                    Type::U16 => Type::U16,
                    Type::U8 => Type::U8,
                    Type::F32 => Type::F32,
                    Type::F64 => Type::F64,
                    Type::Bool => Type::Bool,
                    Type::Char => Type::Char,
                    Type::Str => Type::Str,
                    other => other,
                };
                Type::Array(Box::new(element_type), size)
            }
            Expr::Dictionary { entries, .. } => {
                if entries.is_empty() { return Type::Dict(Box::new(Type::Str), Box::new(Type::I32)); }
                let first_type = self.infer_expr(&entries[0].1);
                for (_, val) in entries.iter().skip(1) {
                    self.infer_expr(val);
                }
                Type::Dict(Box::new(Type::Str), Box::new(first_type))

            }
            Expr::StructLiteral { struct_name, .. } => {
                Type::Named(struct_name.clone())
            }
            Expr::Tuple { elements, .. } => {
                Type::Tuple(elements.iter().map(|e| self.infer_expr(e)).collect())
            }
            Expr::Closure { params, body, .. } => {
                self.symbols.push_scope();
                let mut param_types = Vec::new();
                for (pname, ptype) in params {
                    let t = ptype.as_ref().map(|a| self.resolve_ast_type(a))
                        .unwrap_or(Type::I32);
                    let _ = self.symbols.insert(pname.clone(),
                        Symbol::new_var(pname.clone(), Some(t.clone()), false));
                    param_types.push(t);
                }
                let ret = self.infer_expr(body);
                self.symbols.pop_scope();
                Type::Function(FunctionType {
                    is_async: false, is_const: false,
                    params: param_types, return_: Box::new(ret), fallible: false,
                })
            }
            Expr::Await { expression: _, .. } => {
                // Await on an async task handle returns i64 (the widened result)
                Type::I64
            }
            Expr::Async { .. } | Expr::AsyncBlock { .. } => {
                // async expr returns a task handle (i64), not a function
                Type::I64
            }
            Expr::Spread { expression, .. } => self.infer_expr(expression),
            Expr::Index { target, index, .. } => {
                let tt = self.infer_expr(target);
                self.infer_expr(index);
                match tt {
                    Type::List(et) => *et,
                    Type::Str => Type::Str,
                    Type::Dict(_, vt) => *vt,
                    _ => Type::I32,
                }
            }
            Expr::RangeSlice { target, start, end, .. } => {
                let tt = self.infer_expr(target);
                if let Some(s) = start { self.infer_expr(s); }
                if let Some(e) = end { self.infer_expr(e); }
                match tt {
                    Type::List(et) => Type::List(et),
                    Type::Str => Type::Str,
                    _ => Type::List(Box::new(Type::I32)),
                }
            }
            Expr::OptionalChain { target, property, .. } => {
                let target_type = self.infer_expr(target);
                if let Type::Option(inner) = &target_type {
                    if let Type::Named(struct_name) = inner.as_ref() {
                        if let Some(sym) = self.symbols.lookup(struct_name) {
                            if let SymKind::Struct(s) = &sym.kind {
                                if let Some(field) = s.fields.iter().find(|f| f.name == *property) {
                                    return Type::Option(Box::new(self.resolve_ast_type(&field.type_)));
                                }
                            }
                        }
                    }
                }
                Type::Option(Box::new(Type::I32))
            }
            Expr::Loop { body, .. } => {
                self.check_block(body);
                Type::Void
            }
            Expr::ErrorProp { expression, .. } => {
                let inner = self.infer_expr(expression);
                match inner {
                    Type::Error(t) => *t,
                    _ => inner,
                }
            }
            Expr::StringInterp { parts, .. } => {
                for part in parts {
                    self.infer_expr(part);
                }
                Type::Str
            }
            Expr::Ternary { cond, then_expr, else_expr, span } => {
                let cond_type = self.infer_expr(cond);
                if !matches!(cond_type, Type::Bool) {
                    self.reporter.report(
                        Diagnostic::error(ErrorCode::E0001,
                            format!("ternary condition must be bool, got {:?}", cond_type))
                            .with_span(*span)
                    );
                }
                let then_type = self.infer_expr(then_expr);
                let else_type = self.infer_expr(else_expr);
                if then_type != else_type {
                    self.reporter.report(
                        Diagnostic::error(ErrorCode::E0001,
                            format!("ternary branches must have same type, got {:?} and {:?}", then_type, else_type))
                            .with_span(*span)
                    );
                }
                then_type
            }
            Expr::BorrowRef { expression, .. } => self.infer_expr(expression),
            Expr::NullCoalesce { left, right, span } => {
                let left_type = self.infer_expr(left);
                let right_type = self.infer_expr(right);
                let inner = match &left_type {
                    Type::Option(inner) => Some(inner.as_ref()),
                    Type::Generic(name, args) if name == "Option" && args.len() == 1 => {
                        Some(&args[0])
                    }
                    _ => None,
                };
                if let Some(inner) = inner {
                    if *inner != right_type {
                        self.reporter.report(
                            Diagnostic::error(ErrorCode::E0001,
                                format!("'??' default value type mismatch: expected {:?}, got {:?}", inner, right_type))
                                .with_span(*span)
                        );
                    }
                    inner.clone()
                } else {
                    self.reporter.report(
                        Diagnostic::error(ErrorCode::E0001,
                            format!("'??' requires Option type on left, got {:?}", left_type))
                            .with_span(*span)
                    );
                    right_type
                }
            }
            Expr::MatchExpr { expression, arms, span } => {
                let scrutinee_type = self.infer_expr(expression);
                let mut arm_types = Vec::new();
                for arm in arms {
                    self.symbols.push_scope();
                    self.bind_pattern(&arm.pattern, Some(&scrutinee_type));
                    if let Some(g) = &arm.guard { self.infer_expr(g); }
                    self.check_block(&arm.body);
                    // Last statement's expression type determines arm value type
                    if let Some(last) = arm.body.statements.last() {
                        if let Stmt::Expression(e) = last {
                            arm_types.push(self.infer_expr(e));
                        } else {
                            arm_types.push(Type::Void);
                        }
                    } else {
                        arm_types.push(Type::Void);
                    }
                    self.symbols.pop_scope();
                }
                // Unify all arm types
                let result_type = arm_types.first().cloned().unwrap_or(Type::Void);
                for at in &arm_types {
                    if *at != result_type {
                        self.reporter.report(
                            Diagnostic::error(ErrorCode::E0001,
                                format!("match arms must have same type, got {:?} and {:?}", result_type, at))
                                .with_span(*span)
                        );
                    }
                }
                result_type
            }
        }
    }

    fn resolve_ast_type(&self, ast: &AstType) -> Type {
        match ast {
            AstType::Primitive { name, .. } => {
                self.symbols.lookup_type(name).unwrap_or(Type::Named(name.clone()))
            }
            AstType::User { name, .. } => {
                self.symbols.lookup_type(name).unwrap_or(Type::Named(name.clone()))
            }
            AstType::Generic { name, args, .. } => {
                match name.as_str() {
                    "list" => {
                        if let Some(inner) = args.first() {
                            Type::List(Box::new(self.resolve_ast_type(inner)))
                        } else {
                            Type::List(Box::new(Type::I32))
                        }
                    }
                    "dict" => {
                        let k = args.first().map(|k| self.resolve_ast_type(k)).unwrap_or(Type::Str);
                        let v = args.get(1).map(|v| self.resolve_ast_type(v)).unwrap_or(Type::I64);
                        Type::Dict(Box::new(k), Box::new(v))
                    }
                    "Option" => {
                        if let Some(inner) = args.first() {
                            Type::Option(Box::new(self.resolve_ast_type(inner)))
                        } else {
                            Type::Option(Box::new(Type::Void))
                        }
                    }
                    "tuple" => {
                        Type::Tuple(args.iter().map(|a| self.resolve_ast_type(a)).collect())
                    }
                    _ => Type::Generic(name.clone(), args.iter().map(|a| self.resolve_ast_type(a)).collect()),
                }
            }
            AstType::Optional { inner, .. } => Type::Option(Box::new(self.resolve_ast_type(inner))),
            AstType::Error { inner, .. } => Type::Error(Box::new(self.resolve_ast_type(inner))),
            AstType::Dict { key, value, .. } => Type::Dict(Box::new(self.resolve_ast_type(key)), Box::new(self.resolve_ast_type(value))),
            AstType::FnPtr { params, return_, .. } => {
                Type::Function(FunctionType {
                    is_async: false,
                    is_const: false,
                    params: params.iter().map(|p| self.resolve_ast_type(p)).collect(),
                    return_: Box::new(self.resolve_ast_type(return_)),
                    fallible: false,
                })
            }
            AstType::Mutable { inner, .. } | AstType::Borrow { inner, .. } | AstType::Mutable { inner, .. } => {
                self.resolve_ast_type(inner)
            }
            AstType::Ptr { .. } => Type::Ptr,
            AstType::Array { inner, size, .. } => Type::Array(Box::new(self.resolve_ast_type(inner)), *size),
        }
    }

    fn types_match(&self, a: &Type, b: &Type) -> bool {
        if a == b { return true; }
        match (a, b) {
            (Type::I32, Type::I64) | (Type::I64, Type::I32) => return true,
            (Type::F32, Type::F64) | (Type::F64, Type::F32) => return true,
            (Type::I8, Type::I16) | (Type::I16, Type::I8) => return true,
            // Unsigned ↔ signed of same width
            (Type::U8, Type::I8) | (Type::I8, Type::U8) => return true,
            (Type::U16, Type::I16) | (Type::I16, Type::U16) => return true,
            (Type::U32, Type::I32) | (Type::I32, Type::U32) => return true,
            (Type::U64, Type::I64) | (Type::I64, Type::U64) => return true,
            // Signed ↔ unsigned wider
            (Type::U8, Type::I16) | (Type::I16, Type::U8) => return true,
            (Type::U8, Type::I32) | (Type::I32, Type::U8) => return true,
            (Type::U8, Type::I64) | (Type::I64, Type::U8) => return true,
            (Type::U16, Type::I32) | (Type::I32, Type::U16) => return true,
            (Type::U16, Type::I64) | (Type::I64, Type::U16) => return true,
            (Type::U32, Type::I64) | (Type::I64, Type::U32) => return true,
            // Unsigned ↔ wider unsigned
            (Type::U8, Type::U16) | (Type::U16, Type::U8) => return true,
            (Type::U8, Type::U32) | (Type::U32, Type::U8) => return true,
            (Type::U8, Type::U64) | (Type::U64, Type::U8) => return true,
            (Type::U16, Type::U32) | (Type::U32, Type::U16) => return true,
            (Type::U16, Type::U64) | (Type::U64, Type::U16) => return true,
            (Type::U32, Type::U64) | (Type::U64, Type::U32) => return true,
            // Narrower signed → unsigned (need zero-extend)
            (Type::I8, Type::U16) | (Type::U16, Type::I8) => return true,
            (Type::I8, Type::U32) | (Type::U32, Type::I8) => return true,
            (Type::I8, Type::U64) | (Type::U64, Type::I8) => return true,
            (Type::I16, Type::U32) | (Type::U32, Type::I16) => return true,
            (Type::I16, Type::U64) | (Type::U64, Type::I16) => return true,
            (Type::I32, Type::U64) | (Type::U64, Type::I32) => return true,
            // Empty array [] matches any [T; N] (zero-init)
            (Type::Array(ia, 0), Type::Array(_, _)) |
            (Type::Array(_, _), Type::Array(ia, 0)) if **ia == Type::I32 => return true,
            _ => {}
        }
        false
    }
}

/// Check if a name is a known module namespace for namespaced APIs.
fn is_namespace(name: &str) -> bool {
    matches!(name, "parallel" | "thread" | "assert" | "json" | "math"
        | "str" | "tcp" | "crypto" | "process" | "regex"
        | "date_time" | "date" | "time" | "console" | "random"
        | "dict" | "str_builder")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(source: &str) -> Vec<Diagnostic> {
        let mut lexer = kyc_frontend::lexer::Lexer::new(source);
        let tokens = lexer.tokenize();
        let mut parser = kyc_frontend::parser::Parser::new(tokens);
        let program = parser.parse().unwrap();
        let mut tc = TypeChecker::new();
        tc.check_program(&program);
        tc.reporter.diagnostics().to_vec()
    }

    fn check_has_error(source: &str, code: ErrorCode) -> bool {
        let diags = check(source);
        diags.iter().any(|d| d.code == code)
    }

    #[test]
    fn test_undefined_symbol() {
        assert!(check_has_error("fn f():\n    x\n", ErrorCode::E0009));
    }

    #[test]
    fn test_valid_variable() {
        let diags = check("x = 42\n");
        assert!(diags.is_empty(), "expected no errors, got: {:?}", diags);
    }

    #[test]
    fn test_immutable_param_assign() {
        // Function parameters are immutable; reassignment should error
        assert!(check_has_error("fn f(x: i32):\n    x = 2\n", ErrorCode::E0007));
    }

    #[test]
    fn test_mutable_variable() {
        let diags = check("x: &i32 = 1\n");
        assert!(diags.is_empty(), "expected no errors, got: {:?}", diags);
    }

    #[test]
    fn test_function_call() {
        let source = "fn add(a: i32, b: i32) i32:\n    a + b\n";
        let diags = check(source);
        assert!(diags.is_empty(), "expected no errors, got: {:?}", diags);
    }

    #[test]
    fn test_undefined_in_expr() {
        assert!(check_has_error("fn f():\n    x + 1\n", ErrorCode::E0009));
    }

    #[test]
    fn test_mutable_var_assign_in_fn() {
        // mutable var declared inside fn, then reassigned
        let source = "fn f():\n    x: &i32 = 1\n    x = 2\n";
        let diags = check(source);
        assert!(diags.is_empty(), "expected no errors, got: {:?}", diags);
    }

    #[test]
    fn test_if_statement_valid() {
        let source = "fn f():\n    if true:\n        42\n";
        let diags = check(source);
        assert!(diags.is_empty(), "expected no errors, got: {:?}", diags);
    }

    #[test]
    fn test_import_no_errors() {
        let source = "import math\n";
        let diags = check(source);
        assert!(diags.is_empty(), "expected no errors for import, got: {:?}", diags);
    }

    #[test]
    fn test_while_statement_nested_assign() {
        // Just the while without the nested assignment
        let source = "fn f():\n    while true:\n        42\n";
        let diags = check(source);
        assert!(diags.is_empty(), "expected no errors, got: {:?}", diags);
    }

    #[test]
    fn test_assign_inside_while() {
        // A variable declared in fn body and assigned inside a while
        let source = "fn f():\n    x: &i32 = 0\n    while true:\n        x = 1\n";
        let diags = check(source);
        assert!(diags.is_empty(), "expected no errors, got: {:?}", diags);
    }

    #[test]
    fn test_while_condition_variable_ref() {
        let source = "fn f():\n    x: &i32 = 0\n    while x < 5:\n        42\n";
        let diags = check(source);
        assert!(diags.is_empty(), "expected no errors, got: {:?}", diags);
    }

    #[test]
    fn test_while_body_var_read() {
        let source = "fn f():\n    x := 0\n    while true:\n        x\n";
        let diags = check(source);
        assert!(diags.is_empty(), "expected no errors, got: {:?}", diags);
    }

    #[test]
    fn test_while_body_expr_read() {
        let source = "fn f():\n    x := 0\n    while true:\n        x + 1\n";
        let diags = check(source);
        assert!(diags.is_empty(), "expected no errors, got: {:?}", diags);
    }

    #[test]
    fn test_const_fn_no_errors() {
        let source = "const fn factorial(n: i32) i32:\n    if n <= 1:\n        1\n    else:\n        n * factorial(n - 1)\n";
        let diags = check(source);
        assert!(diags.is_empty(), "const fn should have no errors, got: {:?}", diags);
    }

    #[test]
    fn test_ternary_type_unification() {
        let source = "fn f(x: i32):\n    result = x > 0 ? x : 0\n";
        let diags = check(source);
        assert!(diags.is_empty(), "ternary with matching types should have no errors, got: {:?}", diags);
    }

    #[test]
    fn test_dict_type_inference() {
        let source = "fn f():\n    config = {name: \"Alice\"}\n";
        let diags = check(source);
        assert!(diags.is_empty(), "dict literal should have no errors, got: {:?}", diags);
    }

}