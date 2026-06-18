// klc_semantic — Type checking and symbol resolution
//
// Depends on: klc_core
//
// Responsibilities:
//   - Symbol table construction
//   - Scope resolution (including `this` handling)
//   - Type inference (Hindley-Milner based)
//   - Generic monomorphization
//   - Contract validation
//   - Error safety validation
//   - Optional safety validation

pub mod symbol_table;
pub mod type_checker;
pub mod scope;
pub mod contracts;
pub mod analyzer;
pub mod module_resolver;
