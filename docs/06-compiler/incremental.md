# Incremental Compilation

> Compilación incremental: solo recompilar archivos modificados.
> **Pendiente de implementación.**

## Estado

La compilación incremental NO está implementada actualmente.
Cada compilación procesa el proyecto completo desde cero.

## Diseño propuesto

### Cache de módulos

```rust
struct IncrementalCache {
    modules: HashMap<PathBuf, CachedModule>,
}

struct CachedModule {
    hash: u64,              // Hash del contenido del archivo
    ast: Module,            // AST cacheado
    mir: MirModule,         // MIR cacheado
    object_path: PathBuf,   // .o file
}
```

### Pipeline

```rust
fn compile_incremental(project: &Project, cache: &mut IncrementalCache) {
    for file in &project.files {
        let current_hash = hash_file(file);
        let cached = cache.get(file);
        
        if cached.map_or(false, |c| c.hash == current_hash) {
            // No changes, use cached
            continue;
        }
        
        // Compile from scratch
        let mir = compile_file(file);
        cache.insert(file, CachedModule { hash: current_hash, mir, ... });
    }
    
    // Link all (changed and unchanged)
    link_all(cache.modules.values());
}
```

### Dependencias

Cuando un módulo cambia, todos los que dependen de él deben recompilarse:

```rust
fn resolve_dependencies(project: &Project) -> DependencyGraph {
    // Analizar imports de cada archivo
    // Construir grafo de dependencias
    // Si A importa B, cuando B cambia, A debe recompilarse
}
```

## Ver también

- `overview.md` — Pipeline completo de compilación
- `pipeline.md` — Orchestación actual (sin incremental)
