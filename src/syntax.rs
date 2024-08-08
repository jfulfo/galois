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

#[derive(Clone)]
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

#[derive(Clone)]
pub enum Value {
    Primitive(Primitive),
    Function(String, Vec<String>, Box<Expr>, Environment),
    PartialApplication(Box<Value>, Vec<Value>),
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Primitive(p) => write!(f, "{}", p),
            Value::Function(name, params, body, _) => {
                write!(f, "function {} ({}) {{ ", name, params.join(", "))?;
                fmt::Debug::fmt(body, f)?;
                write!(f, " }}")
            }
            Value::PartialApplication(func, args) => {
                write!(f, "partial application of {:?} with {:?}", func, args)
            }
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Primitive(p) => write!(f, "{}", p),
            Value::Function(name, params, body, _) => {
                write!(f, "function {} ({}) {{ ", name, params.join(", "))?;
                fmt::Display::fmt(body, f)?;
                write!(f, " }}")
            }
            Value::PartialApplication(func, args) => {
                write!(
                    f,
                    "partial application of {} with {} args",
                    func,
                    args.len()
                )
            }
        }
    }
}

impl fmt::Debug for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Primitive(p) => write!(f, "{:?}", p),
            Expr::Variable(name) => write!(f, "{}", name),
            Expr::FunctionDef(name, params, body) => {
                write!(f, "function {} ({}) {{ ", name, params.join(", "))?;
                fmt::Debug::fmt(body, f)?;
                write!(f, " }}")
            }
            Expr::FunctionCall(func, args) => {
                fmt::Debug::fmt(func, f)?;
                write!(f, "(")?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?
                    }
                    fmt::Debug::fmt(arg, f)?;
                }
                write!(f, ")")
            }
            Expr::Return(e) => {
                write!(f, "return ")?;
                fmt::Debug::fmt(e, f)
            }
            Expr::Block(exprs) => {
                write!(f, "{{ ")?;
                for (i, e) in exprs.iter().enumerate() {
                    if i > 0 {
                        write!(f, "; ")?
                    }
                    fmt::Debug::fmt(e, f)?;
                }
                write!(f, " }}")
            }
            Expr::Assignment(name, e) => {
                write!(f, "{} = ", name)?;
                fmt::Debug::fmt(e, f)
            }
            Expr::NotationDecl(_, _, _) => write!(f, "<notation declaration>"),
            Expr::FFIDecl(name, params) => {
                write!(f, "FFI declaration: {} ({})", name, params.join(", "))
            }
            Expr::FFICall(module, func, _args) => write!(f, "FFI call: {}::{}(...)", module, func),
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

pub type Environment = HashMap<String, Value>;
