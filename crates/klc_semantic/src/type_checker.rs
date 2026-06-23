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
        }
    }

    pub fn with_source(mut self, source_map: SourceMap, name: String) -> Self {
        self.reporter = self.reporter.with_source(source_map, name);
        self
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
            self.check_block(body);
        }
        self.symbols.pop_scope();
    }

    fn check_variable(&mut self, v: &VariableDecl) {
        let is_uninit = matches!(v.value.as_ref(), Expr::Literal { value: Literal::None, .. });
        let inferred = if is_uninit {
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
            if !is_uninit && !self.types_match(&inferred, &declared_ty) {
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
                // For enum variant payloads, use I32 as default (payload is i64 in MIR).
                // For standalone identifiers, leave type as None for inference.
                let inferred = match_type.map(|_| Type::I32);
                let _ = self.symbols.insert(name.clone(),
                    Symbol::new_var(name.clone(), inferred, false));
            }
            Pattern::EnumVariant { args, .. } => {
                for arg in args {
                    self.bind_pattern(arg, Some(&Type::I32));
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
                self.infer_expr(&m.expression);
                for arm in &m.arms {
                    self.symbols.push_scope();
                    self.bind_pattern(&arm.pattern, None);
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
                }
                ty
            }
            Expr::FunctionCall { target, arguments, .. } => {
                let fn_type = self.infer_expr(target);
                for arg in arguments { self.infer_expr(arg); }
                match fn_type {
                    Type::Function(ft) => *ft.return_,
                    _ => {
                        if let Expr::Identifier { name, .. } = target.as_ref() {
                            match name.as_str() {
                                "print" | "println" | "print_err" => Type::Void,
                                "len" => Type::I32,
                                "str" => Type::Str,
                                "input" => Type::Str,
                                "range" => Type::List(Box::new(Type::I32)),
                                "open" | "read_str" | "write_str" | "close" => Type::I32,
                                "sleep" => Type::Void,
                                "now" => Type::I64,
                                "assert" | "assert_eq" | "assert_str" => Type::Void,
                                _ => Type::I32,
                            }
                        } else {
                            Type::I32
                        }
                    }
                }
            }
            Expr::PropertyAccess { object, .. } => {
                self.infer_expr(object);
                Type::I32
            }
            Expr::List { elements, .. } => {
                if elements.is_empty() { return Type::List(Box::new(Type::I32)); }
                Type::List(Box::new(self.infer_expr(&elements[0])))
            }
            Expr::Dictionary { entries, .. } => {
                if entries.is_empty() { return Type::Dict(Box::new(Type::Str), Box::new(Type::I32)); }
                Type::Dict(Box::new(Type::Str), Box::new(Type::I32))
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
                    _ => Type::I32,
                }
            }
            Expr::RangeSlice { target, start, end, .. } => {
                self.infer_expr(target);
                if let Some(s) = start { self.infer_expr(s); }
                if let Some(e) = end { self.infer_expr(e); }
                Type::List(Box::new(Type::I32))
            }
            Expr::OptionalChain { target, .. } => {
                self.infer_expr(target);
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
                Type::Generic(name.clone(), args.iter().map(|a| self.resolve_ast_type(a)).collect())
            }
            AstType::Optional { inner, .. } => Type::Option(Box::new(self.resolve_ast_type(inner))),
            AstType::Error { inner, .. } => Type::Error(Box::new(self.resolve_ast_type(inner))),
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
        let diags = check("mut x = 1\n");
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
        let source = "fn f():\n    mut x = 1\n    x = 2\n";
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
        let source = "fn f():\n    mut x = 0\n    while true:\n        x = 1\n";
        let diags = check(source);
        assert!(diags.is_empty(), "expected no errors, got: {:?}", diags);
    }

    #[test]
    fn test_while_condition_variable_ref() {
        let source = "fn f():\n    mut x = 0\n    while x < 5:\n        42\n";
        let diags = check(source);
        assert!(diags.is_empty(), "expected no errors, got: {:?}", diags);
    }

    #[test]
    fn test_while_body_var_read() {
        let source = "fn f():\n    mut x = 0\n    while true:\n        x\n";
        let diags = check(source);
        assert!(diags.is_empty(), "expected no errors, got: {:?}", diags);
    }

    #[test]
    fn test_while_body_expr_read() {
        let source = "fn f():\n    mut x = 0\n    while true:\n        x + 1\n";
        let diags = check(source);
        assert!(diags.is_empty(), "expected no errors, got: {:?}", diags);
    }


}