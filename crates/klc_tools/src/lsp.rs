use std::collections::HashMap;
use lsp_server::{Connection, Message, Notification, Request, Response};
use lsp_types::*;
use klc_core::ast::*;
use klc_core::span::Span;
use klc_core::types::Type;
use klc_frontend::token::TokenKind;
use klc_semantic::symbol_table::{Symbol, SymKind, SymbolTable};

/// KL Language Server — provides diagnostics, completion, hover, and
/// go-to-definition via the Language Server Protocol.
pub struct LanguageServer {
    connection: Connection,
    sources: HashMap<String, String>,
}

impl LanguageServer {
    pub fn new() -> Result<Self, String> {
        let (connection, io_threads) = Connection::stdio();
        let server = Self {
            connection,
            sources: HashMap::new(),
        };
        std::mem::forget(io_threads);
        Ok(server)
    }

    pub fn run(&mut self) -> Result<(), String> {
        let caps = self.server_capabilities();
        let init_params = serde_json::to_value(&caps).unwrap();
        let _init = self.connection.initialize(init_params)
            .map_err(|e| format!("initialize error: {}", e))?;

        loop {
            let msg = self.connection.receiver.recv()
                .map_err(|e| format!("recv error: {}", e))?;
            match msg {
                Message::Request(req) => {
                    if self.connection.handle_shutdown(&req).unwrap_or(false) {
                        return Ok(());
                    }
                    self.handle_request(req);
                }
                Message::Notification(not) => {
                    self.handle_notification(not);
                }
                Message::Response(_) => {}
            }
        }
    }

    fn server_capabilities(&self) -> ServerCapabilities {
        ServerCapabilities {
            text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
            completion_provider: Some(CompletionOptions {
                trigger_characters: Some(vec![".".to_string()]),
                ..Default::default()
            }),
            hover_provider: Some(HoverProviderCapability::Simple(true)),
            definition_provider: Some(OneOf::Left(true)),
            references_provider: Some(OneOf::Left(true)),
            document_symbol_provider: Some(OneOf::Left(true)),
            workspace_symbol_provider: Some(OneOf::Left(true)),
            signature_help_provider: Some(SignatureHelpOptions {
                trigger_characters: Some(vec!["(".to_string(), ",".to_string()]),
                ..Default::default()
            }),
            code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
            document_formatting_provider: Some(OneOf::Left(true)),
            ..Default::default()
        }
    }

    fn handle_request(&mut self, req: Request) {
        match req.method.as_str() {
            "textDocument/completion" => self.handle_completion(req),
            "textDocument/hover" => self.handle_hover(req),
            "textDocument/definition" => self.handle_definition(req),
            "textDocument/references" => self.handle_references(req),
            "textDocument/documentSymbol" => self.handle_document_symbol(req),
            "workspace/symbol" => self.handle_workspace_symbol(req),
            "textDocument/signatureHelp" => self.handle_signature_help(req),
            "textDocument/codeAction" => self.handle_code_action(req),
            "textDocument/formatting" => self.handle_formatting(req),
            _ => {
                let resp = Response::new_err(
                    req.id,
                    lsp_server::ErrorCode::MethodNotFound as i32,
                    format!("unknown request: {}", req.method),
                );
                let _ = self.connection.sender.send(Message::Response(resp));
            }
        }
    }

    fn handle_notification(&mut self, not: Notification) {
        match not.method.as_str() {
            "textDocument/didOpen" => self.handle_did_open(not),
            "textDocument/didChange" => self.handle_did_change(not),
            _ => {}
        }
    }

    fn handle_did_open(&mut self, not: Notification) {
        let params: DidOpenTextDocumentParams = serde_json::from_value(not.params).unwrap();
        let uri = params.text_document.uri.to_string();
        self.sources.insert(uri.clone(), params.text_document.text);
        self.publish_diagnostics(&uri);
    }

    fn handle_did_change(&mut self, not: Notification) {
        let params: DidChangeTextDocumentParams = serde_json::from_value(not.params).unwrap();
        let uri = params.text_document.uri.to_string();
        if let Some(change) = params.content_changes.into_iter().last() {
            self.sources.insert(uri.clone(), change.text);
        }
        self.publish_diagnostics(&uri);
    }

