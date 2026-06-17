// klc_backend — LLVM code generation
//
// Generates native machine code via LLVM.
// Depends on: klc_core, inkwell (LLVM bindings)
//
// Uses Homebrew LLVM at: /opt/homebrew/opt/llvm
// LLVM version: 22.1.7
//
// Configuration in: .cargo/config.toml (LLVM_SYS_220_PREFIX)
//
// NOTE: Backend is placeholder until Phase 3.
// During Phase 1, this crate exists as a stub.

pub mod codegen;
pub mod linker;
