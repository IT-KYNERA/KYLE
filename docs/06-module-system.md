# KL Module System Specification v1.0

---

# Philosophy

The KL module system is designed around:

```text
Simplicity

Predictability

Fast Compilation

Scalability

Enterprise Projects

Monorepos

Dependency Isolation

Explicit Imports
```

---

# Design Goals

```text
No Circular Dependencies

No Hidden Imports

No Global Namespace Pollution

Fast Dependency Resolution

Simple Package Structure

Easy Refactoring

Enterprise Ready
```

---

# Core Concepts

KL organizes code using:

```text
Module

Package

Workspace

Project
```

Hierarchy:

```text
Workspace

 └── Project

      └── Package

           └── Module
```

---

# Module

A module is a single file.

Example:

```text
user.kl
```

```kl
class User:

    name: str
```

---

# Module Name Resolution

File:

```text
src/models/user.kl
```

Module Path:

```text
models.user
```

Import:

```kl
import models.user
```

---

# Imports

Import entire module:

```kl
import models.user
```

Usage:

```kl
user = user.User()
```

---

# Specific Imports

```kl
from models.user import User
```

Usage:

```kl
user = User()
```

---

# Multiple Imports

```kl
from models.user import (

    User,

    UserRole,

    UserStatus
)
```

---

# Alias Imports

```kl
import database.connection as db
```

Usage:

```kl
db.connect()
```

---

# Nested Modules

Structure:

```text
src/

    database/

        postgres/

            connection.kl
```

Import:

```kl
import database.postgres.connection
```

---

# Relative Imports

Current Module:

```text
src/services/auth.kl
```

Import sibling:

```kl
from .user import User
```

Import parent:

```kl
from ..database import Connection
```

---

# Visibility

Default:

```text
Public
```

---

# Private Members

Single underscore:

```kl
_secret: str

fn _validate()
```

Visible only inside module hierarchy.

---

# Internal Members

Double underscore:

```kl
__token: str

fn __encrypt()
```

Visible only inside current module.

---

# Visibility

KL uses naming conventions for visibility — no keywords needed.

```kl
name          # public — visible to all modules
_name         # protected — visible only inside the module hierarchy
__name        # private — visible only inside the current module
```

Rules:

```text
Public (default):  name without prefix. Visible everywhere.
Protected:         single underscore prefix. Visible in current module
                   and its sub-modules. Not visible to sibling modules.
Private:           double underscore prefix. Visible only in the
                   current file/module. Never exposed outside.
```

Examples:

```kl
class User:           # public — usable from any module

    name: str         # public field

    _role: str        # protected — only in module hierarchy

    __token: str      # private — only inside this file


fn _validate():       # protected

    ...

fn __encrypt():       # private

    ...
```

This model requires no `export`, `pub`, or `public` keyword.
Visibility is determined entirely by the name convention.

---

# FFI (Foreign Function Interface)

Call C functions directly:

```kl
extern fn printf(fmt: str, ...) -> i32
extern fn malloc(size: i64) -> *void
extern fn free(ptr: *void)
```

Usage:

```kl
unsafe:
    ptr = malloc(1024)
    free(ptr)
```

Link directives in `kl.toml`:

```toml
[ffi]
libraries = ["m", "pthread", "ssl"]
link_paths = ["/usr/local/lib"]
```

Rules:

```text
extern declares a foreign function signature (no body).
FFI calls must be inside an unsafe block.
Pointer types use *T syntax (only available in unsafe context).
```

---

# Unsafe Blocks

Escape hatch for low-level operations:

```kl
unsafe:
    ptr = memory_address
    value = ptr[0]
    ptr[0] = new_value
```

Allowed inside unsafe:

```text
Pointer arithmetic and dereference
FFI calls
Raw memory access
Type punning (reinterpreting memory)
```

The compiler must enforce that `unsafe` is explicit
and contained to the smallest possible scope.

---

# Package

A package is a directory.

Example:

```text
models/

    user.kl

    role.kl

    permission.kl
```

Package Name:

```text
models
```

---

