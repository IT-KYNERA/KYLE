# KL Formal Grammar Specification v1.0

## Introduction

This document defines the formal grammar of the KL programming language.

Notation is based on EBNF (Extended Backus-Naur Form).

---

# Lexical Rules

## Identifiers

```ebnf
identifier =
    letter
    { letter | digit | "_" }
;
```

Examples:

```kl
name
user
user_name
_age
__password
Repository
```

---

## Letters

```ebnf
letter =
    "A"..."Z"
    | "a"..."z"
;
```

---

## Digits

```ebnf
digit =
    "0"..."9"
;
```

---

# Literals

## Integer

```ebnf
integer =
    digit
    { digit }
;
```

Hex:

```ebnf
hex_integer =
    "0x"
    hex_digit
    { hex_digit }
;
```

Binary:

```ebnf
binary_integer =
    "0b"
    "0" | "1"
    { "0" | "1" }
;
```

Examples:

```kl
0
10
25
1000
0xFF
0b1010
```

---

## Float

```ebnf
float =
    integer
    "."
    integer
;
```

Examples:

```kl
3.14
0.5
100.25
```

---

## String

```ebnf
string =
    '"'
    { character | interpolation }
    '"'
;

interpolation =
    "{"
    expression
    "}"
;
```

Examples:

```kl
"John"
"Hello World"
""
"Hello {name}, age {age}"
```

---

## Boolean

```ebnf
boolean =
    "true"
    | "false"
;
```

---

# Types

```ebnf
type =
      primitive_type
    | identifier
    | generic_type
    | optional_type
    | error_type
    | function_type
    | list_type
    | "void"
;

function_type =
    [ "async" ]
    "fn"
    "("
    [ type { "," type } ]
    ")"
    "->"
    type
;

list_type =
    "list"
    "<"
    type
    ">"
;
```

---

## Primitive Types

```ebnf
primitive_type =
      "i8"
    | "i16"
    | "i32"
    | "i64"
    | "u8"
    | "u16"
    | "u32"
    | "u64"
    | "f32"
    | "f64"
    | "bool"
    | "char"
    | "str"
    | "void"
;
```

---

## Optional Type

Optional types use the generic type `Option<T>`:

```ebnf
optional_type =
    "Option"
    "<"
    type
    ">"
;
```

Examples:

```kl
Option<str>
Option<User>
```

---

## Error Type

```ebnf
error_type =
    type
    "!"
;
```

Examples:

```kl
User!
File!
```

---

# Program

```ebnf
program =
    { declaration }
;
```

---

# Declarations

```ebnf
declaration =
    [ attribute ]
    ( variable_declaration
    | constant_declaration
    | function_declaration
    | class_declaration
    | abstract_class_declaration
    | struct_declaration
    | enum_declaration
    | contract_declaration
    | import_declaration
    | type_alias
    )
;

attribute =
    "#["
    identifier
    [ "(" attribute_args ")" ]
    "]"
;
```

---

# Variables

## Inferred Type

```ebnf
variable_declaration =
    [ "mut" ]
    identifier
    "="
    expression
;
```

Example:

```kl
age = 25        # immutable (default)
mut score = 10  # mutable
```

---

## Explicit Type

```ebnf
typed_variable =
    [ "mut" ]
    identifier
    ":"
    type
    "="
    expression
;
```

Example:

```kl
age: i32 = 25
```

---

# Constants

Constants use UPPERCASE naming convention.
Syntactically identical to variable declarations (without `mut`).

```ebnf
constant_declaration =
    UPPERCASE_IDENTIFIER
    "="
    expression
;
```

The compiler enforces immutability based on the ALL_CAPS naming.
`mut` is NOT allowed with UPPERCASE constants (compile error).

Example:

```kl
PI = 3.141592
```

---

# Functions

```ebnf
function_declaration =
    [ attribute ]
    [ "const" ]
    [ "async" ]
    [ "abs" ]
    "fn"
    identifier
    "("
    parameter_list
    ")"
    [ return_type ]
    [ "!" ]
    ":"
    [ block ]
;
```

---

## Parameters

```ebnf
parameter_list =
    [ parameter
    { "," parameter } ]
    [ "," variadic_parameter ]
;
```

```ebnf
parameter =
    identifier
    ":"
    type
    [ "=" expression ]
;
```

```ebnf
variadic_parameter =
    "..."
    identifier
    ":"
    type
;
```

---

## Return Type

```ebnf
return_type =
    "->"
    type
;
```

---

# Closures

```ebnf
closure =
    "("
    parameter_names
    ")"
    "=>"
    expression
;
```

Example:

```kl
double = (x) => x * 2
```

---

# Classes

```ebnf
class_declaration =
    "class"
    identifier
    [ inheritance ]
    ":"
    block
;
```

---

## Abstract Classes

```ebnf
abstract_class_declaration =
    "abs" "class"
    identifier
    [ inheritance ]
    ":"
    block
;
```

---

## Inheritance

```ebnf
inheritance =
    ":"
    identifier
;
```

Example:

```kl
class Dog : Animal:
```

---

# Contracts

```ebnf
contract_declaration =
    "contract"
    identifier
    ":"
    block
;
```

---

# Structs

```ebnf
struct_declaration =
    "struct"
    identifier
    ":"
    block
;
```

---

# Enums

```ebnf
enum_declaration =
    "enum"
    identifier
    ":"
    block
;
```

---

# Properties

