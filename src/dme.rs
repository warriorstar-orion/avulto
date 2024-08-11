extern crate dreammaker;

use itertools::Itertools;
use pyo3::{
    exceptions::PyRuntimeError,
    prelude::*,
    types::{PyList, PyString},
};

use crate::path::Path;
use crate::{path, typedecl::TypeDecl};

#[pyclass(module = "avulto", name = "DME")]
pub struct Dme {
    pub objtree: dreammaker::objtree::ObjectTree,
    #[pyo3(get)]
    filepath: Py<PyAny>,
}

#[pymethods]
impl Dme {
    #[staticmethod]
    #[pyo3(signature = (filename, parse_procs=false))]
    fn from_file(filename: &Bound<PyAny>, parse_procs: bool, py: Python<'_>) -> PyResult<Dme> {
        let path = if let Ok(path) = filename.extract::<std::path::PathBuf>() {
            path
        } else if let Ok(pystr) = filename.downcast::<PyString>() {
            std::path::Path::new(&pystr.to_string()).to_path_buf()
        } else {
            return Err(PyRuntimeError::new_err(format!(
                "invalid filename {}",
                filename
            )));
        };
        let pathlib = py.import_bound(pyo3::intern!(py, "pathlib"))?;

        let ctx = dreammaker::Context::default();
        let pp = dreammaker::preprocessor::Preprocessor::new(&ctx, path.clone())
            .expect("i/o error opening .dme");
        let indents = dreammaker::indents::IndentProcessor::new(&ctx, pp);
        let mut parser = dreammaker::parser::Parser::new(&ctx, indents);
        if parse_procs {
            parser.enable_procs();
        }

        let (fatal_errored, tree) = parser.parse_object_tree_2();
        if fatal_errored {
            return Err(PyRuntimeError::new_err(format!(
                "failed to parse DME environment {}",
                filename
            )));
        }

        let pathlib_path = pathlib.call_method1(pyo3::intern!(py, "Path"), (path,))?;
        Ok(Dme {
            objtree: tree,
            filepath: pathlib_path.into_py(py),
        })
    }

    fn typedecl(self_: PyRef<'_, Self>, path: &Bound<PyAny>, py: Python<'_>) -> PyResult<Py<PyAny>> {
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

    fn paths_prefixed(&self, prefix: &Bound<PyAny>, py: Python<'_>) -> PyResult<PyObject> {
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
        Ok(PyList::new_bound(py, x.into_iter().map(|m| m.into_py(py))).to_object(py))
    }
}
