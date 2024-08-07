// parser/mod.rs

mod base;
mod notation;

use crate::syntax::Expr;
use nom::Finish;

pub use self::base::parse_program as parse_base_program;
pub use self::notation::apply_notations;

pub fn parse_program(input: &str) -> Result<Vec<Expr>, String> {
    let (remaining, base_ast) = parse_base_program(input)
        .finish()
        .map_err(|e| format!("Parse error: {:?}", e))?;

    if !remaining.trim().is_empty() {
        eprintln!(
            "Warning: Parsing incomplete. Remaining input: {:?}",
            remaining.trim()
        );
    }

    apply_notations(base_ast)
}
