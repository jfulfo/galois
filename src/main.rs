// main.rs

mod parser;
mod syntax;
mod debug;
mod interpreter;

use std::fs;
use std::env;
use parser::parse_program;
use syntax::Environment;
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

    if debug_mode {
        debug_printer.log_program(&content);
    }

    match parse_program(&content) {
        Ok((_, exprs)) => {  // Note: parse_program should now return Vec<Expr>
            if debug_mode {
                debug_printer.log_parsed(&exprs);
            }

            let mut env = Environment::new();
            let mut final_result = None;

            for expr in exprs {
                match interpret(&expr, &mut env, &mut debug_printer) {
                    Ok(result) => {
                        final_result = Some(result);
                    },
                    Err(e) => {
                        eprintln!("Runtime error: {}", e);
                        std::process::exit(1);
                    }
                }
            }

            if let Some(result) = final_result {
                println!("{:?}", result);
            }
        }
        Err(e) => eprintln!("Parse error: {}", e),
    }
}
