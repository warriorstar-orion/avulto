extern crate dreammaker;

use dreammaker::constants::Constant;
use itertools::Itertools;
use pyo3::{exceptions::PyRuntimeError, prelude::*, types::PyList};

use crate::{dme::Dme, helpers, path};

#[pyclass(module = "avulto")]
pub struct TypeDecl {
    pub dme: Py<PyAny>,
    #[pyo3(get)]
    pub path: Py<PyAny>,
}

#[pymethods]
impl TypeDecl {
    pub fn var_names(&self, py: Python<'_>) -> PyResult<PyObject> {
        let mut out: Vec<String> = Vec::new();
        let bound = self.dme.downcast_bound::<Dme>(py).unwrap();

        let _path = self.path.extract::<path::Path>(py)?;
        for ty in bound.borrow().objtree.iter_types() {
            if ty.path == _path.rel {
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
        let _path = self.path.extract::<path::Path>(py)?;

        for ty in bound.borrow().objtree.iter_types() {
            if ty.path == _path.rel {
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

    pub fn proc_names(&self, py: Python<'_>) -> PyResult<PyObject> {
        let mut out: Vec<String> = Vec::new();
        let bound = self.dme.downcast_bound::<Dme>(py).unwrap();
        let _path = self.path.extract::<path::Path>(py)?;

        for ty in bound.borrow().objtree.iter_types() {
            if ty.path == _path.rel {
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

    pub fn walk_proc(
        &self,
        proc: &Bound<PyAny>,
        walker: &Bound<PyAny>,
        py: Python<'_>,
    ) -> PyResult<()> {
        let bound = self.dme.downcast_bound::<Dme>(py).unwrap();
        bound.borrow().walk_proc(self.path.bind(py), proc, walker, py)
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("<TypeDecl {}>", self.path))
    }
}
