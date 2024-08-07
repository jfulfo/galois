// syntax.rs

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Primitive {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
}

#[derive(Debug, Clone)]
pub enum Expr {
    Primitive(Primitive),
    Variable(String),
    FunctionDef(String, Vec<String>, Box<Expr>),
    FunctionCall(Box<Expr>, Vec<Expr>),
    Return(Box<Expr>),
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

pub type Environment = HashMap<String, Value>;
