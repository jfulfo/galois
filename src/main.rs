// main.rs

mod parser;
mod syntax;
mod debug;
mod interpreter;

use std::fs;
use std::env;
use syntax::Environment;
use parser::parse_program;
use debug::DebugPrinter;
use interpreter::interpret;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <filename.gal> [--debug]", args[0]);
        std::process::exit(1);
    }

    let filename = &args[1];
    let debug_mode = args.contains(&"--debug".to_string());
    
    let content = fs::read_to_string(filename)
        .unwrap_or_else(|e| {
            eprintln!("Error reading file '{}': {}", filename, e);
            std::process::exit(1);
        });

    let mut debug_printer = DebugPrinter::new(debug_mode);

    match parse_program(&content) {
        Ok(exprs) => {
            if debug_mode {
                for expr in &exprs {
                    debug_printer.log_expr(expr, &Environment::new());
                }
            }

            match interpret(exprs, &mut debug_printer) {
                Ok(result) => println!("Final result: {:?}", result),
                Err(e) => eprintln!("Runtime error: {}", e),
            }
        }
        Err(e) => eprintln!("Parse error: {}", e),
    }
}
