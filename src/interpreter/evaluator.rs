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
    StackOverflow,
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
            InterpreterError::StackOverflow => {
                write!(f, "Stack overflow: maximum recursion depth exceeded")
            }
        }
    }
}

const MAX_STACK_DEPTH: usize = 1000;

pub fn interpret(exprs: Vec<Expr>, debug: &mut DebugPrinter) -> Result<Value, InterpreterError> {
    let mut env = Environment::new();
    let mut result = Value::Primitive(Primitive::Bool(false));
    let stack_depth = Rc::new(RefCell::new(0));

    for expr in exprs {
        result = eval_expr(&expr, &mut env, debug, Rc::clone(&stack_depth))?;
    }

    Ok(result)
}

fn eval_expr(
    expr: &Expr,
    env: &mut Environment,
    debug: &DebugPrinter,
    stack_depth: Rc<RefCell<usize>>,
) -> Result<Value, InterpreterError> {
    *stack_depth.borrow_mut() += 1;
    if *stack_depth.borrow() > MAX_STACK_DEPTH {
        return Err(InterpreterError::StackOverflow);
    }

    debug.log_expr(expr, env);

    let result = match expr {
        Expr::Primitive(p) => Ok(Value::Primitive(p.clone())),
        Expr::Variable(name) => env
            .get(name)
            .cloned()
            .ok_or_else(|| InterpreterError::UndefinedVariable(name.clone())),
        Expr::FunctionDef(name, params, body) => {
            let func_value =
                Value::Function(name.clone(), params.clone(), body.clone(), env.clone());
            env.insert(name.clone(), func_value.clone());
            Ok(func_value)
        }
        Expr::FunctionCall(func, args) => {
            let func_value = eval_expr(func, env, debug, Rc::clone(&stack_depth))?;
            let arg_values: Result<Vec<Value>, InterpreterError> = args
                .iter()
                .map(|arg| eval_expr(arg, env, debug, Rc::clone(&stack_depth)))
                .collect();
            apply_function(func_value, arg_values?, env, debug, Rc::clone(&stack_depth))
        }
        Expr::Return(e) => eval_expr(e, env, debug, Rc::clone(&stack_depth)),
        Expr::NotationDecl(_, _, _) => Ok(Value::Primitive(Primitive::Bool(true))),
        Expr::FFIDecl(_, _) => Ok(Value::Primitive(Primitive::Bool(true))),
        Expr::FFICall(_, _, _) => Ok(Value::Primitive(Primitive::Bool(true))),
    };

    *stack_depth.borrow_mut() -= 1;
    result
}

fn apply_function(
    func: Value,
    args: Vec<Value>,
    env: &mut Environment,
    debug: &DebugPrinter,
    stack_depth: Rc<RefCell<usize>>,
) -> Result<Value, InterpreterError> {
    match func {
        Value::Function(name, params, body, mut closure_env) => {
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

            for (param, arg) in params.iter().zip(args.iter()) {
                closure_env.insert(param.clone(), arg.clone());
            }

            let result = eval_expr(&body, &mut closure_env, debug, stack_depth);
            debug.log_exit(&name, &result.clone().map_err(|e| e.to_string()));
            result
        }
        Value::PartialApplication(func, prev_args) => {
            let mut all_args = prev_args;
            all_args.extend(args);
            apply_function(*func, all_args, env, debug, stack_depth)
        }
        _ => Err(InterpreterError::TypeMismatch(
            "Attempted to call a non-function value".to_string(),
        )),
    }
}



