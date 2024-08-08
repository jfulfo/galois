// parser/notation.rs

use crate::syntax::{Expr, NotationPattern};
use std::collections::HashMap;

#[derive(Clone, Debug)]
struct Notation {
    pattern: NotationPattern,
    expansion: Box<Expr>,
}

pub fn apply_notations(ast: Vec<Expr>) -> Result<Vec<Expr>, String> {
    let (notations, expressions): (Vec<_>, Vec<_>) = ast
        .into_iter()
        .partition(|expr| matches!(expr, Expr::NotationDecl(_, _)));

    let notations: Vec<Notation> = notations
        .into_iter()
        .filter_map(|expr| {
            if let Expr::NotationDecl(pattern, expansion) = expr {
                Some(Notation { pattern, expansion })
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

fn expand_expr(expr: Expr, notations: &[Notation]) -> Result<Expr, String> {
    let expanded = match expr {
        Expr::FunctionDef(name, params, body) => {
            Expr::FunctionDef(name, params, Box::new(expand_expr(*body, notations)?))
        }
        Expr::FunctionCall(func, args) => {
            let expanded_func = Box::new(expand_expr(*func, notations)?);
            let expanded_args = args
                .into_iter()
                .map(|arg| expand_expr(arg, notations))
                .collect::<Result<Vec<_>, _>>()?;
            Expr::FunctionCall(expanded_func, expanded_args)
        }
        Expr::Return(e) => Expr::Return(Box::new(expand_expr(*e, notations)?)),
        Expr::Block(exprs) => {
            let expanded_exprs = exprs
                .into_iter()
                .map(|e| expand_expr(e, notations))
                .collect::<Result<Vec<_>, _>>()?;
            Expr::Block(expanded_exprs)
        }
        Expr::Assignment(name, e) => Expr::Assignment(name, Box::new(expand_expr(*e, notations)?)),
        Expr::InfixOp(left, op, right) => {
            let expanded_left = Box::new(expand_expr(*left, notations)?);
            let expanded_right = Box::new(expand_expr(*right, notations)?);
            Expr::InfixOp(expanded_left, op, expanded_right)
        }
        _ => expr,
    };

    // Try to match and expand notations
    for notation in notations {
        if let Some(bindings) = match_pattern(&expanded, &notation.pattern) {
            return expand_notation(&notation.expansion, &bindings);
        }
    }

    Ok(expanded)
}

fn match_pattern(expr: &Expr, pattern: &NotationPattern) -> Option<HashMap<String, Expr>> {
    match expr {
        Expr::InfixOp(left, op, right) if pattern.pattern == format!("$x {} $y", op) => {
            let mut bindings = HashMap::new();
            bindings.insert("x".to_string(), (**left).clone());
            bindings.insert("y".to_string(), (**right).clone());
            Some(bindings)
        }
        _ => None,
    }
}

fn expand_notation(expansion: &Expr, bindings: &HashMap<String, Expr>) -> Result<Expr, String> {
    match expansion {
        Expr::Variable(name) => Ok(bindings
            .get(name)
            .cloned()
            .unwrap_or_else(|| expansion.clone())),
        Expr::FunctionCall(func, args) => {
            let expanded_func = Box::new(expand_notation(func, bindings)?);
            let expanded_args = args
                .iter()
                .map(|arg| expand_notation(arg, bindings))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Expr::FunctionCall(expanded_func, expanded_args))
        }
        Expr::InfixOp(left, op, right) => {
            let expanded_left = Box::new(expand_notation(left, bindings)?);
            let expanded_right = Box::new(expand_notation(right, bindings)?);
            Ok(Expr::InfixOp(expanded_left, op.clone(), expanded_right))
        }
        _ => Ok(expansion.clone()),
    }
}
