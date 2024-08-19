// main.rs

mod debug;
mod ffi;
mod interpreter;
mod parser;
mod syntax;

use debug::DebugPrinter;
use interpreter::interpret;
use parser::parse_program;
use std::env;
use std::fs;
use std::time::Instant;
use syntax::Environment;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <filename.gal> [--debug]", args[0]);
        std::process::exit(1);
    }

    let filename = &args[1];
    let debug_mode = args.contains(&"--debug".to_string());

    let content = fs::read_to_string(filename)?;

    let mut debug_printer = DebugPrinter::new(debug_mode);

    let parse_time = Instant::now();
    let parsed = parse_program(&content);
    let parse_duration = parse_time.elapsed();

    let start_time = Instant::now();

    match parsed {
        Ok(exprs) => {
            if debug_mode {
                for expr in &exprs {
                    debug_printer.log_expr(expr, &Environment::new(), 0);
                }
            }
            match interpret(exprs, &mut debug_printer) {
                Ok(result) => {
                    if debug_mode {
                        println!("Result:");
                        debug_printer.log_value(&result, 0);
                        debug_printer.print_timings();
                    }
                }
                Err(e) => {
                    eprintln!("Runtime error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("Parse error: {}", e);
            std::process::exit(1);
        }
    }

    let duration = start_time.elapsed();
    if debug_mode {
        println!("parse took {:?}", parse_duration);
        println!("execution took {:?}", duration);
    } else {
        println!("took {:?}", duration);
    }

    Ok(())
}
