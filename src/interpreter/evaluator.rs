// interpreter/evaluator.rs

use crate::debug::DebugPrinter;
use crate::ffi::{FFIBackend, FFIProtocol};
use crate::syntax::{Environment, Expr, Primitive, Value};
use std::cell::RefCell;
use std::rc::Rc;

pub struct Interpreter {
    env: Rc<RefCell<Environment>>,
    debug: DebugPrinter,
    ffi: FFIBackend,
}

#[derive(Debug, Clone)]
pub enum InterpreterError {
    UndefinedVariable(String),
    TypeMismatch(String),
    ArityMismatch(String),
    FFIError(String),
    NotReachable(String),
}

impl std::fmt::Display for InterpreterError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            InterpreterError::UndefinedVariable(name) => write!(f, "Undefined variable: {}", name),
            InterpreterError::TypeMismatch(msg) => write!(f, "Type mismatch: {}", msg),
            InterpreterError::ArityMismatch(msg) => write!(f, "Arity mismatch: {}", msg),
            InterpreterError::FFIError(msg) => write!(f, "FFI error: {}", msg),
            InterpreterError::NotReachable(msg) => write!(f, "Not reachable: {}", msg),
        }
    }
}

impl Interpreter {
    pub fn new(debug_mode: bool) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Interpreter {
            env: Rc::new(RefCell::new(Environment::new())),
            debug: DebugPrinter::new(debug_mode),
            ffi: FFIBackend::new(),
        })
    }

    pub fn interpret(&mut self, exprs: Vec<Rc<Expr>>) -> Result<Value, InterpreterError> {
        let mut result = Value::Primitive(Primitive::Bool(false));

        for expr in exprs {
            result = self.eval_expr(&expr)?;
        }

        Ok(result)
    }

    fn eval_expr(&mut self, expr: &Expr) -> Result<Value, InterpreterError> {
        match expr {
            Expr::Primitive(p) => Ok(Value::Primitive(p.clone())),
            Expr::Variable(name) => self
                .env
                .borrow()
                .get(name)
                .cloned()
                .ok_or_else(|| InterpreterError::UndefinedVariable(name.clone())),
            Expr::FunctionDef(name, params, body) => {
                let func_value = Value::Function(
                    name.clone(),
                    params.clone(),
                    body.to_vec(),
                    Rc::clone(&self.env),
                );
                self.env
                    .borrow_mut()
                    .insert(name.clone(), func_value.clone());
                Ok(func_value)
            }
            Expr::FunctionCall(func, args) => {
                let func_value = self.eval_expr(func)?;
                // bug here because in the .map self.env gets overwritten?
                let arg_values: Result<Vec<Value>, InterpreterError> =
                    args.iter().map(|arg| self.eval_expr(arg)).collect();
                self.apply_function(func_value, arg_values?)
            }
            Expr::Return(e) => self.eval_expr(e),
            Expr::Assignment(name, expr) => {
                let value = self.eval_expr(expr)?;
                self.env.borrow_mut().insert(name.clone(), value.clone());
                Ok(value)
            }
            Expr::FFIDecl(module, name, alias) => {
                self.ffi
                    .load_module(module)
                    .map_err(|e| InterpreterError::FFIError(e.to_string()))?;

                let ffi_name = alias.as_ref().unwrap_or(name);
                // implicit aliasing
                self.env
                    .borrow_mut()
                    .insert(ffi_name.to_string(), Value::Ffi(name.to_string()));
                Ok(Value::Primitive(Primitive::Bool(true)))
            }
            Expr::InfixOp(_, _, _) => Err(InterpreterError::NotReachable(
                "Infix operations should be handled by the parser".to_string(),
            )),
            Expr::NotationDecl(_, _) => Err(InterpreterError::NotReachable(
                "Notation declarations should be handled by the parser".to_string(),
            )),
        }
    }

    fn apply_function(&mut self, func: Value, args: Vec<Value>) -> Result<Value, InterpreterError> {
        match func {
            Value::Function(name, params, body, closure_env) => {
                self.debug.log_entry(&name, &args);
                if args.len() != params.len() {
                    let error = Err(InterpreterError::ArityMismatch(format!(
                        "Function '{}' expects {} arguments, but got {}",
                        name,
                        params.len(),
                        args.len()
                    )));
                    self.debug
                        .log_exit(&name, &error.clone().map_err(|e| e.to_string()));
                    return error;
                }

                let mut new_env = (*closure_env).borrow().clone();
                for (param, arg) in params.iter().zip(args.iter()) {
                    new_env.insert(param.clone(), arg.clone());
                }
                self.env = Rc::new(RefCell::new(new_env));

                let result = body
                    .iter()
                    .try_fold(Value::Primitive(Primitive::Bool(false)), |_, expr| {
                        self.eval_expr(expr)
                    });
                self.debug
                    .log_exit(&name, &result.clone().map_err(|e| e.to_string()));

                Ok(result?)
            }
            Value::Ffi(ffi_name) => {
                self.debug.log_entry(&ffi_name, &args);
                let result = self
                    .ffi
                    .call_function(&ffi_name, args)
                    .map_err(|e| InterpreterError::FFIError(e.to_string()));
                self.debug
                    .log_exit(&ffi_name, &result.clone().map_err(|e| e.to_string()));
                result
            }
            Value::PartialApplication(func, prev_args) => {
                let mut all_args = prev_args;
                all_args.extend(args);
                self.apply_function((*func).clone(), all_args)
            }
            _ => Err(InterpreterError::TypeMismatch(
                "Attempted to call a non-function value".to_string(),
            )),
        }
    }
}

pub fn interpret(
    exprs: Vec<Rc<Expr>>,
    debug: &mut DebugPrinter,
) -> Result<Value, InterpreterError> {
    let mut interpreter = Interpreter::new(debug.debug_mode)
        .map_err(|e| InterpreterError::FFIError(e.to_string()))?;
    interpreter.interpret(exprs)
}
