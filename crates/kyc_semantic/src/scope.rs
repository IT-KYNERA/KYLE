use kyc_core::ast::*;
use kyc_core::types::{Type, FunctionType};
use kyc_core::diagnostic::{Diagnostic, ErrorCode, DiagnosticReporter};
use crate::symbol_table::{SymbolTable, Symbol, SymKind};

pub struct ScopeResolver {
    pub symbols: SymbolTable,
    pub reporter: DiagnosticReporter,
}

impl ScopeResolver {
    pub fn new(reporter: DiagnosticReporter) -> Self {
        Self { symbols: SymbolTable::new(), reporter }
    }

    pub fn resolve(&mut self, program: &Program) {
        for decl in &program.declarations {
            self.register_decl(decl);
        }
        for decl in &program.declarations {
            self.resolve_decl(decl);
        }
    }

    fn register_decl(&mut self, decl: &Decl) {
        let (name, kind) = match decl {
            Decl::Function(f) => (f.name.clone(), SymKind::Function(f.clone())),
            Decl::Class(c) => (c.name.clone(), SymKind::Class(c.clone())),
            Decl::AbstractClass(c) => (c.name.clone(), SymKind::Class(ClassDecl {
                name: c.name.clone(), type_params: c.type_params.clone(),
                parent: c.parent.clone(), contracts: c.contracts.clone(),
                members: c.members.clone(), span: c.span,
            })),
            Decl::Struct(s) => (s.name.clone(), SymKind::Struct(s.clone())),
            Decl::Enum(e) => (e.name.clone(), SymKind::Enum(e.clone())),
            Decl::Contract(c) => (c.name.clone(), SymKind::Contract(c.clone())),
            Decl::TypeAlias(t) => (t.name.clone(), SymKind::TypeAlias(t.clone())),
            Decl::Constant(c) => {
                let sym = Symbol::new(c.name.clone(), SymKind::Constant(Type::I32));
                if let Err(e) = self.symbols.insert(c.name.clone(), sym) {
                    self.reporter.report(Diagnostic::error(ErrorCode::E0001, e));
                }
                return;
            }
            Decl::Import(i) => {
                // Use alias if present, otherwise module name
                let scope_name = i.alias.clone().unwrap_or_else(|| i.module_name.clone());
                let sym = Symbol::new(scope_name.clone(), SymKind::Module(vec![]));
                let _ = self.symbols.insert(scope_name, sym);
                return;
            }
            Decl::FromImport(_) => {
                // The actual declaration is inserted into the program by the pipeline
                // and will be registered separately as its proper type (Function, Struct, etc.)
                return;
            }
            Decl::Variable(_) => return,
        };
        let sym = Symbol::new(name.clone(), kind);
        if let Err(e) = self.symbols.insert(name, sym) {
            self.reporter.report(Diagnostic::error(ErrorCode::E0001, e));
        }
    }

    fn resolve_decl(&mut self, decl: &Decl) {
        match decl {
            Decl::Function(f) => self.resolve_function(f),
            Decl::Variable(v) => self.resolve_variable(v),
            Decl::Constant(_) => {}
            Decl::Class(c) => self.resolve_class_body(c),
            Decl::AbstractClass(c) => {
                let cd = ClassDecl {
                    name: c.name.clone(), type_params: c.type_params.clone(),
                    parent: c.parent.clone(), contracts: c.contracts.clone(),
                    members: c.members.clone(), span: c.span,
                };
                self.resolve_class_body(&cd);
            }
            Decl::Struct(_) | Decl::Enum(_) | Decl::Contract(_) | Decl::TypeAlias(_) => {}
            Decl::Import(_) | Decl::FromImport(_) => {}
        }
    }

    fn resolve_function(&mut self, f: &FunctionDecl) {
        self.symbols.push_scope();
        for param in &f.params {
            let ty = self.resolve_ast_type(&param.type_);
            let is_mutable = matches!(param.mode, ParamMode::MutableBorrow | ParamMode::Move);
            let sym = Symbol::new_var(param.name.clone(), Some(ty), is_mutable);
            let _ = self.symbols.insert(param.name.clone(), sym);
        }
        if let Some(body) = &f.body {
            self.resolve_block(body);
        }
        self.symbols.pop_scope();
    }

    fn resolve_variable(&mut self, v: &VariableDecl) {
        let ty = v.type_.as_ref().map(|t| self.resolve_ast_type(t));
        let sym = Symbol::new_var(v.name.clone(), ty, v.is_mutable);
        if let Err(e) = self.symbols.insert(v.name.clone(), sym) {
            self.reporter.report(Diagnostic::error(ErrorCode::E0009, e));
        }
    }

