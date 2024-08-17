pub mod python;

use crate::syntax::Value;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

pub trait FFIProtocol {
    fn load_module(&mut self, module_path: &str) -> Result<(), Box<dyn Error>>;
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
    pub module_to_language: HashMap<String, String>,
}

impl FFIBackend {
    pub fn new() -> Self {
        FFIBackend {
            modules: HashMap::new(),
            module_to_language: HashMap::new(),
        }
    }
}

impl FFIProtocol for FFIBackend {
    fn load_module(&mut self, module_path: &str) -> Result<(), Box<dyn Error>> {
        let parts: Vec<&str> = module_path.split('.').collect();
        let language = parts[0];
        let module_name = parts[1..].join(".");

        if !self.modules.contains_key(language) {
            match language {
                "python" => {
                    self.modules.insert(
                        language.to_string(),
                        Box::new(crate::ffi::python::PythonFFI::new()?),
                    );
                }
                _ => return Err(format!("Unsupported language: {}", language).into()),
            }
        }

        self.modules
            .get_mut(language)
            .unwrap()
            .load_module(&module_name)?;

        self.module_to_language
            .insert(module_name, language.to_string());

        Ok(())
    }

    fn call_function(&self, func_name: &str, args: Vec<Value>) -> Result<Value, Box<dyn Error>> {
        let module_name = "ffi_effects";
        let language = "python";
        self.modules
            .get(language)
            .ok_or_else(|| format!("Language not loaded: {}", language))?
            .call_function(&format!("{}.{}", module_name, func_name), args)
    }
}
