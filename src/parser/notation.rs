// parser/notation.rs

use crate::syntax::Expr;
use std::collections::HashMap;

pub fn apply_notations(ast: Vec<Expr>) -> Result<Vec<Expr>, String> {
    let (notations, rest) = extract_notations(ast);
    let expanded = expand_notations(rest, &notations);
    Ok(expanded)
}

fn extract_notations(ast: Vec<Expr>) -> (HashMap<String, (Vec<String>, Box<Expr>)>, Vec<Expr>) {
    let mut notations = HashMap::new();
    let mut rest = Vec::new();

    for expr in ast {
        match expr {
            Expr::NotationDecl(pattern, vars, expansion) => {
                notations.insert(pattern, (vars, expansion));
            }
            _ => rest.push(expr),
        }
    }

    (notations, rest)
}

fn expand_notations(
    ast: Vec<Expr>,
    notations: &HashMap<String, (Vec<String>, Box<Expr>)>,
) -> Vec<Expr> {
    ast.into_iter()
        .map(|expr| expand_expr(expr, notations))
        .collect()
}

fn expand_expr(expr: Expr, notations: &HashMap<String, (Vec<String>, Box<Expr>)>) -> Expr {
    match expr {
        Expr::FunctionCall(func, args) => {
            let expanded_func = expand_expr(*func, notations);
            let expanded_args = args
                .into_iter()
                .map(|arg| expand_expr(arg, notations))
                .collect::<Vec<_>>();

            // Check if this function call matches any notation
            for (pattern, (vars, expansion)) in notations {
                if let Some(matched_args) =
                    match_notation(&expanded_func, &expanded_args, pattern, vars)
                {
                    let mut env = HashMap::new();
                    for (var, arg) in vars.iter().zip(matched_args) {
                        env.insert(var.clone(), arg);
                    }
                    return substitute_expr(&expansion, &env);
                }
            }

            Expr::FunctionCall(Box::new(expanded_func), expanded_args)
        }
        Expr::FunctionDef(name, params, body) => {
            Expr::FunctionDef(name, params, Box::new(expand_expr(*body, notations)))
        }
        Expr::Return(e) => Expr::Return(Box::new(expand_expr(*e, notations))),
        _ => expr,
    }
}

fn match_notation(func: &Expr, args: &[Expr], pattern: &str, vars: &[String]) -> Option<Vec<Expr>> {
    let pattern_parts: Vec<&str> = pattern.split_whitespace().collect();
    if pattern_parts.len() != args.len() + 1 {
        return None;
    }

    if let Expr::Variable(func_name) = func {
        if func_name != pattern_parts[0] {
            return None;
        }
    } else {
        return None;
    }

    let mut matched_args = Vec::new();
    for (i, arg) in args.iter().enumerate() {
        if vars.contains(&pattern_parts[i + 1].to_string()) {
            matched_args.push(arg.clone());
        } else if pattern_parts[i + 1] != "+"
            && pattern_parts[i + 1] != "-"
            && pattern_parts[i + 1] != "*"
            && pattern_parts[i + 1] != "/"
        {
            return None;
        }
    }

    Some(matched_args)
}

fn substitute_expr(expr: &Expr, env: &HashMap<String, Expr>) -> Expr {
    match expr {
        Expr::Variable(name) => env.get(name).cloned().unwrap_or_else(|| expr.clone()),
        Expr::FunctionCall(func, args) => Expr::FunctionCall(
            Box::new(substitute_expr(func, env)),
            args.iter().map(|arg| substitute_expr(arg, env)).collect(),
        ),
        _ => expr.clone(),
    }
}
