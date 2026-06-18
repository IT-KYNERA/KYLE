// klc_mir — Mid-level IR and optimization
//
// Depends on: klc_core
//
// Responsibilities:
//   - MIR definition
//   - AST to MIR lowering
//   - Optimization passes
//   - Control flow analysis

pub mod mir;
pub mod lower;
pub mod optimize;
pub mod ownership;
