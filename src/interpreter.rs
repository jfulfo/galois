// interpreter.rs

use crate::syntax::{Expr, Value, Environment};
use crate::debug::DebugPrinter;
use std::fmt;

#[derive(Debug)]
pub enum InterpreterError {
    UndefinedVariable(String),
    TypeMismatch(String),
    ArityMismatch(String),
    // ...
}

impl fmt::Display for InterpreterError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InterpreterError::UndefinedVariable(name) => write!(f, "Undefined variable: {}", name),
            InterpreterError::TypeMismatch(msg) => write!(f, "Type mismatch: {}", msg),
            InterpreterError::ArityMismatch(msg) => write!(f, "Arity mismatch: {}", msg),
        }
    }
}

pub fn interpret(expr: &Expr, env: &mut Environment, debug: &mut DebugPrinter) -> Result<Value, InterpreterError> {
    debug.log_step(expr, env);
    debug.increase_indent();

    let result = match expr {
        Expr::Primitive(p) => Ok(Value::Primitive(p.clone())),
        Expr::Variable(name) => env.get(name)
            .cloned()
            .ok_or_else(|| InterpreterError::UndefinedVariable(name.clone())),
        Expr::FunctionDef(name, params, body) => {
            Ok(Value::Function(name.clone(), params.clone(), body.clone(), env.clone()))
        },
        Expr::FunctionCall(func, args) => {
            let func_value = interpret(func, env, debug)?;
            let arg_values: Result<Vec<Value>, InterpreterError> = 
                args.iter().map(|arg| interpret(arg, env, debug)).collect();
            apply_function(func_value, arg_values?, debug)
        }
    };

    debug.decrease_indent();
    debug.log_result(&result);
    result
}

fn apply_function(mut func: Value, args: Vec<Value>, debug: &mut DebugPrinter) -> Result<Value, InterpreterError> {
    match func {
        Value::Function(ref name, ref params, ref body, ref mut closure_env) => {
            if args.len() < params.len() {
                Ok(Value::PartialApplication(Box::new(func), args))
            } else if args.len() == params.len() {
                for (param, arg) in params.iter().zip(args.iter()) {
                    closure_env.insert(param.clone(), arg.clone());
                }
                interpret(&body, closure_env, debug)
            } else {
                Err(InterpreterError::ArityMismatch(format!("Too many arguments for function: {}", name)))
            }
        }
        Value::PartialApplication(func, mut prev_args) => {
            prev_args.extend(args);
            apply_function(*func, prev_args, debug)
        }
        _ => Err(InterpreterError::TypeMismatch("Attempted to call a non-function value".to_string())),
    }
}