    fn publish_diagnostics(&self, uri: &str) {
        let source = match self.sources.get(uri) {
            Some(s) => s.clone(),
            None => return,
        };

        let mut lexer = klc_frontend::lexer::Lexer::new(&source);
        let tokens = lexer.tokenize();
        let mut parser = klc_frontend::parser::Parser::new(tokens);
        let mut lsp_diags = Vec::new();

        match parser.parse() {
            Ok(program) => {
                let file_name = uri.trim_start_matches("file://");
                let mut source_map = klc_core::source_map::SourceMap::new();
                let _file_id = source_map.add(file_name.to_string(), source);
                let mut analyzer = klc_semantic::analyzer::SemanticAnalyzer::new()
                    .with_source(source_map, file_name.to_string());
                analyzer.analyze(&program);

                for diag in analyzer.reporter().diagnostics() {
                    let range = diag.span
                        .as_ref()
                        .map(span_to_range)
                        .unwrap_or(Range {
                            start: Position { line: 0, character: 0 },
                            end: Position { line: 0, character: 0 },
                        });
                    let severity = match diag.severity {
                        klc_core::diagnostic::Severity::Error => Some(DiagnosticSeverity::ERROR),
                        klc_core::diagnostic::Severity::Warning => Some(DiagnosticSeverity::WARNING),
                        klc_core::diagnostic::Severity::Note => Some(DiagnosticSeverity::INFORMATION),
                        klc_core::diagnostic::Severity::Ice => Some(DiagnosticSeverity::ERROR),
                    };
                    lsp_diags.push(Diagnostic {
                        range,
                        severity,
                        source: Some("klc".to_string()),
                        message: format!("[{}] {}", diag.code, diag.message),
                        ..Default::default()
                    });
                }
            }
            Err(e) => {
                lsp_diags.push(Diagnostic {
                    range: Range {
                        start: Position { line: 0, character: 0 },
                        end: Position { line: 0, character: 0 },
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    source: Some("klc".to_string()),
                    message: format!("Parse error: {}", e),
                    ..Default::default()
                });
            }
        }

        let params = PublishDiagnosticsParams {
            uri: lsp_types::Url::parse(uri).unwrap(),
            diagnostics: lsp_diags,
            version: None,
        };
        let not = Notification::new(
            "textDocument/publishDiagnostics".to_string(),
            serde_json::to_value(params).unwrap(),
        );
        let _ = self.connection.sender.send(Message::Notification(not));
    }

    fn handle_completion(&mut self, req: Request) {
        let params: CompletionParams = serde_json::from_value(req.params).unwrap();
        let uri = params.text_document_position.text_document.uri.to_string();
        let source = self.sources.get(&uri).cloned().unwrap_or_default();
        let pos = params.text_document_position.position;

        // Check if this is a dot-triggered completion (user typed "expr.")
        let is_dot = self.is_after_dot(&source, pos.line as usize, pos.character as usize, &params);
        if is_dot {
            let expr = self.word_before_dot(&source, pos.line as usize, pos.character as usize);
            if !expr.is_empty() {
                if let Some(items) = self.dot_completions(&source, &expr, pos.line as usize, pos.character as usize) {
                    let result = CompletionResponse::Array(items);
                    let resp = Response::new_ok(req.id, serde_json::to_value(result).unwrap());
                    let _ = self.connection.sender.send(Message::Response(resp));
                    return;
                }
            }
        }

        let prefix = self.word_at(&source, pos.line as usize, pos.character as usize);
        let prefix_lower = prefix.to_lowercase();

        let mut items: Vec<CompletionItem> = Vec::new();

        // Built-in functions (28 total)
        let builtins: &[(&str, &str)] = &[
            ("print", "print(value) — Print to stdout"),
            ("println", "println(value) — Print with newline"),
            ("print_int", "print_int(n) — Print integer"),
            ("println_int", "println_int(n) — Print integer with newline"),
            ("print_err", "print_err(value) — Print to stderr"),
            ("str", "str(value) — Convert to string"),
            ("len", "len(value) — Get length"),
            ("int", "int(value) — Convert to integer"),
            ("float", "float(value) — Convert to float"),
            ("bool", "bool(value) — Convert to boolean"),
            ("input", "input() — Read line from stdin"),
            ("open", "open(path) -> i64 — Open file"),
            ("read_str", "read_str(fd) -> str — Read file content"),
            ("write_str", "write_str(fd, str) — Write to file"),
            ("close", "close(fd) — Close file"),
            ("sleep", "sleep(ms) — Sleep in milliseconds"),
            ("now", "now() -> i64 — Current timestamp"),
            ("contains", "contains(str, substr) -> bool"),
            ("to_upper", "to_upper(str) -> str"),
            ("to_lower", "to_lower(str) -> str"),
            ("trim", "trim(str) -> str"),
            ("replace", "replace(str, from, to) -> str"),
            ("substr", "substr(str, start, len) -> str"),
            ("char_at", "char_at(str, index) -> char"),
            ("ord", "ord(c) -> i32 — Char to ASCII code"),
            ("is_digit", "is_digit(c) -> bool"),
            ("is_alpha", "is_alpha(c) -> bool"),
            ("is_alnum", "is_alnum(c) -> bool"),
            ("is_whitespace", "is_whitespace(c) -> bool"),
            ("is_upper", "is_upper(c) -> bool"),
            ("is_lower", "is_lower(c) -> bool"),
            ("assert", "assert(condition)"),
            ("assert_eq", "assert_eq(a, b)"),
            ("assert_str", "assert_str(a, b)"),
            ("assert_ne", "assert_ne(a, b)"),
            ("range", "range(start, end) -> [i32]"),
            ("json_parse", "json_parse(str) -> i64"),
            ("json_stringify", "json_stringify(value) -> str"),
            ("list_push", "list_push(list, value)"),
            ("list_pop", "list_pop(list) -> i64"),
            ("list_len", "list_len(list) -> i32"),
            ("ceil", "ceil(f64) -> f64"),
            ("floor", "floor(f64) -> f64"),
            ("round", "round(f64) -> f64"),
        ];
        for (label, detail) in builtins {
            if prefix.is_empty() || label.starts_with(&prefix) || label.to_lowercase().starts_with(&prefix_lower) {
                items.push(CompletionItem {
                    label: label.to_string(),
                    kind: Some(CompletionItemKind::FUNCTION),
                    detail: Some(detail.to_string()),
                    insert_text: Some(label.to_string()),
                    sort_text: Some(format!("1{}", label)),
                    ..Default::default()
                });
            }
        }

        // Project symbols — parse source and extract all declarations
        let mut lexer = klc_frontend::lexer::Lexer::new(&source);
        let tokens = lexer.tokenize();
        let mut parser = klc_frontend::parser::Parser::new(tokens);
        if let Ok(prog) = parser.parse() {
            for decl in &prog.declarations {
                let (name, kind, detail): (String, CompletionItemKind, String) = match decl {
                    Decl::Function(f) => {
                        let params: Vec<String> = f.params.iter().map(|p| format!("{}: {}", p.name, Self::fmt_ast_type(&p.type_))).collect();
                        let sig = format!("fn {}({})", f.name, params.join(", "));
                        (f.name.clone(), CompletionItemKind::FUNCTION, sig)
                    }
                    Decl::Variable(v) => (v.name.clone(), CompletionItemKind::VARIABLE, format!("var {}", v.name)),
                    Decl::Constant(c) => (c.name.clone(), CompletionItemKind::CONSTANT, c.name.clone()),
                    Decl::Class(c) => (c.name.clone(), CompletionItemKind::CLASS, format!("class {}", c.name)),
                    Decl::Struct(s) => (s.name.clone(), CompletionItemKind::STRUCT, format!("struct {}", s.name)),
                    Decl::Enum(e) => (e.name.clone(), CompletionItemKind::ENUM, format!("enum {}", e.name)),
                    Decl::Contract(c) => (c.name.clone(), CompletionItemKind::INTERFACE, format!("contract {}", c.name)),
                    Decl::TypeAlias(t) => (t.name.clone(), CompletionItemKind::TYPE_PARAMETER, format!("type {}", t.name)),
                    Decl::AbstractClass(c) => (c.name.clone(), CompletionItemKind::CLASS, format!("abstract class {}", c.name)),
                    _ => continue,
                };
                if prefix.is_empty() || name.starts_with(&prefix) || name.to_lowercase().starts_with(&prefix_lower) {
                    items.push(CompletionItem {
                        label: name.clone(),
                        kind: Some(kind),
                        detail: Some(detail),
                        insert_text: None,
                        sort_text: Some(format!("2{}", &name)),
                        ..Default::default()
                    });
                }
            }
        }

        // Keywords
        let keywords = [
            ("if", "if condition:"),
            ("elif", "elif condition:"),
            ("else", "else:"),
            ("while", "while condition:"),
            ("for", "for item in list:"),
            ("in", "in"),
            ("match", "match value:"),
            ("return", "return value"),
            ("fn", "fn name(params):"),
            ("mut", "mut variable"),
            ("class", "class Name:"),
            ("struct", "struct Name:"),
            ("enum", "enum Name:"),
            ("contract", "contract Name:"),
            ("import", "import module"),
            ("from", "from module import name"),
            ("as", "as alias"),
            ("type", "type Alias = Type"),
            ("const", "const NAME = value"),
            ("abs", "abstract — abs class/fn"),
            ("async", "async expression"),
            ("await", "await expression"),
            ("defer", "defer expression"),
            ("guard", "guard condition"),
            ("loop", "loop:"),
            ("unsafe", "unsafe:"),
            ("break", "break"),
            ("true", "true"),
            ("false", "false"),
            ("None", "None"),
            ("ok", "ok(value)"),
            ("error", "error(message)"),
            ("this", "this.field"),
            ("next", "next (loop iteration)"),
        ];
        for (kw, detail) in &keywords {
            if prefix.is_empty() || kw.starts_with(&prefix) {
                items.push(CompletionItem {
                    label: kw.to_string(),
                    kind: Some(CompletionItemKind::KEYWORD),
                    detail: Some(detail.to_string()),
                    insert_text: None,
                    sort_text: Some(format!("3{}", kw)),
                    ..Default::default()
                });
            }
        }

        let result = CompletionResponse::Array(items);
        let resp = Response::new_ok(req.id, serde_json::to_value(result).unwrap());
        let _ = self.connection.sender.send(Message::Response(resp));
    }

    fn handle_hover(&mut self, req: Request) {
        let params: HoverParams = serde_json::from_value(req.params).unwrap();
        let uri = params.text_document_position_params.text_document.uri.to_string();
        let source = self.sources.get(&uri).cloned().unwrap_or_default();
        let pos = params.text_document_position_params.position;

        let hover_text = self.find_hover_info(&source, pos.line as usize, pos.character as usize);

        let result = Hover {
            contents: HoverContents::Scalar(MarkedString::String(hover_text)),
            range: None,
        };
        let resp = Response::new_ok(req.id, serde_json::to_value(result).unwrap());
        let _ = self.connection.sender.send(Message::Response(resp));
    }

    fn find_hover_info(&self, source: &str, line: usize, col: usize) -> String {
        let mut lexer = klc_frontend::lexer::Lexer::new(source);
        let tokens = lexer.tokenize();
        let mut parser = klc_frontend::parser::Parser::new(tokens);
        if let Ok(prog) = parser.parse() {
            for decl in &prog.declarations {
                match decl {
                    Decl::Function(f) => {
                        let l1 = f.span.start.line;
                        let l2 = f.span.end.line;
                        if (l1 <= line + 1 && line + 1 <= l2) || l1 == line + 1 {
                            let mut info = format!("**fn {}**", f.name);
                            if let Some(rt) = &f.return_type {
                                info.push_str(&format!(" -> {}", Self::fmt_ast_type(rt)));
                            }
                            return info;
                        }
                    }
                    _ => {}
                }
            }
            let word = self.word_at(source, line, col);
            match word.as_str() {
                "print" => return "`print(value)` — Print value to stdout".to_string(),
                "println" => return "`println(value)` — Print value with newline".to_string(),
                "str" => return "`str(value)` — Convert to string".to_string(),
                "len" => return "`len(value)` — Get length".to_string(),
                "int" => return "`int(value)` — Convert to integer".to_string(),
                "float" => return "`float(value)` — Convert to float".to_string(),
                "bool" => return "`bool(value)` — Convert to boolean".to_string(),
                _ => {
                    if !word.is_empty() {
                        return format!("`{}` — KL identifier", word);
                    }
                }
            }
        }
        "KL source file".to_string()
    }

    fn word_at(&self, source: &str, line: usize, col: usize) -> String {
        for (i, l) in source.lines().enumerate() {
            if i == line {
                let chars: Vec<char> = l.chars().collect();
                if col >= chars.len() || !chars[col].is_alphanumeric() && chars[col] != '_' {
                    return String::new();
                }
                let mut start = col;
                let mut end = col;
                while start > 0 && (chars[start - 1].is_alphanumeric() || chars[start - 1] == '_') {
                    start -= 1;
                }
                while end < chars.len() && (chars[end].is_alphanumeric() || chars[end] == '_') {
                    end += 1;
                }
                return chars[start..end].iter().collect();
            }
        }
        String::new()
    }

    /// Check if the cursor is right after a `.` (either triggered by the
    /// character itself or invoked manually while positioned after one).
    fn is_after_dot(&self, source: &str, line: usize, col: usize, params: &CompletionParams) -> bool {
        if let Some(ctx) = &params.context {
            if ctx.trigger_kind == CompletionTriggerKind::TRIGGER_CHARACTER
                && ctx.trigger_character.as_deref() == Some(".")
            {
                return true;
            }
        }
        // Manual invocation — check if char before cursor is '.'
        for (i, l) in source.lines().enumerate() {
            if i == line && col > 0 {
                let chars: Vec<char> = l.chars().collect();
                if col - 1 < chars.len() && chars[col - 1] == '.' {
                    return true;
                }
            }
        }
        false
    }

    /// Extract the identifier immediately before a `.` at the cursor.
    /// Cursor position (`col`) is *after* the dot, so we look at `col - 2 ..`.
    fn word_before_dot(&self, source: &str, line: usize, col: usize) -> String {
        for (i, l) in source.lines().enumerate() {
            if i == line {
                let chars: Vec<char> = l.chars().collect();
                if col == 0 {
                    return String::new();
                }
                let dot_idx = col.saturating_sub(1);
                if dot_idx >= chars.len() || chars[dot_idx] != '.' {
                    return String::new();
                }
                let mut start = dot_idx;
                while start > 0 && (chars[start - 1].is_alphanumeric() || chars[start - 1] == '_') {
                    start -= 1;
                }
                return chars[start..dot_idx].iter().collect();
            }
        }
        String::new()
    }

    /// Resolve dot completions for an expression by running the semantic
    /// pipeline and looking up the expression's type in the symbol table.
    /// `line`/`col` are the cursor position (after the dot) used to find the
    /// enclosing function for local variable resolution.
    fn dot_completions(&self, source: &str, expr: &str, line: usize, col: usize) -> Option<Vec<CompletionItem>> {
        let mut lexer = klc_frontend::lexer::Lexer::new(source);
        let tokens = lexer.tokenize();
        let mut parser = klc_frontend::parser::Parser::new(tokens);
        let program = parser.parse().ok()?;

        let mut source_map = klc_core::source_map::SourceMap::new();
        let _file_id = source_map.add("input.kl".to_string(), source.to_string());
        let mut analyzer = klc_semantic::analyzer::SemanticAnalyzer::new()
            .with_source(source_map, "input.kl".to_string());
        analyzer.analyze(&program);

        let symbols = analyzer.type_checker.symbols();

        // Build local scope for variable-to-type mapping
        let local_types = Self::build_local_scope(&program, line, col);

        // Handle chained dot completions: "user.address" → resolve base "user" → get "address" type
        let parts: Vec<&str> = expr.split('.').collect();
        if parts.len() > 1 {
            let base = parts[0];
            // Walk the chain: resolve each part's type
            let mut current_type = Self::resolve_type_name(base, symbols, &local_types);
            for i in 1..parts.len() {
                let prop = parts[i];
                // Look up `prop` as a field of `current_type`
                if let Some(ref tn) = current_type {
                    if let Some(sym) = symbols.lookup(tn) {
                        let field_type = Self::field_type_for_sym(sym, prop);
                        if let Some(ft) = field_type {
                            current_type = Some(ft);
                        } else {
                            // If this is the last part, return completions for current type
                            if i == parts.len() - 1 {
                                return Self::completions_for_named_type(tn, symbols);
                            }
                            return None;
                        }
                    } else {
                        return None;
                    }
                } else {
                    return None;
                }
            }
            // Return completions for the resolved final type
            if let Some(tn) = current_type {
                return Self::completions_for_named_type(&tn, symbols);
            }
            return None;
        }

        // First, try global symbol table
        if let Some(sym) = symbols.lookup(expr) {
            return Self::completions_for_sym(sym, symbols);
        }

        // Not found in globals — try local variables from the enclosing function
        if let Some(type_name) = local_types.get(expr) {
            return Self::completions_for_named_type(type_name, symbols);
        }

        None
    }

    /// Resolve an expression name to its type name using symbol table and local scope.
    fn resolve_type_name<'a>(name: &str, symbols: &'a SymbolTable, local_types: &HashMap<String, String>) -> Option<String> {
        // Try global symbols first
        if let Some(sym) = symbols.lookup(name) {
            return match &sym.kind {
                SymKind::Variable { type_: Some(t), .. } => Some(Self::type_to_name(t)),
                SymKind::Constant(t) => Some(Self::type_to_name(t)),
                SymKind::Function(f) => f.return_type.as_ref().map(|t| Self::fmt_ast_type(t)),
                _ => None,
            };
        }
        // Try local scope
        if let Some(type_name) = local_types.get(name) {
            return Some(type_name.clone());
        }
        None
    }

