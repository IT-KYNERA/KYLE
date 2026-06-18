// klc_tools — Developer tooling
//
// Depends on: klc_core, klc_frontend
//
// Responsibilities:
//   - LSP (Language Server Protocol) implementation
//   - Code formatter (kl fmt)
//   - Code completion
//   - Diagnostics
//   - Refactoring tools

pub mod lsp;
pub mod formatter;
pub mod completion;
pub mod package;
