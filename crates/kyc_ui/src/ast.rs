use std::fmt;

/// A parsed .kyx file
#[derive(Clone, Debug)]
pub struct KyxFile {
    /// Optional view path declarations: view("/path", "/alias")
    pub view_paths: Vec<String>,
    /// Kyle code blocks: @(...)
    pub code_blocks: Vec<String>,
    /// Root XML elements
    pub body: Vec<KyxNode>,
}

/// An XML element or control-flow directive
#[derive(Clone, Debug)]
pub enum KyxNode {
    /// XML element: <tag attrs>children</tag>
    Element {
        tag: String,
        attrs: Vec<KyxAttr>,
        children: Vec<KyxNode>,
    },
    /// Self-closing element: <tag attrs />
    SelfClosing {
        tag: String,
        attrs: Vec<KyxAttr>,
    },
    /// Slot declaration: <slot name="header" />
    Slot {
        name: String,
        fallback: Vec<KyxNode>,
    },
    /// @if(condition): ...
    If {
        condition: String,
        then_branch: Vec<KyxNode>,
        else_branch: Vec<KyxNode>,
    },
    /// @for(item in list): ...
    For {
        item: String,
        list: String,
        body: Vec<KyxNode>,
    },
    /// @match(expr): cases
    Match {
        expr: String,
        cases: Vec<MatchCase>,
    },
    /// @expr — inline Kyle expression
    Expr(String),
    /// @(code) — Kyle code block
    CodeBlock(String),
    /// Raw text content
    Text(String),
}

/// A match case in @match
#[derive(Clone, Debug)]
pub struct MatchCase {
    pub pattern: String,
    pub body: Vec<KyxNode>,
}

/// An XML attribute
#[derive(Clone, Debug)]
pub struct KyxAttr {
    pub name: String,
    pub value: AttrValue,
}

/// Attribute value: literal string or Kyle expression
#[derive(Clone, Debug)]
pub enum AttrValue {
    /// Plain string: attr="value"
    String(String),
    /// Kyle expression: attr=@expr
    Expr(String),
    /// Boolean flag: attr (no value)
    Flag,
}

impl fmt::Display for KyxFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for path in &self.view_paths {
            writeln!(f, "view(\"{}\")", path)?;
        }
        for block in &self.code_blocks {
            writeln!(f, "@({})", block)?;
        }
        for node in &self.body {
            write!(f, "{}", node)?;
        }
        Ok(())
    }
}

impl fmt::Display for KyxNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KyxNode::Element { tag, attrs, children } => {
                write!(f, "<{}", tag)?;
                for a in attrs { write!(f, " {}", a)?; }
                if children.is_empty() {
                    writeln!(f, " />")?;
                } else {
                    writeln!(f, ">")?;
                    for c in children { write!(f, "  {}", c)?; }
                    writeln!(f, "</{}>", tag)?;
                }
                Ok(())
            }
            KyxNode::SelfClosing { tag, attrs } => {
                write!(f, "<{}", tag)?;
                for a in attrs { write!(f, " {}", a)?; }
                writeln!(f, " />")
            }
            KyxNode::Slot { name, .. } => writeln!(f, "<slot name=\"{}\" />", name),
            KyxNode::If { condition, .. } => writeln!(f, "@if({}): ...", condition),
            KyxNode::For { item, list, .. } => writeln!(f, "@for({} in {}): ...", item, list),
            KyxNode::Match { expr, .. } => writeln!(f, "@match({}): ...", expr),
            KyxNode::Expr(e) => writeln!(f, "@{}", e),
            KyxNode::CodeBlock(b) => writeln!(f, "@({})", b),
            KyxNode::Text(t) => write!(f, "{}", t),
        }
    }
}

impl fmt::Display for KyxAttr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.value {
            AttrValue::String(s) => write!(f, "{}=\"{}\"", self.name, s),
            AttrValue::Expr(e) => write!(f, "{}={}", self.name, e),
            AttrValue::Flag => write!(f, "{}", self.name),
        }
    }
}