    /// Extract a simple name from a Type for resolution purposes.
    fn type_to_name(t: &Type) -> String {
        match t {
            Type::Named(name) => name.clone(),
            Type::Generic(name, _) => name.clone(),
            Type::Str => "str".to_string(),
            Type::I32 | Type::I64 => "i32".to_string(),
            Type::Bool => "bool".to_string(),
            Type::F32 | Type::F64 => "f64".to_string(),
            Type::Char => "char".to_string(),
            Type::List(_) => "list".to_string(),
            Type::Dict(_, _) => "dict".to_string(),
            Type::Option(inner) => Self::type_to_name(inner),
            _ => String::new(),
        }
    }

    /// Return the field type name for a symbol at a given property, or None.
    fn field_type_for_sym(sym: &Symbol, property: &str) -> Option<String> {
        match &sym.kind {
            SymKind::Struct(s) => {
                s.fields.iter().find(|f| f.name == property)
                    .map(|f| Self::fmt_ast_type(&f.type_))
            }
            SymKind::Class(c) => {
                for member in &c.members {
                    if let ClassMember::Field(f) = member {
                        if f.name == property {
                            return Some(Self::fmt_ast_type(&f.type_));
                        }
                    }
                }
                None
            }
            SymKind::Enum(e) => {
                e.variants.iter().find(|v| v.name == property)
                    .map(|_| format!("{}", e.name)) // enum variant → returns the enum type
            }
            SymKind::Variable { type_: Some(t), .. } => Self::field_type_for_type(t, property),
            SymKind::Constant(t) => Self::field_type_for_type(t, property),
            _ => None,
        }
    }

