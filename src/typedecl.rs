extern crate dreammaker;

use dreammaker::constants::Constant;
use itertools::Itertools;
use pyo3::{exceptions::PyRuntimeError, prelude::*, types::PyList};

use crate::{dme::Dme, helpers};

#[pyclass(module = "avulto")]
pub struct TypeDecl {
    pub dme: Py<PyAny>,
    #[pyo3(get)]
    pub path: String,
}

#[pymethods]
impl TypeDecl {
    pub fn varnames(&self, py: Python<'_>) -> PyResult<PyObject> {
        let mut out: Vec<String> = Vec::new();
        let bound = self.dme.downcast_bound::<Dme>(py).unwrap();

        for ty in bound.borrow().objtree.iter_types() {
            if ty.path == self.path {
                for (name, _) in ty.vars.iter() {
                    out.push(name.clone());
                }
                let mut x = out.into_iter().unique().collect::<Vec<String>>();
                x.sort();
                return Ok(PyList::new_bound(py, x).into_py(py));
            }
        }

        Err(PyRuntimeError::new_err(format!(
            "cannot find type {}",
            self.path
        )))
    }

    pub fn value(&self, name: String, py: Python<'_>) -> PyResult<PyObject> {
        let bound = self.dme.downcast_bound::<Dme>(py).unwrap();

        for ty in bound.borrow().objtree.iter_types() {
            if ty.path == self.path {
                if let Some(c) = ty.get_value(&name) {
                    return Ok(helpers::constant_to_python_value(
                        c.constant.as_ref().unwrap_or(Constant::null()),
                    ));
                } else {
                    return Ok(py.None());
                }
            }
        }

        Err(PyRuntimeError::new_err(format!(
            "cannot find value for {}/{}",
            self.path, name
        )))
    }

    pub fn procnames(&self, py: Python<'_>) -> PyResult<PyObject> {
        let mut out: Vec<String> = Vec::new();
        let bound = self.dme.downcast_bound::<Dme>(py).unwrap();

        for ty in bound.borrow().objtree.iter_types() {
            if ty.path == self.path {
                for (name, _) in ty.procs.iter() {
                    out.push(name.clone());
                }
                let mut x = out.into_iter().unique().collect::<Vec<String>>();
                x.sort();
                return Ok(PyList::new_bound(py, x).into_py(py));
            }
        }

        Err(PyRuntimeError::new_err(format!(
            "cannot find type {}",
            self.path
        )))
    }
}
