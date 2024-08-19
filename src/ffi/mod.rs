pub mod python;

use crate::ffi::python::PythonFFI;
use crate::syntax::Value;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

pub trait FFIProtocol {
    fn load_module(&mut self, module_path: &str) -> Result<Vec<String>, Box<dyn Error>>;
    fn call_function(&self, func_name: &str, args: Vec<Value>) -> Result<Value, Box<dyn Error>>;
}

#[derive(Debug)]
pub enum FFIError {
    ModuleNotFound(String),
    ProtocolNotImplemented(String),
    FunctionNotFound(String),
    AmbiguousFunction(String),
    LoadError(String),
    CallError(String),
}

impl fmt::Display for FFIError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FFIError::ModuleNotFound(name) => write!(f, "Module not found: {}", name),
            FFIError::ProtocolNotImplemented(name) => {
                write!(f, "FFI protocol not implemented for: {}", name)
            }
            FFIError::FunctionNotFound(name) => write!(f, "Function not found: {}", name),
            FFIError::AmbiguousFunction(name) => write!(f, "Ambiguous function call: {}", name),
            FFIError::LoadError(msg) => write!(f, "Error loading module: {}", msg),
            FFIError::CallError(msg) => write!(f, "Error calling function: {}", msg),
        }
    }
}

impl Error for FFIError {}

pub struct FFIBackend {
    pub modules: HashMap<String, Box<dyn FFIProtocol>>,
    pub function_to_module: HashMap<String, (String, String)>, // function -> (language, module)
}

impl FFIBackend {
    pub fn new() -> Self {
        FFIBackend {
            modules: HashMap::new(),
            function_to_module: HashMap::new(),
        }
    }
}

impl FFIProtocol for FFIBackend {
    fn load_module(&mut self, module_path: &str) -> Result<Vec<String>, Box<dyn Error>> {
        let parts: Vec<&str> = module_path.split('.').collect();
        let language = parts[0];
        let module_name = parts[1..].join(".");

        if !self.modules.contains_key(language) {
            match language {
                "python" => {
                    self.modules
                        .insert(language.to_string(), Box::new(PythonFFI::new()?));
                }
                _ => return Err(format!("Unsupported language: {}", language).into()),
            }
        }

        let functions = self
            .modules
            .get_mut(language)
            .unwrap()
            .load_module(&module_name)?;

        functions.iter().for_each(|func| {
            self.function_to_module
                .insert(func.clone(), (language.to_string(), module_name.clone()));
        });

        Ok(functions)
    }

    fn call_function(&self, function: &str, args: Vec<Value>) -> Result<Value, Box<dyn Error>> {
        let (language, module_name) = self
            .function_to_module
            .get(function)
            .ok_or_else(|| FFIError::FunctionNotFound(function.to_string()))?;

        self.modules
            .get(language)
            .ok_or_else(|| format!("Language not loaded: {}", language))?
            .call_function(&format!("{}.{}", module_name, function), args)
    }
}