    /// Return completions for a type name (resolves str, list, dict and user types).
    fn completions_for_named_type(type_name: &str, symbols: &SymbolTable) -> Option<Vec<CompletionItem>> {
        if let Some(sym) = symbols.lookup(type_name) {
            return Self::completions_for_sym(sym, symbols);
        }
        match type_name {
            "str" => Self::completions_for_type(&Type::Str, symbols),
            "list" => Self::completions_for_type(&Type::List(Box::new(Type::TypeVar(0))), symbols),
            "dict" => Self::completions_for_type(&Type::Dict(Box::new(Type::Str), Box::new(Type::TypeVar(0))), symbols),
            _ => None,
        }
    }

    /// Look up a field's type from a Type.
    fn field_type_for_type(t: &Type, _property: &str) -> Option<String> {
        match t {
            Type::Named(name) | Type::Generic(name, _) => {
                // We don't have symbol table access here, so return None
                Some(name.clone()) // best effort
            }
            _ => None,
        }
    }

    /// Walk the program AST to build a map of local variable names to their
    /// type names, scoped to the function containing `(line, col)`.
    fn build_local_scope(program: &Program, line: usize, _col: usize) -> HashMap<String, String> {
        let mut vars: HashMap<String, String> = HashMap::new();
        for decl in &program.declarations {
            if let Decl::Function(f) = decl {
                let l1 = f.span.start.line;
                let l2 = f.span.end.line;
                // KL spans are 1-indexed; cursor is 0-indexed
                if l1 <= line + 1 && line + 1 <= l2 {
                    // Register params
                    for param in &f.params {
                        if param.name == "this" { continue; }
                        let tn = Self::ast_type_name(&param.type_);
                        if !tn.is_empty() {
                            vars.insert(param.name.clone(), tn);
                        }
                    }
                    // Walk function body
                    if let Some(body) = &f.body {
                        Self::walk_block_for_vars(body, &mut vars);
                    }
                }
            }
        }
        vars
    }

