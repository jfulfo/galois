// parser/mod.rs

mod base;
mod notation;

use crate::syntax::Expr;

pub use self::base::parse_program as parse_base_program;
pub use self::notation::apply_notations;

pub fn parse_program(input: &str) -> Result<Vec<Expr>, String> {
    let (_, base_ast) = parse_base_program(input).map_err(|e| e.to_string())?;
    apply_notations(base_ast)
}
