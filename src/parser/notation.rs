// parser/notation.rs

use crate::syntax::{Expr, NotationPattern};
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone, Debug)]
struct Notation {
    pattern: NotationPattern,
    expansion: Rc<Expr>,
}

pub fn apply_notations(ast: Vec<Rc<Expr>>) -> Result<Vec<Rc<Expr>>, String> {
    let (notations, expressions): (Vec<_>, Vec<_>) = ast
        .into_iter()
        .partition(|expr| matches!(&**expr, Expr::NotationDecl(_, _)));

    let notations: Vec<Notation> = notations
        .into_iter()
        .filter_map(|expr| {
            if let Expr::NotationDecl(pattern, expansion) = &*expr {
                Some(Notation {
                    pattern: pattern.clone(),
                    expansion: Rc::clone(expansion),
                })
            } else {
                None
            }
        })
        .collect();

    expressions
        .into_iter()
        .map(|expr| expand_expr(expr, &notations))
        .collect()
}

fn expand_expr(expr: Rc<Expr>, notations: &[Notation]) -> Result<Rc<Expr>, String> {
    let expanded = match &*expr {
        Expr::FunctionDef(name, params, body) => Rc::new(Expr::FunctionDef(
            name.clone(),
            params.clone(),
            body.iter()
                .map(|e| expand_expr(Rc::clone(e), notations))
                .collect::<Result<_, _>>()?,
        )),
        Expr::FunctionCall(func, args) => {
            let expanded_func = expand_expr(Rc::clone(func), notations)?;
            let expanded_args = args
                .iter()
                .map(|arg| expand_expr(Rc::clone(arg), notations))
                .collect::<Result<Vec<_>, _>>()?;
            Rc::new(Expr::FunctionCall(expanded_func, expanded_args))
        }
        Expr::Return(e) => Rc::new(Expr::Return(expand_expr(Rc::clone(e), notations)?)),
        Expr::Assignment(name, e) => Rc::new(Expr::Assignment(
            name.clone(),
            expand_expr(Rc::clone(e), notations)?,
        )),
        Expr::InfixOp(left, op, right) => {
            let expanded_left = expand_expr(Rc::clone(left), notations)?;
            let expanded_right = expand_expr(Rc::clone(right), notations)?;
            Rc::new(Expr::InfixOp(expanded_left, op.clone(), expanded_right))
        }
        _ => Rc::clone(&expr),
    };

    // Try to match and expand notations
    for notation in notations {
        if let Some(bindings) = match_pattern(&expanded, &notation.pattern) {
            return expand_notation(&notation.expansion, &bindings);
        }
    }

    Ok(expanded)
}

fn match_pattern(expr: &Expr, pattern: &NotationPattern) -> Option<HashMap<String, Rc<Expr>>> {
    match expr {
        Expr::InfixOp(left, op, right) if pattern.pattern == format!("$x {} $y", op) => {
            let mut bindings = HashMap::new();
            bindings.insert("x".to_string(), Rc::clone(left));
            bindings.insert("y".to_string(), Rc::clone(right));
            Some(bindings)
        }
        _ => None,
    }
}

fn expand_notation(
    expansion: &Expr,
    bindings: &HashMap<String, Rc<Expr>>,
) -> Result<Rc<Expr>, String> {
    match expansion {
        Expr::Variable(name) => Ok(bindings
            .get(name)
            .cloned()
            .unwrap_or_else(|| Rc::new(Expr::Variable(name.clone())))),
        Expr::FunctionCall(func, args) => {
            let expanded_func = expand_notation(func, bindings)?;
            let expanded_args = args
                .iter()
                .map(|arg| expand_notation(arg, bindings))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Rc::new(Expr::FunctionCall(expanded_func, expanded_args)))
        }
        Expr::InfixOp(left, op, right) => {
            let expanded_left = expand_notation(left, bindings)?;
            let expanded_right = expand_notation(right, bindings)?;
            Ok(Rc::new(Expr::InfixOp(
                expanded_left,
                op.clone(),
                expanded_right,
            )))
        }
        _ => Ok(Rc::new(expansion.clone())),
    }
}
