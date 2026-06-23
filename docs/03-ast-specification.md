# Kyle AST Specification v1.0

## Introduction

The Abstract Syntax Tree (AST) is the internal representation of a Kyle source file after parsing.

The parser transforms source code into AST nodes.

Example:

```kl
age = 10 + 20
```

Produces:

```text
VariableDeclarationNode

├── name: age

└── BinaryExpressionNode

    ├── left: IntegerNode(10)

    ├── operator: +

    └── right: IntegerNode(20)
```

---

# Root Node

## ProgramNode

Represents a complete source file.

```text
ProgramNode

├── declarations[]
```

Example:

```kl
import math

fn main():
```

AST:

```text
ProgramNode

├── ImportNode

└── FunctionNode
```

---

# Declaration Nodes

## ImportNode

```text
ImportNode

├── module_name

├── alias?
```

Examples:

```kl
import math

import database as db
```

---

## FromImportNode

```text
FromImportNode

├── module_name

├── imported_name
```

Example:

```kl
from math import sqrt
```

---

## VariableDeclarationNode

```text
VariableDeclarationNode

├── name

├── type?

└── value
```

Examples:

```kl
name = "John"

age: i32 = 25
```

---

## ConstantDeclarationNode

```text
ConstantDeclarationNode

├── name

└── value
```

Example:

```kl
PI = 3.14
```

---

# Function Nodes

## FunctionNode

```text
FunctionNode

├── name

├── parameters[]

├── return_type?

├── is_async

├── is_const

├── is_abstract

└── body?
```

Example:

```kl
fn greet(name: str) -> str:
```

---

## ParameterNode

```text
ParameterNode

├── name

└── type
```

---

## ReturnNode

```text
ReturnNode

└── expression
```

Example:

```kl
return name
```

---

# Class Nodes

## ClassNode

```text
ClassNode

├── name

├── parent?

├── contracts[]

└── members[]
```

Example:

```kl
class User:
```

---

## AbstractClassNode

```text
AbstractClassNode

├── name

├── parent?

├── contracts[]

└── members[]
```

Example:

```kl
abs class Animal:
```

---

## ConstructorNode

```text
ConstructorNode

├── parameters[]

└── body
```

Example:

```kl
User(name: str):
```

---

## FieldNode

```text
FieldNode

├── name

├── type

└── visibility
```

---

## PropertyNode

```text
PropertyNode

├── name

├── type

├── getter?

└── setter?
```

Example:

```kl
name: str:

    get:

    set(value):
```

---

# Contract Nodes

## ContractNode

```text
ContractNode

├── name

└── methods[]
```

Example:

```kl
contract Serializable:
```

---

## ContractMethodNode

```text
ContractMethodNode

├── name

├── parameters[]

└── return_type?
```

---

# Struct Nodes

## StructNode

```text
StructNode

├── name

└── fields[]
```

Example:

```kl
struct Point:
```

---

# Enum Nodes

## EnumNode

```text
EnumNode

├── name

└── variants[]
```

Example:

```kl
enum Status:
```

---

## EnumVariantNode

```text
EnumVariantNode

└── name
```

---

# Statement Nodes

## BlockNode

```text
BlockNode

├── statements[]
```

---

## ExpressionStatementNode

```text
ExpressionStatementNode

└── expression
```

---

## IfNode

```text
IfNode

├── condition

├── body

├── elif_branches[]

└── else_branch?
```

---

## ElifNode

```text
ElifNode

├── condition

└── body
```

---

## WhileNode

```text
WhileNode

├── condition

└── body
```

---

## ForNode

```text
ForNode

├── variable

├── iterable

└── body
```

---

## MatchNode

```text
MatchNode

├── expression

└── cases[]
```

---

## MatchCaseNode

```text
MatchCaseNode

├── pattern

└── body
```

---

## BreakNode

```text
BreakNode
```

Example:

```kl
break
```

---

# Expression Nodes

## IdentifierNode

```text
IdentifierNode

└── name
```

Example:

```kl
user
```

---

## IntegerNode

```text
IntegerNode

└── value
```

---

## FloatNode

```text
FloatNode

└── value
```

---

## StringNode

```text
StringNode

└── value
```

---

## BooleanNode

```text
BooleanNode

└── value
```

---

## BinaryExpressionNode

```text
BinaryExpressionNode

├── left

├── operator

└── right
```

Example:

```kl
a + b
```

---

## UnaryExpressionNode

```text
UnaryExpressionNode

├── operator

└── operand
```

Example:

```kl
!active
```

---

## AssignmentNode

```text
AssignmentNode

├── target

└── value
```

Example:

```kl
age = 25
```

---

## FunctionCallNode

```text
FunctionCallNode

├── target

└── arguments[]
```

Example:

```kl
print(name)
```

---

## PropertyAccessNode

```text
PropertyAccessNode

├── object

└── property
```

Example:

```kl
user.name
```

---

## ListNode

```text
ListNode

└── elements[]
```

Example:

```kl
[
    1,
    2,
    3
]
```

---

## DictionaryNode

```text
DictionaryNode

└── entries[]
```

---

## DictionaryEntryNode

```text
DictionaryEntryNode

├── key

└── value
```

Example:

```kl
{
    name: "John"
}
```

---

## TupleNode

```text
TupleNode

└── elements[]
```

Example:

```kl
(
    10,
    20
)
```

---

## ClosureNode

```text
ClosureNode

├── parameters[]

└── expression
```

Example:

```kl
(x) => x * 2
```

---

## AwaitNode

```text
AwaitNode

└── expression
```

Example:

```kl
await load_users()
```

---

## TernaryNode

```text
TernaryNode

├── condition

├── then_expr

└── else_expr
```

The ternary operator evaluates `condition`, then returns `then_expr` if truthy, `else_expr` otherwise.

Example:

```kl
status = age >= 18 ? "Adult" : "Minor"
```

---

# Type Nodes

## PrimitiveTypeNode

```text
PrimitiveTypeNode

└── name
```

Examples:

```text
i32
f64
bool
str
```

---

## UserTypeNode

```text
UserTypeNode

└── name
```

Example:

```text
User
```

---

## GenericTypeNode

```text
GenericTypeNode

├── name

└── type_arguments[]
```

Example:

```text
list<User>
```

---

## OptionalTypeNode

```text
OptionalTypeNode

└── wrapped_type
```

Example:

```text
Option<User>
```

---

## ErrorTypeNode

```text
ErrorTypeNode

└── wrapped_type
```

Example:

```text
User!
```

---

# AST Processing Pipeline

```text
Source Code

↓

Lexer

↓

Tokens

↓

Parser

↓

AST

↓

Semantic Analyzer

↓

Type Checker

↓

LLVM IR

↓

Machine Code
```

---

# AST Design Principles

```text
Immutable Nodes

Strongly Typed Nodes

No Runtime Information

No LLVM Information

Language Independent

Easy to Optimize

Easy to Traverse
```

---

# End of AST Specification
