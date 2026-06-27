// klc_core::ast — Full AST node type definitions
//
// Reference: docs/03-ast-specification.md
// Every node carries a `span: Span` for diagnostic reporting.

use std::fmt;
use crate::span::Span;

// ---------------------------------------------------------------------------
// Literal values
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub enum Literal {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    None,
}

// ---------------------------------------------------------------------------
// Types (inline AST nodes)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub enum AstType {
    Primitive { name: String, span: Span },
    User { name: String, span: Span },
    Generic { name: String, args: Vec<AstType>, span: Span },
    Optional { inner: Box<AstType>, span: Span },
    Error { inner: Box<AstType>, span: Span },
    Dict { key: Box<AstType>, value: Box<AstType>, span: Span },
}

// ---------------------------------------------------------------------------
// Patterns (for match statements)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub enum Pattern {
    Identifier { name: String, span: Span },
    Literal { value: Literal, span: Span },
    Wildcard { span: Span },
    EnumVariant { enum_name: String, variant: String, args: Vec<Pattern>, span: Span },
    IsType { type_: AstType, span: Span },
}

// ---------------------------------------------------------------------------
// Declarations
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub struct Import {
    pub module_name: String,
    pub alias: Option<String>,
    pub relative: bool,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FromImport {
    pub module_name: String,
    pub imported_name: String,
    pub relative: bool,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct VariableDecl {
    pub name: String,
    pub type_: Option<AstType>,
    pub value: Box<Expr>,
    pub is_mutable: bool,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ConstantDecl {
    pub name: String,
    pub value: Box<Expr>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub type_: AstType,
    pub default: Option<Box<Expr>>,
    pub variadic: bool,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TypeParam {
    pub name: String,
    pub constraint: Option<AstType>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FunctionDecl {
    pub name: String,
    pub type_params: Vec<TypeParam>,
    pub params: Vec<Parameter>,
    pub return_type: Option<AstType>,
    pub is_async: bool,
    pub is_const: bool,
    pub is_abstract: bool,
    pub visibility: Visibility,
    pub body: Option<Block>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Field {
    pub name: String,
    pub type_: AstType,
    pub visibility: Visibility,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Visibility {
    Public,
    Protected,
    Private,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClassDecl {
    pub name: String,
    pub type_params: Vec<TypeParam>,
    pub parent: Option<String>,
    pub contracts: Vec<String>,
    pub members: Vec<ClassMember>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AbstractClassDecl {
    pub name: String,
    pub type_params: Vec<TypeParam>,
    pub parent: Option<String>,
    pub contracts: Vec<String>,
    pub members: Vec<ClassMember>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ClassMember {
    Field(Field),
    Property(Property),
    Constructor(Constructor),
    Method(FunctionDecl),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Property {
    pub name: String,
    pub type_: AstType,
    pub getter: Option<Block>,
    pub setter: Option<(String, Block)>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Constructor {
    pub params: Vec<Parameter>,
    pub body: Block,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ContractDecl {
    pub name: String,
    pub methods: Vec<ContractMethod>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ContractMethod {
    pub name: String,
    pub params: Vec<Parameter>,
    pub return_type: Option<AstType>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StructDecl {
    pub name: String,
    pub type_params: Vec<TypeParam>,
    pub fields: Vec<Field>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EnumVariant {
    pub name: String,
    pub payload: Vec<AstType>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EnumDecl {
    pub name: String,
    pub type_params: Vec<TypeParam>,
    pub variants: Vec<EnumVariant>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TypeAlias {
    pub name: String,
    pub type_params: Vec<TypeParam>,
    pub type_: AstType,
    pub span: Span,
}

// ---------------------------------------------------------------------------
// Top-level declaration enum
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub enum Decl {
    Import(Import),
    FromImport(FromImport),
    Variable(VariableDecl),
    Constant(ConstantDecl),
    Function(FunctionDecl),
    Class(ClassDecl),
    AbstractClass(AbstractClassDecl),
    Struct(StructDecl),
    Enum(EnumDecl),
    Contract(ContractDecl),
    TypeAlias(TypeAlias),
}

// ---------------------------------------------------------------------------
// Statements
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    pub statements: Vec<Stmt>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IfStmt {
    pub condition: Box<Expr>,
    pub body: Block,
    pub elif_branches: Vec<ElifBranch>,
    pub else_branch: Option<Block>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ElifBranch {
    pub condition: Box<Expr>,
    pub body: Block,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BindingIf {
    pub name: String,
    pub value: Box<Expr>,
    pub body: Block,
    pub else_branch: Option<Block>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WhileStmt {
    pub condition: Box<Expr>,
    pub body: Block,
    pub else_branch: Option<Block>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WhileBind {
    pub name: String,
    pub iterable: Box<Expr>,
    pub body: Block,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ForStmt {
    pub variable: String,
    pub iterable: Box<Expr>,
    pub body: Block,
    pub else_branch: Option<Block>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Box<Expr>>,
    pub body: Block,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MatchStmt {
    pub expression: Box<Expr>,
    pub arms: Vec<MatchArm>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DeferStmt {
    pub call: Box<Expr>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GuardStmt {
    pub condition: Box<Expr>,
    pub body: Block,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UnsafeBlock {
    pub body: Block,
    pub span: Span,
}

// ---------------------------------------------------------------------------
// Statement enum
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub enum Stmt {
    Variable(VariableDecl),
    TypedVariable(VariableDecl),
    Constant(ConstantDecl),
    Expression(Expr),
    Return(Option<Box<Expr>>),
    Break(Option<Box<Expr>>),
    Continue,
    If(IfStmt),
    BindingIf(BindingIf),
    While(WhileStmt),
    WhileBind(WhileBind),
    For(ForStmt),
    Match(MatchStmt),
    Defer(DeferStmt),
    Guard(GuardStmt),
    Unsafe(UnsafeBlock),
}

// ---------------------------------------------------------------------------
// Expressions
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Literal {
        value: Literal,
        span: Span,
    },
    Identifier {
        name: String,
        span: Span,
    },
    Binary {
        left: Box<Expr>,
        operator: BinaryOp,
        right: Box<Expr>,
        span: Span,
    },
    Unary {
        operator: UnaryOp,
        operand: Box<Expr>,
        span: Span,
    },
    Assignment {
        target: Box<Expr>,
        operator: Option<BinaryOp>,
        value: Box<Expr>,
        span: Span,
    },
    FunctionCall {
        target: Box<Expr>,
        arguments: Vec<Expr>,
        span: Span,
    },
    PropertyAccess {
        object: Box<Expr>,
        property: String,
        span: Span,
    },
    List {
        elements: Vec<Expr>,
        span: Span,
    },
    Dictionary {
        entries: Vec<(String, Expr)>,
        span: Span,
    },
    StructLiteral {
        struct_name: String,
        fields: Vec<(String, Expr)>,
        span: Span,
    },
    Tuple {
        elements: Vec<Expr>,
        span: Span,
    },
    Closure {
        params: Vec<String>,
        body: Box<Expr>,
        span: Span,
    },
    Await {
        expression: Box<Expr>,
        span: Span,
    },
    Async {
        expression: Box<Expr>,
        span: Span,
    },
    Spread {
        expression: Box<Expr>,
        span: Span,
    },
    Index {
        target: Box<Expr>,
        index: Box<Expr>,
        span: Span,
    },
    RangeSlice {
        target: Box<Expr>,
        start: Option<Box<Expr>>,
        end: Option<Box<Expr>>,
        span: Span,
    },
    OptionalChain {
        target: Box<Expr>,
        property: String,
        span: Span,
    },
    Loop {
        body: Block,
        span: Span,
    },
    ErrorProp {
        expression: Box<Expr>,
        span: Span,
    },
    StringInterp {
        parts: Vec<Expr>,
        span: Span,
    },
    Ternary {
        cond: Box<Expr>,
        then_expr: Box<Expr>,
        else_expr: Box<Expr>,
        span: Span,
    },
    MatchExpr {
        expression: Box<Expr>,
        arms: Vec<MatchArm>,
        span: Span,
    },
}

// ---------------------------------------------------------------------------
// Operators
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Pow,
    AddPercent,
    SubPercent,
    MulPercent,
    Eq,
    Neq,
    Lt,
    Gt,
    Le,
    Ge,
    And,
    Or,
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
    Is,
    Assign,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    RemAssign,
    BitAndAssign,
    BitOrAssign,
    BitXorAssign,
    ShlAssign,
    ShrAssign,
    Range,
}

#[derive(Clone, Debug, PartialEq)]
pub enum UnaryOp {
    Not,
    Neg,
    BitNot,
}

// ---------------------------------------------------------------------------
// Program node (root)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub struct Program {
    pub declarations: Vec<Decl>,
    pub span: Span,
}

// ---------------------------------------------------------------------------
// Display / pretty-printing
// ---------------------------------------------------------------------------

fn write_indent(f: &mut fmt::Formatter<'_>, indent: usize) -> fmt::Result {
    for _ in 0..indent {
        write!(f, "  ")?;
    }
    Ok(())
}

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Program")?;
        for decl in &self.declarations {
            decl.fmt_depth(f, 1)?;
        }
        Ok(())
    }
}

trait DisplayDepth {
    fn fmt_depth(&self, f: &mut fmt::Formatter<'_>, d: usize) -> fmt::Result;
}

fn display_block(f: &mut fmt::Formatter<'_>, d: usize, label: &str, block: &Block) -> fmt::Result {
    write_indent(f, d)?;
    writeln!(f, "{}", label)?;
    for stmt in &block.statements {
        stmt.fmt_depth(f, d + 1)?;
    }
    Ok(())
}

impl DisplayDepth for Decl {
    fn fmt_depth(&self, f: &mut fmt::Formatter<'_>, d: usize) -> fmt::Result {
        match self {
            Decl::Import(i) => i.fmt_depth(f, d),
            Decl::FromImport(fi) => fi.fmt_depth(f, d),
            Decl::Variable(v) => v.fmt_depth(f, d),
            Decl::Constant(c) => c.fmt_depth(f, d),
            Decl::Function(fd) => fd.fmt_depth(f, d),
            Decl::Class(c) => c.fmt_depth(f, d),
            Decl::AbstractClass(a) => a.fmt_depth(f, d),
            Decl::Struct(s) => s.fmt_depth(f, d),
            Decl::Enum(e) => e.fmt_depth(f, d),
            Decl::Contract(c) => c.fmt_depth(f, d),
            Decl::TypeAlias(t) => t.fmt_depth(f, d),
        }
    }
}

impl DisplayDepth for Import {
    fn fmt_depth(&self, f: &mut fmt::Formatter<'_>, d: usize) -> fmt::Result {
        write_indent(f, d)?;
        write!(f, "Import module=\"{}\"", self.module_name)?;
        if self.relative {
            write!(f, " relative")?;
        }
        if let Some(alias) = &self.alias {
            write!(f, " as=\"{}\"", alias)?;
        }
        writeln!(f)
    }
}

impl DisplayDepth for FromImport {
    fn fmt_depth(&self, f: &mut fmt::Formatter<'_>, d: usize) -> fmt::Result {
        write_indent(f, d)?;
        writeln!(f, "FromImport module=\"{}\" name=\"{}\"{}",
            self.module_name, self.imported_name,
            if self.relative { " relative" } else { "" })
    }
}

impl DisplayDepth for VariableDecl {
    fn fmt_depth(&self, f: &mut fmt::Formatter<'_>, d: usize) -> fmt::Result {
        write_indent(f, d)?;
        write!(f, "Var name=\"{}\"", self.name)?;
        if self.is_mutable {
            write!(f, " mut")?;
        }
        if let Some(t) = &self.type_ {
            write!(f, " type=\"{}\"", t)?;
        }
        writeln!(f)?;
        self.value.fmt_depth(f, d + 1)?;
        Ok(())
    }
}

impl DisplayDepth for ConstantDecl {
    fn fmt_depth(&self, f: &mut fmt::Formatter<'_>, d: usize) -> fmt::Result {
        write_indent(f, d)?;
        writeln!(f, "Const name=\"{}\"", self.name)?;
        self.value.fmt_depth(f, d + 1)?;
        Ok(())
    }
}

impl DisplayDepth for Parameter {
    fn fmt_depth(&self, f: &mut fmt::Formatter<'_>, d: usize) -> fmt::Result {
        write_indent(f, d)?;
        write!(f, "Param name=\"{}\" type=\"{}\"", self.name, self.type_)?;
        if self.variadic {
            write!(f, " variadic")?;
        }
        if let Some(default) = &self.default {
            writeln!(f)?;
            write_indent(f, d + 1)?;
            write!(f, "default=")?;
            default.fmt_depth(f, 0)?;
        }
        writeln!(f)
    }
}

impl DisplayDepth for FunctionDecl {
    fn fmt_depth(&self, f: &mut fmt::Formatter<'_>, d: usize) -> fmt::Result {
        write_indent(f, d)?;
        write!(f, "Fn name=\"{}\"", self.name)?;
        if self.is_async { write!(f, " async")?; }
        if self.is_const { write!(f, " const")?; }
        if self.is_abstract { write!(f, " abstract")?; }
        if let Some(rt) = &self.return_type {
            write!(f, " -> \"{}\"", rt)?;
        }
        writeln!(f)?;
        for param in &self.params {
            param.fmt_depth(f, d + 1)?;
        }
        if let Some(body) = &self.body {
            display_block(f, d + 1, "body:", body)?;
        }
        Ok(())
    }
}

impl DisplayDepth for Field {
    fn fmt_depth(&self, f: &mut fmt::Formatter<'_>, d: usize) -> fmt::Result {
        write_indent(f, d)?;
        writeln!(f, "Field name=\"{}\" type=\"{}\" vis={:?}", self.name, self.type_, self.visibility)
    }
}

impl DisplayDepth for ClassDecl {
    fn fmt_depth(&self, f: &mut fmt::Formatter<'_>, d: usize) -> fmt::Result {
        write_indent(f, d)?;
        write!(f, "Class name=\"{}\"", self.name)?;
        if let Some(p) = &self.parent {
            write!(f, " extends=\"{}\"", p)?;
        }
        if !self.contracts.is_empty() {
            write!(f, " implements={:?}", self.contracts)?;
        }
        writeln!(f)?;
        for member in &self.members {
            member.fmt_depth(f, d + 1)?;
        }
        Ok(())
    }
}

impl DisplayDepth for AbstractClassDecl {
    fn fmt_depth(&self, f: &mut fmt::Formatter<'_>, d: usize) -> fmt::Result {
        write_indent(f, d)?;
        write!(f, "AbsClass name=\"{}\"", self.name)?;
        if let Some(p) = &self.parent {
            write!(f, " extends=\"{}\"", p)?;
        }
        if !self.contracts.is_empty() {
            write!(f, " implements={:?}", self.contracts)?;
        }
        writeln!(f)?;
        for member in &self.members {
            member.fmt_depth(f, d + 1)?;
        }
        Ok(())
    }
}

impl DisplayDepth for ClassMember {
    fn fmt_depth(&self, f: &mut fmt::Formatter<'_>, d: usize) -> fmt::Result {
        match self {
            ClassMember::Field(field) => field.fmt_depth(f, d),
            ClassMember::Property(prop) => prop.fmt_depth(f, d),
            ClassMember::Constructor(ctor) => ctor.fmt_depth(f, d),
            ClassMember::Method(method) => method.fmt_depth(f, d),
        }
    }
}

impl DisplayDepth for Property {
    fn fmt_depth(&self, f: &mut fmt::Formatter<'_>, d: usize) -> fmt::Result {
        write_indent(f, d)?;
        writeln!(f, "Property name=\"{}\" type=\"{}\"", self.name, self.type_)?;
        if let Some(getter) = &self.getter {
            display_block(f, d + 1, "get:", getter)?;
        }
        if let Some((_param, setter)) = &self.setter {
            display_block(f, d + 1, "set:", setter)?;
        }
        Ok(())
    }
}

impl DisplayDepth for Constructor {
    fn fmt_depth(&self, f: &mut fmt::Formatter<'_>, d: usize) -> fmt::Result {
        write_indent(f, d)?;
        writeln!(f, "Constructor")?;
        for param in &self.params {
            param.fmt_depth(f, d + 1)?;
        }
        display_block(f, d + 1, "body:", &self.body)?;
        Ok(())
    }
}

impl DisplayDepth for ContractDecl {
    fn fmt_depth(&self, f: &mut fmt::Formatter<'_>, d: usize) -> fmt::Result {
        write_indent(f, d)?;
        writeln!(f, "Contract name=\"{}\"", self.name)?;
        for method in &self.methods {
            method.fmt_depth(f, d + 1)?;
        }
        Ok(())
    }
}

impl DisplayDepth for ContractMethod {
    fn fmt_depth(&self, f: &mut fmt::Formatter<'_>, d: usize) -> fmt::Result {
        write_indent(f, d)?;
        write!(f, "Method name=\"{}\"", self.name)?;
        if let Some(rt) = &self.return_type {
            write!(f, " -> \"{}\"", rt)?;
        }
        writeln!(f)?;
        for param in &self.params {
            param.fmt_depth(f, d + 1)?;
        }
        Ok(())
    }
}

impl DisplayDepth for StructDecl {
    fn fmt_depth(&self, f: &mut fmt::Formatter<'_>, d: usize) -> fmt::Result {
        write_indent(f, d)?;
        writeln!(f, "Struct name=\"{}\"", self.name)?;
        for field in &self.fields {
            field.fmt_depth(f, d + 1)?;
        }
        Ok(())
    }
}

impl DisplayDepth for EnumDecl {
    fn fmt_depth(&self, f: &mut fmt::Formatter<'_>, d: usize) -> fmt::Result {
        write_indent(f, d)?;
        writeln!(f, "Enum name=\"{}\"", self.name)?;
        for variant in &self.variants {
            variant.fmt_depth(f, d + 1)?;
        }
        Ok(())
    }
}

impl DisplayDepth for EnumVariant {
    fn fmt_depth(&self, f: &mut fmt::Formatter<'_>, d: usize) -> fmt::Result {
        write_indent(f, d)?;
        write!(f, "Variant name=\"{}\"", self.name)?;
        if !self.payload.is_empty() {
            write!(f, " payload=[")?;
            for (i, t) in self.payload.iter().enumerate() {
                if i > 0 { write!(f, ", ")?; }
                write!(f, "\"{}\"", t)?;
            }
            write!(f, "]")?;
        }
        writeln!(f)
    }
}

impl DisplayDepth for TypeAlias {
    fn fmt_depth(&self, f: &mut fmt::Formatter<'_>, d: usize) -> fmt::Result {
        write_indent(f, d)?;
        writeln!(f, "TypeAlias name=\"{}\" type=\"{}\"", self.name, self.type_)
    }
}

impl DisplayDepth for Stmt {
    fn fmt_depth(&self, f: &mut fmt::Formatter<'_>, d: usize) -> fmt::Result {
        match self {
            Stmt::Variable(v) => v.fmt_depth(f, d),
            Stmt::TypedVariable(v) => {
                write_indent(f, d)?;
                let type_str = v.type_.as_ref().map(|t| format!("{}", t)).unwrap_or_default();
                writeln!(f, "TypedVar name=\"{}\" type=\"{}\"", v.name, type_str)?;
                v.value.fmt_depth(f, d + 1)
            }
            Stmt::Constant(c) => c.fmt_depth(f, d),
            Stmt::Expression(e) => {
                write_indent(f, d)?;
                writeln!(f, "ExprStmt")?;
                e.fmt_depth(f, d + 1)
            }
            Stmt::Return(e) => {
                write_indent(f, d)?;
                if let Some(val) = e {
                    writeln!(f, "Return")?;
                    val.fmt_depth(f, d + 1)
                } else {
                    writeln!(f, "Return (void)")
                }
            }
            Stmt::Break(v) => {
                write_indent(f, d)?;
                if let Some(val) = v {
                    writeln!(f, "Break")?;
                    val.fmt_depth(f, d + 1)
                } else {
                    writeln!(f, "Break")
                }
            }
            Stmt::Continue => {
                write_indent(f, d)?;
                writeln!(f, "Continue")
            }
            Stmt::If(s) => {
                write_indent(f, d)?;
                writeln!(f, "If")?;
                s.condition.fmt_depth(f, d + 1)?;
                display_block(f, d + 1, "then:", &s.body)?;
                for elif in &s.elif_branches {
                    write_indent(f, d)?;
                    writeln!(f, "Elif")?;
                    elif.condition.fmt_depth(f, d + 1)?;
                    display_block(f, d + 1, "then:", &elif.body)?;
                }
                if let Some(el) = &s.else_branch {
                    display_block(f, d + 1, "else:", el)?;
                }
                Ok(())
            }
            Stmt::BindingIf(b) => {
                write_indent(f, d)?;
                writeln!(f, "BindingIf name=\"{}\"", b.name)?;
                b.value.fmt_depth(f, d + 1)?;
                display_block(f, d + 1, "then:", &b.body)?;
                if let Some(el) = &b.else_branch {
                    display_block(f, d + 1, "else:", el)?;
                }
                Ok(())
            }
            Stmt::While(s) => {
                write_indent(f, d)?;
                writeln!(f, "While")?;
                s.condition.fmt_depth(f, d + 1)?;
                display_block(f, d + 1, "body:", &s.body)?;
                if let Some(el) = &s.else_branch {
                    display_block(f, d + 1, "else:", el)?;
                }
                Ok(())
            }
            Stmt::WhileBind(w) => {
                write_indent(f, d)?;
                writeln!(f, "WhileBind name=\"{}\"", w.name)?;
                w.iterable.fmt_depth(f, d + 1)?;
                display_block(f, d + 1, "body:", &w.body)?;
                Ok(())
            }
            Stmt::For(s) => {
                write_indent(f, d)?;
                writeln!(f, "For var=\"{}\"", s.variable)?;
                s.iterable.fmt_depth(f, d + 1)?;
                display_block(f, d + 1, "body:", &s.body)?;
                if let Some(el) = &s.else_branch {
                    display_block(f, d + 1, "else:", el)?;
                }
                Ok(())
            }
            Stmt::Match(s) => {
                write_indent(f, d)?;
                writeln!(f, "Match")?;
                s.expression.fmt_depth(f, d + 1)?;
                for arm in &s.arms {
                    arm.fmt_depth(f, d + 1)?;
                }
                Ok(())
            }
            Stmt::Defer(s) => {
                write_indent(f, d)?;
                writeln!(f, "Defer")?;
                s.call.fmt_depth(f, d + 1)
            }
            Stmt::Guard(s) => {
                write_indent(f, d)?;
                writeln!(f, "Guard")?;
                s.condition.fmt_depth(f, d + 1)?;
                display_block(f, d + 1, "body:", &s.body)
            }
            Stmt::Unsafe(s) => {
                write_indent(f, d)?;
                writeln!(f, "Unsafe")?;
                display_block(f, d + 1, "body:", &s.body)
            }
        }
    }
}

impl DisplayDepth for MatchArm {
    fn fmt_depth(&self, f: &mut fmt::Formatter<'_>, d: usize) -> fmt::Result {
        write_indent(f, d)?;
        writeln!(f, "Arm")?;
        self.pattern.fmt_depth(f, d + 1)?;
        if let Some(g) = &self.guard {
            write_indent(f, d)?;
            writeln!(f, "guard")?;
            g.fmt_depth(f, d + 1)?;
        }
        display_block(f, d + 1, "body:", &self.body)
    }
}

impl DisplayDepth for Pattern {
    fn fmt_depth(&self, f: &mut fmt::Formatter<'_>, d: usize) -> fmt::Result {
        match self {
            Pattern::Identifier { name, .. } => {
                write_indent(f, d)?;
                writeln!(f, "Pattern::Ident \"{}\"", name)
            }
            Pattern::Literal { value, .. } => {
                write_indent(f, d)?;
                writeln!(f, "Pattern::Lit {}", value)
            }
            Pattern::Wildcard { .. } => {
                write_indent(f, d)?;
                writeln!(f, "Pattern::Wildcard")
            }
            Pattern::EnumVariant { enum_name, variant, args, .. } => {
                write_indent(f, d)?;
                writeln!(f, "Pattern::EnumVariant {}.{} ({} args)", enum_name, variant, args.len())
            }
            Pattern::IsType { type_, .. } => {
                write_indent(f, d)?;
                writeln!(f, "Pattern::IsType \"{}\"", type_)
            }
        }
    }
}

impl DisplayDepth for Expr {
    fn fmt_depth(&self, f: &mut fmt::Formatter<'_>, d: usize) -> fmt::Result {
        match self {
            Expr::Literal { value, .. } => {
                write_indent(f, d)?;
                writeln!(f, "Lit {}", value)
            }
            Expr::Identifier { name, .. } => {
                write_indent(f, d)?;
                writeln!(f, "Ident \"{}\"", name)
            }
            Expr::Binary { left, operator, right, .. } => {
                write_indent(f, d)?;
                writeln!(f, "BinaryOp {:?}", operator)?;
                left.fmt_depth(f, d + 1)?;
                right.fmt_depth(f, d + 1)
            }
            Expr::Unary { operator, operand, .. } => {
                write_indent(f, d)?;
                writeln!(f, "UnaryOp {:?}", operator)?;
                operand.fmt_depth(f, d + 1)
            }
            Expr::Assignment { target, operator, value, .. } => {
                write_indent(f, d)?;
                if let Some(op) = operator {
                    writeln!(f, "AssignOp {:?}", op)?;
                } else {
                    writeln!(f, "Assign")?;
                }
                target.fmt_depth(f, d + 1)?;
                value.fmt_depth(f, d + 1)
            }
            Expr::FunctionCall { target, arguments, .. } => {
                write_indent(f, d)?;
                writeln!(f, "Call")?;
                target.fmt_depth(f, d + 1)?;
                for arg in arguments {
                    arg.fmt_depth(f, d + 1)?;
                }
                Ok(())
            }
            Expr::PropertyAccess { object, property, .. } => {
                write_indent(f, d)?;
                writeln!(f, "PropAccess \"{}\"", property)?;
                object.fmt_depth(f, d + 1)
            }
            Expr::List { elements, .. } => {
                write_indent(f, d)?;
                writeln!(f, "List ({} elements)", elements.len())?;
                for elem in elements {
                    elem.fmt_depth(f, d + 1)?;
                }
                Ok(())
            }
            Expr::Dictionary { entries, .. } => {
                write_indent(f, d)?;
                writeln!(f, "Dict ({} entries)", entries.len())?;
                for (key, val) in entries {
                    write_indent(f, d + 1)?;
                    writeln!(f, "key=\"{}\"", key)?;
                    val.fmt_depth(f, d + 2)?;
                }
                Ok(())
            }
            Expr::StructLiteral { struct_name, fields, .. } => {
                write_indent(f, d)?;
                writeln!(f, "StructLiteral \"{}\" ({} fields)", struct_name, fields.len())?;
                for (key, val) in fields {
                    write_indent(f, d + 1)?;
                    writeln!(f, "{}", key)?;
                    val.fmt_depth(f, d + 2)?;
                }
                Ok(())
            }
            Expr::Tuple { elements, .. } => {
                write_indent(f, d)?;
                writeln!(f, "Tuple ({} elements)", elements.len())?;
                for elem in elements {
                    elem.fmt_depth(f, d + 1)?;
                }
                Ok(())
            }
            Expr::Closure { params, body, .. } => {
                write_indent(f, d)?;
                writeln!(f, "Closure params={:?}", params)?;
                body.fmt_depth(f, d + 1)
            }
            Expr::Await { expression, .. } => {
                write_indent(f, d)?;
                writeln!(f, "Await")?;
                expression.fmt_depth(f, d + 1)
            }
            Expr::Async { expression, .. } => {
                write_indent(f, d)?;
                writeln!(f, "Async")?;
                expression.fmt_depth(f, d + 1)
            }
            Expr::Spread { expression, .. } => {
                write_indent(f, d)?;
                writeln!(f, "Spread")?;
                expression.fmt_depth(f, d + 1)
            }
            Expr::Index { target, index, .. } => {
                write_indent(f, d)?;
                writeln!(f, "Index")?;
                target.fmt_depth(f, d + 1)?;
                index.fmt_depth(f, d + 1)
            }
            Expr::RangeSlice { target, start, end, .. } => {
                write_indent(f, d)?;
                writeln!(f, "RangeSlice")?;
                target.fmt_depth(f, d + 1)?;
                if let Some(s) = start {
                    write_indent(f, d)?;
                    writeln!(f, "start")?;
                    s.fmt_depth(f, d + 1)?;
                }
                if let Some(e) = end {
                    write_indent(f, d)?;
                    writeln!(f, "end")?;
                    e.fmt_depth(f, d + 1)?;
                }
                Ok(())
            }
            Expr::OptionalChain { target, property, .. } => {
                write_indent(f, d)?;
                writeln!(f, "OptionalChain \"{}\"", property)?;
                target.fmt_depth(f, d + 1)
            }
            Expr::Loop { body, .. } => {
                write_indent(f, d)?;
                writeln!(f, "Loop")?;
                display_block(f, d + 1, "body:", body)
            }
            Expr::ErrorProp { expression, .. } => {
                write_indent(f, d)?;
                writeln!(f, "ErrorProp")?;
                expression.fmt_depth(f, d + 1)
            }
            Expr::StringInterp { parts, .. } => {
                write_indent(f, d)?;
                writeln!(f, "StringInterp ({} parts)", parts.len())?;
                for part in parts {
                    part.fmt_depth(f, d + 1)?;
                }
                Ok(())
            }
            Expr::Ternary { cond, then_expr, else_expr, .. } => {
                write_indent(f, d)?;
                writeln!(f, "Ternary")?;
                write_indent(f, d + 1)?;
                writeln!(f, "Cond:")?;
                cond.fmt_depth(f, d + 2)?;
                write_indent(f, d + 1)?;
                writeln!(f, "Then:")?;
                then_expr.fmt_depth(f, d + 2)?;
                write_indent(f, d + 1)?;
                writeln!(f, "Else:")?;
                else_expr.fmt_depth(f, d + 2)
            }
            Expr::MatchExpr { expression, arms, .. } => {
                write_indent(f, d)?;
                writeln!(f, "MatchExpr")?;
                write_indent(f, d + 1)?;
                writeln!(f, "Expression:")?;
                expression.fmt_depth(f, d + 2)?;
                for arm in arms {
                    write_indent(f, d + 1)?;
                    writeln!(f, "Arm:")?;
                    arm.fmt_depth(f, d + 2)?;
                }
                Ok(())
            }
        }
    }
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Literal::Integer(n) => write!(f, "{}", n),
            Literal::Float(n) => write!(f, "{}", n),
            Literal::String(s) => write!(f, "\"{}\"", s),
            Literal::Boolean(b) => write!(f, "{}", b),
            Literal::None => write!(f, "None"),
        }
    }
}

impl fmt::Display for AstType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AstType::Primitive { name, .. } => write!(f, "{}", name),
            AstType::User { name, .. } => write!(f, "{}", name),
            AstType::Generic { name, args, .. } => {
                write!(f, "{}[", name)?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", arg)?;
                }
                write!(f, "]")
            }
            AstType::Optional { inner, .. } => write!(f, "Option<{}>", inner),
            AstType::Error { inner, .. } => write!(f, "Result<{}>", inner),
            AstType::Dict { key, value, .. } => write!(f, "Dict<{}, {}>", key, value),
        }
    }
}
