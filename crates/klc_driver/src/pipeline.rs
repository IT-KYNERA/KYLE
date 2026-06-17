use klc_core::ast::Program;
use klc_frontend::lexer::Lexer;
use klc_frontend::parser::Parser;

/// Compilation pipeline stages.
#[derive(Default)]
pub struct Pipeline;

impl Pipeline {
    /// Run the full frontend pipeline: source → lexical analysis → parsing → AST.
    pub fn parse_source(source: &str) -> Result<ParsedOutput, String> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let program = parser.parse()?;
        Ok(ParsedOutput { program })
    }
}

/// Output of the parse stage.
pub struct ParsedOutput {
    pub program: Program,
}
