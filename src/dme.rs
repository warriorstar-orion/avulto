extern crate dreammaker;

use dreammaker::constants::Constant;
use itertools::Itertools;
use pyo3::{
    exceptions::PyRuntimeError,
    prelude::*,
    types::{PyList, PyString},
};

use crate::path;
use crate::{helpers, path::Path};

#[pyclass(module = "avulto", name = "DME")]
pub struct Dme {
    objtree: dreammaker::objtree::ObjectTree,
    #[pyo3(get)]
    filepath: Py<PyAny>,
}

#[pyclass(module = "avulto")]
pub struct TypeDecl {
    dme: Py<PyAny>,
    #[pyo3(get)]
    path: String,
}

#[pymethods]
impl Dme {
    #[staticmethod]
    fn from_file(filename: &PyAny, py: Python<'_>) -> PyResult<Dme> {
        let pathlib = py.import(pyo3::intern!(py, "pathlib"))?;

        let ctx = dreammaker::Context::default();
        if let Ok(path) = filename.extract::<std::path::PathBuf>() {
            let pathlib_path = pathlib.call_method1(pyo3::intern!(py, "Path"), (path.clone(),))?;
            return Ok(Dme {
                objtree: ctx.parse_environment(&path).unwrap(),
                filepath: pathlib_path.into_py(py),
            });
        } else if let Ok(pystr) = filename.downcast::<PyString>() {
            let pathlib_path = pathlib.call_method1(pyo3::intern!(py, "Path"), (pystr,))?;
            return Ok(Dme {
                objtree: ctx
                    .parse_environment(std::path::Path::new(&pystr.to_string()))
                    .unwrap(),
                filepath: pathlib_path.into_py(py),
            });
        };

        Err(PyRuntimeError::new_err(format!(
            "invalid filename {}",
            filename
        )))
    }

    fn typedecl(self_: PyRef<'_, Self>, path: &PyAny, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let objpath = if let Ok(patht) = path.extract::<path::Path>() {
            patht.0
        } else if let Ok(pystr) = path.downcast::<PyString>() {
            pystr.to_string()
        } else {
            return Err(PyRuntimeError::new_err(
                "cannot coerce path to string".to_string(),
            ));
        };

        match self_.objtree.find(objpath.as_str()) {
            Some(_) => Ok(TypeDecl {
                dme: self_.into_py(py),
                path: objpath.to_string(),
            }
            .into_py(py)),
            None => Err(PyRuntimeError::new_err(format!(
                "cannot find path {}",
                objpath
            ))),
        }
    }

    fn paths_prefixed(&self, prefix: &PyAny, py: Python<'_>) -> PyResult<Py<PyList>> {
        let mut out: Vec<Path> = Vec::new();

        if let Ok(path) = prefix.extract::<path::Path>() {
            for ty in self.objtree.iter_types() {
                if ty.path.starts_with(&path.0) {
                    out.push(Path(ty.path.clone()));
                }
            }
        } else if let Ok(pystr) = prefix.downcast::<PyString>() {
            for ty in self.objtree.iter_types() {
                if ty.path.starts_with(&pystr.to_string()) {
                    out.push(Path(ty.path.clone()));
                }
            }
        }

        let mut x = out.into_iter().unique().collect::<Vec<Path>>();
        x.sort();
        Ok(PyList::new(py, x.into_iter().map(|m| m.into_py(py))).into_py(py))
    }
}

#[pymethods]
impl TypeDecl {
    pub fn varnames(&self, py: Python<'_>) -> PyResult<Py<PyList>> {
        let mut out: Vec<String> = Vec::new();
        let dme: &PyCell<Dme> = self.dme.downcast(py).unwrap();

        for ty in dme.borrow().objtree.iter_types() {
            if ty.path == self.path {
                for (name, _) in ty.vars.iter() {
                    out.push(name.clone());
                }
                let mut x = out.into_iter().unique().collect::<Vec<String>>();
                x.sort();
                return Ok(PyList::new(py, x).into_py(py));
            }
        }

        Err(PyRuntimeError::new_err(format!(
            "cannot find type {}",
            self.path
        )))
    }

    pub fn value(&self, name: String, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let dme: &PyCell<Dme> = self.dme.downcast(py).unwrap();

        for ty in dme.borrow().objtree.iter_types() {
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
}
