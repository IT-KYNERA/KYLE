# 06-compiler

> Documentación del compilador de Kyle: pipeline completo, desde el código fuente hasta el binario.

## Archivos

| Documento | Descripción | Estado |
|-----------|-------------|--------|
| `overview.md` | Pipeline general y arquitectura del compilador | ✅ |
| `pipeline.md` | Orchestrador que conecta todas las fases | ✅ |
| `lexer.md` | Tokenización: fuente → tokens | ✅ |
| `parser.md` | Parsing: tokens → AST | ✅ |
| `ast.md` | Definiciones del Abstract Syntax Tree | ✅ |
| `hir.md` | High-Level IR: desugaring | ✅ |
| `semantic.md` | Type checking, scope resolution | ✅ |
| `mir.md` | Mid-Level IR: lowering | ✅ |
| `borrow-analysis.md` | Análisis de ownership y borrows | ✅ |
| `ssa.md` | Transformación a SSA form | ✅ |
| `optimizer.md` | Optimizaciones a nivel MIR | ✅ |
| `codegen.md` | Generación de LLVM IR | ✅ |
| `backend.md` | Configuración LLVM target | ✅ |
| `linker.md` | Linking con runtime library | ✅ |
| `diagnostics.md` | Sistema de errores y warnings | ✅ |
| `incremental.md` | Compilación incremental | 🔶 Diseñado, no implementado |
| `wasm.md` | Target WebAssembly | ✅ |
