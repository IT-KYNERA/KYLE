# KL Package Manager Specification v1.0

---

## Philosophy

The KL package manager is built-in. No third-party tools needed.

```text
No npm
No pip
No cargo install
No gem install
No nuget
Everything is: kl
```

---

## Design Goals

```text
Fast dependency resolution
Deterministic builds
Simple configuration
Secure packages
Easy publishing
Workspace support
Monorepo friendly
Offline support
```

---

## Commands

### Initialize Project

```bash
kl init
```

Creates:

```text
kl.toml
src/main.kl
.gitignore
```

### Initialize with Name

```bash
kl init my_app
```

### Build

```bash
kl build
```

Compiles the project.

### Build Release

```bash
kl build --release
```

### Run

```bash
kl run
```

### Run Specific File

```bash
kl run main.kl
```

### Test

```bash
kl test
```

### Run Specific Test

```bash
kl test test_name
```

### Add Dependency

```bash
kl add web@1.0
```

### Add Latest

```bash
kl add web
```

### Add Dev Dependency

```bash
kl add testing --dev
```

### Remove Dependency

```bash
kl remove web
```

### Update Dependencies

```bash
kl update
```

### Publish Package

```bash
kl publish
```

### Install from Registry

```bash
kl install web
```

### Format Code

```bash
kl fmt
```

### Check/Lint

```bash
kl check
```

### Doctor

```bash
kl doctor
```

Checks KL installation.

### Clean

```bash
kl clean
```

### Info

```bash
kl info
```

### Tree

```bash
kl tree
```

Shows dependency tree.

---

## Package Manifest

File: `kl.toml`

```toml
name = "my_app"
version = "1.0.0"
edition = "1"
authors = ["Kynera"]
license = "MIT"
description = "My KL application"

[compiler]
optimization = "O2"
target = "native"
debug = false

[dependencies]
web = "1.0"
json = "2.0"
database = "1.5"

[dev-dependencies]
testing = "1.0"

[features]
default = ["full"]
full = ["database", "web"]
lite = []
```

---

## Dependency Resolution

### Version Format

```text
MAJOR.MINOR.PATCH
```

### Version Constraints

```toml
web = "1.0"           # >=1.0.0 <2.0.0
web = "1.2.0"         # exact 1.2.0
web = ">=1.0"         # >=1.0.0
web = "<2.0"          # <2.0.0
web = ">=1.0 <2.0"    # range
web = "*"             # any version
```

### Resolution Algorithm

```text
1. Read kl.toml
2. Build dependency graph
3. Find compatible versions for each dependency
4. Check for conflicts
5. Lock exact versions in kl.lock
6. Download missing packages
7. Cache downloaded packages
```

---

## Package Registry

### Default Registry

```text
https://registry.kl-lang.org
```

### Custom Registry

```toml
[registry]
url = "https://registry.mycompany.com"
```

### Private Packages

```toml
[dependencies]
internal-lib = { registry = "private", version = "1.0" }
```

---

## Package Structure

```text
my_package/
├── kl.toml
├── src/
│   ├── main.kl
│   └── lib.kl
├── tests/
├── examples/
├── README.md
└── LICENSE
```

---

## Lock File

File: `kl.lock`

```text
Purpose: Deterministic builds

Generated automatically
Committed to version control
Pinned exact versions
Content-hash verification
```

Example:

```toml
version = 1

[[packages]]
name = "web"
version = "1.2.0"
checksum = "sha256:abc123..."

[[packages]]
name = "json"
version = "2.0.1"
checksum = "sha256:def456..."
```

---

## Package Resolution Order

```text
1. Local path dependencies
2. Workspace members
3. Registry packages
4. Git dependencies
5. Standard library
```

---

## Local Dependencies

```toml
[dependencies]
core = { path = "../core" }
```

---

## Git Dependencies

```toml
[dependencies]
web = { git = "https://github.com/company/web", tag = "v1.0" }
```

```toml
web = { git = "https://github.com/company/web", branch = "main" }
```

```toml
web = { git = "https://github.com/company/web", rev = "abc123" }
```

---

## Workspace

Root `kl.toml`:

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

## Publishing

### Requirements

```text
Valid kl.toml
Version bumped
Tests pass
License specified
README exists
```

### Publish Command

```bash
kl publish
```

### Version Bump

```bash
kl publish --patch
kl publish --minor
kl publish --major
```

---

## Cache

Location:

```text
~/.kl/cache/
```

Structure:

```text
~/.kl/cache/
├── packages/
│   ├── web-1.0.0/
│   └── json-2.0.0/
├── registry/
│   └── index.json
└── temp/
```

---

## Offline Support

```text
Cached packages used first
No network required for cached builds
kl update refreshes cache
```

---

## Security

```text
Checksum verification
HTTPS for registry
Package signing (future)
Dependency audit (future)
Vulnerability scanning (future)
```

---

## Version

```text
KL Package Manager Specification v1.0
```