    fn resolve_class_body(&mut self, c: &ClassDecl) {
        self.symbols.push_scope();
        let _ = self.symbols.insert("this".to_string(),
            Symbol::new_var("this".to_string(), Some(Type::Named(c.name.clone())), false));
        for member in &c.members {
            match member {
                ClassMember::Field(f) => {
                    let ty = self.resolve_ast_type(&f.type_);
                    let _ = self.symbols.insert(f.name.clone(),
                        Symbol::new_var(f.name.clone(), Some(ty), true));
                }
                ClassMember::Method(m) => self.resolve_function(m),
                ClassMember::Constructor(con) => {
                    self.symbols.push_scope();
                    for param in &con.params {
                        let ty = self.resolve_ast_type(&param.type_);
                        let is_mutable = matches!(param.mode, ParamMode::MutableBorrow | ParamMode::Move);
                        let _ = self.symbols.insert(param.name.clone(),
                            Symbol::new_var(param.name.clone(), Some(ty), is_mutable));
                    }
                    self.resolve_block(&con.body);
                    self.symbols.pop_scope();
                }
                ClassMember::Property(_) => {}
            }
        }
        self.symbols.pop_scope();
    }

    fn resolve_block(&mut self, block: &Block) {
        for stmt in &block.statements {
            self.resolve_stmt(stmt);
        }
    }

    /// Bind identifiers in a pattern into the current scope.
    fn bind_pattern(&mut self, pattern: &Pattern) {
        match pattern {
            Pattern::Identifier { name, .. } => {
                let _ = self.symbols.insert(name.clone(),
                    Symbol::new_var(name.clone(), None, false));
            }
            Pattern::EnumVariant { args, .. } => {
                for arg in args {
                    self.bind_pattern(arg);
                }
            }
            Pattern::Or { patterns, .. } => {
                for p in patterns {
                    self.bind_pattern(p);
                }
            }
            _ => {}
        }
    }

