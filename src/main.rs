/*
* For now we only implement a basic interpreter
*/

mod parser;
mod syntax;

use std::fs;
use parser::parse_program;
use syntax::{Environment, Expr, Value};

fn eval(expr: &Expr, env: &mut Environment) -> Value {
    match expr {
        Expr::Primitive(p) => Value::Primitive(p.clone()),
        Expr::Variable(name) => env
            .get(name)
            .expect(&format!("Variable not found: {}", name))
            .clone(),
        Expr::FunctionDef(name, params, body) => Value::Function(
            name.clone(),
            params.clone(),
            body.clone(),
            env.clone(),
        ),
        Expr::FunctionCall(func, args) => {
            let func_value = eval(func, env);
            let arg_values: Vec<Value> = args.iter().map(|arg| eval(arg, env)).collect();
            apply_function(func_value, arg_values)
        }
    }
}

fn apply_function(mut func: Value, args: Vec<Value>) -> Value {
    match func {
        Value::Function(ref name, ref params, ref body, ref mut closure_env) => {
            if args.len() < params.len() {
                Value::PartialApplication(Box::new(func), args)
            } else if args.len() == params.len() {
                for (param, arg) in params.iter().zip(args.iter()) {
                    closure_env.insert(param.clone(), arg.clone());
                }
                eval(&body, closure_env)
            } else {
                panic!("Too many arguments for function: {}", name);
            }
        }
        Value::PartialApplication(func, mut prev_args) => {
            prev_args.extend(args);
            apply_function(*func, prev_args)
        }
        _ => panic!("Attempted to call a non-function value"),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <filename.gal>", args[0]);
        std::process::exit(1);
    }

    let filename = &args[1];
    let content = fs::read_to_string(filename).expect("Failed to read file");

    println!("=========PROGRAM=========\n{}\n=========================", content);
    let parse = parse_program(&content);
    println!("");
    println!("=========PARSED==========\n{:?}\n========================", parse);

    match parse {
        Ok((_, expr)) => {
            let mut env = Environment::new();
            let result = eval(&expr, &mut env);
            match result {
                Value::Primitive(p) => println!("Result: {:?}", p),
                Value::Function(name, params, _, _) => {
                    println!("Final state: Function '{}' with parameters {:?}", name, params);
                    println!("Note: Program ended with a fully defined function.");
                }
                Value::PartialApplication(func, args) => {
                    if let Value::Function(name, params, _, _) = *func {
                        println!("Final state: Partial application of function '{}' with {} out of {} arguments supplied", name, args.len(), params.len());
                        println!("Note: Program could not step further due to insufficient arguments.");
                    }
                }
            }
        }
        Err(e) => eprintln!("Failed to parse program: {:?}", e),
    }
}
