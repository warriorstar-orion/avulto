extern crate dreammaker;

use dreammaker::ast::Statement;
use nodes::Node;
use pyo3::{
    create_exception,
    exceptions::{PyException, PyOSError, PyRuntimeError, PyValueError},
    pyclass, pymethods,
    types::{PyAnyMethods, PyList, PyString, PyStringMethods},
    Bound, IntoPy, Py, PyAny, PyObject, PyRef, PyResult, Python, ToPyObject,
};

use crate::{
    path::{self, Path},
    typedecl::TypeDecl,
};

pub mod expression;
pub mod nodes;
pub mod operators;
pub mod prefab;

create_exception!(avulto.exceptions, EmptyProcError, PyException);
create_exception!(avulto.exceptions, MissingTypeError, PyException);
create_exception!(avulto.exceptions, MissingProcError, PyException);

#[pyclass(module = "avulto", name = "DME")]
pub struct Dme {
    pub objtree: dreammaker::objtree::ObjectTree,
    #[pyo3(get)]
    filepath: Py<PyAny>,
    procs_parsed: bool,
}

impl Dme {
    fn collect_child_paths(&self, needle: &Path, strict: bool, out: &mut Vec<Path>) {
        for ty in self.objtree.iter_types() {
            // special handling for root
            if ty.path.is_empty() && needle.abs.eq("/") {
                if !strict {
                    out.push(Path::root());
                }
                continue;
            }
            let trusted = Path::make_trusted(&ty.path.clone());
            if needle.internal_parent_of_string(&trusted.abs, strict) {
                out.push(trusted);
            }
        }

        out.sort();
        out.dedup();
    }

    pub fn walk_stmt(
        &self,
        stmt: &Statement,
        walker: &Bound<PyAny>,
        py: Python<'_>,
    ) -> PyResult<()> {
        let node = Node::from_statement(py, stmt);
        Node::walk(
            node.into_py(py).downcast_bound::<Node>(py).unwrap(),
            py,
            walker,
        )?;
        Ok(())
    }
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
            return Err(PyValueError::new_err(format!(
                "invalid filename {}",
                filename
            )));
        };
        let pathlib = py.import_bound(pyo3::intern!(py, "pathlib"))?;
        if !path.is_file() {
            return Err(PyOSError::new_err(format!("file not found: {:?}", path)));
        }
        let ctx = dreammaker::Context::default();
        let pp = match dreammaker::preprocessor::Preprocessor::new(&ctx, path.clone()) {
            Ok(pp) => pp,
            Err(e) => {
                return Err(PyOSError::new_err(format!(
                    "error opening {:?}: {}",
                    path, e
                )));
            }
        };
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
            procs_parsed: parse_procs,
        })
    }

    fn typedecl(
        self_: PyRef<'_, Self>,
        path: &Bound<PyAny>,
        py: Python<'_>,
    ) -> PyResult<Py<PyAny>> {
        let objpath = if let Ok(patht) = path.extract::<path::Path>() {
            patht.rel
        } else if let Ok(pystr) = path.downcast::<PyString>() {
            pystr.to_string()
        } else {
            return Err(PyValueError::new_err(format!("invalid path {:?}", path)));
        };

        let search_string = if objpath.as_str().eq("/") {
            ""
        } else {
            objpath.as_str()
        };
        match self_.objtree.find(search_string) {
            Some(_) => Ok(TypeDecl {
                dme: self_.into_py(py),
                path: Path::make_trusted(objpath.as_str()).into_py(py),
            }
            .into_py(py)),
            None => Err(PyRuntimeError::new_err(format!(
                "cannot find path {}",
                objpath
            ))),
        }
    }

    fn typesof(&self, prefix: &Bound<PyAny>, py: Python<'_>) -> PyResult<PyObject> {
        let mut out: Vec<Path> = Vec::new();

        let prefix_path = if let Ok(path) = prefix.extract::<path::Path>() {
            path
        } else if let Ok(pystr) = prefix.downcast::<PyString>() {
            match Path::make_untrusted(pystr.to_str()?) {
                Ok(p) => p,
                Err(e) => {
                    return Err(PyRuntimeError::new_err(e));
                }
            }
        } else {
            return Err(PyValueError::new_err(format!("invalid path {:?}", prefix)));
        };
        self.collect_child_paths(&prefix_path, false, &mut out);

        Ok(PyList::new_bound(py, out.into_iter().map(|m| m.into_py(py))).to_object(py))
    }

    fn subtypesof(&self, prefix: &Bound<PyAny>, py: Python<'_>) -> PyResult<PyObject> {
        let mut out: Vec<Path> = Vec::new();

        let prefix_path = if let Ok(path) = prefix.extract::<path::Path>() {
            path
        } else if let Ok(pystr) = prefix.downcast::<PyString>() {
            match Path::make_untrusted(pystr.to_str()?) {
                Ok(p) => p,
                Err(e) => {
                    return Err(PyRuntimeError::new_err(e));
                }
            }
        } else {
            return Err(PyValueError::new_err(format!("invalid path {:?}", prefix)));
        };
        self.collect_child_paths(&prefix_path, true, &mut out);

        Ok(PyList::new_bound(py, out.into_iter().map(|m| m.into_py(py))).to_object(py))
    }

    pub fn walk_proc(
        &self,
        path: &Bound<PyAny>,
        proc: &Bound<PyAny>,
        walker: &Bound<PyAny>,
        py: Python<'_>,
    ) -> PyResult<()> {
        if !&self.procs_parsed {
            return Err(PyRuntimeError::new_err(
                "parse_procs=True was not included in DME's constructor",
            ));
        }
        let objtree = &self.objtree;
        let objpath = if let Ok(patht) = path.extract::<path::Path>() {
            patht.rel
        } else if let Ok(pystr) = path.downcast::<PyString>() {
            pystr.to_string()
        } else {
            return Err(PyRuntimeError::new_err(
                "cannot coerce path to string".to_string(),
            ));
        };
        let procname = if let Ok(proct) = proc.downcast::<PyString>() {
            proct.to_string()
        } else {
            return Err(PyRuntimeError::new_err(
                "cannot coerce proc name to string".to_string(),
            ));
        };

        if let Some(ty) = objtree.find(&objpath) {
            if let Some(p) = ty.get_proc(&procname) {
                if let Some(ref code) = p.get().code {
                    for stmt in code.iter() {
                        self.walk_stmt(&stmt.elem, walker, py)?;
                    }
                } else {
                    return Err(EmptyProcError::new_err(format!(
                        "no code statements found in proc {} on type {}",
                        procname, objpath
                    )));
                }
            } else {
                return Err(MissingProcError::new_err(format!(
                    "cannot find proc {} on type {}",
                    procname, objpath
                )));
            }
        } else {
            return Err(MissingTypeError::new_err(format!(
                "cannot find type {}",
                objpath
            )));
        };

        Ok(())
    }

    fn __repr__(&self, py: Python<'_>) -> PyResult<String> {
        Ok(format!(
            "<DME {}>",
            self.filepath.getattr(py, "name").unwrap()
        ))
    }
}
