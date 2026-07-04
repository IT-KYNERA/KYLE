# Package Registry System

## How it works

Kyle packages are distributed through a registry — an HTTP server that serves package metadata and tarballs.

### Registry API

| Endpoint | Method | Response |
|----------|--------|----------|
| `GET /packages/{name}` | Version list | `{ "versions": [{ "version": "0.1.0" }] }` |
| `GET /packages/{name}/{version}/dependencies` | Dependencies | `{ "dependencies": [{ "name": "...", "version": "..." }] }` |
| `GET /packages/{name}/{version}/download` | Binary tarball | `.tar.gz` file |

### Default registry

```
https://registry.kyle-lang.org/v1
```

Override with `KL_REGISTRY` environment variable.

## Local file registry (current)

Until the production registry server is built, packages are distributed via a **file registry** — a local directory structure that mimics the HTTP API.

### Structure

```
registry/
├── http/
│   ├── 0.1.0/                # package source
│   │   ├── ky.toml
│   │   └── src/lib.ky
│   └── 0.1.0.tar.gz          # tarball for download
├── json/
│   ├── 0.1.0/
│   │   ├── ky.toml
│   │   └── src/lib.ky
│   └── 0.1.0.tar.gz
└── sqlite/
    ├── 0.1.0/
    │   ├── ky.toml
    │   └── src/lib.ky
    └── 0.1.0.tar.gz
```

### How to use

```bash
# Point to local registry
export KL_REGISTRY=file:///path/to/ky/registry

# Add a package
ky add http
ky add json
ky add sqlite
```

### How to publish a new version

```bash
# 1. Create the package directory
mkdir -p registry/<name>/<version>/src
cp packages/<name>/ky.toml registry/<name>/<version>/
cp packages/<name>/src/lib.ky registry/<name>/<version>/src/

# 2. Create the tarball
cd registry
tar czf <name>/<version>.tar.gz -C <name>/<version> .

# 3. Commit and push to GitHub
git add registry/
git commit -m "registry: add <name> v<version>"
```

## Future: production registry server

The file registry is a temporary solution. The long-term plan is:

| Phase | Description | Status |
|-------|-------------|--------|
| **File registry** | Static files in repo, `KL_REGISTRY=file://` | ✅ Working |
| **GitHub registry** | Registry data hosted on GitHub Pages | 📅 |
| **Registry server** | Dedicated HTTP server with auth, yanking, etc. | 📅 |

### Directory vs HTTP mapping

| File registry path | HTTP endpoint |
|-------------------|---------------|
| `registry/http/` | `GET /packages/http` |
| `registry/http/0.1.0/ky.toml` | `GET /packages/http/0.1.0/ky.toml` |
| `registry/http/0.1.0.tar.gz` | `GET /packages/http/0.1.0/download` |
