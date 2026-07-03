// kyc_frontend::parser — Recursive descent parser for KL
//
// Transforms a token stream into an AST.
// Reference: docs/02-formal-grammar.md, docs/03-ast-specification.md

use kyc_core::ast::*;
use kyc_core::span::Span;
use crate::token::{Token, TokenKind};

/// The KL recursive-descent parser.
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    errors: Vec<String>,
    links: Vec<String>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0, errors: vec![], links: vec![] }
    }

    fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    fn error(&mut self, msg: String) {
        self.errors.push(msg);
    }

    /// Return true if the current token starts a statement.
    fn at_stmt_start(&self) -> bool {
        self.current().map_or(false, |t| matches!(t.kind,
            TokenKind::If | TokenKind::While | TokenKind::For | TokenKind::Match
            | TokenKind::Loop | TokenKind::Return | TokenKind::Break
            | TokenKind::Continue | TokenKind::Fn
        ))
    }

    /// Skip tokens until a synchronization point: dedent, eof, newline,
    /// or a keyword that begins a statement or declaration.
    fn sync_to_stmt_boundary(&mut self) {
        // Always advance past the token that caused the error first.
        self.advance();
        let mut safety = 0;
        while safety < 100 {
            safety += 1;
            if self.pos >= self.tokens.len() {
                return;
            }
            if self.at(TokenKind::Dedent) || self.at(TokenKind::Eof) {
                return;
            }
            if self.at(TokenKind::Newline) || self.at(TokenKind::Indent) {
                self.advance();
                return;
            }
            if self.at_stmt_start() {
                return;
            }
            self.advance();
        }
    }

    /// Build a span covering from `start_pos` (token index at start) to the
    /// last consumed token (self.pos - 1). If no tokens consumed, returns a
    /// span around the current token or a dummy span.
    fn span_from(&self, start_pos: usize) -> Span {
        let start = &self.tokens[start_pos].span.start;
        let end_pos = self.pos.saturating_sub(1).max(start_pos);
        let end = &self.tokens[end_pos].span.end;
        Span {
            start: *start,
            end: *end,
            file_id: 0,
        }
    }

    /// Parse the full token stream into a Program AST node.
    pub fn parse(&mut self) -> Result<Program, String> {
        let start = self.pos;
        let mut declarations = Vec::new();
        loop {
            if self.at(TokenKind::Eof) {
                break;
            }
            if self.at(TokenKind::Newline) || self.at(TokenKind::Indent) || self.at(TokenKind::Dedent) {
                self.advance();
                continue;
            }
            match self.parse_decl() {
                Ok(decl) => declarations.push(decl),
                Err(msg) => {
                    self.error(msg);
                    self.sync_to_stmt_boundary();
                }
            }
        }
        if self.has_errors() {
            Err(self.errors.join("\n"))
        } else {
            Ok(Program {
                declarations,
                links: std::mem::take(&mut self.links),
                span: self.span_from(start),
            })
        }
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
        // `#[test]` attribute before function
        if self.at(TokenKind::Hash) {
            return self.parse_attr_function();
        }
        // extern fn — declare external C function (no body)
        if self.at(TokenKind::Extern) {
            let start = self.pos;
            self.advance();
            if !self.at(TokenKind::Fn) {
                return Err("expected 'fn' after 'extern'".to_string());
            }
            self.advance();
            let raw_name = self.eat_identifier();
            let (name, _visibility) = if raw_name.starts_with("__") {
                (raw_name.trim_start_matches('_').to_string(), Visibility::Private)
            } else if raw_name.starts_with('_') {
                (raw_name.trim_start_matches('_').to_string(), Visibility::Protected)
            } else {
                (raw_name.clone(), Visibility::Public)
            };
            self.expect(TokenKind::LParen)?;
            let params = self.parse_params()?;
            self.expect(TokenKind::RParen)?;
            let return_type = if self.at(TokenKind::Colon) || self.at(TokenKind::Newline)
                || self.at(TokenKind::Dedent) || self.at(TokenKind::Eof)
            {
                None
            } else {
                Some(self.parse_type()?)
            };
            return Ok(Decl::Function(FunctionDecl {
                name, params, return_type,
                type_params: vec![],
                is_async: false, is_const: false, is_abstract: false,
                is_static: false, is_test: false, is_extern: true,
                visibility: Visibility::Public,
                body: None,
                span: self.span_from(start),
            }));
        }
        // @link "libname" — link against a native library
        if self.at(TokenKind::At) {
            let start = self.pos;
            self.advance();
            if self.at_identifier() && self.eat_identifier() == "link" {
                if let TokenKind::String(s) = &self.current()?.kind {
                    let name = s.clone();
                    self.advance();
                    return Ok(Decl::Link(name, self.span_from(start)));
                }
                return Err("expected string literal after '@link'".to_string());
            }
            self.pos = start; // backtrack — not @link
        }
        if self.at(TokenKind::Fn)
            || self.at(TokenKind::Async)
            || self.at(TokenKind::Const)
        {
            return self.parse_function(false).map(Decl::Function);
        }
        // Class: `final class X:`, `class X:`, `abstract class X:`
        if self.at(TokenKind::Final) {
            self.advance();
            if self.at(TokenKind::Class) {
                return self.parse_class(false); // final class = normal class
            }
            return Err("expected 'class' after 'final'".to_string());
        }
        if self.at(TokenKind::Abstract) {
            self.advance();
            if self.at(TokenKind::Class) {
                return self.parse_class(true);
            }
            return Err("expected 'class' after 'abstract'".to_string());
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
        // Variable/constant declaration: `name = expr`, `name: &type = expr`, or `NAME := expr`
        if self.at_identifier() {
            let start = self.pos;
            let name = self.eat_identifier();
            // Check for typed declaration: `ident : type = expr`
            if self.at(TokenKind::Colon) {
                self.advance();
                let type_ = self.parse_type()?;
                let is_mutable = matches!(type_, AstType::Mutable { .. });
                if self.at(TokenKind::Equals) {
                    self.advance();
                    let value = self.parse_expr()?;
                    return Ok(Decl::Variable(VariableDecl {
                        name, type_: Some(type_), value: Box::new(value), is_mutable, span: self.span_from(start),
                    }));
                }
                return Err("expected '=' after typed declaration".to_string());
            }
            if self.at(TokenKind::Walrus) {
                // `NAME := expr` → constant
                self.advance();
                let value = self.parse_expr()?;
                return Ok(Decl::Constant(ConstantDecl {
                    name, value: Box::new(value), span: self.span_from(start),
                }));
            } else if self.at(TokenKind::Equals) {
                self.advance();
                let value = self.parse_expr()?;
                return Ok(Decl::Variable(VariableDecl {
                    name, type_: None, value: Box::new(value), is_mutable: false, span: self.span_from(start),
                }));
            }
            return Err(format!("expected '=', ':=', or ': type =' after identifier '{}'", name));
        }
        let found = self.current().map(|t| format!("{:?}", t.kind)).unwrap_or_else(|_| "EOF".into());
        Err(format!("unexpected token at declaration start: {}", found))
    }

    /// Read a dotted name: ident . ident . ident ...
    fn read_dotted_name(&mut self) -> Result<String, String> {
        let mut name = self.eat_identifier();
        while self.at(TokenKind::Dot) {
            self.advance(); // consume .
            name.push('.');
            name.push_str(&self.eat_identifier());
        }
        Ok(name)
    }

    fn parse_import(&mut self) -> Result<Decl, String> {
        let start = self.pos;
        self.advance(); // import
        let relative = if self.at(TokenKind::Tilde) {
            self.advance();
            true
        } else {
            false
        };
        let module_name = self.read_dotted_name()?;
        let alias = if self.at(TokenKind::As) {
            self.advance();
            Some(self.eat_identifier())
        } else {
            None
        };
        Ok(Decl::Import(Import { module_name, alias, relative, span: self.span_from(start) }))
    }

    fn parse_from_import(&mut self) -> Result<Decl, String> {
        let start = self.pos;
        self.advance(); // from
        let relative = if self.at(TokenKind::Tilde) {
            self.advance();
            true
        } else {
            false
        };
        let module_name = self.read_dotted_name()?;
        self.expect_keyword("import")?;
        let imported_name = self.eat_identifier();
        let alias = if self.at(TokenKind::As) {
            self.advance();
            Some(self.eat_identifier())
        } else {
            None
        };
        Ok(Decl::FromImport(FromImport { module_name, imported_name, alias, relative, span: self.span_from(start) }))
    }

    fn parse_type_alias(&mut self) -> Result<Decl, String> {
        let start = self.pos;
        self.advance(); // type
        let name = self.eat_identifier();
        let type_params = if self.at(TokenKind::Less) {
            self.parse_type_params()?
        } else {
            vec![]
        };
        if !self.at(TokenKind::Equals) {
            return Err("expected '=' in type alias".to_string());
        }
        self.advance();
        let type_ = self.parse_type()?;
        Ok(Decl::TypeAlias(TypeAlias { name, type_params, type_, span: self.span_from(start) }))
    }

    /// Parse `#[attr]` before a function declaration.
    /// Currently only `#[test]` is recognized.
    fn parse_attr_function(&mut self) -> Result<Decl, String> {
        let start = self.pos;
        self.advance(); // consume #
        if !self.at(TokenKind::LBracket) {
            return Err("expected '[' after '#' for attribute".to_string());
        }
        self.advance(); // consume [
        let attr_name = self.eat_identifier();
        if !self.at(TokenKind::RBracket) {
            return Err("expected ']' after attribute name".to_string());
        }
        self.advance(); // consume ]
        if attr_name != "test" {
            return Err(format!("unknown attribute: #[{}]", attr_name));
        }
        let mut func = self.parse_function(true)?;
        func.span = self.span_from(start);
        Ok(Decl::Function(func))
    }

    fn parse_function(&mut self, is_test: bool) -> Result<FunctionDecl, String> {
        let start = self.pos;
        let is_const = if self.at(TokenKind::Const) { self.advance(); true } else { false };
        let is_async = if self.at(TokenKind::Async) { self.advance(); true } else { false };
        let is_static = if self.at(TokenKind::Static) { self.advance(); true } else { false };
        if !self.at(TokenKind::Fn) {
            return Err("expected 'fn'".to_string());
        }
        self.advance();
        let mut raw_name = self.eat_identifier();
        // Operator overloading: "op_" followed by operator token → "op_+", "op_-", etc.
        if raw_name == "op" || raw_name == "op_" {
            let op_sym = match self.current()?.kind {
                TokenKind::Plus => { self.advance(); "+" }
                TokenKind::Minus => { self.advance(); "-" }
                TokenKind::Star => { self.advance(); "*" }
                TokenKind::Slash => { self.advance(); "/" }
                TokenKind::Percent => { self.advance(); "%" }
                TokenKind::Equals => { self.advance(); "==" }
                TokenKind::BangEquals => { self.advance(); "!=" }
                TokenKind::Less => { self.advance(); "<" }
                TokenKind::Greater => { self.advance(); ">" }
                TokenKind::LessEquals => { self.advance(); "<=" }
                TokenKind::GreaterEquals => { self.advance(); ">=" }
                _ => "",
            };
            if !op_sym.is_empty() {
                raw_name = format!("op_{}", op_sym);
            }
        }
        // Visibility convention: `__` prefix → private, `_` prefix → protected, none → public.
        // The stored name is always stripped of the convention prefixes.
        let (name, visibility) = if raw_name.starts_with("__") {
            (raw_name.trim_start_matches('_').to_string(), Visibility::Private)
        } else if raw_name.starts_with('_') {
            (raw_name.trim_start_matches('_').to_string(), Visibility::Protected)
        } else {
            (raw_name.clone(), Visibility::Public)
        };
        // Parse optional type params: `<T, U>`
        let type_params = if self.at(TokenKind::Less) {
            self.parse_type_params()?
        } else {
            vec![]
        };
        self.expect(TokenKind::LParen)?;
        let params = self.parse_params()?;
        self.expect(TokenKind::RParen)?;
        let return_type = if self.at(TokenKind::Colon)
            || self.at(TokenKind::Newline) || self.at(TokenKind::Dedent) || self.at(TokenKind::Eof)
        {
            None
        } else {
            let type_start = self.pos;
            let base = self.parse_type()?;
            if self.at(TokenKind::Bang) {
                self.advance();
                Some(AstType::Error { inner: Box::new(base), span: self.span_from(type_start) })
            } else {
                Some(base)
            }
        };
        let is_abstract = !self.at(TokenKind::Colon);
        let body = if is_abstract {
            None
        } else {
            self.advance(); // ':'
            Some(self.parse_block()?)
        };
        Ok(FunctionDecl {
            name, type_params, params, return_type, is_async, is_const, is_static, is_abstract, is_extern: false, is_test, visibility, body,
            span: self.span_from(start),
        })
    }

    fn parse_class(&mut self, is_abstract: bool) -> Result<Decl, String> {
        let start = self.pos;
        self.advance(); // class
        let name = self.eat_identifier();
        // Parse optional type params: `<T, U>`
        let type_params = if self.at(TokenKind::Less) {
            self.parse_type_params()?
        } else {
            vec![]
        };

        // `class Name :: Parent, Contract1, Contract2:`
        // `::` is lexed as two consecutive Colons (not ConstDecl)
        let mut parent: Option<String> = None;
        let mut contracts: Vec<String> = Vec::new();

        if self.at(TokenKind::Colon) {
            let saved = self.pos;
            self.advance();
            if self.at(TokenKind::Colon) {  // second colon → :: syntax
                self.advance();
                // Parse comma-separated list of identifiers
                let mut items: Vec<String> = Vec::new();
                if self.at_identifier() {
                    items.push(self.eat_identifier());
                    while self.at(TokenKind::Comma) {
                        self.advance();
                        if self.at_identifier() {
                            items.push(self.eat_identifier());
                        } else { break; }
                    }
                }
                // First item = parent (if it exists), rest = contracts
                if !items.is_empty() {
                    parent = Some(items.remove(0));
                    contracts = items;
                }
            } else {
                // Single colon without :: — simple class, no parent/contracts
                self.pos = saved;
            }
        }

        // Expect body start
        self.expect(TokenKind::Colon)?;
        let members = self.parse_class_members()?;
        self.make_class_decl(start, name, type_params, parent, contracts, members, is_abstract)
    }

    fn make_class_decl(
        &self,
        start_pos: usize,
        name: String,
        type_params: Vec<TypeParam>,
        parent: Option<String>,
        contracts: Vec<String>,
        mut members: Vec<ClassMember>,
        is_abstract: bool,
    ) -> Result<Decl, String> {
        // If the class has no explicit constructor, synthesize a default empty
        // one so that `ClassName()` instantiations work without user-written
        // boilerplate.
        let has_ctor = members.iter().any(|m| matches!(m, ClassMember::Constructor(_)));
        if !has_ctor && !is_abstract {
            members.push(ClassMember::Constructor(Constructor {
                params: Vec::new(),
                body: Block { statements: Vec::new(), span: self.span_from(start_pos) },
                span: self.span_from(start_pos),
            }));
        }
        if is_abstract {
            Ok(Decl::AbstractClass(AbstractClassDecl {
                name, type_params, parent, contracts, members, span: self.span_from(start_pos),
            }))
        } else {
            Ok(Decl::Class(ClassDecl {
                name, type_params, parent, contracts, members, span: self.span_from(start_pos),
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
                    let constructor_start = self.pos;
                    self.advance();
                    let params = self.parse_params()?;
                    self.expect(TokenKind::RParen)?;
                    self.expect(TokenKind::Colon)?;
                    let body = self.parse_block()?;
                    members.push(ClassMember::Constructor(Constructor {
                        params, body, span: self.span_from(constructor_start),
                    }));
                    continue;
                }
                // Check for field or property: identifier followed by ':'
                if self.at(TokenKind::Colon) {
                    let member_start = self.pos;
                    self.advance();
                    let type_ = self.parse_type()?;
                    let is_mutable = matches!(type_, AstType::Mutable { .. });
                    let visibility = if name.starts_with("__") {
                        Visibility::Private
                    } else if name.starts_with('_') {
                        Visibility::Protected
                    } else {
                        Visibility::Public
                    };
                    // Field default: `name: type = expr`
                    let default = if self.at(TokenKind::Equals) {
                        self.advance();
                        Some(Box::new(self.parse_expr()?))
                    } else {
                        None
                    };
                    // Check for property getter/setter blocks (indented under the field)
                    let mut getter = None;
                    let mut setter = None;
                    if self.at(TokenKind::Newline) {
                        self.advance();
                        if self.at(TokenKind::Indent) {
                            self.advance(); // enter property block
                            loop {
                                if self.at(TokenKind::Newline) { self.advance(); continue; }
                                if self.at(TokenKind::Dedent) { self.advance(); break; }
                                if self.at(TokenKind::Get) {
                                    self.advance();
                                    self.expect(TokenKind::Colon)?;
                                    getter = Some(self.parse_block()?);
                                } else if self.at(TokenKind::Set) {
                                    self.advance();
                                    let set_param = if self.at(TokenKind::LParen) {
                                        self.advance();
                                        let p = self.eat_identifier();
                                        self.expect(TokenKind::RParen)?;
                                        p
                                    } else {
                                        "value".to_string()
                                    };
                                    self.expect(TokenKind::Colon)?;
                                    setter = Some((set_param, self.parse_block()?));
                                } else {
                                    break;
                                }
                            }
                        }
                    }
                    if getter.is_some() || setter.is_some() {
                        members.push(ClassMember::Property(Property {
                            name, type_, getter, setter, span: self.span_from(member_start),
                        }));
                    } else {
                        members.push(ClassMember::Field(Field {
                            name, type_, is_mutable, default, visibility, span: self.span_from(member_start),
                        }));
                    }
                    continue;
                }
                // Unknown — break and let the outer parser handle it
                break;
            }
            // Method: `static fn name(params):` or `fn name(params):`
            if self.at(TokenKind::Static) || self.at(TokenKind::Fn) || self.at(TokenKind::Async) || self.at(TokenKind::Const) || self.at(TokenKind::Abstract) {
                let method = self.parse_function(false)?;
                members.push(ClassMember::Method(method));
                continue;
            }
            break;
        }
        Ok(members)
    }

    fn parse_struct(&mut self) -> Result<StructDecl, String> {
        let start = self.pos;
        self.advance(); // struct
        let name = self.eat_identifier();
        let type_params = if self.at(TokenKind::Less) {
            self.parse_type_params()?
        } else {
            vec![]
        };
        self.expect(TokenKind::Colon)?;
        if self.at(TokenKind::Newline) { self.advance(); }
        if self.at(TokenKind::Indent) { self.advance(); }
        let mut fields = Vec::new();
        loop {
            if self.at(TokenKind::Dedent) { self.advance(); break; }
            if self.at(TokenKind::Eof) { break; }
            if self.at(TokenKind::Newline) { self.advance(); continue; }
            let field_start = self.pos;
            let field_name = self.eat_identifier();
            self.expect(TokenKind::Colon)?;
            let type_ = self.parse_type()?;
            let is_mutable = matches!(type_, AstType::Mutable { .. });
            fields.push(Field { name: field_name, type_, is_mutable, default: None, visibility: Visibility::Public, span: self.span_from(field_start) });
        }
        Ok(StructDecl { name, type_params, fields, span: self.span_from(start) })
    }

    fn parse_enum(&mut self) -> Result<EnumDecl, String> {
        let start = self.pos;
        self.advance(); // enum
        let name = self.eat_identifier();
        let type_params = if self.at(TokenKind::Less) {
            self.parse_type_params()?
        } else {
            vec![]
        };
        self.expect(TokenKind::Colon)?;
        if self.at(TokenKind::Newline) { self.advance(); }
        if self.at(TokenKind::Indent) { self.advance(); }
        let mut variants = Vec::new();
        loop {
            if self.at(TokenKind::Dedent) { self.advance(); break; }
            if self.at(TokenKind::Eof) { break; }
            if self.at(TokenKind::Newline) { self.advance(); continue; }
            let variant_start = self.pos;
            let variant_name = self.eat_identifier();
            if variant_name.is_empty() {
                let found = format!("{:?}", self.current()?.kind);
                return Err(format!("expected enum variant name, found {}", found));
            }
            let payload = if self.at(TokenKind::LParen) {
                self.advance();
                let mut types = Vec::new();
                loop {
                    types.push(self.parse_type()?);
                    if self.at(TokenKind::Comma) { self.advance(); }
                    else { break; }
                }
                self.expect(TokenKind::RParen)?;
                types
            } else {
                vec![]
            };
            variants.push(EnumVariant { name: variant_name, payload, span: self.span_from(variant_start) });
        }
        Ok(EnumDecl { name, type_params, variants, span: self.span_from(start) })
    }

    fn parse_contract(&mut self) -> Result<ContractDecl, String> {
        let start = self.pos;
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
            let method_start = self.pos;
            self.advance();
            let method_name = self.eat_identifier();
            self.expect(TokenKind::LParen)?;
            let params = self.parse_params()?;
            self.expect(TokenKind::RParen)?;
            let return_type = if self.at(TokenKind::Colon)
                || self.at(TokenKind::Newline) || self.at(TokenKind::Dedent) || self.at(TokenKind::Eof)
            {
                None
            } else {
                Some(self.parse_type()?)
            };
            methods.push(ContractMethod { name: method_name, params, return_type, span: self.span_from(method_start) });
        }
        Ok(ContractDecl { name, methods, span: self.span_from(start) })
    }

    fn parse_params(&mut self) -> Result<Vec<Parameter>, String> {
        let mut params = Vec::new();
        loop {
            if self.at(TokenKind::RParen) { break; }
            let param_start = self.pos;
            // `^name: type` — move/ownership transfer parameter
            let is_move = self.at(TokenKind::Caret);
            if is_move {
                self.advance();
            }
            let variadic = if self.at(TokenKind::DotDotDot) {
                self.advance();
                true
            } else {
                false
            };
            let name = if variadic {
                if self.at_identifier() {
                    self.eat_identifier()
                } else {
                    "_args".to_string()
                }
            } else {
                self.eat_identifier()
            };
            let type_ = if self.at(TokenKind::Colon) {
                self.advance();
                self.parse_type()?
            } else {
                AstType::Primitive { name: "void".into(), span: self.span_from(param_start) }
            };
            // Determine param mode from type prefix and ^ prefix
            let mode = if is_move {
                ParamMode::Move
            } else if matches!(type_, AstType::Mutable { .. }) {
                ParamMode::MutableBorrow
            } else {
                ParamMode::Borrow
            };
            let default = if self.at(TokenKind::Equals) {
                self.advance();
                Some(Box::new(self.parse_expr()?))
            } else { None };
            params.push(Parameter { name, type_, default, variadic, mode, span: self.span_from(param_start) });
            if self.at(TokenKind::Comma) { self.advance(); } else { break; }
        }
        Ok(params)
    }

    fn parse_type_params(&mut self) -> Result<Vec<TypeParam>, String> {
        self.advance(); // '<'
        let mut params = Vec::new();
        loop {
            if self.at(TokenKind::Greater) { break; }
            let param_start = self.pos;
            let name = self.eat_identifier();
            let constraint = if self.at(TokenKind::Colon) {
                self.advance();
                Some(self.parse_type()?)
            } else {
                None
            };
            params.push(TypeParam { name, constraint, span: self.span_from(param_start) });
            if self.at(TokenKind::Comma) { self.advance(); } else { break; }
        }
        self.expect(TokenKind::Greater)?;
        Ok(params)
    }

    // -----------------------------------------------------------------------
    // Type parsing
    // -----------------------------------------------------------------------

    fn parse_type(&mut self) -> Result<AstType, String> {
        let start = self.pos;
        // Handle `&T` — mutable reference type
        if self.at(TokenKind::Ampersand) {
            self.advance();
            let inner = self.parse_type()?;
            return Ok(AstType::Mutable { inner: Box::new(inner), span: self.span_from(start) });
        }
        // Handle `^T` — move type
        if self.at(TokenKind::Caret) {
            self.advance();
            let inner = self.parse_type()?;
            return Ok(AstType::Move { inner: Box::new(inner), span: self.span_from(start) });
        }
        // Handle list shorthand: [T] → List<T>
        if self.at(TokenKind::LBracket) {
            self.advance();
            let inner = self.parse_type()?;
            self.expect(TokenKind::RBracket)?;
            return Ok(AstType::Generic {
                name: "list".to_string(),
                args: vec![inner],
                span: self.span_from(start),
            });
        }
        // Handle function pointer type: fn(T, U) V
        if self.at(TokenKind::Fn) {
            self.advance(); // 'fn'
            self.expect(TokenKind::LParen)?;
            let mut elems = Vec::new();
            if !self.at(TokenKind::RParen) {
                elems.push(self.parse_type()?);
                while self.at(TokenKind::Comma) {
                    self.advance();
                    if self.at(TokenKind::RParen) { break; }
                    elems.push(self.parse_type()?);
                }
            }
            self.expect(TokenKind::RParen)?;
            // Optional return type (void if absent)
            let return_ = if self.at(TokenKind::Colon) || self.at(TokenKind::Newline)
                || self.at(TokenKind::Dedent) || self.at(TokenKind::Eof)
                || self.at(TokenKind::Comma) || self.at(TokenKind::RParen)
                || self.at(TokenKind::Greater) || self.at(TokenKind::RBracket)
            {
                None
            } else {
                Some(self.parse_type()?)
            };
            let return_type = return_.unwrap_or(
                AstType::Primitive { name: "void".to_string(), span: self.span_from(start) }
            );
            return Ok(AstType::FnPtr {
                params: elems,
                return_: Box::new(return_type),
                span: self.span_from(start),
            });
        }
        // Tuple type: (T, U) — only if not followed by more type syntax
        if self.at(TokenKind::LParen) {
            self.advance();
            let mut elems = Vec::new();
            if !self.at(TokenKind::RParen) {
                elems.push(self.parse_type()?);
                while self.at(TokenKind::Comma) {
                    self.advance();
                    if self.at(TokenKind::RParen) { break; }
                    elems.push(self.parse_type()?);
                }
            }
            self.expect(TokenKind::RParen)?;
            // Tuples are represented as a generic with name "tuple" for now
            return Ok(AstType::Generic {
                name: "tuple".to_string(),
                args: elems,
                span: self.span_from(start),
            });
        }
        let name = self.eat_identifier();
        if name.is_empty() {
            let found = self.current().map(|t| format!("{:?}", t.kind)).unwrap_or_else(|_| "EOF".into());
            return Err(format!("expected type name, found {}", found));
        }
        // Handle ptr as a built-in type
        if name == "ptr" {
            return Ok(AstType::Ptr { span: self.span_from(start) });
        }
        let mut base = if self.at(TokenKind::Less) {
            self.advance();
            let mut args = Vec::new();
            args.push(self.parse_type()?);
            while self.at(TokenKind::Comma) {
                self.advance();
                args.push(self.parse_type()?);
            }
            self.expect(TokenKind::Greater)?;
            AstType::Generic { name, args, span: self.span_from(start) }
        } else {
            AstType::User { name, span: self.span_from(start) }
        };
        // Postfix `?` for optional types: T?, list<i32>?
        if self.at(TokenKind::Question) {
            self.advance();
            base = AstType::Optional { inner: Box::new(base), span: self.span_from(start) };
        }
        // Postfix `!` for error-returning types: T!, list<i32>!
        // (already uses AstType::Error which exists)
        if self.at(TokenKind::Bang) {
            self.advance();
            base = AstType::Error { inner: Box::new(base), span: self.span_from(start) };
        }
        Ok(base)
    }

    // -----------------------------------------------------------------------
    // Expression parsing
    // -----------------------------------------------------------------------

    fn parse_expr(&mut self) -> Result<Expr, String> {
        self.parse_binary(0)
    }

    fn parse_binary(&mut self, min_prec: u8) -> Result<Expr, String> {
        let start = self.pos;
        let mut left = self.parse_unary()?;
        loop {
            // Check for '??' — default operator (expr ?? default)
            if self.at(TokenKind::QuestionQuestion) {
                const NULL_COALESCE_PREC: u8 = 3;
                if NULL_COALESCE_PREC < min_prec { break; }
                let start = self.pos;
                self.advance();
                let right = self.parse_binary(NULL_COALESCE_PREC + 1)?;
                left = Expr::NullCoalesce {
                    left: Box::new(left),
                    right: Box::new(right),
                    span: self.span_from(start),
                };
                continue;
            }
            // Check for '?' — BOTH ternary (cond ? then : else) and error-prop (expr?)
            if self.at(TokenKind::Question) {
                const TERNARY_PREC: u8 = 2;
                if TERNARY_PREC < min_prec { break; }
                let saved = self.pos;
                self.advance(); // consume '?'
                // Try ternary first: parse middle expression, check for ':'
                match self.parse_binary(0) {
                    Ok(middle) => {
                        if self.at(TokenKind::Colon) {
                            self.advance(); // consume ':'
                            let right = self.parse_binary(TERNARY_PREC)?;
                            left = Expr::Ternary {
                                cond: Box::new(left),
                                then_expr: Box::new(middle),
                                else_expr: Box::new(right),
                                span: self.span_from(start),
                            };
                            continue; // allow chaining: a ? b : c ? d : e
                        }
                        // Not ternary — fall through to error prop
                    }
                    Err(_) => {
                        // Not ternary — fall through to error prop
                    }
                }
                // Restore and handle as error propagation
                self.pos = saved;
                self.advance(); // consume '?'
                left = Expr::ErrorProp { expression: Box::new(left), span: self.span_from(start) };
                continue;
            }
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
                span: self.span_from(start),
            };
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, String> {
        if self.at(TokenKind::Minus) {
            let start = self.pos;
            self.advance();
            return Ok(Expr::Unary { operator: UnaryOp::Neg, operand: Box::new(self.parse_primary()?), span: self.span_from(start) });
        }
        if self.at(TokenKind::Bang) {
            let start = self.pos;
            self.advance();
            return Ok(Expr::Unary { operator: UnaryOp::Not, operand: Box::new(self.parse_primary()?), span: self.span_from(start) });
        }
        if self.at(TokenKind::Tilde) {
            let start = self.pos;
            self.advance();
            return Ok(Expr::Unary { operator: UnaryOp::BitNot, operand: Box::new(self.parse_primary()?), span: self.span_from(start) });
        }
        if self.at(TokenKind::Await) {
            let start = self.pos;
            self.advance();
            return Ok(Expr::Await { expression: Box::new(self.parse_primary()?), span: self.span_from(start) });
        }
        // `&expr` — mutable reference at call site (coercion for &T params)
        if self.at(TokenKind::Ampersand) {
            let start = self.pos;
            self.advance();
            return Ok(Expr::MutableRef { expression: Box::new(self.parse_primary()?), span: self.span_from(start) });
        }
        // `^expr` — ownership transfer at call site (for ^T params)
        if self.at(TokenKind::Caret) {
            let start = self.pos;
            self.advance();
            return Ok(Expr::MoveExpr { expression: Box::new(self.parse_primary()?), span: self.span_from(start) });
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        let start = self.pos;
        let tok = self.current()?;
        let expr = match &tok.kind {
            TokenKind::Integer(s) => {
                let val = if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
                    i64::from_str_radix(hex, 16).unwrap_or(0)
                } else if let Some(bin) = s.strip_prefix("0b").or_else(|| s.strip_prefix("0B")) {
                    i64::from_str_radix(bin, 2).unwrap_or(0)
                } else if let Some(oct) = s.strip_prefix("0o").or_else(|| s.strip_prefix("0O")) {
                    i64::from_str_radix(oct, 8).unwrap_or(0)
                } else {
                    s.parse::<i64>().unwrap_or(0)
                };
                self.advance();
                Expr::Literal { value: Literal::Integer(val), span: self.span_from(start) }
            }
            TokenKind::Float(s) => {
                let val = s.parse::<f64>().unwrap_or(0.0);
                self.advance();
                Expr::Literal { value: Literal::Float(val), span: self.span_from(start) }
            }
            TokenKind::String(s) => {
                let val = s.clone();
                self.advance();
                if val.contains('{') {
                    self.parse_string_interp(&val, start)?
                } else {
                    Expr::Literal { value: Literal::String(val), span: self.span_from(start) }
                }
            }
            TokenKind::True => {
                self.advance();
                Expr::Literal { value: Literal::Boolean(true), span: self.span_from(start) }
            }
            TokenKind::False => {
                self.advance();
                Expr::Literal { value: Literal::Boolean(false), span: self.span_from(start) }
            }
            TokenKind::None => {
                self.advance();
                Expr::Literal { value: Literal::None, span: self.span_from(start) }
            }
            TokenKind::Null => {
                self.advance();
                Expr::Literal { value: Literal::Null, span: self.span_from(start) }
            }
            TokenKind::Char(s) => {
                let val = s.chars().next().unwrap_or('\0') as u8 as i64;
                self.advance();
                Expr::Literal { value: Literal::Integer(val), span: self.span_from(start) }
            }
            TokenKind::Identifier(name) => {
                let val = name.clone();
                self.advance();
                Expr::Identifier { name: val, span: self.span_from(start) }
            }
            TokenKind::LParen => {
                let start = self.pos;
                self.advance(); // consume '('
                // Try closure: (params) => expr
                let saved = self.pos;
                let params = self.parse_closure_params();
                if self.at(TokenKind::RParen) {
                    self.advance(); // consume ')'
                    if self.at(TokenKind::FatArrow) {
                        self.advance(); // consume '=>'
                        let body = self.parse_expr()?;
                        Expr::Closure { params, body: Box::new(body), span: self.span_from(start) }
                    } else {
                        // Not a closure — backtrack and try tuple or parenthesized expr
                        self.pos = saved;
                        let first = self.parse_expr()?;
                        if self.at(TokenKind::Comma) {
                            let mut elements = vec![first];
                            while self.at(TokenKind::Comma) {
                                self.advance();
                                elements.push(self.parse_expr()?);
                            }
                            self.expect(TokenKind::RParen)?;
                            Expr::Tuple { elements, span: self.span_from(start) }
                        } else {
                            self.expect(TokenKind::RParen)?;
                            first
                        }
                    }
                } else {
                    // Not a closure — try tuple or parenthesized expr
                    self.pos = saved;
                    let first = self.parse_expr()?;
                    if self.at(TokenKind::Comma) {
                        let mut elements = vec![first];
                        while self.at(TokenKind::Comma) {
                            self.advance();
                            elements.push(self.parse_expr()?);
                        }
                        self.expect(TokenKind::RParen)?;
                        Expr::Tuple { elements, span: self.span_from(start) }
                    } else {
                        self.expect(TokenKind::RParen)?;
                        first
                    }
                }
            }
            TokenKind::LBracket => {
                self.advance();
                let mut elements = Vec::new();
                while !self.at(TokenKind::RBracket) && !self.at(TokenKind::Eof) {
                    if self.at(TokenKind::DotDotDot) {
                        let span_start = self.pos;
                        self.advance();
                        let expr = self.parse_expr()?;
                        elements.push(Expr::Spread { expression: Box::new(expr), span: self.span_from(span_start) });
                    } else {
                        elements.push(self.parse_expr()?);
                    }
                    if self.at(TokenKind::Comma) { self.advance(); }
                }
                self.expect(TokenKind::RBracket)?;
                Expr::List { elements, span: self.span_from(start) }
            }
            TokenKind::LBrace => {
                self.advance();
                let mut entries = Vec::new();
                while !self.at(TokenKind::RBrace) && !self.at(TokenKind::Eof) {
                    let key = if let Ok(tok) = self.current() {
                        match &tok.kind {
                            TokenKind::String(s) => { let val = s.clone(); self.advance(); val }
                            _ => self.eat_identifier()
                        }
                    } else { String::new() };
                    self.expect(TokenKind::Colon)?;
                    let value = self.parse_expr()?;
                    entries.push((key, value));
                    if self.at(TokenKind::Comma) { self.advance(); }
                }
                self.expect(TokenKind::RBrace)?;
                Expr::Dictionary { entries, span: self.span_from(start) }
            }
            TokenKind::Async => {
                self.advance();
                let expr = self.parse_expr()?;
                Expr::Async { expression: Box::new(expr), span: self.span_from(start) }
            }
            TokenKind::Match => {
                return self.parse_match_expr();
            }
            TokenKind::Super => {
                self.advance();
                // V1: super is an alias for this (parent resolution not yet implemented)
                Expr::Identifier { name: "this".to_string(), span: self.span_from(start) }
            }
            _ => return Err(format!("unexpected token in expression: {:?}", tok.kind)),
        };
        self.parse_postfix(expr)
    }

    fn parse_postfix(&mut self, mut expr: Expr) -> Result<Expr, String> {
        loop {
            // Generic type args on identifiers: Name<T>(args) or Name<T>{ ... }
            if self.at(TokenKind::Less) && matches!(&expr, Expr::Identifier { .. }) {
                let saved = self.pos;
                self.advance();
                if let Ok(first_arg) = self.parse_type() {
                    let mut type_args = vec![first_arg];
                    while self.at(TokenKind::Comma) {
                        self.advance();
                        if let Ok(t) = self.parse_type() { type_args.push(t); } else { break; }
                    }
                    if self.at(TokenKind::Greater) {
                        self.advance();
                        let start = self.pos;
                        if self.at(TokenKind::LParen) {
                            // Function call with type args: identity<i32>(42)
                            self.advance();
                            let mut arguments = Vec::new();
                            while !self.at(TokenKind::RParen) && !self.at(TokenKind::Eof) {
                                arguments.push(self.parse_expr()?);
                                if self.at(TokenKind::Comma) { self.advance(); }
                            }
                            self.expect(TokenKind::RParen)?;
                            expr = Expr::FunctionCall {
                                target: Box::new(expr), arguments, type_args, span: self.span_from(start),
                            };
                        } else if self.at(TokenKind::LBrace) {
                            // Struct literal with type args: Container<i32>{ ... }
                            self.advance();
                            let mut fields = Vec::new();
                            while !self.at(TokenKind::RBrace) && !self.at(TokenKind::Eof) {
                                let key = self.eat_identifier();
                                self.expect(TokenKind::Colon)?;
                                let value = self.parse_expr()?;
                                fields.push((key, value));
                                if self.at(TokenKind::Comma) { self.advance(); }
                            }
                            self.expect(TokenKind::RBrace)?;
                            expr = Expr::StructLiteral {
                                struct_name: if let Expr::Identifier { name, .. } = &expr { name.clone() } else { String::new() },
                                type_args, fields, span: self.span_from(start),
                            };
                        } else {
                            // Not followed by ( or { — backtrack
                            self.pos = saved;
                        }
                        continue; // handled, go to next postfix iteration
                    }
                }
                self.pos = saved;
            }
            if self.at(TokenKind::LParen) {
                let start = self.pos;
                self.advance();
                let mut arguments = Vec::new();
                while !self.at(TokenKind::RParen) && !self.at(TokenKind::Eof) {
                    arguments.push(self.parse_expr()?);
                    if self.at(TokenKind::Comma) { self.advance(); }
                }
                self.expect(TokenKind::RParen)?;
                expr = Expr::FunctionCall { target: Box::new(expr), arguments, type_args: vec![], span: self.span_from(start) };
            } else if self.at(TokenKind::Dot) {
                let start = self.pos;
                self.advance();
                let property = self.eat_identifier();
                expr = Expr::PropertyAccess { object: Box::new(expr), property, span: self.span_from(start) };
            } else if self.at(TokenKind::QuestionDot) {
                let start = self.pos;
                self.advance();
                let property = self.eat_identifier();
                expr = Expr::OptionalChain { target: Box::new(expr), property, span: self.span_from(start) };
            } else if self.at(TokenKind::LBracket) {
                let start = self.pos;
                self.advance();
                let index = self.parse_expr()?;
                self.expect(TokenKind::RBracket)?;
                // Detect range expression inside brackets: items[start..end] → RangeSlice
                if let Expr::Binary { left, operator: BinaryOp::Range, right, .. } = &index {
                    expr = Expr::RangeSlice {
                        target: Box::new(expr),
                        start: Some(left.clone()),
                        end: Some(right.clone()),
                        span: self.span_from(start),
                    };
                } else if let Expr::Binary { left, operator: BinaryOp::RangeInclusive, right, .. } = &index {
                    expr = Expr::RangeSlice {
                        target: Box::new(expr),
                        start: Some(left.clone()),
                        end: Some(right.clone()),
                        span: self.span_from(start),
                    };
                } else if let Expr::Binary { left, operator: BinaryOp::RangeExclusive, right, .. } = &index {
                    expr = Expr::RangeSlice {
                        target: Box::new(expr),
                        start: Some(left.clone()),
                        end: Some(right.clone()),
                        span: self.span_from(start),
                    };
                } else {
                    expr = Expr::Index { target: Box::new(expr), index: Box::new(index), span: self.span_from(start) };
                }
            } else if self.at(TokenKind::LBrace) {
                // Struct literal (no generics): Identifier { field: value, ... }
                let start = self.pos;
                if let Expr::Identifier { name: struct_name, .. } = &expr {
                    let sname = struct_name.clone();
                    self.advance(); // consume '{'
                    let mut fields = Vec::new();
                    while !self.at(TokenKind::RBrace) && !self.at(TokenKind::Eof) {
                        let key = self.eat_identifier();
                        self.expect(TokenKind::Colon)?;
                        let value = self.parse_expr()?;
                        fields.push((key, value));
                        if self.at(TokenKind::Comma) { self.advance(); }
                    }
                    self.expect(TokenKind::RBrace)?;
                    expr = Expr::StructLiteral { struct_name: sname, type_args: vec![], fields, span: self.span_from(start) };
                } else {
                    break;
                }
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
        let start = self.pos;
        let single_line = !self.at(TokenKind::Newline);
        if self.at(TokenKind::Newline) { self.advance(); }
        if self.at(TokenKind::Indent) { self.advance(); }
        let mut statements = Vec::new();
        let mut had_error = false;
        loop {
            if self.at(TokenKind::Dedent) { self.advance(); break; }
            if self.at(TokenKind::Eof) { break; }
            if self.at(TokenKind::Newline) { self.advance(); continue; }
            match self.parse_stmt() {
                Ok(stmt) => statements.push(stmt),
                Err(msg) => {
                    had_error = true;
                    self.error(msg);
                    if !single_line {
                        self.sync_to_stmt_boundary();
                    }
                }
            }
            if single_line { break; }
        }
        if had_error && !single_line {
            // After recovering within the block, return an empty block — the
            // caller will see the error but we've already consumed all tokens
            // up to the dedent/eof. Return the partially-parsed block anyway
            // so the outer level can continue.
            return Ok(Block { statements, span: self.span_from(start) });
        }
        if single_line {
            if had_error {
                // Single-line body: can't recover, propagate upward
                return Err(self.errors.join("\n"));
            }
            while self.at(TokenKind::Newline) {
                self.advance();
            }
        }
        Ok(Block { statements, span: self.span_from(start) })
    }

    fn parse_stmt(&mut self) -> Result<Stmt, String> {
        // Check for labeled loops: `label: for/while/loop`
        let label = self.peek_loop_label();
        if label.is_some() {
            self.advance(); // consume identifier
            self.advance(); // consume ':'
        }
        if self.at(TokenKind::If) { return self.parse_if(); }
        if self.at(TokenKind::While) { return self.parse_while(label); }
        if self.at(TokenKind::For) { return self.parse_for(label); }
        if self.at(TokenKind::Match) { return self.parse_match(); }
        if self.at(TokenKind::Loop) {
            let start = self.pos;
            self.advance();
            self.expect(TokenKind::Colon)?;
            let body = self.parse_block()?;
            return Ok(Stmt::Expression(Expr::Loop { body, label, span: self.span_from(start) }));
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
            let value = if self.at(TokenKind::At) {
                None
            } else if self.current_is_expr_start() {
                Some(Box::new(self.parse_expr()?))
            } else { None };
            let label = if self.at(TokenKind::At) {
                self.advance();
                Some(self.eat_identifier())
            } else { None };
            return Ok(Stmt::Break(value, label));
        }
        if self.at(TokenKind::Continue) {
            self.advance();
            let label = if self.at(TokenKind::At) {
                self.advance();
                Some(self.eat_identifier())
            } else { None };
            return Ok(Stmt::Continue(label));
        }
        if self.at(TokenKind::Defer) {
            let start = self.pos;
            self.advance();
            let call = self.parse_expr()?;
            return Ok(Stmt::Defer(DeferStmt { call: Box::new(call), span: self.span_from(start) }));
        }
        if self.at(TokenKind::Guard) {
            let start = self.pos;
            self.advance();
            let condition = self.parse_expr()?;
            self.expect_keyword("else")?;
            self.expect(TokenKind::Colon)?;
            let body = self.parse_block()?;
            return Ok(Stmt::Guard(GuardStmt { condition: Box::new(condition), body, span: self.span_from(start) }));
        }
        if self.at(TokenKind::Unsafe) {
            let start = self.pos;
            self.advance();
            self.expect(TokenKind::Colon)?;
            let body = self.parse_block()?;
            return Ok(Stmt::Unsafe(UnsafeBlock { body, span: self.span_from(start) }));
        }
        // Variable declaration: `ident [":" type] "=" expr` or `ident ":" "&" type "=" expr` (mutable)
        // Handle typed declarations first: `ident : type = expr`
        if self.at_identifier() && self.tokens.get(self.pos + 1).map_or(false, |t| t.is(&TokenKind::Colon)) {
            let start = self.pos;
            let name = self.eat_identifier();
            self.advance(); // ':'
            let type_ = self.parse_type()?;
            // Check if the type starts with `&` → mutable variable
            let is_mutable = matches!(type_, AstType::Mutable { .. });
            if self.at(TokenKind::Equals) {
                self.advance();
                let value = self.parse_expr()?;
                return Ok(Stmt::Variable(VariableDecl {
                    name, type_: Some(type_), value: Box::new(value), is_mutable, span: self.span_from(start),
                }));
            }
            return Err("expected '=' after typed declaration".to_string());
        }
        // Constant declaration: `NAME := expr` (unambiguous)
        if self.at_identifier() && self.tokens.get(self.pos + 1).map_or(false, |t| t.is(&TokenKind::Walrus)) {
            let start = self.pos;
            let name = self.eat_identifier();
            self.advance(); // consume :=
            let value = self.parse_expr()?;
            return Ok(Stmt::Variable(VariableDecl {
                name, type_: None, value: Box::new(value), is_mutable: false, span: self.span_from(start),
            }));
        }
        // Destructuring: `(x, y) = expr` or `(x, y) := expr`
        if self.peek_destructure() {
            return self.parse_destructure();
        }
        // Binding-if or assignment: `ident = expr [: block]`
        if self.at_identifier() && self.peek_equals() {
            let start = self.pos;
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
                    name, value: Box::new(value), body, else_branch, span: self.span_from(start),
                }))
            } else {
                Ok(Stmt::Expression(Expr::Assignment {
                    target: Box::new(Expr::Identifier { name, span: self.span_from(start) }),
                    operator: None,
                    value: Box::new(value),
                    span: self.span_from(start),
                }))
            }
        } else {
            self.parse_expr_stmt()
        }
    }

    fn parse_if(&mut self) -> Result<Stmt, String> {
        let start = self.pos;
        self.advance();
        // `if name = expr:` — BindingIf
        if self.peek_equals() {
            return self.parse_binding_if();
        }
        let condition = self.parse_expr()?;
        self.expect(TokenKind::Colon)?;
        let body = self.parse_block()?;
        while self.at(TokenKind::Newline) { self.advance(); }
        let mut elif_branches = Vec::new();
        let mut else_branch = None;
        while self.at(TokenKind::Elif) {
            let elif_start = self.pos;
            self.advance();
            let cond = self.parse_expr()?;
            self.expect(TokenKind::Colon)?;
            let body = self.parse_block()?;
            while self.at(TokenKind::Newline) { self.advance(); }
            elif_branches.push(ElifBranch {
                condition: Box::new(cond),
                body,
                span: self.span_from(elif_start),
            });
        }
        if self.at(TokenKind::Else) {
            self.advance();
            self.expect(TokenKind::Colon)?;
            else_branch = Some(self.parse_block()?);
        }
        Ok(Stmt::If(IfStmt { condition: Box::new(condition), body, elif_branches, else_branch, span: self.span_from(start) }))
    }

    fn parse_binding_if(&mut self) -> Result<Stmt, String> {
        let start = self.pos;
        let name = self.eat_identifier();
        self.expect(TokenKind::Equals)?;
        let value = self.parse_expr()?;
        self.expect(TokenKind::Colon)?;
        let body = self.parse_block()?;
        let else_branch = if self.at(TokenKind::Else) {
            self.advance();
            self.expect(TokenKind::Colon)?;
            Some(self.parse_block()?)
        } else { None };
        Ok(Stmt::BindingIf(BindingIf {
            name, value: Box::new(value), body, else_branch, span: self.span_from(start),
        }))
    }

    fn parse_while(&mut self, label: Option<String>) -> Result<Stmt, String> {
        let start = self.pos;
        self.advance();
        let condition = self.parse_expr()?;
        self.expect(TokenKind::Colon)?;
        let body = self.parse_block()?;
        let else_branch = if self.at(TokenKind::Else) {
            self.advance();
            self.expect(TokenKind::Colon)?;
            Some(self.parse_block()?)
        } else { None };
        Ok(Stmt::While(WhileStmt { condition: Box::new(condition), body, else_branch, label, span: self.span_from(start) }))
    }

    fn parse_for(&mut self, label: Option<String>) -> Result<Stmt, String> {
        let start = self.pos;
        self.advance();
        let variable = self.eat_identifier();
        self.expect_keyword("in")?;
        let iterable = self.parse_expr()?;
        self.expect(TokenKind::Colon)?;
        let body = self.parse_block()?;
        let else_branch = if self.at(TokenKind::Else) {
            self.advance();
            self.expect(TokenKind::Colon)?;
            Some(self.parse_block()?)
        } else { None };
        Ok(Stmt::For(ForStmt { variable, iterable: Box::new(iterable), body, else_branch, label, span: self.span_from(start) }))
    }

    fn parse_match(&mut self) -> Result<Stmt, String> {
        let start = self.pos;
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
            let arm_start = self.pos;
            let pattern = self.parse_pattern_or()?;
            let guard = if self.at(TokenKind::If) {
                self.advance();
                Some(Box::new(self.parse_expr()?))
            } else { None };
            self.expect(TokenKind::Colon)?;
            let body = self.parse_block()?;
            arms.push(MatchArm { pattern, guard, body, span: self.span_from(arm_start) });
        }
        Ok(Stmt::Match(MatchStmt { expression: Box::new(expression), arms, span: self.span_from(start) }))
    }

    fn parse_match_expr(&mut self) -> Result<Expr, String> {
        let start = self.pos;
        self.advance(); // consume 'match'
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
            let arm_start = self.pos;
            let pattern = self.parse_pattern_or()?;
            let guard = if self.at(TokenKind::If) {
                self.advance();
                Some(Box::new(self.parse_expr()?))
            } else { None };
            self.expect(TokenKind::Colon)?;
            let body = self.parse_block()?;
            arms.push(MatchArm { pattern, guard, body, span: self.span_from(arm_start) });
        }
        Ok(Expr::MatchExpr { expression: Box::new(expression), arms, span: self.span_from(start) })
    }

    fn parse_pattern(&mut self) -> Result<Pattern, String> {
        let start = self.pos;
        // `is Type:` — type test pattern
        if self.at(TokenKind::Is) {
            self.advance();
            let type_ = self.parse_type()?;
            return Ok(Pattern::IsType { type_, span: self.span_from(start) });
        }
        if self.at_identifier() {
            let name = self.eat_identifier();
            // Check for enum variant pattern: Option.Some(v)
            if self.at(TokenKind::Dot) {
                self.advance();
                let variant = self.eat_identifier();
                let args = if self.at(TokenKind::LParen) {
                    self.advance();
                    let mut patterns = Vec::new();
                    loop {
                        patterns.push(self.parse_pattern()?);
                        if self.at(TokenKind::Comma) { self.advance(); }
                        else { break; }
                    }
                    self.expect(TokenKind::RParen)?;
                    patterns
                } else {
                    vec![]
                };
                return Ok(Pattern::EnumVariant {
                    enum_name: name,
                    variant,
                    args,
                    span: self.span_from(start),
                });
            }
            return Ok(Pattern::Identifier { name, span: self.span_from(start) });
        }
        // Tuple pattern: (p1, p2, ...)
        if self.at(TokenKind::LParen) {
            self.advance();
            let mut elements = Vec::new();
            loop {
                elements.push(self.parse_pattern_or()?);
                if self.at(TokenKind::Comma) { self.advance(); }
                else { break; }
            }
            self.expect(TokenKind::RParen)?;
            return Ok(Pattern::Tuple { elements, span: self.span_from(start) });
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
            TokenKind::None => Literal::None,
            _ => {
                let found = format!("{:?}", self.current()?.kind);
                return Err(format!("expected pattern, found {}", found));
            }
        };
        self.advance();
        Ok(Pattern::Literal {
            value: lit,
            span: self.span_from(start),
        })
    }

    /// Parse a pattern, possibly with `|` for or-patterns.
    fn parse_pattern_or(&mut self) -> Result<Pattern, String> {
        let start = self.pos;
        let mut patterns = vec![self.parse_pattern()?];
        while self.at(TokenKind::Pipe) {
            self.advance();
            patterns.push(self.parse_pattern()?);
        }
        if patterns.len() == 1 {
            Ok(patterns.into_iter().next().unwrap())
        } else {
            Ok(Pattern::Or { patterns, span: self.span_from(start) })
        }
    }

    /// Parse string interpolation: split `"Hello {name}"` into parts.
    fn parse_string_interp(&mut self, s: &str, start: usize) -> Result<Expr, String> {
        let mut parts: Vec<Expr> = Vec::new();
        let mut current = String::new();
        let chars: Vec<char> = s.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            if chars[i] == '\\' && i + 1 < chars.len() && (chars[i + 1] == '{' || chars[i + 1] == '}') {
                current.push(chars[i + 1]);
                i += 2;
            } else if chars[i] == '{' {
                if !current.is_empty() {
                    parts.push(Expr::Literal {
                        value: Literal::String(std::mem::take(&mut current)),
                        span: self.span_from(start),
                    });
                }
                i += 1;
                let expr_start = i;
                let mut depth = 1;
                while i < chars.len() && depth > 0 {
                    if chars[i] == '{' { depth += 1; }
                    else if chars[i] == '}' { depth -= 1; }
                    if depth > 0 { i += 1; }
                }
                let expr_text: String = chars[expr_start..i].iter().collect();
                let expr = self.parse_interp_expr_text(&expr_text)?;
                parts.push(expr);
                i += 1;
            } else {
                current.push(chars[i]);
                i += 1;
            }
        }

        if !current.is_empty() {
            parts.push(Expr::Literal {
                value: Literal::String(current),
                span: self.span_from(start),
            });
        }

        if parts.is_empty() {
            parts.push(Expr::Literal {
                value: Literal::String(String::new()),
                span: self.span_from(start),
            });
        }

        // If only one part and it's a string literal, return a plain literal (no interpolation)
        if parts.len() == 1 {
            if let Expr::Literal { .. } = &parts[0] {
                return Ok(parts.remove(0));
            }
        }

        Ok(Expr::StringInterp { parts, span: self.span_from(start) })
    }

    /// Parse an expression from raw text (used inside `{...}` in string interpolation).
    fn parse_interp_expr_text(&self, text: &str) -> Result<Expr, String> {
        use crate::lexer::Lexer;
        let mut lexer = Lexer::new(text);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        parser.parse_expr()
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

    /// Returns true if the current position looks like a destructuring assignment:
    /// `(ident, ...) =|:= expr`
    fn peek_destructure(&self) -> bool {
        if !self.at(TokenKind::LParen) { return false; }
        if self.tokens.get(self.pos + 1).map_or(false, |t| matches!(&t.kind, TokenKind::Identifier(_)))
            && self.tokens.get(self.pos + 2).map_or(false, |t| t.is(&TokenKind::Comma))
        {
            let mut depth = 1;
            let mut i = self.pos + 1;
            while i < self.tokens.len() && depth > 0 {
                if self.tokens[i].is(&TokenKind::LParen) { depth += 1; }
                else if self.tokens[i].is(&TokenKind::RParen) { depth -= 1; }
                i += 1;
            }
            if depth == 0 && i < self.tokens.len() {
                let next = &self.tokens[i];
                return next.is(&TokenKind::Equals) || next.is(&TokenKind::Walrus);
            }
        }
        false
    }

    /// Parse destructuring: `(x, y) = expr` or `(x, y) := expr`
    fn parse_destructure(&mut self) -> Result<Stmt, String> {
        let start = self.pos;
        self.advance(); // '('
        let mut elements = Vec::new();
        loop {
            let name = self.eat_identifier();
            if name.is_empty() {
                return Err("expected identifier in destructuring pattern".to_string());
            }
            elements.push(Expr::Identifier { name, span: self.span_from(start) });
            if self.at(TokenKind::Comma) { self.advance(); }
            else { break; }
        }
        self.expect(TokenKind::RParen)?;
        let is_mutable = self.at(TokenKind::Walrus);
        if self.at(TokenKind::Equals) || is_mutable {
            self.advance();
            let value = self.parse_expr()?;
            Ok(Stmt::Expression(Expr::Assignment {
                target: Box::new(Expr::Tuple { elements, span: self.span_from(start) }),
                operator: None,
                value: Box::new(value),
                span: self.span_from(start),
            }))
        } else {
            Err("expected '=' or ':=' after destructuring pattern".to_string())
        }
    }

    /// Check if the current position has a label before a loop keyword.
    /// Returns the label name if found, or None.
    fn peek_loop_label(&self) -> Option<String> {
        // Pattern: `identifier : for`, `identifier : while`, or `identifier : loop`
        if !self.at_identifier() { return None; }
        if !self.tokens.get(self.pos + 1).map_or(false, |t| t.is(&TokenKind::Colon)) { return None; }
        let after_colon = self.tokens.get(self.pos + 2)?;
        let is_loop = after_colon.is(&TokenKind::For)
            || after_colon.is(&TokenKind::While)
            || after_colon.is(&TokenKind::Loop);
        if is_loop {
            if let Ok(tok) = self.current() {
                if let TokenKind::Identifier(name) = &tok.kind {
                    return Some(name.clone());
                }
            }
        }
        None
    }

    /// Returns true if the next token is `:=` (Walrus) — constant declaration.
    fn peek_walrus(&self) -> bool {
        self.tokens.get(self.pos).map_or(false, |t| t.is(&TokenKind::Walrus))
    }

    /// Returns the kind of declaration operator at pos+1: Equals or Walrus.
    fn peek_decl_op(&self) -> Option<TokenKind> {
        self.tokens.get(self.pos + 1).and_then(|t| {
            if t.is(&TokenKind::Equals) { Some(TokenKind::Equals) }
            else if t.is(&TokenKind::Walrus) { Some(TokenKind::Walrus) }
            else { None }
        })
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
    /// Also accepts soft keywords (get, set) and common value keywords (None, True, False) as identifiers.
    fn eat_identifier(&mut self) -> String {
        if let Ok(tok) = self.current() {
            let name = match &tok.kind {
                TokenKind::Identifier(n) => Some(n.clone()),
                TokenKind::Get => Some("get".to_string()),
                TokenKind::Set => Some("set".to_string()),
                TokenKind::None => Some("None".to_string()),
                TokenKind::True => Some("True".to_string()),
                TokenKind::False => Some("False".to_string()),
                _ => None,
            };
            if let Some(n) = name {
                self.advance();
                return n;
            }
        }
        String::new()
    }

    /// Try to parse closure parameter list: zero or more comma-separated identifiers.
    /// Supports optional type annotations: `name: Type` (type is parsed but discarded for V1).
    /// Advancement stops BEFORE the closing `)` (if any).
    fn parse_closure_params(&mut self) -> Vec<String> {
        let mut params = Vec::new();
        if self.at(TokenKind::RParen) { return params; }
        loop {
            let name = self.eat_identifier();
            if name.is_empty() { break; }
            // Skip optional type annotation: `name: Type` (discarded for V1)
            if self.at(TokenKind::Colon) {
                self.advance();
                let _ = self.parse_type();
            }
            params.push(name);
            if self.at(TokenKind::Comma) { self.advance(); }
            else { break; }
        }
        params
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
            TokenKind::And => Some(BinaryOp::And),
            TokenKind::Or => Some(BinaryOp::Or),
            TokenKind::DotDot => Some(BinaryOp::Range),
            TokenKind::DotDotEquals => Some(BinaryOp::RangeInclusive),
            TokenKind::DotDotLess => Some(BinaryOp::RangeExclusive),
            TokenKind::Is => Some(BinaryOp::Is),
            TokenKind::As => Some(BinaryOp::As),
            _ => Option::None,
        }
    }

    fn operator_precedence(&self, op: &BinaryOp) -> u8 {
        match op {
            BinaryOp::Assign | BinaryOp::AddAssign | BinaryOp::SubAssign
            | BinaryOp::MulAssign | BinaryOp::DivAssign | BinaryOp::RemAssign
            | BinaryOp::BitAndAssign | BinaryOp::BitOrAssign | BinaryOp::BitXorAssign
            | BinaryOp::ShlAssign | BinaryOp::ShrAssign => 1,
            BinaryOp::Range | BinaryOp::RangeInclusive | BinaryOp::RangeExclusive => 2,
            BinaryOp::Or => 2,
            BinaryOp::And => 3,
            BinaryOp::Eq | BinaryOp::Neq => 4,
            BinaryOp::Lt | BinaryOp::Gt | BinaryOp::Le | BinaryOp::Ge | BinaryOp::Is => 5,
            BinaryOp::BitOr => 6,
            BinaryOp::BitXor => 7,
            BinaryOp::BitAnd => 8,
            BinaryOp::Shl | BinaryOp::Shr => 9,
            BinaryOp::Add | BinaryOp::Sub | BinaryOp::AddPercent | BinaryOp::SubPercent => 10,
            BinaryOp::Mul | BinaryOp::Div | BinaryOp::Rem | BinaryOp::MulPercent => 11,
            BinaryOp::Pow => 12,
            BinaryOp::As => 13,
        }
    }

    fn current_is_expr_start(&self) -> bool {
        self.current().map_or(false, |t| match t.kind {
            TokenKind::Integer(_) | TokenKind::Float(_) | TokenKind::String(_)
            | TokenKind::True | TokenKind::False | TokenKind::Identifier(_)
            | TokenKind::LParen | TokenKind::LBracket | TokenKind::LBrace
            | TokenKind::Minus | TokenKind::Bang | TokenKind::Tilde
            | TokenKind::Async | TokenKind::Match => true,
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
        let p = parse("fn add(x: i32, y: i32) i32:\n    x + y\n").unwrap();
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
    fn speak() str
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
    fn test_destructure_assign() {
        let source = "fn test():\n    (x, y) = (1, 2)\n    print(x)\n";
        let result = parse(source);
        match &result {
            Ok(_) => {},
            Err(e) => panic!("destructure assign parse failed: {}", e),
        }
    }

    #[test]
    fn test_destructure_mut() {
        let source = "fn test():\n    (x, y) := (1, 2)\n";
        let result = parse(source);
        match &result {
            Ok(_) => {},
            Err(e) => panic!("destructure mut parse failed: {}", e),
        }
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
        let source = "fn add(a: i32, b: i32) i32:\n    a + b\n";
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

    #[test]
    fn test_dict_multi_entry() {
        let source = "\
fn test():\n\
    config = {name: \"Alice\", age: 30}\n";
        assert!(parse(source).is_ok());
    }

    #[test]
    fn test_ternary_expression() {
        let source = "\
fn test():\n\
    result = x > 0 ? \"pos\" : \"neg\"\n";
        assert!(parse(source).is_ok());
    }

    #[test]
    fn test_match_expression() {
        let source = "fn test():\n    result = match value:\n        1:\n            \"one\"\n        2:\n            \"two\"\n";
        assert!(parse(source).is_ok());
    }

    #[test]
    fn test_or_pattern_match() {
        let source = "fn f():\n    match 1:\n        1 | 2:\n            0\n";
        match parse(source) {
            Ok(p) => assert!(p.declarations.len() >= 1, "expected 1+ decls, got {}", p.declarations.len()),
            Err(e) => panic!("Parse error: {}", e),
        }
    }

    #[test]
    fn test_or_pattern_match_expr() {
        let source = "fn f():\n    result = match 1:\n        1 | 2:\n            0\n";
        match parse(source) {
            Ok(p) => assert!(p.declarations.len() >= 1, "expected 1+ decls, got {}", p.declarations.len()),
            Err(e) => panic!("Parse error: {}", e),
        }
    }

    #[test]
    fn test_optional_chain() {
        let source = "\
fn test():\n\
    name = user?.name\n";
        assert!(parse(source).is_ok());
    }

    #[test]
    fn test_spread_list() {
        let source = "\
fn test():\n\
    items = [...a, 4, 5]\n";
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
        let result = parse("fn f() :\n    x\n");
        assert!(result.is_ok());
    }

    #[test]
    fn test_error_unexpected_declaration() {
        let result = parse("if x:\n    y\n");
        assert!(result.is_err());
    }

    // -----------------------------------------------------------------------
    // Integration — large file smoke test
    // -----------------------------------------------------------------------

    #[test]
    fn test_large_file() {
        let source = r##"
## Large synthetic test — 200 functions to stress the parser
"##.to_string();
        let mut body = String::new();
        for i in 0..200 {
            body.push_str(&format!("fn fn_{}(x: i32) i32:\n    return x + {}\n\n", i, i));
        }
        body.push_str("fn main() i32:\n    return fn_0(0)\n");
        let src = format!("{}{}", source, body);
        if let Err(e) = parse(&src) {
            panic!("Large file parse error: {}", e);
        }
    }
}
