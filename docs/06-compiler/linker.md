# Linker

> Enlazado del código objeto con la librería runtime para producir el binario final.
> Crate: `kyc_backend/src/linker.rs` (189 líneas).

## Responsabilidad

El linker toma los archivos objeto (`.o`) generados por LLVM y los enlaza con
`libkyc_runtime.a` y las bibliotecas del sistema para producir un ejecutable nativo.

## Proceso

```rust
 fn link(&self, object_files: &[&Path], output: &Path, 
            runtime_lib: Option<&Path>, release: bool, links: &[String]) -> Result<(), String> {
    let mut cmd = linker_cmd();
    cmd.arg("-o").arg(output);
    
    // Optimization flags
    if release {
        cmd.arg("-O3");
        cmd.arg("-flto");   // Link-Time Optimization
    }
    
    // Object files
    for obj in object_files { cmd.arg(obj); }
    
    // Runtime library
    if let Some(runtime) = runtime_lib { cmd.arg(runtime); }
    
    // Additional libraries
    for link in links {
        if link.starts_with("-framework ") {
            cmd.arg("-framework");
            cmd.arg(framework_name);
        } else {
            cmd.arg(format!("-l{}", link));
        }
    }
    
    // macOS specific
    if release { cmd.arg("-lm"); }
    if macos { cmd.arg("-framework").arg("CoreFoundation"); }
    cmd.arg("-macosx_version_min").arg(&deployment_target);
    
    cmd.status()?;
}
```

## Runtime library discovery

```rust
 fn find_runtime_lib() -> Option<PathBuf> {
    // 1. Relative to ky binary
    //    /usr/local/bin/ky → /usr/local/lib/kl/libkyc_runtime.a
    // 2. Cargo workspace
    //    target/debug/libkyc_runtime.a
    //    target/release/libkyc_runtime.a
    // 3. Current directory
    //    ./target/debug/libkyc_runtime.a
}
```

La librería runtime se busca en (por orden):

| Path | Ejemplo |
|------|---------|
| Junto al binario `ky` | `~/.ky/bin/libkyc_runtime.a` |
| Workspace debug | `<project>/target/debug/libkyc_runtime.a` |
| Workspace release | `<project>/target/release/libkyc_runtime.a` |
| Directorio actual | `./target/debug/libkyc_runtime.a` |

## Link-Time Optimization (LTO)

En modo release, se habilita LTO con `-flto`, permitiendo que LLVM optimice
a través de los límites entre el código generado y la librería runtime.

## macOS specifics

- `-framework CoreFoundation` necesario para `chrono`/`iana-time-zone`
- `-macosx_version_min` detectado vía `sw_vers -productVersion`
- `MACOSX_DEPLOYMENT_TARGET` env var para override

## Verification

```llvm
$ clang -o output input.o libkyc_runtime.a -framework CoreFoundation -lm -O3 -flto
```

## Ver también

- `codegen.md` — Genera los .o que el linker consume
- `backend.md` — Configuración del target
- `05-runtime/` — Documentación del runtime
