// debug.rs

use crate::syntax::{Expr, Environment};
use colored::*;
use std::fmt;

pub struct DebugPrinter {
    debug_mode: bool,
    indent_level: usize,
}

impl DebugPrinter {
    pub fn new(debug_mode: bool) -> Self {
        DebugPrinter {
            debug_mode,
            indent_level: 0,
        }
    }

    pub fn log_program(&self, content: &str) {
        if !self.debug_mode { return; }
        println!("{}", "Program:".blue().bold());
        println!("{}", content);
        println!();
    }

    pub fn log_parsed(&self, exprs: &[Expr]) {
        if !self.debug_mode { return; }
        println!("{}", "Parsed Expressions:".blue().bold());
        for (i, expr) in exprs.iter().enumerate() {
            println!("Expression {}:", i + 1);
            println!("{:?}", expr);
            println!();
        }
    }

    pub fn log_step(&mut self, expr: &Expr, env: &Environment) {
        if !self.debug_mode { return; }
        println!("{}", "┌─ Evaluation Step ".blue().bold());
        self.print_indented("Expression:", expr);
        self.print_indented("Environment:", env);
        println!("{}", "└─────────".blue().bold());
    }

    pub fn log_result<T: fmt::Debug>(&self, result: &Result<T, impl fmt::Debug>) {
        if !self.debug_mode { return; }
        println!("{}", "┌─ Step Result ".green().bold());
        match result {
            Ok(value) => self.print_indented("Value:", value),
            Err(e) => self.print_indented("Error:", e),
        }
        println!("{}", "└─────────".green().bold());
    }

    pub fn increase_indent(&mut self) {
        self.indent_level += 1;
    }

    pub fn decrease_indent(&mut self) {
        if self.indent_level > 0 {
            self.indent_level -= 1;
        }
    }

    fn print_indented<T: fmt::Debug>(&self, label: &str, value: &T) {
        let indent = "│ ".repeat(self.indent_level);
        println!("{}{}:", indent, label.yellow());
        println!("{}{:?}", indent, value);
    }
}
