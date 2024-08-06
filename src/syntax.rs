/*
* Syntax for the language
*/

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Primitive {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    List(Vec<Expr>),
    Dict(Vec<(Expr, Expr)>),
}

#[derive(Debug, Clone)]
pub enum Expr {
    Primitive(Primitive),
    Variable(String),
    FunctionDef(String, Vec<String>, Box<Expr>),
    FunctionCall(Box<Expr>, Vec<Expr>),
}

#[derive(Debug, Clone)]
pub enum Value {
    Primitive(Primitive),
    Function(String, Vec<String>, Box<Expr>, Environment),
    PartialApplication(Box<Value>, Vec<Value>),
}

pub type Environment = HashMap<String, Value>;
