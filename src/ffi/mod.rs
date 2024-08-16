pub mod python;

use crate::syntax::Value;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::path::PathBuf;

pub trait FFIProtocol {
    fn load_module(&mut self, module_path: &str, alias: Option<&str>)
        -> Result<(), Box<dyn Error>>;
    fn call_function(&self, func_path: &str, args: Vec<Value>) -> Result<Value, Box<dyn Error>>;
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

impl std::fmt::Display for FFIError {
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
}

impl FFIBackend {
    pub fn new() -> Self {
        FFIBackend {
            modules: HashMap::new(),
        }
    }

    fn get_galois_path() -> Vec<PathBuf> {
        env::var("GALOIS_PATH")
            .unwrap_or_else(|_| String::from("./std/ffi"))
            .split(':')
            .map(PathBuf::from)
            .collect()
    }

    fn find_module_file(module_name: &str) -> Option<PathBuf> {
        for path in Self::get_galois_path() {
            let full_path = path.join(format!("{}.py", module_name));
            if full_path.exists() {
                return Some(full_path);
            }
        }
        None
    }

    fn load_ffi_protocol(&self, _module_name: &str) -> Result<Box<dyn FFIProtocol>, FFIError> {
        // Dynamically load the appropriate FFI protocol
        Ok(Box::new(
            crate::ffi::python::PythonFFI::new().map_err(|e| FFIError::LoadError(e.to_string()))?,
        ))
    }
}

impl FFIProtocol for FFIBackend {
    fn load_module(
        &mut self,
        module_path: &str,
        alias: Option<&str>,
    ) -> Result<(), Box<dyn Error>> {
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
            .load_module(&module_name, alias)
    }

    fn call_function(&self, func_path: &str, args: Vec<Value>) -> Result<Value, Box<dyn Error>> {
        let parts: Vec<&str> = func_path.split('.').collect();
        let language = parts[0];
        let function_path = parts[1..].join(".");

        self.modules
            .get(language)
            .ok_or_else(|| format!("Language not loaded: {}", language))?
            .call_function(&function_path, args)
    }
}
