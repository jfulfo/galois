// debug.rs

use crate::syntax::{Environment, Expr, Value};
use colored::*;
use std::cell::RefCell;
use std::fmt;
use std::time::{Duration, Instant};

thread_local! {
    static CALL_STACK: RefCell<Vec<CallFrame>> = const { RefCell::new(Vec::new()) };
    static TIMINGS: RefCell<Vec<(String, Duration)>> = const { RefCell::new(Vec::new()) };
}

#[derive(Clone, Debug)]
pub struct CallFrame {
    function_name: String,
    args: Vec<String>,
    start_time: Instant,
}

impl fmt::Display for CallFrame {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}({})", self.function_name, self.args.join(", "))
    }
}

pub struct DebugPrinter {
    pub debug_mode: bool,
}

impl DebugPrinter {
    pub fn new(debug_mode: bool) -> Self {
        DebugPrinter { debug_mode }
    }

    pub fn log_entry(&self, name: &str, args: &[Value]) {
        if !self.debug_mode {
            return;
        }
        let frame = CallFrame {
            function_name: name.to_string(),
            args: args.iter().map(|arg| format!("{:?}", arg)).collect(),
            start_time: Instant::now(),
        };
        CALL_STACK.with(|stack| {
            stack.borrow_mut().push(frame.clone());
        });
        println!("{} Entering: {}", "→".green(), frame);
        self.print_call_stack();
    }

    pub fn log_exit(&self, name: &str, result: &Result<Value, String>) {
        if !self.debug_mode {
            return;
        }
        CALL_STACK.with(|stack| {
            if let Some(frame) = stack.borrow_mut().pop() {
                let duration = frame.start_time.elapsed();
                TIMINGS.with(|timings| {
                    timings
                        .borrow_mut()
                        .push((frame.function_name.clone(), duration));
                });
                println!("{} Exiting: {} (took {:?})", "←".blue(), name, duration);
            }
        });
        match result {
            Ok(value) => println!("  Result: {:?}", value),
            Err(e) => println!("  Error: {}", e),
        }
        self.print_call_stack();
    }

    pub fn print_call_stack(&self) {
        if !self.debug_mode {
            return;
        }
        println!("{}", "Call Stack:".yellow());
        CALL_STACK.with(|stack| {
            for (i, frame) in stack.borrow().iter().rev().enumerate() {
                println!("  {}: {}", i, frame);
            }
        });
        println!();
    }

    pub fn log_value(&self, value: &Value, depth: usize) {
        if !self.debug_mode {
            return;
        }
        let indent = "  ".repeat(depth);
        match value {
            Value::Primitive(p) => println!("{}Value: {:?}", indent, p),
            Value::Function(name, params, body, _) => {
                println!("{}Function: {} ({})", indent, name, params.join(", "));
                println!("{}Body:", indent);
                body.iter()
                    .for_each(|e| self.log_expr(e, &Environment::new(), depth + 1));
            }
            Value::PartialApplication(func, args) => {
                println!("{}Partial Application:", indent);
                self.log_value(func, depth + 1);
                println!("{}Applied Arguments:", indent);
                for (i, arg) in args.iter().enumerate() {
                    println!("{}Arg {}: {:?}", indent, i, arg);
                }
            }
            Value::Ffi(s) => {
                println!("{}Foreign Function Interface: {:?}", indent, s);
            }
        }
    }

    pub fn log_expr(&self, expr: &Expr, _env: &Environment, depth: usize) {
        if !self.debug_mode {
            return;
        }
        let indent = "  ".repeat(depth);
        match expr {
            Expr::Primitive(p) => println!("{}Primitive: {:?}", indent, p),
            Expr::Variable(name) => println!("{}Variable: {}", indent, name),
            Expr::FunctionDef(name, params, body) => {
                println!(
                    "{}Function Definition: {} ({})",
                    indent,
                    name,
                    params.join(", ")
                );
                println!("{}Body:", indent);
                body.iter().for_each(|e| self.log_expr(e, _env, depth + 1));
            }
            Expr::FunctionCall(func, args) => {
                println!("{}Function Call:", indent);
                self.log_expr(func, _env, depth + 1);
                println!("{}Arguments:", indent);
                for (i, arg) in args.iter().enumerate() {
                    println!("{}Arg {}:", indent, i);
                    self.log_expr(arg, _env, depth + 2);
                }
            }
            Expr::Return(e) => {
                println!("{}Return:", indent);
                self.log_expr(e, _env, depth + 1);
            }
            Expr::Assignment(name, e) => {
                println!("{}Assignment: {}", indent, name);
                self.log_expr(e, _env, depth + 1);
            }
            Expr::FFIDecl(module, name, given_name) => match given_name {
                Some(given_name) => {
                    println!(
                        "{}FFI Declaration: from {} use {} as {}",
                        indent, module, name, given_name
                    );
                }
                None => {
                    println!("{}FFI Declaration: from {} use {}", indent, module, name);
                }
            },
            Expr::NotationDecl(pattern, expansion) => {
                println!("{}Notation Declaration:", indent);
                println!("{}Pattern: {}", indent, pattern);
                println!("{}Expansion: {}", indent, expansion);
            }
            Expr::InfixOp(left, op, right) => {
                println!("{}Infix Operation: {} {} {}", indent, left, op, right);
            }
        }
    }

    pub fn print_timings(&self) {
        if !self.debug_mode {
            return;
        }
        println!("{}", "Function Timings:".yellow());
        TIMINGS.with(|timings| {
            let mut timings = timings.borrow_mut();
            timings.sort_by(|a, b| b.1.cmp(&a.1));
            for (name, duration) in timings.iter() {
                println!("  {}: {:?}", name, duration);
            }
        });
    }
}