# Package Entry Point

Optional:

```text
models/

    index.kl
```

Example:

```kl
from .user import User

from .role import Role
```

All imported symbols are publicly accessible through the package.

Usage:

```kl
from models import User
```

---

# Dependency Resolution

Compiler search order:

```text
Current Module

Current Package

Current Project

Workspace Packages

Installed Packages

Standard Library
```

---

# Standard Library Imports

```kl
import io

import math

import json

import filesystem
```

---

# Third Party Packages

Install:

```bash
kl add web
```

Import:

```kl
import web
```

---

# Package Manifest

File:

```text
kl.toml
```

Example:

```toml
name = "my_app"

version = "1.0.0"

edition = "1"

authors = ["Kynera"]

license = "MIT"
```

---

# Dependencies

```toml
[dependencies]

web = "1.0"

json = "2.0"

postgres = "1.5"
```

---

# Dev Dependencies

```toml
[dev-dependencies]

testing = "1.0"
```

Used only for:

```text
Tests

Benchmarks

Development Tools
```

---

# Workspace

A workspace contains multiple projects.

Structure:

```text
workspace/

    kl.toml

    apps/

        api/

        dashboard/

    libraries/

        auth/

        core/
```

---

# Workspace Manifest

```toml
[workspace]

members = [

    "apps/api",

    "apps/dashboard",

    "libraries/auth",

    "libraries/core"
]
```

---

# Monorepo Support

Example:

```text
kynera/

    apps/

        web/

        mobile/

        desktop/

    services/

        auth/

        billing/

        notifications/

    libraries/

        core/

        database/

        security/
```

---

# Local Package References

```toml
[dependencies]

core = { path = "../core" }
```

---

# Versioning

Format:

```text
MAJOR.MINOR.PATCH
```

Example:

```text
1.0.0

1.1.0

2.0.0
```

---

# Package Registry

Default:

```text
KL Registry
```

Commands:

```bash
kl publish

kl install

kl update

kl remove
```

---

# Lock File

Generated:

```text
kl.lock
```

Purpose:

```text
Deterministic Builds

Version Reproducibility

Dependency Integrity
```

---

# Compiler Rules

Rule 1:

```text
Imports must be explicit.
```

Rule 2:

```text
No wildcard imports.
```

Rejected:

```kl
from math import *
```

---

Rule 3:

```text
Circular dependencies are forbidden.
```

---

Rule 4:

```text
Module paths are case-sensitive.
```

---

Rule 5:

```text
Unused imports generate warnings.
```

---

# Build Graph

Compiler generates:

```text
Dependency Graph
```

Example:

```text
main

 ├── services.auth

 │    └── models.user

 └── database.connection
```

Purpose:

```text
Parallel Compilation

Incremental Compilation

Dependency Validation
```

---

# Incremental Compilation

Compiler caches:

```text
AST

Type Information

Module Metadata
```

Benefits:

```text
Faster Rebuilds

Large Project Scalability
```

---

# Enterprise Structure

Recommended Layout:

```text
src/

    app/

    domain/

    infrastructure/

    application/

    contracts/

    shared/

    tests/
```

---

# Layer Responsibilities

Domain:

```text
Business Rules
```

Application:

```text
Use Cases
```

Infrastructure:

```text
Database

Network

Filesystem
```

Contracts:

```text
Interfaces

DTOs

Public APIs
```

Shared:

```text
Utilities

Helpers

Extensions
```

---

# Future Features

Planned:

```text
Private Packages

Enterprise Registries

Package Signing

Package Verification

Binary Distribution

Remote Workspaces

Build Caching Servers
```

---

# Internal Compiler Representation

Module:

```text
ModuleNode
```

Package:

```text
PackageNode
```

Workspace:

```text
WorkspaceNode
```

Dependency:

```text
DependencyGraph
```

---

# Long-Term Goals

```text
Cargo-Level Experience

Go-Level Simplicity

Rust-Level Reliability

Enterprise Scalability

Monorepo First

Cloud Native
```

---

# Version

```text
KL Module System Specification v1.0
```
