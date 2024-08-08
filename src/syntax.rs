// syntax.rs

// syntax.rs

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

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

#[derive(Clone, Debug)]
pub struct NotationPattern {
    pub pattern: String,
    pub variables: Vec<String>,
    pub precedence: Option<i32>,
    pub associativity: Associativity,
}

impl fmt::Display for NotationPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}",
            self.pattern,
            match &self.precedence {
                Some(p) => format!(" [{}]", p),
                None => "".to_string(),
            }
        )
    }
}

#[derive(Clone, Debug)]
pub enum Associativity {
    Left,
    Right,
    None,
}

#[derive(Clone)]
pub enum Expr {
    Primitive(Primitive),
    Variable(String),
    FunctionDef(String, Vec<String>, Rc<Expr>),
    FunctionCall(Rc<Expr>, Vec<Rc<Expr>>),
    Return(Rc<Expr>),
    Block(Vec<Rc<Expr>>),
    Assignment(String, Rc<Expr>),
    FFIDecl(String, Vec<String>),
    FFICall(String, String, Vec<Rc<Expr>>),
    InfixOp(Rc<Expr>, String, Rc<Expr>),
    NotationDecl(NotationPattern, Rc<Expr>),
}

#[derive(Clone)]
pub enum Value {
    Primitive(Primitive),
    Function(String, Vec<String>, Rc<Expr>, Rc<RefCell<Environment>>),
    PartialApplication(Rc<Value>, Vec<Value>),
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
            Expr::FFIDecl(name, params) => {
                write!(f, "FFI declaration: {} ({})", name, params.join(", "))
            }
            Expr::FFICall(module, func, args) => {
                write!(f, "FFI call: {}::{}(", module, func)?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?
                    }
                    fmt::Debug::fmt(arg, f)?;
                }
                write!(f, ")")
            }
            Expr::NotationDecl(pattern, expansion) => {
                write!(f, "notation declaration: {} -> ", pattern)?;
                fmt::Debug::fmt(expansion, f)
            }
            Expr::InfixOp(left, op, right) => write!(f, "({:?} {} {:?})", left, op, right),
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

pub type Environment = HashMap<String, Value>;
