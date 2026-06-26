use std::collections::HashMap;
use lsp_server::{Connection, Message, Notification, Request, Response};
use lsp_types::*;
use klc_core::ast::*;
use klc_core::span::Span;
use klc_frontend::token::TokenKind;

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
                        let params: Vec<String> = f.params.iter().map(|p| format!("{}: {}", p.name, Self::fmt_type(&p.type_))).collect();
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
                                info.push_str(&format!(" -> {}", Self::fmt_type(rt)));
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

    fn fmt_type(t: &AstType) -> String {
        match t {
            AstType::Primitive { name, .. } => name.clone(),
            AstType::User { name, .. } => name.clone(),
            AstType::Generic { name, args, .. } => {
                let args: Vec<String> = args.iter().map(Self::fmt_type).collect();
                format!("{}<{}>", name, args.join(", "))
            }
            AstType::Optional { inner, .. } => format!("{}?", Self::fmt_type(inner)),
            AstType::Error { inner, .. } => format!("{}!", Self::fmt_type(inner)),
            AstType::Dict { key, value, .. } => format!("Dict<{}, {}>", Self::fmt_type(key), Self::fmt_type(value)),
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
                                .map(|p| format!("{}: {}", p.name, Self::fmt_type(&p.type_)))
                                .collect();
                            let return_info = f.return_type.as_ref()
                                .map(|rt| format!(" -> {}", Self::fmt_type(rt)))
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
