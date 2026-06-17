// klc_frontend::parser — Recursive descent parser for KL
//
// Transforms a token stream into an AST.
// Reference: docs/02-formal-grammar.md, docs/03-ast-specification.md

use klc_core::ast::*;
use klc_core::span::Span;
use crate::token::{Token, TokenKind};

/// The KL recursive-descent parser.
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    /// Parse the full token stream into a Program AST node.
    pub fn parse(&mut self) -> Result<Program, String> {
        let mut declarations = Vec::new();
        loop {
            if self.at(TokenKind::Eof) {
                break;
            }
            if self.at(TokenKind::Newline) {
                self.advance();
                continue;
            }
            declarations.push(self.parse_decl()?);
        }
        Ok(Program {
            declarations,
            span: Span::dummy(),
        })
    }

    // -----------------------------------------------------------------------
    // Declaration parsing
    // -----------------------------------------------------------------------

    fn parse_decl(&mut self) -> Result<Decl, String> {
        if self.at(TokenKind::From) {
            return self.parse_from_import();
        }
        if self.at(TokenKind::Import) {
            return self.parse_import();
        }
        if self.at(TokenKind::Type) {
            return self.parse_type_alias();
        }
        if self.at(TokenKind::Fn)
            || self.at(TokenKind::Async)
            || self.at(TokenKind::Const)
        {
            return self.parse_function().map(Decl::Function);
        }
        // Class: `class X:` or `abs class X:`
        if self.at(TokenKind::Abs) {
            self.advance(); // consume 'abs'
            if self.at(TokenKind::Class) {
                return self.parse_class(true);
            }
            return Err("expected 'class' after 'abs'".to_string());
        }
        if self.at(TokenKind::Class) {
            return self.parse_class(false);
        }
        if self.at(TokenKind::Struct) {
            return self.parse_struct().map(Decl::Struct);
        }
        if self.at(TokenKind::Enum) {
            return self.parse_enum().map(Decl::Enum);
        }
        if self.at(TokenKind::Contract) {
            return self.parse_contract().map(Decl::Contract);
        }
        // Variable/constant declaration: `name = expr`
        if self.at_identifier() {
            let name = self.eat_identifier();
            if self.at(TokenKind::Equals) {
                self.advance();
                let value = self.parse_expr()?;
                if name.chars().all(|c| c.is_uppercase() || c == '_' || c.is_ascii_digit()) {
                    return Ok(Decl::Constant(ConstantDecl {
                        name,
                        value: Box::new(value),
                        span: Span::dummy(),
                    }));
                }
                return Ok(Decl::Variable(VariableDecl {
                    name,
                    type_: None,
                    value: Box::new(value),
                    span: Span::dummy(),
                }));
            }
            return Err(format!("expected '=' after identifier '{}'", name));
        }
        let found = self.current().map(|t| format!("{:?}", t.kind)).unwrap_or_else(|_| "EOF".into());
        Err(format!("unexpected token at declaration start: {}", found))
    }

    fn parse_import(&mut self) -> Result<Decl, String> {
        self.advance(); // import
        let module_name = self.eat_identifier();
        let alias = if self.at(TokenKind::As) {
            self.advance();
            Some(self.eat_identifier())
        } else {
            None
        };
        Ok(Decl::Import(Import { module_name, alias, span: Span::dummy() }))
    }

    fn parse_from_import(&mut self) -> Result<Decl, String> {
        self.advance(); // from
        let module_name = self.eat_identifier();
        self.expect_keyword("import")?;
        let imported_name = self.eat_identifier();
        Ok(Decl::FromImport(FromImport { module_name, imported_name, span: Span::dummy() }))
    }

    fn parse_type_alias(&mut self) -> Result<Decl, String> {
        self.advance(); // type
        let name = self.eat_identifier();
        if !self.at(TokenKind::Equals) {
            return Err("expected '=' in type alias".to_string());
        }
        self.advance();
        let type_ = self.parse_type()?;
        Ok(Decl::TypeAlias(TypeAlias { name, type_, span: Span::dummy() }))
    }

    fn parse_function(&mut self) -> Result<FunctionDecl, String> {
        let is_const = if self.at(TokenKind::Const) { self.advance(); true } else { false };
        let is_async = if self.at(TokenKind::Async) { self.advance(); true } else { false };
        if !self.at(TokenKind::Fn) {
            return Err("expected 'fn'".to_string());
        }
        self.advance();
        let name = self.eat_identifier();
        self.expect(TokenKind::LParen)?;
        let params = self.parse_params()?;
        self.expect(TokenKind::RParen)?;
        let return_type = if self.at(TokenKind::Arrow) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };
        let is_abstract = !self.at(TokenKind::Colon);
        let body = if is_abstract {
            None
        } else {
            self.advance(); // ':'
            Some(self.parse_block()?)
        };
        Ok(FunctionDecl {
            name, params, return_type, is_async, is_const, is_abstract, body,
            span: Span::dummy(),
        })
    }

    fn parse_class(&mut self, is_abstract: bool) -> Result<Decl, String> {
        self.advance(); // class
        let name = self.eat_identifier();
        // Check for inheritance: `class Dog : Animal:`
        let parent = if self.at(TokenKind::Colon) {
            let saved = self.pos;
            self.advance();
            if self.at_identifier() {
                let parent_name = self.eat_identifier();
                if self.at(TokenKind::Colon) {
                    self.advance();
                    let members = self.parse_class_members()?;
                    return self.make_class_decl(name, Some(parent_name), members, is_abstract);
                }
            }
            self.pos = saved;
            None
        } else {
            None
        };
        self.expect(TokenKind::Colon)?;
        let members = self.parse_class_members()?;
        self.make_class_decl(name, parent, members, is_abstract)
    }

    fn make_class_decl(
        &self,
        name: String,
        parent: Option<String>,
        members: Vec<ClassMember>,
        is_abstract: bool,
    ) -> Result<Decl, String> {
        if is_abstract {
            Ok(Decl::AbstractClass(AbstractClassDecl {
                name, parent, contracts: vec![], members, span: Span::dummy(),
            }))
        } else {
            Ok(Decl::Class(ClassDecl {
                name, parent, contracts: vec![], members, span: Span::dummy(),
            }))
        }
    }

    fn parse_class_members(&mut self) -> Result<Vec<ClassMember>, String> {
        // Consume the Newline and Indent that precede the class body.
        if self.at(TokenKind::Newline) { self.advance(); }
        if self.at(TokenKind::Indent) { self.advance(); }
        let mut members = Vec::new();
        loop {
            if self.at(TokenKind::Dedent) { self.advance(); break; }
            if self.at(TokenKind::Eof) { break; }
            if self.at(TokenKind::Newline) { self.advance(); continue; }
            // Constructor: `Name(params):`
            if self.at_identifier() {
                let name = self.eat_identifier();
                // Check for constructor: identifier followed by '('
                if self.at(TokenKind::LParen) {
                    self.advance();
                    let params = self.parse_params()?;
                    self.expect(TokenKind::RParen)?;
                    self.expect(TokenKind::Colon)?;
                    let body = self.parse_block()?;
                    members.push(ClassMember::Constructor(Constructor {
                        params, body, span: Span::dummy(),
                    }));
                    continue;
                }
                // Check for field or property: identifier followed by ':'
                if self.at(TokenKind::Colon) {
                    self.advance();
                    if self.at(TokenKind::Get) || self.at(TokenKind::Set) {
                        // Property — stub, will expand in Phase 2
                        members.push(ClassMember::Property(Property {
                            name, type_: AstType::Primitive { name: "str".into(), span: Span::dummy() },
                            getter: None, setter: None, span: Span::dummy(),
                        }));
                    } else {
                        // Field: `name: Type`
                        let type_ = self.parse_type()?;
                        let visibility = if name.starts_with("__") {
                            Visibility::Private
                        } else if name.starts_with('_') {
                            Visibility::Protected
                        } else {
                            Visibility::Public
                        };
                        members.push(ClassMember::Field(Field {
                            name, type_, visibility, span: Span::dummy(),
                        }));
                    }
                    continue;
                }
                // Unknown — break and let the outer parser handle it
                break;
            }
            // Method: `fn name(params):`
            if self.at(TokenKind::Fn) || self.at(TokenKind::Async) || self.at(TokenKind::Const) || self.at(TokenKind::Abs) {
                let method = self.parse_function()?;
                members.push(ClassMember::Method(method));
                continue;
            }
            break;
        }
        Ok(members)
    }

    fn parse_struct(&mut self) -> Result<StructDecl, String> {
        self.advance(); // struct
        let name = self.eat_identifier();
        self.expect(TokenKind::Colon)?;
        if self.at(TokenKind::Newline) { self.advance(); }
        if self.at(TokenKind::Indent) { self.advance(); }
        let mut fields = Vec::new();
        loop {
            if self.at(TokenKind::Dedent) { self.advance(); break; }
            if self.at(TokenKind::Eof) { break; }
            if self.at(TokenKind::Newline) { self.advance(); continue; }
            let field_name = self.eat_identifier();
            self.expect(TokenKind::Colon)?;
            let type_ = self.parse_type()?;
            fields.push(Field { name: field_name, type_, visibility: Visibility::Public, span: Span::dummy() });
        }
        Ok(StructDecl { name, fields, span: Span::dummy() })
    }

    fn parse_enum(&mut self) -> Result<EnumDecl, String> {
        self.advance(); // enum
        let name = self.eat_identifier();
        self.expect(TokenKind::Colon)?;
        if self.at(TokenKind::Newline) { self.advance(); }
        if self.at(TokenKind::Indent) { self.advance(); }
        let mut variants = Vec::new();
        loop {
            if self.at(TokenKind::Dedent) { self.advance(); break; }
            if self.at(TokenKind::Eof) { break; }
            if self.at(TokenKind::Newline) { self.advance(); continue; }
            let variant_name = self.eat_identifier();
            variants.push(EnumVariant { name: variant_name, payload: vec![], span: Span::dummy() });
        }
        Ok(EnumDecl { name, variants, span: Span::dummy() })
    }

    fn parse_contract(&mut self) -> Result<ContractDecl, String> {
        self.advance(); // contract
        let name = self.eat_identifier();
        self.expect(TokenKind::Colon)?;
        if self.at(TokenKind::Newline) { self.advance(); }
        if self.at(TokenKind::Indent) { self.advance(); }
        let mut methods = Vec::new();
        loop {
            if self.at(TokenKind::Dedent) { self.advance(); break; }
            if self.at(TokenKind::Eof) { break; }
            if self.at(TokenKind::Newline) { self.advance(); continue; }
            if !self.at(TokenKind::Fn) { break; }
            self.advance();
            let method_name = self.eat_identifier();
            self.expect(TokenKind::LParen)?;
            let params = self.parse_params()?;
            self.expect(TokenKind::RParen)?;
            let return_type = if self.at(TokenKind::Arrow) {
                self.advance();
                Some(self.parse_type()?)
            } else { None };
            methods.push(ContractMethod { name: method_name, params, return_type, span: Span::dummy() });
        }
        Ok(ContractDecl { name, methods, span: Span::dummy() })
    }

    fn parse_params(&mut self) -> Result<Vec<Parameter>, String> {
        let mut params = Vec::new();
        loop {
            if self.at(TokenKind::RParen) { break; }
            let name = self.eat_identifier();
            self.expect(TokenKind::Colon)?;
            let type_ = self.parse_type()?;
            let default = if self.at(TokenKind::Equals) {
                self.advance();
                Some(Box::new(self.parse_expr()?))
            } else { None };
            params.push(Parameter { name, type_, default, variadic: false, span: Span::dummy() });
            if self.at(TokenKind::Comma) { self.advance(); } else { break; }
        }
        Ok(params)
    }

    // -----------------------------------------------------------------------
    // Type parsing
    // -----------------------------------------------------------------------

    fn parse_type(&mut self) -> Result<AstType, String> {
        let name = self.eat_identifier();
        if name.is_empty() {
            let found = self.current().map(|t| format!("{:?}", t.kind)).unwrap_or_else(|_| "EOF".into());
            return Err(format!("expected type name, found {}", found));
        }
        if self.at(TokenKind::Less) {
            self.advance();
            let mut args = Vec::new();
            args.push(self.parse_type()?);
            while self.at(TokenKind::Comma) {
                self.advance();
                args.push(self.parse_type()?);
            }
            self.expect(TokenKind::Greater)?;
            Ok(AstType::Generic { name, args, span: Span::dummy() })
        } else {
            Ok(AstType::User { name, span: Span::dummy() })
        }
    }

    // -----------------------------------------------------------------------
    // Expression parsing
    // -----------------------------------------------------------------------

    fn parse_expr(&mut self) -> Result<Expr, String> {
        self.parse_binary(0)
    }

    fn parse_binary(&mut self, min_prec: u8) -> Result<Expr, String> {
        let mut left = self.parse_unary()?;
        loop {
            let op = match self.current_operator() {
                Some(op) => op,
                None => break,
            };
            let prec = self.operator_precedence(&op);
            if prec < min_prec { break; }
            self.advance();
            let right = self.parse_binary(prec + 1)?;
            left = Expr::Binary {
                left: Box::new(left),
                operator: op,
                right: Box::new(right),
                span: Span::dummy(),
            };
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, String> {
        if self.at(TokenKind::Minus) {
            self.advance();
            return Ok(Expr::Unary { operator: UnaryOp::Neg, operand: Box::new(self.parse_primary()?), span: Span::dummy() });
        }
        if self.at(TokenKind::Bang) {
            self.advance();
            return Ok(Expr::Unary { operator: UnaryOp::Not, operand: Box::new(self.parse_primary()?), span: Span::dummy() });
        }
        if self.at(TokenKind::Tilde) {
            self.advance();
            return Ok(Expr::Unary { operator: UnaryOp::BitNot, operand: Box::new(self.parse_primary()?), span: Span::dummy() });
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        let tok = self.current()?;
        let expr = match &tok.kind {
            TokenKind::Integer(s) => {
                let val = s.parse::<i64>().unwrap_or(0);
                self.advance();
                Expr::Literal { value: Literal::Integer(val), span: Span::dummy() }
            }
            TokenKind::Float(s) => {
                let val = s.parse::<f64>().unwrap_or(0.0);
                self.advance();
                Expr::Literal { value: Literal::Float(val), span: Span::dummy() }
            }
            TokenKind::String(s) => {
                let val = s.clone();
                self.advance();
                Expr::Literal { value: Literal::String(val), span: Span::dummy() }
            }
            TokenKind::True => {
                self.advance();
                Expr::Literal { value: Literal::Boolean(true), span: Span::dummy() }
            }
            TokenKind::False => {
                self.advance();
                Expr::Literal { value: Literal::Boolean(false), span: Span::dummy() }
            }
            TokenKind::Identifier(name) => {
                let val = name.clone();
                self.advance();
                Expr::Identifier { name: val, span: Span::dummy() }
            }
            TokenKind::LParen => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect(TokenKind::RParen)?;
                expr
            }
            TokenKind::LBracket => {
                self.advance();
                let mut elements = Vec::new();
                while !self.at(TokenKind::RBracket) && !self.at(TokenKind::Eof) {
                    elements.push(self.parse_expr()?);
                    if self.at(TokenKind::Comma) { self.advance(); }
                }
                self.expect(TokenKind::RBracket)?;
                Expr::List { elements, span: Span::dummy() }
            }
            TokenKind::LBrace => {
                self.advance();
                let mut entries = Vec::new();
                while !self.at(TokenKind::RBrace) && !self.at(TokenKind::Eof) {
                    let key = self.eat_identifier();
                    self.expect(TokenKind::Colon)?;
                    let value = self.parse_expr()?;
                    entries.push((key, value));
                    if self.at(TokenKind::Comma) { self.advance(); }
                }
                self.expect(TokenKind::RBrace)?;
                Expr::Dictionary { entries, span: Span::dummy() }
            }
            _ => return Err(format!("unexpected token in expression: {:?}", tok.kind)),
        };
        self.parse_postfix(expr)
    }

    fn parse_postfix(&mut self, mut expr: Expr) -> Result<Expr, String> {
        loop {
            if self.at(TokenKind::LParen) {
                self.advance();
                let mut arguments = Vec::new();
                while !self.at(TokenKind::RParen) && !self.at(TokenKind::Eof) {
                    arguments.push(self.parse_expr()?);
                    if self.at(TokenKind::Comma) { self.advance(); }
                }
                self.expect(TokenKind::RParen)?;
                expr = Expr::FunctionCall { target: Box::new(expr), arguments, span: Span::dummy() };
            } else if self.at(TokenKind::Dot) {
                self.advance();
                let property = self.eat_identifier();
                expr = Expr::PropertyAccess { object: Box::new(expr), property, span: Span::dummy() };
            } else if self.at(TokenKind::QuestionDot) {
                self.advance();
                let property = self.eat_identifier();
                expr = Expr::OptionalChain { target: Box::new(expr), property, span: Span::dummy() };
            } else {
                break;
            }
        }
        Ok(expr)
    }

    // -----------------------------------------------------------------------
    // Block and statement parsing
    // -----------------------------------------------------------------------

    fn parse_block(&mut self) -> Result<Block, String> {
        // Consume the Newline and Indent that precede every indented block.
        if self.at(TokenKind::Newline) { self.advance(); }
        if self.at(TokenKind::Indent) { self.advance(); }
        let mut statements = Vec::new();
        loop {
            if self.at(TokenKind::Dedent) { self.advance(); break; }
            if self.at(TokenKind::Eof) { break; }
            // Newlines between statements are ignored.
            if self.at(TokenKind::Newline) { self.advance(); continue; }
            statements.push(self.parse_stmt()?);
        }
        Ok(Block { statements, span: Span::dummy() })
    }

    fn parse_stmt(&mut self) -> Result<Stmt, String> {
        if self.at(TokenKind::If) { return self.parse_if(); }
        if self.at(TokenKind::While) { return self.parse_while(); }
        if self.at(TokenKind::For) { return self.parse_for(); }
        if self.at(TokenKind::Match) { return self.parse_match(); }
        if self.at(TokenKind::Loop) {
            self.advance();
            self.expect(TokenKind::Colon)?;
            let body = self.parse_block()?;
            return Ok(Stmt::Expression(Expr::Loop { body, span: Span::dummy() }));
        }
        if self.at(TokenKind::Return) {
            self.advance();
            let expr = if self.current_is_expr_start() {
                Some(Box::new(self.parse_expr()?))
            } else { None };
            return Ok(Stmt::Return(expr));
        }
        if self.at(TokenKind::Break) {
            self.advance();
            let value = if self.current_is_expr_start() {
                Some(Box::new(self.parse_expr()?))
            } else { None };
            return Ok(Stmt::Break(value));
        }
        if self.at(TokenKind::Defer) {
            self.advance();
            let call = self.parse_expr()?;
            return Ok(Stmt::Defer(DeferStmt { call: Box::new(call), span: Span::dummy() }));
        }
        if self.at(TokenKind::Guard) {
            self.advance();
            let condition = self.parse_expr()?;
            self.expect_keyword("else")?;
            self.expect(TokenKind::Colon)?;
            let body = self.parse_block()?;
            return Ok(Stmt::Guard(GuardStmt { condition: Box::new(condition), body, span: Span::dummy() }));
        }
        if self.at(TokenKind::Unsafe) {
            self.advance();
            self.expect(TokenKind::Colon)?;
            let body = self.parse_block()?;
            return Ok(Stmt::Unsafe(UnsafeBlock { body, span: Span::dummy() }));
        }
        // Variable declaration or binding-if: `ident = expr`
        // Disambiguate by checking if the next token after the expr is ':'
        // (binding-if) or something else (expression statement).
        if self.at_identifier() && self.peek_equals() {
            let name = self.eat_identifier();
            self.advance(); // '='
            let value = self.parse_expr()?;
            if self.at(TokenKind::Colon) {
                self.advance();
                let body = self.parse_block()?;
                let else_branch = if self.at(TokenKind::Else) {
                    self.advance();
                    Some(self.parse_block()?)
                } else { None };
                Ok(Stmt::BindingIf(BindingIf {
                    name, value: Box::new(value), body, else_branch, span: Span::dummy(),
                }))
            } else {
                Ok(Stmt::Expression(Expr::Assignment {
                    target: Box::new(Expr::Identifier { name, span: Span::dummy() }),
                    operator: None,
                    value: Box::new(value),
                    span: Span::dummy(),
                }))
            }
        } else {
            self.parse_expr_stmt()
        }
    }

    fn parse_if(&mut self) -> Result<Stmt, String> {
        self.advance();
        let condition = self.parse_expr()?;
        self.expect(TokenKind::Colon)?;
        let body = self.parse_block()?;
        let mut elif_branches = Vec::new();
        let mut else_branch = None;
        while self.at(TokenKind::Elif) {
            self.advance();
            let cond = self.parse_expr()?;
            self.expect(TokenKind::Colon)?;
            let body = self.parse_block()?;
            elif_branches.push(ElifBranch {
                condition: Box::new(cond),
                body,
                span: Span::dummy(),
            });
        }
        if self.at(TokenKind::Else) {
            self.advance();
            self.expect(TokenKind::Colon)?;
            else_branch = Some(self.parse_block()?);
        }
        Ok(Stmt::If(IfStmt { condition: Box::new(condition), body, elif_branches, else_branch, span: Span::dummy() }))
    }

    fn parse_binding_if(&mut self) -> Result<Stmt, String> {
        let name = self.eat_identifier();
        self.expect(TokenKind::Equals)?;
        let value = self.parse_expr()?;
        self.expect(TokenKind::Colon)?;
        let body = self.parse_block()?;
        let else_branch = if self.at(TokenKind::Else) {
            self.advance();
            Some(self.parse_block()?)
        } else { None };
        Ok(Stmt::BindingIf(BindingIf {
            name, value: Box::new(value), body, else_branch, span: Span::dummy(),
        }))
    }

    fn parse_while(&mut self) -> Result<Stmt, String> {
        self.advance();
        let condition = self.parse_expr()?;
        self.expect(TokenKind::Colon)?;
        let body = self.parse_block()?;
        let else_branch = if self.at(TokenKind::Else) {
            self.advance();
            Some(self.parse_block()?)
        } else { None };
        Ok(Stmt::While(WhileStmt { condition: Box::new(condition), body, else_branch, span: Span::dummy() }))
    }

    fn parse_for(&mut self) -> Result<Stmt, String> {
        self.advance();
        let variable = self.eat_identifier();
        self.expect_keyword("in")?;
        let iterable = self.parse_expr()?;
        self.expect(TokenKind::Colon)?;
        let body = self.parse_block()?;
        let else_branch = if self.at(TokenKind::Else) {
            self.advance();
            Some(self.parse_block()?)
        } else { None };
        Ok(Stmt::For(ForStmt { variable, iterable: Box::new(iterable), body, else_branch, span: Span::dummy() }))
    }

    fn parse_match(&mut self) -> Result<Stmt, String> {
        self.advance();
        let expression = self.parse_expr()?;
        self.expect(TokenKind::Colon)?;
        // Consume Newline and Indent that precede the match body
        if self.at(TokenKind::Newline) { self.advance(); }
        if self.at(TokenKind::Indent) { self.advance(); }
        let mut arms = Vec::new();
        loop {
            if self.at(TokenKind::Newline) { self.advance(); continue; }
            if self.at(TokenKind::Dedent) { self.advance(); break; }
            if self.at(TokenKind::Eof) { break; }
            let pattern = self.parse_pattern()?;
            let guard = if self.at(TokenKind::If) {
                self.advance();
                Some(Box::new(self.parse_expr()?))
            } else { None };
            self.expect(TokenKind::Colon)?;
            let body = self.parse_block()?;
            arms.push(MatchArm { pattern, guard, body, span: Span::dummy() });
        }
        Ok(Stmt::Match(MatchStmt { expression: Box::new(expression), arms, span: Span::dummy() }))
    }

    fn parse_pattern(&mut self) -> Result<Pattern, String> {
        if self.at_identifier() {
            let name = self.eat_identifier();
            return Ok(Pattern::Identifier { name, span: Span::dummy() });
        }
        let lit = match &self.current()?.kind {
            TokenKind::Integer(s) => {
                let n: i64 = s.parse().unwrap_or(0);
                Literal::Integer(n)
            }
            TokenKind::Float(s) => {
                let n: f64 = s.parse().unwrap_or(0.0);
                Literal::Float(n)
            }
            TokenKind::String(s) => Literal::String(s.clone()),
            TokenKind::True => Literal::Boolean(true),
            TokenKind::False => Literal::Boolean(false),
            _ => {
                let found = format!("{:?}", self.current()?.kind);
                return Err(format!("expected pattern, found {}", found));
            }
        };
        self.advance();
        Ok(Pattern::Literal {
            value: lit,
            span: Span::dummy(),
        })
    }

    fn parse_expr_stmt(&mut self) -> Result<Stmt, String> {
        let expr = self.parse_expr()?;
        Ok(Stmt::Expression(expr))
    }

    // -----------------------------------------------------------------------
    // Token inspection helpers
    // -----------------------------------------------------------------------

    /// Returns true if the current token has the given kind (by discriminant).
    fn at(&self, kind: TokenKind) -> bool {
        self.current().map_or(false, |t| t.is(&kind))
    }

    /// Returns true if the current token is an Identifier.
    fn at_identifier(&self) -> bool {
        self.current().map_or(false, |t| matches!(t.kind, TokenKind::Identifier(_)))
    }

    /// Returns true if the current and next token look like `ident =`.
    fn peek_equals(&self) -> bool {
        if !self.at_identifier() { return false; }
        self.tokens.get(self.pos + 1).map_or(false, |t| t.is(&TokenKind::Equals))
    }

    /// Returns the current token, or an error.
    fn current(&self) -> Result<&Token, String> {
        self.tokens.get(self.pos).ok_or_else(|| "unexpected end of input".to_string())
    }

    /// Advance one token.
    fn advance(&mut self) {
        self.pos += 1;
    }

    /// Consume the current identifier and return its name.
    fn eat_identifier(&mut self) -> String {
        if let Ok(tok) = self.current() {
            if let TokenKind::Identifier(name) = &tok.kind {
                let name = name.clone();
                self.advance();
                return name;
            }
        }
        String::new()
    }

    /// Expect a specific token kind; return error if not found.
    fn expect(&mut self, kind: TokenKind) -> Result<(), String> {
        if self.at(kind.clone()) {
            self.advance();
            Ok(())
        } else {
            let found = self.current().map(|t| format!("{:?}", t.kind)).unwrap_or_else(|_| "EOF".into());
            Err(format!("expected {:?}, found {}", kind, found))
        }
    }

    /// Expect a keyword by its string representation and consume the token.
    fn expect_keyword(&mut self, word: &str) -> Result<(), String> {
        let tok = self.current()?;
        // Map keyword strings to their token kinds.
        let expected = match word {
            "else" => TokenKind::Else,
            "in" => TokenKind::In,
            "as" => TokenKind::As,
            "import" => TokenKind::Import,
            _ => return Err(format!("unknown expected keyword '{}'", word)),
        };
        if tok.is(&expected) {
            self.advance();
            Ok(())
        } else {
            Err(format!("expected keyword '{}', found {:?}", word, tok.kind))
        }
    }

    fn current_operator(&self) -> Option<BinaryOp> {
        let kind = self.current().ok()?;
        match &kind.kind {
            TokenKind::Plus => Some(BinaryOp::Add),
            TokenKind::Minus => Some(BinaryOp::Sub),
            TokenKind::Star => Some(BinaryOp::Mul),
            TokenKind::Slash => Some(BinaryOp::Div),
            TokenKind::Percent => Some(BinaryOp::Rem),
            TokenKind::StarStar => Some(BinaryOp::Pow),
            TokenKind::EqualsEquals => Some(BinaryOp::Eq),
            TokenKind::BangEquals => Some(BinaryOp::Neq),
            TokenKind::Less => Some(BinaryOp::Lt),
            TokenKind::Greater => Some(BinaryOp::Gt),
            TokenKind::LessEquals => Some(BinaryOp::Le),
            TokenKind::GreaterEquals => Some(BinaryOp::Ge),
            TokenKind::Ampersand => Some(BinaryOp::BitAnd),
            TokenKind::Pipe => Some(BinaryOp::BitOr),
            TokenKind::Caret => Some(BinaryOp::BitXor),
            TokenKind::LessLess => Some(BinaryOp::Shl),
            TokenKind::GreaterGreater => Some(BinaryOp::Shr),
            TokenKind::PlusPercent => Some(BinaryOp::AddPercent),
            TokenKind::MinusPercent => Some(BinaryOp::SubPercent),
            TokenKind::StarPercent => Some(BinaryOp::MulPercent),
            TokenKind::Equals => Some(BinaryOp::Assign),
            TokenKind::PlusEquals => Some(BinaryOp::AddAssign),
            TokenKind::MinusEquals => Some(BinaryOp::SubAssign),
            TokenKind::StarEquals => Some(BinaryOp::MulAssign),
            TokenKind::SlashEquals => Some(BinaryOp::DivAssign),
            TokenKind::PercentEquals => Some(BinaryOp::RemAssign),
            TokenKind::AmpersandEquals => Some(BinaryOp::BitAndAssign),
            TokenKind::PipeEquals => Some(BinaryOp::BitOrAssign),
            TokenKind::CaretEquals => Some(BinaryOp::BitXorAssign),
            TokenKind::LessLessEquals => Some(BinaryOp::ShlAssign),
            TokenKind::GreaterGreaterEquals => Some(BinaryOp::ShrAssign),
            _ => Option::None,
        }
    }

    fn operator_precedence(&self, op: &BinaryOp) -> u8 {
        match op {
            BinaryOp::Assign | BinaryOp::AddAssign | BinaryOp::SubAssign
            | BinaryOp::MulAssign | BinaryOp::DivAssign | BinaryOp::RemAssign
            | BinaryOp::BitAndAssign | BinaryOp::BitOrAssign | BinaryOp::BitXorAssign
            | BinaryOp::ShlAssign | BinaryOp::ShrAssign => 1,
            BinaryOp::Or => 2,
            BinaryOp::And => 3,
            BinaryOp::Eq | BinaryOp::Neq => 4,
            BinaryOp::Lt | BinaryOp::Gt | BinaryOp::Le | BinaryOp::Ge => 5,
            BinaryOp::BitOr => 6,
            BinaryOp::BitXor => 7,
            BinaryOp::BitAnd => 8,
            BinaryOp::Shl | BinaryOp::Shr => 9,
            BinaryOp::Add | BinaryOp::Sub | BinaryOp::AddPercent | BinaryOp::SubPercent => 10,
            BinaryOp::Mul | BinaryOp::Div | BinaryOp::Rem | BinaryOp::MulPercent => 11,
            BinaryOp::Pow => 12,
            _ => 0,
        }
    }

    fn current_is_expr_start(&self) -> bool {
        self.current().map_or(false, |t| match t.kind {
            TokenKind::Integer(_) | TokenKind::Float(_) | TokenKind::String(_)
            | TokenKind::True | TokenKind::False | TokenKind::Identifier(_)
            | TokenKind::LParen | TokenKind::LBracket | TokenKind::LBrace
            | TokenKind::Minus | TokenKind::Bang | TokenKind::Tilde => true,
            _ => false,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    fn parse(source: &str) -> Result<Program, String> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        parser.parse()
    }

    // -----------------------------------------------------------------------
    // Declarations
    // -----------------------------------------------------------------------

    #[test]
    fn test_empty_program() {
        let p = parse("").unwrap();
        assert!(p.declarations.is_empty());
    }

    #[test]
    fn test_function_no_args_no_return() {
        let p = parse("fn main():\n    pass\n").unwrap();
        assert_eq!(p.declarations.len(), 1);
    }

    #[test]
    fn test_function_with_args_and_return() {
        let p = parse("fn add(x: i32, y: i32) -> i32:\n    x + y\n").unwrap();
        assert_eq!(p.declarations.len(), 1);
    }

    #[test]
    fn test_import_decl() {
        let p = parse("import math\n").unwrap();
        assert_eq!(p.declarations.len(), 1);
    }

    #[test]
    fn test_from_import() {
        let source = "from math import sqrt\n";
        match parse(source) {
            Ok(p) => assert_eq!(p.declarations.len(), 1),
            Err(e) => panic!("from_import parse failed: {}", e),
        }
    }

    // -----------------------------------------------------------------------
    // Variables and constants
    // -----------------------------------------------------------------------

    #[test]
    fn test_variable_declaration() {
        let p = parse("name = 42\n").unwrap();
        assert_eq!(p.declarations.len(), 1);
    }

    #[test]
    fn test_constant_declaration() {
        let p = parse("MAX_SIZE = 1024\n").unwrap();
        assert_eq!(p.declarations.len(), 1);
    }

    // -----------------------------------------------------------------------
    // Classes and structs
    // -----------------------------------------------------------------------

    #[test]
    fn test_class_with_fields() {
        let source = "\
class User:
    name: str
    age: i32
";
        let p = parse(source).unwrap();
        assert_eq!(p.declarations.len(), 1);
    }

    #[test]
    fn test_abstract_class() {
        let source = "\
abs class Animal:
    fn speak() -> str
";
        let p = parse(source).unwrap();
        assert_eq!(p.declarations.len(), 1);
    }

    #[test]
    fn test_struct() {
        let source = "\
struct Point:
    x: f64
    y: f64
";
        let p = parse(source).unwrap();
        assert_eq!(p.declarations.len(), 1);
    }

    #[test]
    fn test_enum() {
        let source = "\
enum Color:
    Red
    Green
    Blue
";
        let p = parse(source).unwrap();
        assert_eq!(p.declarations.len(), 1);
    }

    #[test]
    fn test_contract() {
        let source = "\
contract Drawable:
    fn draw()
";
        let p = parse(source).unwrap();
        assert_eq!(p.declarations.len(), 1);
    }

    #[test]
    fn test_type_alias() {
        let p = parse("type Age = i32\n").unwrap();
        assert_eq!(p.declarations.len(), 1);
    }

    // -----------------------------------------------------------------------
    // Statements
    // -----------------------------------------------------------------------

    #[test]
    fn test_if_statement() {
        let source = "\
fn test():\n\
    if x > 0:\n\
        print(\"pos\")\n";
        let p = parse(source).unwrap();
        assert_eq!(p.declarations.len(), 1);
        // Verify the function body exists
        if let Decl::Function(f) = &p.declarations[0] {
            assert!(f.body.is_some());
        } else {
            panic!("expected function declaration");
        }
    }

    #[test]
    fn test_if_elif_else() {
        let source = "fn test():\n    if x > 0:\n        print(\"pos\")\n    elif x < 0:\n        print(\"neg\")\n    else:\n        print(\"zero\")\n";
        match parse(source) {
            Ok(_) => {},
            Err(e) => panic!("if_elif_else parse failed: {}", e),
        }
    }

    #[test]
    fn test_while_loop() {
        let source = "\
fn test():\n\
    while running:\n\
        process()\n";
        assert!(parse(source).is_ok());
    }

    #[test]
    fn test_for_loop() {
        let source = "\
fn test():\n\
    for item in items:\n\
        print(item)\n";
        assert!(parse(source).is_ok());
    }

    #[test]
    fn test_match_statement() {
        let source = "fn test():\n    match value:\n        1:\n            print(\"one\")\n        2:\n            print(\"two\")\n";
        match parse(source) {
            Ok(_) => {},
            Err(e) => panic!("match parse failed: {}", e),
        }
    }

    #[test]
    fn test_return_with_value() {
        let source = "fn add(a: i32, b: i32) -> i32:\n    a + b\n";
        assert!(parse(source).is_ok());
    }

    #[test]
    fn test_defer_statement() {
        let source = "\
fn test():\n\
    defer cleanup()\n";
        assert!(parse(source).is_ok());
    }

    #[test]
    fn test_guard_statement() {
        let source = "fn test():\n    guard valid else:\n        return\n";
        match parse(source) {
            Ok(_) => {},
            Err(e) => panic!("guard parse failed: {}", e),
        }
    }

    #[test]
    fn test_unsafe_block() {
        let source = "\
fn test():\n\
    unsafe:\n\
        ptr = addr\n";
        assert!(parse(source).is_ok());
    }

    // -----------------------------------------------------------------------
    // Expressions
    // -----------------------------------------------------------------------

    #[test]
    fn test_binary_expression() {
        let source = "\
fn test():\n\
    x = 1 + 2 * 3\n";
        assert!(parse(source).is_ok());
    }

    #[test]
    fn test_function_call() {
        let source = "\
fn test():\n\
    print(\"hello\")\n";
        assert!(parse(source).is_ok());
    }

    #[test]
    fn test_property_access() {
        let source = "\
fn test():\n\
    name = user.name\n";
        assert!(parse(source).is_ok());
    }

    #[test]
    fn test_list_literal() {
        let source = "\
fn test():\n\
    items = [1, 2, 3]\n";
        assert!(parse(source).is_ok());
    }

    #[test]
    fn test_dict_literal() {
        let source = "\
fn test():\n\
    config = {key: \"value\"}\n";
        assert!(parse(source).is_ok());
    }

    // -----------------------------------------------------------------------
    // Error cases
    // -----------------------------------------------------------------------

    #[test]
    fn test_error_bad_expression() {
        let result = parse("fn test():\n    +\n");
        assert!(result.is_err());
    }

    #[test]
    fn test_error_missing_type() {
        let result = parse("fn f() -> :\n    x\n");
        assert!(result.is_err());
    }

    #[test]
    fn test_error_unexpected_declaration() {
        let result = parse("if x:\n    y\n");
        assert!(result.is_err());
    }

    // -----------------------------------------------------------------------
    // Integration — example files
    // -----------------------------------------------------------------------

    #[test]
    fn test_hello_example() {
        let source = include_str!("../../../examples/hello.kl");
        assert!(parse(source).is_ok(), "hello.kl should parse");
    }

    #[test]
    fn test_fibonacci_example() {
        let source = include_str!("../../../examples/fibonacci.kl");
        assert!(parse(source).is_ok(), "fibonacci.kl should parse");
    }

    #[test]
    fn test_user_example() {
        let source = include_str!("../../../examples/user.kl");
        assert!(parse(source).is_ok(), "user.kl should parse");
    }
}