    /// Walk a block to collect variable declarations with their types.
    fn walk_block_for_vars(block: &Block, vars: &mut HashMap<String, String>) {
        for stmt in &block.statements {
            match stmt {
                Stmt::Variable(vd) | Stmt::TypedVariable(vd) => {
                    if let Some(t) = &vd.type_ {
                        let tn = Self::ast_type_name(t);
                        if !tn.is_empty() {
                            vars.insert(vd.name.clone(), tn);
                        }
                    } else {
                        let val = &vd.value;
                        if let Some(tn) = Self::infer_type_from_expr(val) {
                            vars.insert(vd.name.clone(), tn);
                        }
                    }
                }
                // Auto-declared variables: `x = expr` (no mut/let)
                Stmt::Expression(Expr::Assignment { target, value, .. }) => {
                    if let Expr::Identifier { name, .. } = target.as_ref() {
                        if !vars.contains_key(name) {
                            if let Some(tn) = Self::infer_type_from_expr(value) {
                                vars.insert(name.clone(), tn);
                            }
                        }
                    }
                }
                Stmt::For(fs) => {
                    vars.insert(fs.variable.clone(), "i32".to_string());
                    Self::walk_block_for_vars(&fs.body, vars);
                    if let Some(eb) = &fs.else_branch {
                        Self::walk_block_for_vars(eb, vars);
                    }
                }
                Stmt::If(is) => {
                    Self::walk_block_for_vars(&is.body, vars);
                    for eb in &is.elif_branches {
                        Self::walk_block_for_vars(&eb.body, vars);
                    }
                    if let Some(eb) = &is.else_branch {
                        Self::walk_block_for_vars(eb, vars);
                    }
                }
                Stmt::While(ws) => {
                    Self::walk_block_for_vars(&ws.body, vars);
                }
                Stmt::Match(ms) => {
                    for arm in &ms.arms {
                        Self::walk_block_for_vars(&arm.body, vars);
                    }
                }
                Stmt::Guard(gs) => {
                    Self::walk_block_for_vars(&gs.body, vars);
                }
                Stmt::Unsafe(us) => {
                    Self::walk_block_for_vars(&us.body, vars);
                }
                _ => {}
            }
        }
    }

    /// Extract the type name string from an AstType.
    fn ast_type_name(t: &AstType) -> String {
        match t {
            AstType::Primitive { name, .. } => name.clone(),
            AstType::User { name, .. } => name.clone(),
            AstType::Generic { name, .. } => name.clone(),
            AstType::Optional { inner, .. } => Self::ast_type_name(inner),
            AstType::Error { inner, .. } => Self::ast_type_name(inner),
            AstType::Dict { .. } => "dict".to_string(),
        }
    }

    /// Rough type inference from an expression AST for building local scope.
    fn infer_type_from_expr(expr: &Expr) -> Option<String> {
        match expr {
            Expr::Literal { value, .. } => match value {
                Literal::Integer(_) => Some("i32".to_string()),
                Literal::Float(_) => Some("f64".to_string()),
                Literal::String(_) => Some("str".to_string()),
                Literal::Boolean(_) => Some("bool".to_string()),
                Literal::None => None,
            },
            Expr::StructLiteral { struct_name, .. } => Some(struct_name.clone()),
            Expr::Identifier { name, .. } => {
                // Identifiers have unknown type without symbol table — return None
                // but special-case common patterns
                if name.chars().all(|c| c.is_uppercase() || c == '_' || c.is_ascii_digit()) {
                    Some("i32".to_string()) // UPPERCASE constants are usually i32
                } else {
                    None
                }
            }
            Expr::Unary { operand, .. } => Self::infer_type_from_expr(operand),
            Expr::Binary { left, operator, right, .. } => {
                use klc_core::ast::BinaryOp;
                // String concatenation: if either side is str, result is str
                if matches!(operator, BinaryOp::Add) {
                    let lt = Self::infer_type_from_expr(left);
                    let rt = Self::infer_type_from_expr(right);
                    if lt.as_deref() == Some("str") || rt.as_deref() == Some("str") {
                        return Some("str".to_string());
                    }
                }
                // Comparisons always return bool
                if matches!(operator, BinaryOp::Eq | BinaryOp::Neq
                    | BinaryOp::Lt | BinaryOp::Gt | BinaryOp::Le | BinaryOp::Ge) {
                    return Some("bool".to_string());
                }
                // Arithmetic: use left operand type (both should match)
                Self::infer_type_from_expr(left)
            }
            Expr::PropertyAccess { object: _object, property, .. } => {
                // For chained completions like `obj.field`, infer from property name
                // This is a simplification — real type resolution needs symbol table
                if property == "len" { return Some("i32".to_string()); }
                if property == "contains" || property == "is_some" || property == "is_none" {
                    return Some("bool".to_string());
                }
                // For struct fields, we'd need the type — fallback to str
                Some("str".to_string())
            }
            Expr::FunctionCall { target, .. } => {
                if let Expr::Identifier { name, .. } = target.as_ref() {
                    match name.as_str() {
                        "input" => Some("str".to_string()),
                        "len" => Some("i32".to_string()),
                        "str" => Some("str".to_string()),
                        "int" => Some("i32".to_string()),
                        "float" => Some("f64".to_string()),
                        "bool" => Some("bool".to_string()),
                        "open" => Some("i64".to_string()),
                        "now" => Some("i64".to_string()),
                        "abs" | "sqrt" | "pow" | "gcd" => Some("i32".to_string()),
                        "json_parse" => Some("dict".to_string()),
                        "json_stringify" => Some("str".to_string()),
                        _ => None,
                    }
                } else if let Expr::PropertyAccess { object, property, .. } = target.as_ref() {
                    // Method call: obj.method() — infer from method name
                    if property == "to_upper" || property == "to_lower" || property == "trim" || property == "replace" || property == "substr" {
                        Some("str".to_string())
                    } else if property == "contains" || property == "is_some" || property == "is_none" {
                        Some("bool".to_string())
                    } else if property == "len" {
                        Some("i32".to_string())
                    } else if property == "pop" {
                        Some("i64".to_string())
                    } else {
                        Self::infer_type_from_expr(object)
                    }
                } else {
                    None
                }
            }
            Expr::Ternary { then_expr, .. } => Self::infer_type_from_expr(then_expr),
            Expr::List { .. } => Some("list".to_string()),
            Expr::Dictionary { .. } => Some("dict".to_string()),
            Expr::Index { target, .. } => {
                // For list[i], infer element type
                let target_type = Self::infer_type_from_expr(target);
                if target_type.as_deref() == Some("list") || target_type.as_deref() == Some("dict") {
                    Some("i64".to_string()) // default to i64 for list/dict elements
                } else {
                    Some("i64".to_string())
                }
            }
            Expr::OptionalChain { target, .. } => Self::infer_type_from_expr(target),
            Expr::StringInterp { .. } => Some("str".to_string()),
            Expr::Async { .. } => Some("i64".to_string()), // task handle
            Expr::Spread { expression, .. } => Self::infer_type_from_expr(expression),
            _ => None,
        }
    }

