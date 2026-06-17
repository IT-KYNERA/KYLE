// klc_core::symbol — Interned symbol table

pub type Symbol = usize;

pub struct SymbolTable {
    strings: Vec<String>,
}