    fn resolve_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Variable(v) => self.resolve_variable(v),
            Stmt::TypedVariable(v) => self.resolve_variable(v),
            Stmt::Constant(_) => {}
            Stmt::Expression(e) => { self.resolve_expr(e); }
            Stmt::Return(e) => { if let Some(e) = e { self.resolve_expr(e); } }
            Stmt::Break(e, _) => { if let Some(e) = e { self.resolve_expr(e); } }
            Stmt::If(s) => {
                self.resolve_expr(&s.condition);
                self.resolve_block(&s.body);
                for el in &s.elif_branches {
                    self.resolve_expr(&el.condition);
                    self.resolve_block(&el.body);
                }
                if let Some(el) = &s.else_branch { self.resolve_block(el); }
            }
            Stmt::BindingIf(b) => {
                self.resolve_expr(&b.value);
                self.symbols.push_scope();
                let _ = self.symbols.insert(b.name.clone(),
                    Symbol::new_var(b.name.clone(), None, false));
                self.resolve_block(&b.body);
                self.symbols.pop_scope();
                if let Some(el) = &b.else_branch { self.resolve_block(el); }
            }
            Stmt::While(w) => {
                self.resolve_expr(&w.condition);
                self.resolve_block(&w.body);
                if let Some(el) = &w.else_branch { self.resolve_block(el); }
            }
            Stmt::WhileBind(w) => {
                self.resolve_expr(&w.iterable);
                self.symbols.push_scope();
                let _ = self.symbols.insert(w.name.clone(),
                    Symbol::new_var(w.name.clone(), None, false));
                self.resolve_block(&w.body);
                self.symbols.pop_scope();
            }
            Stmt::For(f) => {
                self.resolve_expr(&f.iterable);
                self.symbols.push_scope();
                let _ = self.symbols.insert(f.variable.clone(),
                    Symbol::new_var(f.variable.clone(), None, false));
                self.resolve_block(&f.body);
                self.symbols.pop_scope();
                if let Some(el) = &f.else_branch { self.resolve_block(el); }
            }
            Stmt::Match(m) => {
                self.resolve_expr(&m.expression);
                for arm in &m.arms {
                    self.symbols.push_scope();
                    self.bind_pattern(&arm.pattern);
                    if let Some(g) = &arm.guard { self.resolve_expr(g); }
                    self.resolve_block(&arm.body);
                    self.symbols.pop_scope();
                }
            }
            Stmt::Defer(d) => { self.resolve_expr(&d.call); }
            Stmt::Guard(g) => {
                self.resolve_expr(&g.condition);
                self.resolve_block(&g.body);
            }
            Stmt::Unsafe(u) => { self.resolve_block(&u.body); }
            Stmt::Continue(_) => {}
        }
    }

    fn resolve_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Literal { .. } => {}
            Expr::Identifier { name, span } => {
                if self.symbols.lookup(name).is_none() {
                    self.reporter.report(
                        Diagnostic::error(ErrorCode::E0009, format!("undefined symbol '{}'", name))
                            .with_span(*span)
                    );
                }
            }
            Expr::Binary { left, right, .. } => {
                self.resolve_expr(left);
                self.resolve_expr(right);
            }
            Expr::Unary { operand, .. } => self.resolve_expr(operand),
            Expr::Assignment { target, value, .. } => {
                if let Expr::Identifier { name, span } = target.as_ref() {
                    if let Some(sym) = self.symbols.lookup(name) {
                        match &sym.kind {
                            SymKind::Variable { is_mutable, is_auto, .. } if !*is_mutable && !*is_auto => {
                                self.reporter.report(
                                    Diagnostic::error(ErrorCode::E0007, format!("cannot modify immutable variable '{}'", name))
                                        .with_span(*span)
                                );
                            }
                            SymKind::Constant(_) => {
                                self.reporter.report(
                                    Diagnostic::error(ErrorCode::E0007, format!("cannot modify constant '{}'", name))
                                        .with_span(*span)
                                );
                            }
                            _ => {}
                        }
                    } else {
                        let _ = self.symbols.insert(name.clone(), Symbol::new_auto(name.clone(), None));
                    }
                    self.resolve_expr(value);
                } else if let Expr::Tuple { elements, .. } = target.as_ref() {
                    for elem in elements {
                        if let Expr::Identifier { name, .. } = elem {
                            if self.symbols.lookup(name).is_none() {
                                let _ = self.symbols.insert(name.clone(), Symbol::new_auto(name.clone(), None));
                            }
                        }
                    }
                    self.resolve_expr(value);
                } else {
                    self.resolve_expr(target);
                    self.resolve_expr(value);
                }
            }
            Expr::FunctionCall { target, arguments, .. } => {
                self.resolve_expr(target);
                for arg in arguments { self.resolve_expr(arg); }
            }
            Expr::PropertyAccess { object, .. } => self.resolve_expr(object),
            Expr::List { elements, .. } => {
                for e in elements { self.resolve_expr(e); }
            }
            Expr::Dictionary { entries, .. } => {
                for (_, v) in entries { self.resolve_expr(v); }
            }
            Expr::StructLiteral { fields, .. } => {
                for (_, v) in fields { self.resolve_expr(v); }
            }
            Expr::Tuple { elements, .. } => {
                for e in elements { self.resolve_expr(e); }
            }
            Expr::Closure { params, body, .. } => {
                self.symbols.push_scope();
                for p in params {
                    let _ = self.symbols.insert(p.clone(),
                        Symbol::new_var(p.clone(), None, false));
                }
                self.resolve_expr(body);
                self.symbols.pop_scope();
            }
            Expr::Await { expression, .. } => self.resolve_expr(expression),
            Expr::Async { expression, .. } => self.resolve_expr(expression),
            Expr::Spread { expression, .. } => self.resolve_expr(expression),
            Expr::Index { target, index, .. } => {
                self.resolve_expr(target);
                self.resolve_expr(index);
            }
            Expr::RangeSlice { target, start, end, .. } => {
                self.resolve_expr(target);
                if let Some(s) = start { self.resolve_expr(s); }
                if let Some(e) = end { self.resolve_expr(e); }
            }
            Expr::OptionalChain { target, .. } => self.resolve_expr(target),
            Expr::Loop { body, .. } => self.resolve_block(body),
            Expr::ErrorProp { expression, .. } => self.resolve_expr(expression),
            Expr::StringInterp { parts, .. } => {
                for part in parts {
                    self.resolve_expr(part);
                }
            }
            Expr::Ternary { cond, then_expr, else_expr, .. } => {
                self.resolve_expr(cond);
                self.resolve_expr(then_expr);
                self.resolve_expr(else_expr);
            }
            Expr::MutableRef { expression, .. } => self.resolve_expr(expression),
            Expr::NullCoalesce { left, right, .. } => {
                self.resolve_expr(left);
                self.resolve_expr(right);
            }
            Expr::MoveExpr { expression, .. } => self.resolve_expr(expression),
            Expr::MatchExpr { expression, arms, .. } => {
                self.resolve_expr(expression);
                for arm in arms {
                    // Walk pattern variables and guard
                    if let Some(g) = &arm.guard {
                        self.resolve_expr(g);
                    }
                    self.resolve_block(&arm.body);
                }
            }
        }
    }

    pub fn resolve_ast_type(&self, ast: &AstType) -> Type {
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
                    "Option" => {
                        if let Some(inner) = args.first() {
                            Type::Option(Box::new(self.resolve_ast_type(inner)))
                        } else {
                            Type::Option(Box::new(Type::Void))
                        }
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
            AstType::Mutable { inner, .. } | AstType::Move { inner, .. } => {
                self.resolve_ast_type(inner)
            }
            AstType::Ptr { .. } => Type::Ptr,
        }
    }
}