    /// Given a Symbol, return the appropriate dot completions (fields,
    /// methods, enum variants).
    fn completions_for_sym(sym: &Symbol, symbols: &SymbolTable) -> Option<Vec<CompletionItem>> {
        match &sym.kind {
            SymKind::Struct(s) => {
                let items: Vec<CompletionItem> = s.fields.iter().map(|f| {
                    CompletionItem {
                        label: f.name.clone(),
                        kind: Some(CompletionItemKind::FIELD),
                        detail: Some(format!("{}: {}", f.name, Self::fmt_ast_type(&f.type_))),
                        sort_text: Some(format!("1{}", f.name)),
                        ..Default::default()
                    }
                }).collect();
                if items.is_empty() { None } else { Some(items) }
            }
            SymKind::Class(c) => {
                let mut items = Vec::new();
                for member in &c.members {
                    match member {
                        ClassMember::Field(f) => {
                            items.push(CompletionItem {
                                label: f.name.clone(),
                                kind: Some(CompletionItemKind::FIELD),
                                detail: Some(format!("{}: {}", f.name, Self::fmt_ast_type(&f.type_))),
                                sort_text: Some(format!("1{}", f.name)),
                                ..Default::default()
                            });
                        }
                        ClassMember::Method(m) => {
                            let params: Vec<String> = m.params.iter()
                                .filter(|p| p.name != "this")
                                .map(|p| format!("{}: {}", p.name, Self::fmt_ast_type(&p.type_)))
                                .collect();
                            let sig = format!("fn {}({})", m.name, params.join(", "));
                            items.push(CompletionItem {
                                label: m.name.clone(),
                                kind: Some(CompletionItemKind::METHOD),
                                detail: Some(sig),
                                sort_text: Some(format!("2{}", m.name)),
                                ..Default::default()
                            });
                        }
                        _ => {}
                    }
                }
                if items.is_empty() { None } else { Some(items) }
            }
            SymKind::Enum(e) => {
                let items: Vec<CompletionItem> = e.variants.iter().map(|v| {
                    CompletionItem {
                        label: v.name.clone(),
                        kind: Some(CompletionItemKind::ENUM_MEMBER),
                        detail: Some(format!("{}.{}", e.name, v.name)),
                        sort_text: Some(format!("1{}", v.name)),
                        ..Default::default()
                    }
                }).collect();
                if items.is_empty() { None } else { Some(items) }
            }
            SymKind::Variable { type_: Some(t), .. } => {
                Self::completions_for_type(t, symbols)
            }
            SymKind::Constant(t) => {
                Self::completions_for_type(t, symbols)
            }
            _ => None,
        }
    }

    /// Return dot completions for a given resolved Type.
    fn completions_for_type(ty: &Type, symbols: &SymbolTable) -> Option<Vec<CompletionItem>> {
        match ty {
            Type::Named(name) | Type::Generic(name, _) => {
                let sym = symbols.lookup(name)?;
                Self::completions_for_sym(sym, symbols)
            }
            Type::Str => {
                Some(vec![
                    CompletionItem { label: "contains".into(), kind: Some(CompletionItemKind::METHOD), detail: Some("fn contains(substr: str) -> bool".into()), sort_text: Some("1contains".into()), ..Default::default() },
                    CompletionItem { label: "to_upper".into(), kind: Some(CompletionItemKind::METHOD), detail: Some("fn to_upper() -> str".into()), sort_text: Some("2to_upper".into()), ..Default::default() },
                    CompletionItem { label: "to_lower".into(), kind: Some(CompletionItemKind::METHOD), detail: Some("fn to_lower() -> str".into()), sort_text: Some("3to_lower".into()), ..Default::default() },
                    CompletionItem { label: "trim".into(), kind: Some(CompletionItemKind::METHOD), detail: Some("fn trim() -> str".into()), sort_text: Some("4trim".into()), ..Default::default() },
                    CompletionItem { label: "replace".into(), kind: Some(CompletionItemKind::METHOD), detail: Some("fn replace(from: str, to: str) -> str".into()), sort_text: Some("5replace".into()), ..Default::default() },
                    CompletionItem { label: "substr".into(), kind: Some(CompletionItemKind::METHOD), detail: Some("fn substr(start: i64, len: i64) -> str".into()), sort_text: Some("6substr".into()), ..Default::default() },
                    CompletionItem { label: "char_at".into(), kind: Some(CompletionItemKind::METHOD), detail: Some("fn char_at(index: i64) -> char".into()), sort_text: Some("7char_at".into()), ..Default::default() },
                    CompletionItem { label: "len".into(), kind: Some(CompletionItemKind::METHOD), detail: Some("fn len() -> i32".into()), sort_text: Some("8len".into()), ..Default::default() },
                ])
            }
            Type::List(_) => {
                Some(vec![
                    CompletionItem { label: "push".into(), kind: Some(CompletionItemKind::METHOD), detail: Some("fn push(value)".into()), sort_text: Some("1push".into()), ..Default::default() },
                    CompletionItem { label: "pop".into(), kind: Some(CompletionItemKind::METHOD), detail: Some("fn pop() -> i64".into()), sort_text: Some("2pop".into()), ..Default::default() },
                    CompletionItem { label: "len".into(), kind: Some(CompletionItemKind::METHOD), detail: Some("fn len() -> i32".into()), sort_text: Some("3len".into()), ..Default::default() },
                ])
            }
            Type::Dict(_, _) => {
                Some(vec![
                    CompletionItem { label: "len".into(), kind: Some(CompletionItemKind::METHOD), detail: Some("fn len() -> i32".into()), sort_text: Some("1len".into()), ..Default::default() },
                    CompletionItem { label: "get".into(), kind: Some(CompletionItemKind::METHOD), detail: Some("fn get(key)".into()), sort_text: Some("2get".into()), ..Default::default() },
                    CompletionItem { label: "set".into(), kind: Some(CompletionItemKind::METHOD), detail: Some("fn set(key, value)".into()), sort_text: Some("3set".into()), ..Default::default() },
                    CompletionItem { label: "keys".into(), kind: Some(CompletionItemKind::METHOD), detail: Some("fn keys() -> [str]".into()), sort_text: Some("4keys".into()), ..Default::default() },
                    CompletionItem { label: "values".into(), kind: Some(CompletionItemKind::METHOD), detail: Some("fn values() -> [T]".into()), sort_text: Some("5values".into()), ..Default::default() },
                ])
            }
            _ => None,
        }
    }

    fn fmt_ast_type(t: &AstType) -> String {
        match t {
            AstType::Primitive { name, .. } => name.clone(),
            AstType::User { name, .. } => name.clone(),
            AstType::Generic { name, args, .. } => {
                let args: Vec<String> = args.iter().map(Self::fmt_ast_type).collect();
                format!("{}<{}>", name, args.join(", "))
            }
            AstType::Optional { inner, .. } => format!("{}?", Self::fmt_ast_type(inner)),
            AstType::Error { inner, .. } => format!("{}!", Self::fmt_ast_type(inner)),
            AstType::Dict { key, value, .. } => format!("Dict<{}, {}>", Self::fmt_ast_type(key), Self::fmt_ast_type(value)),
        }
    }

