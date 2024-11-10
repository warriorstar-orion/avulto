extern crate dreammaker;

use dreammaker::constants::Constant;
use itertools::Itertools;
use pyo3::{
    exceptions::PyRuntimeError,
    prelude::*,
    types::{PyList, PyString},
};

use crate::{
    dme::Dme,
    helpers,
    path::{self, Path},
};

#[pyclass(module = "avulto")]
pub struct TypeDecl {
    pub dme: Py<PyAny>,
    #[pyo3(get)]
    pub path: Py<PyAny>,
}

#[pyclass(module = "avulto")]
pub struct ProcArg {
    #[pyo3(get)]
    pub arg_name: Py<PyAny>,
    #[pyo3(get)]
    pub arg_type: Py<PyAny>,
}

#[pymethods]
impl ProcArg {
    fn __str__(&self, py: Python<'_>) -> PyResult<String> {
        self.__repr__(py)
    }

    fn __repr__(&self, py: Python<'_>) -> PyResult<String> {
        if self.arg_type.is_none(py) {
            return Ok(format!("{}", self.arg_name));
        }
        Ok(format!("{}/{}", self.arg_type, self.arg_name))
    }
}

#[pyclass(module = "avulto")]
pub struct ProcDecl {
    pub dme: Py<PyAny>,
    #[pyo3(get)]
    pub type_path: Py<PyAny>,
    #[pyo3(get)]
    pub proc_name: Py<PyAny>,
    #[pyo3(get)]
    pub args: Py<PyAny>,
}

#[pymethods]
impl ProcDecl {
    fn __str__(&self) -> PyResult<String> {
        self.__repr__()
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("<Proc {}/proc/{}>", self.type_path, self.proc_name))
    }
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

    pub fn proc_decls(&self, proc: &Bound<PyAny>, py: Python<'_>) -> PyResult<PyObject> {
        let bound = self.dme.downcast_bound::<Dme>(py).unwrap();
        let _path = self.path.extract::<path::Path>(py)?;

        let mut out: Vec<ProcDecl> = Vec::new();

        let proc_str = if let Ok(proc_str) = proc.downcast::<PyString>() {
            proc_str.to_string()
        } else {
            return Err(PyRuntimeError::new_err(
                "cannot coerce path to string".to_string(),
            ));
        };

        for ty in bound.borrow().objtree.iter_types() {
            if ty.path == _path.rel {
                for (name, type_proc) in ty.procs.iter() {
                    if name.eq(&proc_str) {
                        for proc_value in type_proc.value.iter() {
                            if !proc_value.location.is_builtins() {
                                let mut args_out: Vec<ProcArg> = Vec::new();
                                for arg in proc_value.parameters.iter() {
                                    let arg_typepath = if arg.var_type.type_path.is_empty() {
                                        py.None()
                                    } else {
                                        Path::new(
                                            ("/".to_string()
                                                + &arg
                                                    .var_type
                                                    .type_path
                                                    .iter()
                                                    .map(|f| f.as_str())
                                                    .join("/"))
                                                .as_str(),
                                        )?
                                        .into_py(py)
                                    };
                                    args_out.push(ProcArg {
                                        arg_name: arg.name.clone().into_py(py),
                                        arg_type: arg_typepath.into_py(py),
                                    });
                                }

                                out.push(ProcDecl {
                                    dme: self.dme.clone_ref(py),
                                    proc_name: name.clone().into_py(py),
                                    type_path: _path.clone().into_py(py),
                                    args: PyList::new_bound(
                                        py,
                                        args_out
                                            .into_iter()
                                            .map(|f| f.into_py(py))
                                            .collect::<Vec<Py<PyAny>>>(),
                                    )
                                    .into_py(py),
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok(PyList::new_bound(
            py,
            out.into_iter()
                .map(|f| f.into_py(py))
                .collect::<Vec<Py<PyAny>>>(),
        )
        .into_py(py))
    }

    pub fn walk_proc(
        &self,
        proc: &Bound<PyAny>,
        walker: &Bound<PyAny>,
        py: Python<'_>,
    ) -> PyResult<()> {
        let bound = self.dme.downcast_bound::<Dme>(py).unwrap();
        bound
            .borrow()
            .walk_proc(self.path.bind(py), proc, walker, py)
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("<TypeDecl {}>", self.path))
    }
}
