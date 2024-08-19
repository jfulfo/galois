// ffi/python.rs

use super::FFIProtocol;
use crate::syntax::{Primitive, Value};
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::PyTuple;
use std::collections::HashMap;
use std::error::Error;
use std::fs;

pub struct PythonFFI {
    py: Python<'static>,
    modules: HashMap<String, Py<PyModule>>,
}

impl PythonFFI {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        pyo3::prepare_freethreaded_python();
        Ok(PythonFFI {
            py: unsafe { Python::assume_gil_acquired() },
            modules: HashMap::new(),
        })
    }

    fn convert_to_python(&self, value: &Value) -> PyObject {
        pyo3::Python::<'static>::with_gil(|py| match value {
            Value::Primitive(p) => match p {
                Primitive::Int(i) => i.to_object(py),
                Primitive::Float(f) => f.to_object(py),
                Primitive::String(s) => s.to_object(py),
                Primitive::Bool(b) => b.to_object(py),
                Primitive::Array(_arr) => {
                    unimplemented!()
                }
            },
            Value::Function(name, params, body, _) => {
                todo!()
            }
            _ => py.None(),
        })
    }

    fn convert_from_python(&self, obj: PyObject) -> Value {
        pyo3::Python::<'static>::with_gil(|py| {
            if let Ok(i) = obj.extract::<i64>(py) {
                Value::Primitive(Primitive::Int(i))
            } else if let Ok(f) = obj.extract::<f64>(py) {
                Value::Primitive(Primitive::Float(f))
            } else if let Ok(s) = obj.extract::<String>(py) {
                Value::Primitive(Primitive::String(s))
            } else if let Ok(b) = obj.extract::<bool>(py) {
                Value::Primitive(Primitive::Bool(b))
            } else if let Ok(_list) = obj.extract::<Vec<PyObject>>(py) {
                Value::Primitive(Primitive::Array(unimplemented!()))
            } else {
                // TODO: Handle other types
                Value::Primitive(Primitive::Bool(false))
            }
        })
    }
}

impl FFIProtocol for PythonFFI {
    // returns a list of function names
    fn load_module(&mut self, module_path: &str) -> Result<Vec<String>, Box<dyn Error>> {
        Python::with_gil(|py| {
            let module_file = format!("std/ffi/python/{}.py", module_path.replace('.', "/"));
            let module_code = fs::read_to_string(&module_file)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;

            let module = PyModule::from_code_bound(py, &module_code, &module_file, module_path)?;

            self.modules
                .insert(module_path.to_string(), module.clone().into());

            let function_names = module
                .dir()?
                .iter()
                .map(|item| item.to_string())
                .collect::<Vec<String>>();

            Ok(function_names)
        })
    }

    fn call_function(&self, func_path: &str, args: Vec<Value>) -> Result<Value, Box<dyn Error>> {
        Python::with_gil(|py| {
            let parts: Vec<&str> = func_path.split('.').collect();
            let (module_parts, func_name) = parts.split_at(parts.len() - 1);
            let module_path = module_parts.join(".");
            let func_name = func_name[0];

            let module = self.modules.get(&module_path).ok_or_else(|| {
                PyRuntimeError::new_err(format!("Module not loaded: {}", module_path))
            })?;
            let func = module.getattr(py, func_name)?;

            let py_args: Vec<PyObject> =
                args.iter().map(|arg| self.convert_to_python(arg)).collect();

            let result = if py_args.is_empty() {
                func.call0(py)?
            } else {
                func.call1(py, PyTuple::new_bound(py, py_args.as_slice()))?
            };
            Ok(self.convert_from_python(result))
        })
        .map_err(|e: PyErr| Box::new(e) as Box<dyn Error>)
    }
}
