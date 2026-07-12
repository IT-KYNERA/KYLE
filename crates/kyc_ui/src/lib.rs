pub mod ast;
pub mod parser;
pub mod ir;
pub mod backend;
pub mod app_config;

// Legacy modules (will be removed in future — functionality migrated to backend::web)
pub mod js_gen;
pub mod style_gen;
pub mod anim_gen;