```ebnf
property_declaration =
    identifier
    ":"
    type
    ":"
    property_block
;
```

---

# Conditionals

```ebnf
if_statement =
    "if"
    expression
    ":"
    block
    {
        "elif"
        expression
        ":"
        block
    }
    [
        "else"
        ":"
        block
    ]
;
```

---

# Match

```ebnf
match_statement =
    "match"
    expression
    ":"
    block
;

match_arm =
    pattern
    [ "if" expression ]
    ":"
    block
;

pattern =
      identifier
    | literal
    | "_"
    | enum_variant_pattern
    | "is" type
;

enum_variant_pattern =
    identifier
    "." identifier
    [ "(" pattern { "," pattern } ")" ]
;
```

---

# Binding Condition

```ebnf
binding_if =
    "if"
    identifier
    "="
    expression
    ":"
    block
    [
        "elif" identifier "=" expression ":" block
    ]
    [
        "else" ":" block
    ]
;
```

---

# While

```ebnf
while_statement =
    "while"
    expression
    ":"
    block
    [ "else" ":" block ]
;
```

---

# While-Bind

```ebnf
while_bind =
    "while"
    identifier
    "="
    expression
    ":"
    block
;
```

---

# For

```ebnf
for_statement =
    "for"
    identifier
    "in"
    expression
    ":"
    block
    [ "else" ":" block ]
;
```

---

# Break

```ebnf
break_statement =
    "break"
    [ expression ]
;
```

---

# Return

```ebnf
return_statement =
    "return"
    expression
;
```

---

# Defer

```ebnf
defer_statement =
    "defer"
    expression
;
```

---

# Guard

```ebnf
guard_statement =
    "guard"
    expression
    "else"
    ":"
    block
;
```

---

# Unsafe

```ebnf
unsafe_block =
    "unsafe"
    ":"
    block
;
```

---

# Type Alias

```ebnf
type_alias =
    "type"
    identifier
    "="
    type
;
```

---

# Expressions

```ebnf
expression =
      literal
    | identifier
    | function_call
    | binary_expression
    | unary_expression
    | property_access
    | list
    | dictionary
    | tuple
    | closure
    | await_expression
    | async_expression
    | spread_expression
    | range_slice
    | optional_chain
    | loop_expression
    | error_propagation
    | "(" expression ")"
;

async_expression =
    "async"
    expression
;

spread_expression =
    "..."
    expression
;

range_slice =
    expression
    "["
    [ expression ]
    ".."
    [ expression ]
    "]"
;

optional_chain =
    expression
    "?"
    "."
    identifier
;

loop_expression =
    "loop"
    ":"
    block
;
```

---

# Function Call

```ebnf
function_call =
    identifier
    "("
    argument_list
    ")"
;
```

---

# Arguments

```ebnf
argument_list =
    [ argument
    { "," argument } ]
;

argument =
    expression
    | named_argument
;

named_argument =
    identifier
    ":"
    expression
;
```

---

# Binary Operators

```ebnf
binary_operator =
      "+"
    | "-"
    | "*"
    | "/"
    | "%"
    | "**"
    | "=="
    | "!="
    | ">"
    | "<"
    | ">="
    | "<="
    | "&&"
    | "||"
    | "&"
    | "|"
    | "^"
    | "<<"
    | ">>"
    | "is"
    | "+%"
    | "-%"
    | "*%"
;

error_propagation =
    expression
    "?"
;
```

---

# Unary Operators

```ebnf
unary_operator =
      "!"
    | "-"
    | "~"
;
```

---

# Lists

```ebnf
list =
    "["
    [ list_element
      { "," list_element } ]
    "]"
;

list_element =
    expression
    | spread_expression
;
```

---

# Object Literals

```ebnf
dictionary =
    "{"
    [ object_entry
      { "," object_entry } ]
    "}"
;

object_entry =
    key_value
    | spread_expression
;
```

---

## Object Entry

```ebnf
key_value =
    identifier
    ":"
    expression
;
```

Object literals are structurally typed. Access is via dot notation.

For dynamic Dict types, use bracket access:

```ebnf
dict_access =
    expression
    "["
    expression
    "]"
;
```

---

# Tuples

```ebnf
tuple =
    "("
    expression
    ","
    [ expression
      { "," expression } ]
    ")"
;
```

---

# Property Access

```ebnf
property_access =
    expression
    "."
    identifier
;
```

Examples:

```kl
user.name

config.database.host
```

---

# Imports

```ebnf
import_declaration =
      module_import
    | from_import
;
```

---

## Module Import

```ebnf
module_import =
    "import"
    identifier
;
```

---

## Alias Import

```ebnf
alias_import =
    "import"
    identifier
    "as"
    identifier
;
```

---

## From Import

```ebnf
from_import =
    "from"
    identifier
    "import"
    identifier
;
```

---

# Async

```ebnf
await_expression =
    "await"
    expression
;
```

---

# Statements

```ebnf
statement =
      expression
    | variable_declaration
    | typed_variable
    | constant_declaration
    | return_statement
    | break_statement
    | if_statement
    | binding_if
    | while_statement
    | while_bind
    | for_statement
    | match_statement
    | defer_statement
    | guard_statement
    | unsafe_block
    | function_call
;
```

# Blocks

```ebnf
block =
    INDENT
    { statement }
    DEDENT
;
```

Rule:

```text
KL uses indentation-based blocks.

No braces {} exist.

No semicolons exist.
```

---

# End of Grammar