    fn handle_definition(&mut self, req: Request) {
        let params: GotoDefinitionParams = serde_json::from_value(req.params).unwrap();
        let uri = params.text_document_position_params.text_document.uri.to_string();
        let source = self.sources.get(&uri).cloned().unwrap_or_default();
        let pos = params.text_document_position_params.position;

        let word = self.word_at(&source, pos.line as usize, pos.character as usize);

        let result: Option<GotoDefinitionResponse> = if word.is_empty() {
            None
        } else {
            self.find_definition(&source, &word)
                .map(|span| {
                    let loc = Location {
                        uri: lsp_types::Url::parse(&uri).unwrap(),
                        range: span_to_range(&span),
                    };
                    GotoDefinitionResponse::Scalar(loc)
                })
        };

        let resp = Response::new_ok(req.id, serde_json::to_value(result).unwrap());
        let _ = self.connection.sender.send(Message::Response(resp));
    }

    fn find_definition(&self, source: &str, name: &str) -> Option<Span> {
        let mut lexer = klc_frontend::lexer::Lexer::new(source);
        let tokens = lexer.tokenize();
        let mut parser = klc_frontend::parser::Parser::new(tokens);
        if let Ok(prog) = parser.parse() {
            for decl in &prog.declarations {
                match decl {
                    Decl::Function(f) if f.name == name => return Some(f.span),
                    Decl::Variable(v) if v.name == name => return Some(v.span),
                    Decl::Constant(c) if c.name == name => return Some(c.span),
                    Decl::Class(c) if c.name == name => return Some(c.span),
                    Decl::Struct(s) if s.name == name => return Some(s.span),
                    Decl::Enum(e) if e.name == name => return Some(e.span),
                    Decl::Contract(c) if c.name == name => return Some(c.span),
                    Decl::TypeAlias(t) if t.name == name => return Some(t.span),
                    _ => {}
                }
            }
        }
        None
    }

    fn handle_document_symbol(&mut self, req: Request) {
        let params: DocumentSymbolParams = serde_json::from_value(req.params).unwrap();
        let uri = params.text_document.uri.to_string();
        let source = self.sources.get(&uri).cloned().unwrap_or_default();
        let symbols = self.collect_symbols(&source, &uri);
        let result = DocumentSymbolResponse::Flat(symbols);
        let resp = Response::new_ok(req.id, serde_json::to_value(result).unwrap());
        let _ = self.connection.sender.send(Message::Response(resp));
    }

    fn handle_workspace_symbol(&mut self, req: Request) {
        let params: WorkspaceSymbolParams = serde_json::from_value(req.params).unwrap();
        let query = params.query.to_lowercase();
        let mut all_symbols: Vec<SymbolInformation> = Vec::new();
        for (uri, source) in &self.sources {
            for sym in self.collect_symbols(source, uri) {
                if query.is_empty() || sym.name.to_lowercase().contains(&query) {
                    all_symbols.push(sym);
                }
            }
        }
        let result = WorkspaceSymbolResponse::Flat(all_symbols);
        let resp = Response::new_ok(req.id, serde_json::to_value(result).unwrap());
        let _ = self.connection.sender.send(Message::Response(resp));
    }

    fn handle_signature_help(&mut self, req: Request) {
        let params: SignatureHelpParams = serde_json::from_value(req.params).unwrap();
        let uri = params.text_document_position_params.text_document.uri.to_string();
        let source = self.sources.get(&uri).cloned().unwrap_or_default();
        let pos = params.text_document_position_params.position;

        let mut lexer = klc_frontend::lexer::Lexer::new(&source);
        let tokens = lexer.tokenize();
        let mut parser = klc_frontend::parser::Parser::new(tokens);
        if let Ok(prog) = parser.parse() {
            let word = self.word_at(&source, pos.line as usize, pos.character as usize);
            if !word.is_empty() {
                for decl in &prog.declarations {
                    if let Decl::Function(f) = decl {
                        if f.name == word {
                            let params_list: Vec<String> = f.params.iter()
                                .map(|p| format!("{}: {}", p.name, Self::fmt_ast_type(&p.type_)))
                                .collect();
                            let return_info = f.return_type.as_ref()
                                .map(|rt| format!(" -> {}", Self::fmt_ast_type(rt)))
                                .unwrap_or_default();
                            let label = format!("fn {}({}){}", f.name, params_list.join(", "), return_info);
                            let sig = SignatureInformation {
                                label,
                                parameters: Some(vec![]),
                                active_parameter: Some(0),
                                documentation: None,
                            };
                            let result = SignatureHelp {
                                signatures: vec![sig],
                                active_signature: Some(0),
                                active_parameter: Some(0),
                            };
                            let resp = Response::new_ok(req.id, serde_json::to_value(result).unwrap());
                            let _ = self.connection.sender.send(Message::Response(resp));
                            return;
                        }
                    }
                }
            }
        }
        let result = SignatureHelp {
            signatures: vec![],
            active_signature: Some(0),
            active_parameter: Some(0),
        };
        let resp = Response::new_ok(req.id, serde_json::to_value(result).unwrap());
        let _ = self.connection.sender.send(Message::Response(resp));
    }

    fn handle_references(&mut self, req: Request) {
        let params: ReferenceParams = serde_json::from_value(req.params).unwrap();
        let uri = params.text_document_position.text_document.uri.to_string();
        let pos = params.text_document_position.position;
        let source = self.sources.get(&uri).cloned().unwrap_or_default();

        let word = self.word_at(&source, pos.line as usize, pos.character as usize);
        let mut locations = Vec::new();
        if !word.is_empty() {
            for (doc_uri, doc_source) in &self.sources {
                if let Ok(url) = lsp_types::Url::parse(doc_uri) {
                    locations.extend(self.find_references_in_source(doc_source, &word, url));
                }
            }
        }
        let resp = Response::new_ok(req.id, serde_json::to_value(locations).unwrap());
        let _ = self.connection.sender.send(Message::Response(resp));
    }

    fn find_references_in_source(&self, source: &str, name: &str, uri: lsp_types::Url) -> Vec<Location> {
        let mut locations = Vec::new();
        let mut lexer = klc_frontend::lexer::Lexer::new(source);
        let tokens = lexer.tokenize();
        for tok in &tokens {
            if let TokenKind::Identifier(ref id) = tok.kind {
                if id == name {
                    locations.push(Location {
                        uri: uri.clone(),
                        range: span_to_range(&tok.span),
                    });
                }
            }
        }
        locations
    }

