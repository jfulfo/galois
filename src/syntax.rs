// syntax.rs

use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone)]
pub enum Primitive {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
}

impl fmt::Display for Primitive {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Primitive::Int(i) => write!(f, "{}", i),
            Primitive::Float(fl) => write!(f, "{}", fl),
            Primitive::String(s) => write!(f, "\"{}\"", s),
            Primitive::Bool(b) => write!(f, "{}", b),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Expr {
    Primitive(Primitive),
    Variable(String),
    FunctionDef(String, Vec<String>, Box<Expr>),
    FunctionCall(Box<Expr>, Vec<Expr>),
    Return(Box<Expr>),
    Block(Vec<Expr>),
    Assignment(String, Box<Expr>),
    NotationDecl(String, Vec<String>, Box<Expr>),
    FFIDecl(String, Vec<String>),
    FFICall(String, String, Vec<Expr>),
}

#[derive(Debug, Clone)]
pub enum Value {
    Primitive(Primitive),
    Function(String, Vec<String>, Box<Expr>, Environment),
    PartialApplication(Box<Value>, Vec<Value>),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Primitive(p) => write!(f, "{}", p),
            Value::Function(name, _, _, _) => write!(f, "<function {}>", name),
            Value::PartialApplication(func, _) => write!(f, "<partial application of {}>", func),
        }
    }
}

pub type Environment = HashMap<String, Value>;
