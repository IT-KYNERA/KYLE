// klc_core::types — Type system representations for the semantic layer
//
// Reference: docs/04-type-system.md
// These are the *semantic* types used after type resolution,
// distinct from AstType which represents type syntax in the AST.

/// A resolved type in the KL type system.
#[derive(Clone, Debug, PartialEq)]
pub enum Type {
    // Primitives
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
    Bool,
    Char,
    Str,
    Void,

    // Named types (user-defined: classes, structs, enums)
    Named(String),

    // Generics
    Generic(String, Vec<Type>),
    TypeVar(usize),

    // Optional
    Option(Box<Type>),

    // Error (fallible)
    Error(Box<Type>),

    // Callable
    Function(FunctionType),

    // Collections
    List(Box<Type>),
    Dict(Box<Type>, Box<Type>),
    Set(Box<Type>),

    // Object literal (structural)
    Object(Vec<(String, Type)>),

    // Tuple
    Tuple(Vec<Type>),
}

/// Represents a function signature in the type system.
#[derive(Clone, Debug, PartialEq)]
pub struct FunctionType {
    pub is_async: bool,
    pub is_const: bool,
    pub params: Vec<Type>,
    pub return_: Box<Type>,
    pub fallible: bool,
}

/// Overflow behavior for integer operations.
#[derive(Clone, Debug, PartialEq)]
pub enum OverflowBehavior {
    /// Panic in debug, wrap in release
    DebugPanicReleaseWrap,
    /// Explicit wrapping (the +%, -%, *% operators)
    ExplicitWrap,
}