    fn handle_code_action(&mut self, req: Request) {
        let params: CodeActionParams = serde_json::from_value(req.params).unwrap();
        let uri = params.text_document.uri.to_string();
        let mut actions: Vec<CodeActionOrCommand> = Vec::new();

        for diag in &params.context.diagnostics {
            if let Some(ref code) = diag.code {
                let code_str = match code {
                    NumberOrString::Number(n) => format!("E{:04}", n),
                    NumberOrString::String(s) => s.clone(),
                };
                if code_str == "E0009" {
                    // Undefined symbol — suggest creating the variable
                    let msg = diag.message.clone();
                    let name = self.extract_symbol_name(&msg);
                    if let Some(var_name) = name {
                        // Create variable code action
                        let edit = WorkspaceEdit {
                            changes: Some({
                                let mut map = HashMap::new();
                                map.insert(
                                    lsp_types::Url::parse(&uri).unwrap(),
                                    vec![TextEdit {
                                        range: Range {
                                            start: Position { line: 0, character: 0 },
                                            end: Position { line: 0, character: 0 },
                                        },
                                        new_text: format!("mut {} = None\n", var_name),
                                    }],
                                );
                                map
                            }),
                            ..Default::default()
                        };
                        actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                            title: format!("Create mutable variable '{}'", var_name),
                            kind: Some(CodeActionKind::QUICKFIX),
                            diagnostics: Some(vec![diag.clone()]),
                            edit: Some(edit),
                            ..Default::default()
                        }));

                        // Import suggestion (if name looks like a module)
                        if var_name.chars().next().unwrap_or(' ').is_uppercase() {
                            let import_edit = WorkspaceEdit {
                                changes: Some({
                                    let mut map = HashMap::new();
                                    map.insert(
                                        lsp_types::Url::parse(&uri).unwrap(),
                                        vec![TextEdit {
                                            range: Range {
                                                start: Position { line: 0, character: 0 },
                                                end: Position { line: 0, character: 0 },
                                            },
                                            new_text: format!("import {}\n", var_name),
                                        }],
                                    );
                                    map
                                }),
                                ..Default::default()
                            };
                            actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                                title: format!("Import module '{}'", var_name),
                                kind: Some(CodeActionKind::QUICKFIX),
                                diagnostics: Some(vec![diag.clone()]),
                                edit: Some(import_edit),
                                ..Default::default()
                            }));
                        }
                    }
                }
            }
        }

        let resp = Response::new_ok(req.id, serde_json::to_value(actions).unwrap());
        let _ = self.connection.sender.send(Message::Response(resp));
    }

    fn handle_formatting(&mut self, req: Request) {
        let params: DocumentFormattingParams = serde_json::from_value(req.params).unwrap();
        let uri = params.text_document.uri.to_string();
        let source = match self.sources.get(&uri) {
            Some(s) => s.clone(),
            None => {
                let resp = Response::new_ok(req.id, serde_json::to_value::<Vec<TextEdit>>(vec![]).unwrap());
                let _ = self.connection.sender.send(Message::Response(resp));
                return;
            }
        };

        let formatter = crate::formatter::Formatter::new();
        match formatter.format(&source) {
            Ok(formatted) => {
                let lines = source.lines().count();
                let last_line_len = source.lines().last().map(|l| l.len()).unwrap_or(0);
                let full_range = Range {
                    start: Position { line: 0, character: 0 },
                    end: Position { line: (lines.max(1) as u32).saturating_sub(1), character: last_line_len.max(1) as u32 },
                };
                let edits = vec![TextEdit {
                    range: full_range,
                    new_text: formatted,
                }];
                let resp = Response::new_ok(req.id, serde_json::to_value(edits).unwrap());
                let _ = self.connection.sender.send(Message::Response(resp));
            }
            Err(_) => {
                let resp = Response::new_ok(req.id, serde_json::to_value::<Vec<TextEdit>>(vec![]).unwrap());
                let _ = self.connection.sender.send(Message::Response(resp));
            }
        }
    }

    /// Extract the symbol name from a diagnostic message like "'foo' is not defined"
    fn extract_symbol_name(&self, msg: &str) -> Option<String> {
        if let Some(start) = msg.find('\'') {
            let rest = &msg[start + 1..];
            if let Some(end) = rest.find('\'') {
                return Some(rest[..end].to_string());
            }
        }
        None
    }

    fn collect_symbols(&self, source: &str, uri: &str) -> Vec<SymbolInformation> {
        let mut symbols = Vec::new();
        let mut lexer = klc_frontend::lexer::Lexer::new(source);
        let tokens = lexer.tokenize();
        let mut parser = klc_frontend::parser::Parser::new(tokens);
        let url = lsp_types::Url::parse(uri).unwrap_or_else(|_| lsp_types::Url::parse("file:///unknown.kl").unwrap());
        if let Ok(prog) = parser.parse() {
            for decl in &prog.declarations {
                let (name, kind, span) = match decl {
                    Decl::Function(f) => (f.name.clone(), SymbolKind::FUNCTION, f.span),
                    Decl::Variable(v) => (v.name.clone(), SymbolKind::VARIABLE, v.span),
                    Decl::Constant(c) => (c.name.clone(), SymbolKind::CONSTANT, c.span),
                    Decl::Class(c) => (c.name.clone(), SymbolKind::CLASS, c.span),
                    Decl::Struct(s) => (s.name.clone(), SymbolKind::STRUCT, s.span),
                    Decl::Enum(e) => (e.name.clone(), SymbolKind::ENUM, e.span),
                    Decl::Contract(c) => (c.name.clone(), SymbolKind::INTERFACE, c.span),
                    Decl::TypeAlias(t) => (t.name.clone(), SymbolKind::TYPE_PARAMETER, t.span),
                    _ => continue,
                };
                #[allow(deprecated)]
                symbols.push(SymbolInformation {
                    name,
                    kind,
                    tags: None,
                    deprecated: None,
                    location: Location {
                        uri: url.clone(),
                        range: span_to_range(&span),
                    },
                    container_name: None,
                });
            }
        }
        symbols
    }
}

/// Convert KL Span (1-indexed) to LSP Range (0-indexed).
fn span_to_range(span: &Span) -> Range {
    Range {
        start: Position {
            line: span.start.line.saturating_sub(1) as u32,
            character: span.start.column.saturating_sub(1) as u32,
        },
        end: Position {
            line: span.end.line.saturating_sub(1) as u32,
            character: span.end.column.saturating_sub(1) as u32,
        },
    }
}
