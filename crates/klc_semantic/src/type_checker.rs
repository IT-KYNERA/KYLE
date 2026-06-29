use std::collections::HashMap;
use klc_core::ast::*;
use klc_core::types::*;
use klc_core::source_map::SourceMap;
use klc_core::diagnostic::{Diagnostic, ErrorCode, DiagnosticReporter};
use crate::symbol_table::{SymbolTable, Symbol, SymKind};
use crate::scope::ScopeResolver;

pub struct TypeChecker {
    pub reporter: DiagnosticReporter,
    symbols: SymbolTable,
    pub fn_return_types: HashMap<String, Type>,
    fn_const_flags: HashMap<String, bool>,
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
            _ => None,
        }
    }

    /// Hardcoded expected argument counts for built-in functions.
    fn builtin_expected_args(name: &str) -> Option<usize> {
        match name {
            "print" | "println" | "print_err" => Some(1),
            "print_int" | "println_int" => Some(1),
            "str" | "int" | "float" | "bool" => Some(1),
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
            "json_parse" | "json_stringify" => Some(1),
            "list_push" => Some(2),
            "list_pop" | "list_len" => Some(1),
            "ceil" | "floor" | "round" => Some(1),
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
                    let _ = self.symbols.insert(c.name.clone(), Symbol::new(c.name.clone(), SymKind::Constant(ty)));
                }
                _ => {}
            }
        }
    }

    fn check_function(&mut self, f: &FunctionDecl) {
        self.symbols.push_scope();
        for param in &f.params {
            let ty = self.resolve_ast_type(&param.type_);
            let _ = self.symbols.insert(param.name.clone(),
                Symbol::new_var(param.name.clone(), Some(ty), false));
        }
        if let Some(body) = &f.body {
            if f.is_const {
                self.check_const_block(body, &f.name);
            } else {
                self.check_block(body);
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

    fn check_const_expr(&mut self, expr: &Expr, fn_name: &str) {
        match expr {
            Expr::Literal { .. } | Expr::Identifier { .. } => {}
            Expr::Binary { left, right, .. } => {
                self.check_const_expr(left, fn_name);
                self.check_const_expr(right, fn_name);
            }
            Expr::Unary { operand, .. } => self.check_const_expr(operand, fn_name),
            Expr::FunctionCall { target, arguments, .. } => {
                // Allow calls to other const fns and pure builtins
                let is_const_call = if let Expr::Identifier { name, .. } = target.as_ref() {
                    let pure_builtins = ["len", "str"];
                    pure_builtins.contains(&name.as_str())
                        || self.fn_const_flags.get(name).copied().unwrap_or(false)
                } else {
                    false
                };
                if !is_const_call {
                    self.reporter.report(
                        Diagnostic::error(ErrorCode::E0001, format!("const fn '{}' cannot call runtime function", fn_name))
                    );
                }
                for arg in arguments {
                    self.check_const_expr(arg, fn_name);
                }
            }
            Expr::Ternary { cond, then_expr, else_expr, .. } => {
                self.check_const_expr(cond, fn_name);
                self.check_const_expr(then_expr, fn_name);
                self.check_const_expr(else_expr, fn_name);
            }
            Expr::Assignment { target, value, .. } => {
                self.check_const_expr(target, fn_name);
                self.check_const_expr(value, fn_name);
            }
            Expr::List { elements, .. } => {
                for e in elements { self.check_const_expr(e, fn_name); }
            }
            // Disallow all other expressions in const fn
            _ => {
                self.reporter.report(
                    Diagnostic::error(ErrorCode::E0001, format!("const fn '{}' contains unsupported expression", fn_name))
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
            Pattern::EnumVariant { args, .. } => {
                // Enum variant payload: each arg takes the type of the parent scrutinee
                for arg in args {
                    self.bind_pattern(arg, match_type);
                }
            }
            _ => {}
        }
    }

    fn check_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Variable(v) => { self.check_variable(v); }
            Stmt::TypedVariable(v) => { self.check_variable(v); }
            Stmt::Constant(_) => {}
            Stmt::Expression(e) => { self.infer_expr(e); }
            Stmt::Return(e) => { if let Some(e) = e { self.infer_expr(e); } }
            Stmt::Break(e) => { if let Some(e) = e { self.infer_expr(e); } }
            Stmt::Continue => {}
            Stmt::If(s) => {
                self.infer_expr(&s.condition);
                self.check_block(&s.body);
                for el in &s.elif_branches { self.check_block(&el.body); }
                if let Some(el) = &s.else_branch { self.check_block(el); }
            }
            Stmt::BindingIf(b) => {
                self.infer_expr(&b.value);
                self.check_block(&b.body);
                if let Some(el) = &b.else_branch { self.check_block(el); }
            }
            Stmt::While(w) => {
                self.infer_expr(&w.condition);
                self.check_block(&w.body);
                if let Some(el) = &w.else_branch { self.check_block(el); }
            }
            Stmt::WhileBind(w) => {
                self.infer_expr(&w.iterable);
                self.check_block(&w.body);
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
                Literal::Integer(_) => Type::I32,
                Literal::Float(_) => Type::F64,
                Literal::String(_) => Type::Str,
                Literal::Boolean(_) => Type::Bool,
                Literal::None => Type::Option(Box::new(Type::Void)),
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
                } else {
                    self.reporter.report(
                        Diagnostic::error(ErrorCode::E0009, format!("undefined symbol '{}'", name))
                            .with_span(*span)
                    );
                    Type::I32
                }
            }
            Expr::Binary { left, right, operator, .. } => {
                let lt = self.infer_expr(left);
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
                    BinaryOp::Assign | BinaryOp::AddAssign | BinaryOp::SubAssign
                    | BinaryOp::MulAssign | BinaryOp::DivAssign | BinaryOp::RemAssign
                    | BinaryOp::BitAndAssign | BinaryOp::BitOrAssign | BinaryOp::BitXorAssign
                    | BinaryOp::ShlAssign | BinaryOp::ShrAssign => { self.infer_expr(right); rt }
                    BinaryOp::Range => Type::Void,
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
                    if let Expr::Tuple { elements: value_elems, .. } = value.as_ref() {
                        for (target_elem, value_elem) in target_elems.iter().zip(value_elems.iter()) {
                            let elem_ty = self.infer_expr(value_elem);
                            if let Expr::Identifier { name, .. } = target_elem {
                                if self.symbols.lookup(name).is_none() {
                                    let _ = self.symbols.insert(name.clone(), Symbol::new_auto(name.clone(), Some(elem_ty)));
                                }
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
                            if arg_count != ft.params.len() {
                                let name = Self::target_name(target).unwrap_or("function");
                                self.reporter.report(
                                    Diagnostic::error(ErrorCode::E0001,
                                        format!("'{}' expects {} argument(s), got {}", name, ft.params.len(), arg_count))
                                );
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
                        // Override return type for builtins
                        if let Some(name) = Self::target_name(target) {
                            match name {
                                "print" | "println" | "print_err" | "print_int" | "println_int"
                                | "sleep" | "assert" | "assert_eq" | "assert_str" => Type::Void,
                                "len" => Type::I32,
                                "str" => Type::Str,
                                "int" => Type::I32,
                                "float" => Type::F64,
                                "bool" => Type::Bool,
                                "input" => Type::Str,
                                "range" => Type::List(Box::new(Type::I32)),
                                "ceil" | "floor" | "round" => Type::F64,
                                "open" | "read_str" | "write_str" | "close" => Type::I32,
                                "now" => Type::I64,
                                "json_parse" => Type::Dict(Box::new(Type::Str), Box::new(Type::I64)),
                                "json_stringify" => Type::Str,
                                "error" => Type::Option(Box::new(Type::Void)),
                                _ => *ft.return_,
                            }
                        } else {
                            *ft.return_
                        }
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
                for p in params {
                    let _ = self.symbols.insert(p.clone(),
                        Symbol::new_var(p.clone(), Some(Type::I32), false));
                }
                let ret = self.infer_expr(body);
                self.symbols.pop_scope();
                let param_types = params.iter().map(|_| Type::I32).collect();
                Type::Function(FunctionType {
                    is_async: false, is_const: false,
                    params: param_types, return_: Box::new(ret), fallible: false,
                })
            }
            Expr::Await { expression, .. } => {
                let inner = self.infer_expr(expression);
                match inner {
                    Type::Function(ft) => *ft.return_,
                    _ => inner,
                }
            }
            Expr::Async { expression, .. } => {
                let ret = self.infer_expr(expression);
                Type::Function(FunctionType {
                    is_async: true, is_const: false,
                    params: vec![], return_: Box::new(ret), fallible: false,
                })
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
        }
    }

    fn types_match(&self, a: &Type, b: &Type) -> bool {
        if a == b { return true; }
        matches!((a, b),
            (Type::I32, Type::I64) | (Type::I64, Type::I32) |
            (Type::F32, Type::F64) | (Type::F64, Type::F32) |
            (Type::I8, Type::I16) | (Type::I16, Type::I8)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(source: &str) -> Vec<Diagnostic> {
        let mut lexer = klc_frontend::lexer::Lexer::new(source);
        let tokens = lexer.tokenize();
        let mut parser = klc_frontend::parser::Parser::new(tokens);
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
        let diags = check("x := 1\n");
        assert!(diags.is_empty(), "expected no errors, got: {:?}", diags);
    }

    #[test]
    fn test_function_call() {
        let source = "fn add(a: i32, b: i32) -> i32:\n    a + b\n";
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
        let source = "fn f():\n    x := 1\n    x = 2\n";
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
        let source = "fn f():\n    x := 0\n    while true:\n        x = 1\n";
        let diags = check(source);
        assert!(diags.is_empty(), "expected no errors, got: {:?}", diags);
    }

    #[test]
    fn test_while_condition_variable_ref() {
        let source = "fn f():\n    x := 0\n    while x < 5:\n        42\n";
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
        let source = "const fn factorial(n: i32) -> i32:\n    if n <= 1:\n        1\n    else:\n        n * factorial(n - 1)\n";
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