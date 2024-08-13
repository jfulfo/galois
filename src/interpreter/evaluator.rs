// interpreter/evaluator.rs

use crate::debug::DebugPrinter;
use crate::syntax::{Environment, Expr, Primitive, Value};
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum InterpreterError {
    UndefinedVariable(String),
    TypeMismatch(String),
    ArityMismatch(String),
    ReturnOutsideFunction,
}

impl fmt::Display for InterpreterError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InterpreterError::UndefinedVariable(name) => write!(f, "Undefined variable: {}", name),
            InterpreterError::TypeMismatch(msg) => write!(f, "Type mismatch: {}", msg),
            InterpreterError::ArityMismatch(msg) => write!(f, "Arity mismatch: {}", msg),
            InterpreterError::ReturnOutsideFunction => {
                write!(f, "Return statement outside of function")
            }
        }
    }
}

pub fn interpret(
    exprs: Vec<Rc<Expr>>,
    debug: &mut DebugPrinter,
) -> Result<Value, InterpreterError> {
    let env = Rc::new(RefCell::new(Environment::new()));
    let mut result = Value::Primitive(Primitive::Bool(false));

    for expr in exprs {
        result = eval_expr(&expr, Rc::clone(&env), debug)?;
    }

    Ok(result)
}

fn eval_expr(
    expr: &Expr,
    env: Rc<RefCell<Environment>>,
    debug: &DebugPrinter,
) -> Result<Value, InterpreterError> {
    // TODO: Fix this when uncommented
    //debug.log_expr(expr, Rc::clone(&env)

    let result = match expr {
        Expr::Primitive(p) => Ok(Value::Primitive(p.clone())),
        Expr::Variable(name) => env
            .try_borrow()
            .map_err(|_| {
                InterpreterError::TypeMismatch("Failed to borrow environment".to_string())
            })?
            .get(name)
            .cloned()
            .ok_or_else(|| InterpreterError::UndefinedVariable(name.clone())),
        Expr::FunctionDef(name, params, body) => {
            let func_value = Value::Function(
                name.clone(),
                params.clone(),
                body.iter().cloned().collect::<Vec<Rc<Expr>>>(),
                Rc::clone(&env),
            );
            env.borrow_mut().insert(name.clone(), func_value.clone());
            Ok(func_value)
        }
        Expr::FunctionCall(func, args) => {
            let func_value = eval_expr(func, Rc::clone(&env), debug)?;
            let arg_values: Result<Vec<Value>, InterpreterError> = args
                .iter()
                .map(|arg| eval_expr(arg, Rc::clone(&env), debug))
                .collect();
            apply_function(func_value, arg_values?, Rc::clone(&env), debug)
        }
        Expr::Return(e) => eval_expr(e, env, debug),
        Expr::Assignment(name, expr) => {
            let value = eval_expr(expr, Rc::clone(&env), debug)?;
            env.borrow_mut().insert(name.clone(), value.clone());
            Ok(value)
        }
        Expr::InfixOp(_, _, _) => Ok(Value::Primitive(Primitive::Bool(true))),
        Expr::NotationDecl(_, _) => Ok(Value::Primitive(Primitive::Bool(true))),
        Expr::FFIDecl(_, _) => Ok(Value::Primitive(Primitive::Bool(true))),
        Expr::FFICall(_, _, _) => Ok(Value::Primitive(Primitive::Bool(true))),
    };

    //*stack_depth.borrow_mut() -= 1;
    result
}

fn apply_function(
    func: Value,
    args: Vec<Value>,
    env: Rc<RefCell<Environment>>,
    debug: &DebugPrinter,
) -> Result<Value, InterpreterError> {
    match func {
        Value::Function(name, params, body, closure_env) => {
            debug.log_entry(&name, &args);

            if args.len() != params.len() {
                let error = Err(InterpreterError::ArityMismatch(format!(
                    "Function '{}' expects {} arguments, but got {}",
                    name,
                    params.len(),
                    args.len()
                )));
                debug.log_exit(&name, &error.clone().map_err(|e| e.to_string()));
                return error;
            }

            let mut new_env = (*closure_env).borrow().clone();
            for (param, arg) in params.iter().zip(args.iter()) {
                new_env.insert(param.clone(), arg.clone());
            }

            let new_env = Rc::new(RefCell::new(new_env));
            let result = body
                .iter()
                .try_fold(Value::Primitive(Primitive::Bool(false)), |_, expr| {
                    eval_expr(expr, Rc::clone(&new_env), debug)
                });
            debug.log_exit(&name, &result.clone().map_err(|e| e.to_string()));
            result
        }
        Value::PartialApplication(func, prev_args) => {
            let mut all_args = prev_args;
            all_args.extend(args);
            apply_function((*func).clone(), all_args, env, debug)
        }
        _ => Err(InterpreterError::TypeMismatch(
            "Attempted to call a non-function value".to_string(),
        )),
    }
}
