// parser/mod.rs

mod base;
mod notation;

use crate::syntax::Expr;
use nom::error::convert_error;
use nom::Finish;
use std::rc::Rc;

pub use self::base::parse_program as parse_base_program;
pub use self::notation::apply_notations;

pub fn parse_program(input: &str) -> Result<Vec<Rc<Expr>>, String> {
    match parse_base_program(input).finish() {
        Ok((_, exprs)) => apply_notations(exprs),
        Err(e) => Err(convert_error(input, e)),
    }
}
