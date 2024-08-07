// debug.rs

use crate::syntax::{Environment, Expr, Value};
use colored::*;
use std::cell::RefCell;
use std::fmt;

thread_local! {
    static CALL_STACK: RefCell<Vec<CallFrame>> = RefCell::new(Vec::new());
    static DEPTH: RefCell<usize> = RefCell::new(0);
}

const MAX_DEPTH: usize = 1000; // Adjust this value as needed

#[derive(Clone, Debug)]
pub struct CallFrame {
    function_name: String,
    args: Vec<String>,
}

impl fmt::Display for CallFrame {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}({})", self.function_name, self.args.join(", "))
    }
}

pub struct DebugPrinter {
    debug_mode: bool,
}

impl DebugPrinter {
    pub fn new(debug_mode: bool) -> Self {
        DebugPrinter { debug_mode }
    }

    pub fn log_entry(&self, name: &str, args: &[Value]) {
        if !self.debug_mode {
            return;
        }
        DEPTH.with(|depth| {
            let mut depth = depth.borrow_mut();
            *depth += 1;
            if *depth > MAX_DEPTH {
                panic!("Maximum recursion depth exceeded");
            }
        });
        let frame = CallFrame {
            function_name: name.to_string(),
            args: args.iter().map(|arg| format!("{:?}", arg)).collect(),
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
        DEPTH.with(|depth| {
            let mut depth = depth.borrow_mut();
            *depth -= 1;
        });
        CALL_STACK.with(|stack| {
            stack.borrow_mut().pop();
        });
        match result {
            Ok(value) => println!("{} Exiting: {} = {:?}", "←".blue(), name, value),
            Err(e) => println!("{} Exiting: {} with error: {}", "←".red(), name, e),
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

    pub fn log_expr(&self, expr: &Expr, env: &Environment) {
        if !self.debug_mode {
            return;
        }
        println!("{}", "Expression:".cyan());
        println!("  {:?}", expr);
        println!("{}", "Environment:".cyan());
        for (key, value) in env {
            println!("  {}: {:?}", key, value);
        }
        println!();
    }
}
