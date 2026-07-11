use crate::ast::*;

/// Parse a .kyx file content into a KyxFile AST.
pub fn parse(source: &str) -> Result<KyxFile, String> {
    let mut parser = KyxParser::new(source);
    parser.parse_file()
}

struct KyxParser {
    chars: Vec<char>,
    pos: usize,
}

impl KyxParser {
    fn new(source: &str) -> Self {
        Self { chars: source.chars().collect(), pos: 0 }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.chars.get(self.pos).copied();
        self.pos += 1;
        c
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c == ' ' || c == '\t' || c == '\n' || c == '\r' { self.advance(); }
            else { break; }
        }
    }

    fn expect_char(&mut self, expected: char) -> Result<(), String> {
        self.skip_whitespace();
        match self.advance() {
            Some(c) if c == expected => Ok(()),
            Some(c) => Err(format!("Expected '{}', found '{}'", expected, c)),
            None => Err(format!("Expected '{}', found EOF", expected)),
        }
    }

    fn parse_file(&mut self) -> Result<KyxFile, String> {
        let mut view_paths = Vec::new();
        let mut code_blocks = Vec::new();
        let mut styles = Vec::new();
        let mut animations = Vec::new();
        let mut body = Vec::new();

        loop {
            self.skip_whitespace();
            match self.peek() {
                None => break,
                Some('#') => {
                    // Skip comment line
                    while self.peek() != Some('\n') && self.peek().is_some() { self.advance(); }
                }
                Some('s') if self.starts_with("style<") => {
                    styles.push(self.parse_style_decl("style", false)?);
                }
                Some('l') if self.starts_with("layout<") => {
                    styles.push(self.parse_style_decl("layout", false)?);
                }
                Some('t') if self.starts_with("tpl<") => {
                    styles.push(self.parse_style_decl("tpl", false)?);
                }
                Some('t') if self.starts_with("theme ") => {
                    self.pos += 5;
                    let name = self.read_while(|c| c != ':' && c != '\n')?.trim().to_string();
                    self.expect_char(':')?;
                    let props = self.parse_style_props()?;
                    styles.push(StyleDecl::Theme { name, props });
                }
                Some('a') if self.starts_with("animation<") => {
                    self.pos += 9; // "animation"
                    self.expect_char('<')?;
                    let component = self.read_while(|c| c != '>')?;
                    self.expect_char('>')?;
                    self.skip_whitespace();
                    let name = self.read_while(|c| c != ':' && c != ' ' && c != '\t')?.trim().to_string();
                    self.expect_char(':')?;
                    let props = self.parse_style_props()?;
                    animations.push(AnimDecl { component, name, props });
                }
                Some(c) if c == 'v' || c == 'V' => {
                    // Try to parse view("...")
                    let saved = self.pos;
                    if let Ok(path) = self.try_parse_view() {
                        view_paths.push(path);
                    } else {
                        self.pos = saved;
                        // Not a view declaration — skip
                        self.advance();
                    }
                }
                Some('@') => {
                    self.advance();
                    if self.peek() == Some('(') {
                        let content_start = self.pos + 1; // after (
                        self.advance();
                        let after_paren = self.find_matching_paren()?;
                        let block = self.extract_and_advance(after_paren, content_start);
                        code_blocks.push(block);
                    } else if self.starts_with("if(") {
                        self.pos += 2;
                        self.advance();
                        let condition = self.read_while(|c| c != ')')?;
                        self.expect_char(')')?;
                        self.expect_char(':')?;
                        let then_branch = self.parse_body_until(&["@else", "@elif"])?;
                        let mut else_branch = Vec::new();
                        if self.starts_with("@else") {
                            self.pos += 5;
                            while self.peek() != Some(':') && self.peek() != Some('\n') && self.peek().is_some() { self.advance(); }
                            if self.peek() == Some(':') { self.advance(); }
                            else_branch = self.parse_body_until(&[])?;
                        }
                        body.push(KyxNode::If { condition, then_branch, else_branch });
                    } else if self.starts_with("for(") {
                        self.pos += 3;
                        let item = self.read_while(|c| c != ' ' && c != '\t')?.trim().to_string();
                        self.skip_whitespace();
                        self.expect_string("in")?;
                        self.skip_whitespace();
                        let list = self.read_while(|c| c != ')')?.trim().to_string();
                        self.expect_char(')')?;
                        self.expect_char(':')?;
                        let body_content = self.parse_body_until(&[])?;
                        body.push(KyxNode::For { item, list, body: body_content });
                    } else if self.starts_with("match(") {
                        self.pos += 5;
                        let expr = self.read_while(|c| c != ')')?.trim().to_string();
                        self.expect_char(')')?;
                        self.expect_char(':')?;
                        let mut cases = Vec::new();
                        loop {
                            self.skip_whitespace();
                            if self.peek().is_none() || self.peek() == Some('<') { break; }
                            let pattern = self.read_while(|c| c != ':')?.trim().to_string();
                            if pattern.is_empty() { break; }
                            self.expect_char(':')?;
                            let case_body = self.parse_body_until(&["@case"])?;
                            cases.push(MatchCase { pattern, body: case_body });
                        }
                        body.push(KyxNode::Match { expr, cases });
                    } else {
                        let expr = self.read_while(|c| c != '\n')?.trim().to_string();
                        body.push(KyxNode::Expr(expr));
                    }
                }
                Some('<') => {
                    let node = self.parse_element()?;
                    body.push(node);
                }
                _ => {
                    self.advance();
                }
            }
        }

        Ok(KyxFile { view_paths, code_blocks, styles, animations, body })
    }

    fn parse_style_decl(&mut self, kind: &str, _is_theme: bool) -> Result<StyleDecl, String> {
        // style<comp> Name: or layout<comp> Name: or tpl<comp> Name:
        for c in kind.chars() { self.advance(); }
        self.expect_char('<')?;
        let component = self.read_while(|c| c != '>')?;
        self.expect_char('>')?;
        self.skip_whitespace();
        let name = self.read_while(|c| c != ':' && c != ' ' && c != '\t')?.trim().to_string();
        self.expect_char(':')?;
        let (props, media) = self.parse_style_block()?;
        match kind {
            "layout" => Ok(StyleDecl::Layout { component, name, props, media }),
            "tpl" => Ok(StyleDecl::Template { component, name, props, media }),
            _ => Ok(StyleDecl::Style { component, name, props, media }),
        }
    }

    fn parse_style_props(&mut self) -> Result<Vec<StyleProp>, String> {
        let mut props = Vec::new();
        // Expect newline + indent
        if !self.at_newline() { return Ok(props); }
        self.advance();
        if !self.at_indent() { return Ok(props); }
        self.advance();
        loop {
            self.skip_whitespace_inline();
            match self.peek() {
                None | Some('\n') | Some('\r') => break,
                _ => {}
            }
            let name = self.read_while(|c| c != ' ' && c != '\t' && c != '=')?.trim().to_string();
            if name.is_empty() { break; }
            self.skip_whitespace();
            if self.peek() == Some('=') {
                self.advance();
                self.skip_whitespace();
                let value = self.read_while(|c| c != '\n' && c != '\r')?.trim().to_string();
                props.push(StyleProp { name, value });
            }
            // Skip to next line
            while self.peek() == Some(' ') || self.peek() == Some('\t') { self.advance(); }
            if self.peek() == Some('\n') || self.peek() == Some('\r') {
                self.advance();
                while self.peek() == Some('\n') || self.peek() == Some('\r') { self.advance(); }
                if !self.at_indent() && !self.at_space_or_tab() { break; }
            }
        }
        Ok(props)
    }

    fn parse_style_block(&mut self) -> Result<(Vec<StyleProp>, Vec<MediaQuery>), String> {
        let mut props = Vec::new();
        let mut media = Vec::new();
        if !self.at_newline() { return Ok((props, media)); }
        self.advance();
        if !self.at_indent() { return Ok((props, media)); }
        self.advance();
        loop {
            self.skip_whitespace_inline();
            match self.peek() {
                None | Some('\n') | Some('\r') => break,
                Some('@') if self.starts_with("@media(") => {
                    // Parse @media query
                    self.pos += 6; // skip "@media"
                    let condition = self.read_while(|c| c != ')')?.trim().to_string();
                    self.expect_char(')')?;
                    self.expect_char(':')?;
                    let media_props = self.parse_style_props()?;
                    media.push(MediaQuery { condition, props: media_props });
                }
                _ => {
                    let name = self.read_while(|c| c != ' ' && c != '\t' && c != '=')?.trim().to_string();
                    if name.is_empty() { break; }
                    self.skip_whitespace();
                    if self.peek() == Some('=') {
                        self.advance();
                        self.skip_whitespace();
                        let value = self.read_while(|c| c != '\n' && c != '\r')?.trim().to_string();
                        props.push(StyleProp { name, value });
                    }
                }
            }
            while self.peek() == Some(' ') || self.peek() == Some('\t') { self.advance(); }
            if self.peek() == Some('\n') || self.peek() == Some('\r') {
                self.advance();
                while self.peek() == Some('\n') || self.peek() == Some('\r') { self.advance(); }
                if !self.at_indent() && !self.at_space_or_tab() { break; }
            }
        }
        Ok((props, media))
    }

    fn at_newline(&self) -> bool {
        self.peek() == Some('\n') || self.peek() == Some('\r')
    }

    fn at_indent(&self) -> bool {
        self.peek() == Some('\t') || self.peek() == Some(' ')
    }

    fn at_space_or_tab(&self) -> bool {
        self.peek() == Some(' ') || self.peek() == Some('\t')
    }

    fn skip_whitespace_inline(&mut self) {
        while let Some(c) = self.peek() {
            if c == ' ' || c == '\t' { self.advance(); }
            else { break; }
        }
    }

    fn try_parse_view(&mut self) -> Result<String, String> {
        // view("/path")
        if !self.starts_with("view(") {
            return Err("Not a view declaration".to_string());
        }
        for c in "view(".chars() {
            self.advance();
        }
        self.expect_char('"')?;
        let path = self.read_until('"')?;
        self.expect_char(')')?;
        Ok(path)
    }

    fn starts_with(&self, s: &str) -> bool {
        let cs: Vec<char> = s.chars().collect();
        self.chars[self.pos..].starts_with(&cs)
    }

    fn parse_view(&mut self) -> Result<String, String> {
        // view("/path")
        self.expect_string("view(")?;
        self.expect_char('"')?;
        let path = self.read_until('"')?;
        self.expect_char('"')?;
        self.expect_char(')')?;
        Ok(path)
    }

    fn parse_at_directive(&mut self) -> Result<KyxNode, String> {
        // @if(cond): ... @else: ... or @for(item in list): or @match(expr): or @expr
        if self.starts_with("if(") {
            self.pos += 2; // skip 'if'
            self.advance(); // skip '('
            let condition = self.read_while(|c| c != ')')?;
            self.expect_char(')')?;
            self.expect_char(':')?;
            let then_branch = self.parse_body_until(&["@else", "@elif"])?;
            let mut else_branch = Vec::new();
            if self.starts_with("@else") {
                self.pos += 5;
                if self.peek() == Some(' ') || self.peek() == Some(':') {
                    while self.peek() != Some(':') && self.peek() != Some('\n') { self.advance(); }
                }
                if self.peek() == Some(':') { self.advance(); }
                else_branch = self.parse_body_until(&[])?;
            }
            return Ok(KyxNode::If { condition, then_branch, else_branch });
        }
        if self.starts_with("for(") {
            self.pos += 3;
            let item = self.read_while(|c| c != ' ' && c != '\t')?.trim().to_string();
            self.skip_whitespace();
            self.expect_string("in")?;
            self.skip_whitespace();
            let list = self.read_while(|c| c != ')')?.trim().to_string();
            self.expect_char(')')?;
            self.expect_char(':')?;
            let body = self.parse_body_until(&[])?;
            return Ok(KyxNode::For { item, list, body });
        }
        if self.starts_with("match(") {
            self.pos += 5;
            let expr = self.read_while(|c| c != ')')?.trim().to_string();
            self.expect_char(')')?;
            self.expect_char(':')?;
            let mut cases = Vec::new();
            loop {
                self.skip_whitespace();
                if self.peek().is_none() || self.peek() == Some('<') { break; }
                let pattern = self.read_while(|c| c != ':')?.trim().to_string();
                if pattern.is_empty() { break; }
                self.expect_char(':')?;
                let case_body = self.parse_body_until(&["@case"])?;
                cases.push(MatchCase { pattern, body: case_body });
            }
            return Ok(KyxNode::Match { expr, cases });
        }
        // Simple @expr (inline expression)
        let expr = self.read_while(|c| c != '\n')?.trim().to_string();
        Ok(KyxNode::Expr(expr))
    }

    fn parse_element(&mut self) -> Result<KyxNode, String> {
        self.expect_char('<')?;
        let tag = self.read_while(|c| c != ' ' && c != '\t' && c != '>' && c != '/' && c != '\n')?;
        let mut attrs = Vec::new();

        loop {
            self.skip_whitespace();
            match self.peek() {
                Some('>') => {
                    self.advance();
                    let children = self.parse_children(&tag)?;
                    return Ok(KyxNode::Element { tag, attrs, children });
                }
                Some('/') => {
                    self.advance();
                    if self.peek() == Some('>') {
                        self.advance();
                        return Ok(KyxNode::SelfClosing { tag, attrs });
                    }
                    return Err(format!("Expected '>' after '/' in tag '{}'", tag));
                }
                None => return Err(format!("Unexpected EOF in tag '{}'", tag)),
                _ => {
                    let attr = self.parse_attr()?;
                    attrs.push(attr);
                }
            }
        }
    }

    fn parse_attr(&mut self) -> Result<KyxAttr, String> {
        let name = self.read_while(|c| c != '=' && c != ' ' && c != '\t' && c != '>' && c != '/' && c != '\n')?;
        self.skip_whitespace();
        match self.peek() {
            Some('=') => {
                self.advance();
                self.skip_whitespace();
                match self.peek() {
                    Some('"') => {
                        self.advance();
                        let val = self.read_until('"')?;
                        Ok(KyxAttr { name, value: AttrValue::String(val) })
                    }
                    Some('@') => {
                        self.advance();
                        let expr = self.read_while(|c| c != ' ' && c != '\t' && c != '>' && c != '/' && c != '\n')?;
                        Ok(KyxAttr { name, value: AttrValue::Expr(expr) })
                    }
                    _ => {
                        let val = self.read_while(|c| c != ' ' && c != '\t' && c != '>' && c != '/' && c != '\n')?;
                        Ok(KyxAttr { name, value: AttrValue::String(val) })
                    }
                }
            }
            _ => Ok(KyxAttr { name, value: AttrValue::Flag }),
        }
    }

    fn parse_children(&mut self, closing_tag: &str) -> Result<Vec<KyxNode>, String> {
        let mut children = Vec::new();
        loop {
            self.skip_whitespace();
            match self.peek() {
                None => return Err(format!("Unexpected EOF, expected </{}>", closing_tag)),
                Some('<') => {
                    if self.starts_with("</") {
                        // Closing tag
                        let start = self.pos;
                        self.pos += 2;
                        let tag = self.read_while(|c| c != '>')?;
                        self.expect_char('>')?;
                        if tag == closing_tag {
                            return Ok(children);
                        }
                        // Not the right closing tag — treat as child element
                        self.pos = start;
                        let child = self.parse_element()?;
                        children.push(child);
                    } else if self.starts_with("<!--") {
                        // HTML comment — skip
                        self.pos += 4;
                        while !self.starts_with("-->") && self.peek().is_some() { self.advance(); }
                        if self.starts_with("-->") { self.pos += 3; }
                    } else {
                        // Child element
                        let child = self.parse_element()?;
                        children.push(child);
                    }
                }
                Some('@') => {
                    // @directive or @expr
                    self.advance();
                    let node = self.parse_at_directive()?;
                    children.push(node);
                }
                _ => {
                    // Text content
                    let text = self.read_while(|c| c != '<' && c != '@')?;
                    if !text.trim().is_empty() {
                        children.push(KyxNode::Text(text));
                    } else { self.advance(); }
                }
            }
        }
    }

    fn parse_body_until(&mut self, delimiters: &[&str]) -> Result<Vec<KyxNode>, String> {
        let mut nodes = Vec::new();
        loop {
            self.skip_whitespace();
            match self.peek() {
                None => break,
                Some('<') => {
                    // Check if this is a closing delimiter tag
                    let mut is_delim = false;
                    for d in delimiters {
                        if self.starts_with(d) { is_delim = true; break; }
                    }
                    if is_delim { break; }
                    let child = self.parse_element()?;
                    nodes.push(child);
                }
                Some('@') => {
                    let mut is_delim = false;
                    for d in delimiters {
                        if self.starts_with(d) { is_delim = true; break; }
                    }
                    if is_delim { break; }
                    self.advance();
                    let node = self.parse_at_directive()?;
                    nodes.push(node);
                }
                _ => break,
            }
        }
        Ok(nodes)
    }

    fn expect_string(&mut self, s: &str) -> Result<(), String> {
        let cs: Vec<char> = s.chars().collect();
        for &c in &cs {
            match self.advance() {
                Some(ch) if ch == c => {}
                Some(ch) => return Err(format!("Expected '{}', found '{}' in string '{}'", c, ch, s)),
                None => return Err(format!("Unexpected EOF, expected '{}'", s)),
            }
        }
        Ok(())
    }

    fn read_while<F>(&mut self, f: F) -> Result<String, String>
    where F: Fn(char) -> bool {
        let mut s = String::new();
        while let Some(c) = self.peek() {
            if f(c) { s.push(c); self.advance(); }
            else { break; }
        }
        Ok(s)
    }

    fn read_until(&mut self, delimiter: char) -> Result<String, String> {
        let mut s = String::new();
        while let Some(c) = self.advance() {
            if c == delimiter { break; }
            s.push(c);
        }
        Ok(s)
    }

    fn find_matching_paren(&mut self) -> Result<usize, String> {
        let mut depth = 1;
        loop {
            match self.advance() {
                Some('(') => depth += 1,
                Some(')') => {
                    depth -= 1;
                    if depth == 0 { return Ok(self.pos); } // pos is AFTER ) now
                }
                Some(_) => {}
                None => return Err("Unexpected EOF in @() block".to_string()),
            }
        }
    }

    fn extract_and_advance(&mut self, after_paren: usize, content_start: usize) -> String {
        self.pos = after_paren;
        self.chars[content_start..after_paren - 1].iter().collect::<String>().trim().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_file() {
        let result = parse("");
        assert!(result.is_ok());
        let file = result.unwrap();
        assert!(file.view_paths.is_empty());
        assert!(file.body.is_empty());
    }

    #[test]
    fn test_view_declaration() {
        let result = parse("view(\"/login\")");
        if let Err(ref e) = result {
            eprintln!("Parse error: {}", e);
        }
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());
        let file = result.unwrap();
        assert_eq!(file.view_paths, vec!["/login"]);
    }

    #[test]
    fn test_simple_element() {
        let result = parse("<view />");
        assert!(result.is_ok());
        let file = result.unwrap();
        assert_eq!(file.body.len(), 1);
    }

    #[test]
    fn test_element_with_attrs() {
        let result = parse(r#"<text value="Hello" />"#);
        assert!(result.is_ok());
        let file = result.unwrap();
        match &file.body[0] {
            KyxNode::SelfClosing { tag, attrs } => {
                assert_eq!(tag, "text");
                assert_eq!(attrs.len(), 1);
                assert_eq!(attrs[0].name, "value");
                match &attrs[0].value {
                    AttrValue::String(s) => assert_eq!(s, "Hello"),
                    _ => panic!("Expected string attr"),
                }
            }
            _ => panic!("Expected self-closing element"),
        }
    }

    #[test]
    fn test_element_with_children() {
        let result = parse("<view>\n  <text />\n</view>");
        assert!(result.is_ok());
        let file = result.unwrap();

        match &file.body[0] {
            KyxNode::Element { tag, children, .. } => {
                assert_eq!(tag, "view");
                assert_eq!(children.len(), 1);
            }
            _ => panic!("Expected element with children"),
        }
    }

    #[test]
    fn test_code_block() {
        let source = r#"@(count: i32
  fn inc(): count = count + 1)"#;
        let result = parse(source);
        if let Err(ref e) = result {
            eprintln!("Parse error: {}", e);
        }
        assert!(result.is_ok());
        let file = result.unwrap();
        assert_eq!(file.code_blocks.len(), 1);
    }

    #[test]
    fn test_full_login() {
        let source = r#"view("/login")

@(
 email: str
 password: str
 fn login():
  print("login")
)

<view>
 <Column layout=Center>
  <text_field bind=@email />
  <button tpl=Primary text="Ingresar" click=@login />
 </Column>
</view>"#;
        let result = parse(source);
        if let Err(ref e) = result {
            eprintln!("Parse error: {}", e);
        }
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());
        let file = result.unwrap();
        assert_eq!(file.view_paths, vec!["/login"]);
        assert_eq!(file.code_blocks.len(), 1);
        assert_eq!(file.body.len(), 1);
    }
}
