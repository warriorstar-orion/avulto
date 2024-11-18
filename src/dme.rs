extern crate dreammaker;

use std::collections::HashMap;

use dreammaker::{
    ast::{Spanned, Statement},
    objtree::NodeIndex,
    FileId, FileList,
};
use nodes::{Node, OriginalSourceLocation};
use pyo3::{
    create_exception,
    exceptions::{PyException, PyOSError, PyRuntimeError, PyValueError},
    pyclass, pymethods,
    types::{PyAnyMethods, PyList, PyString, PyStringMethods},
    Bound, IntoPyObject, Py, PyAny, PyObject, PyRef, PyResult, Python,
};

use crate::{
    helpers,
    path::{self, Path},
    typedecl::{TypeDecl, VarDecl},
};

pub mod expr_parse;
pub mod expr_walk;
pub mod expression;
pub mod node_parse;
pub mod node_walk;
pub mod nodes;
pub mod operators;
pub mod prefab;

create_exception!(avulto.exceptions, EmptyProcError, PyException);
create_exception!(avulto.exceptions, MissingTypeError, PyException);
create_exception!(avulto.exceptions, MissingProcError, PyException);

#[pyclass(module = "avulto")]
pub struct DmeTypeAccessor {
    pub dme: Py<Dme>,
}

#[pymethods]
impl DmeTypeAccessor {
    fn __getitem__(&self, path: &Bound<PyAny>, py: Python<'_>) -> PyResult<Py<TypeDecl>> {
        let dme = self.dme.bind(py).borrow();
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
        match dme.objtree.find(search_string) {
            Some(type_ref) => {
                let type_ref_index = type_ref.index();
                let dme = dme
                    .into_pyobject(py)
                    .expect("passing dme")
                    .clone()
                    .as_unbound()
                    .clone_ref(py)
                    .into_any();
                Ok(TypeDecl {
                    dme,
                    path: Path::make_trusted(objpath.as_str()),
                    node_index: type_ref_index,
                }
                .into_pyobject(py)
                .expect("building typedecl")
                .into())
            }
            None => Err(PyRuntimeError::new_err(format!(
                "cannot find path {}",
                objpath
            ))),
        }
    }
}

#[pyclass(module = "avulto", name = "DME")]
pub struct Dme {
    pub objtree: dreammaker::objtree::ObjectTree,
    #[pyo3(get)]
    filepath: Py<PyAny>,
    procs_parsed: bool,
    pub(crate) file_data: Py<FileData>,
}

#[pyclass]
pub struct FileData {
    pub(crate) file_ids: HashMap<FileId, Py<PyAny>>,
}

#[pyclass]
pub struct FilledSourceLocation {
    #[pyo3(get)]
    pub file_path: Py<PyAny>,
    /// The line number, starting at 1.
    #[pyo3(get)]
    pub line: u32,
    /// The column number, starting at 1.
    #[pyo3(get)]
    pub column: u16,
}

impl FileData {
    fn from_file_list(file_list: &FileList, py: Python<'_>) -> Self {
        let pathlib = py.import(pyo3::intern!(py, "pathlib")).unwrap();
        let mut result = FileData {
            file_ids: HashMap::default(),
        };
        file_list.for_each(|path| {
            result.file_ids.insert(
                file_list.get_id(path).unwrap(),
                pathlib
                    .call_method1(pyo3::intern!(py, "Path"), (path,))
                    .unwrap()
                    .unbind(),
            );
        });

        result
    }
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

    pub fn populate_source_loc(
        &self,
        loc: &Option<Py<OriginalSourceLocation>>,
        py: Python<'_>,
    ) -> Py<PyAny> {
        // TODO: what the fuck
        loc.as_ref()
            .map(|f| {
                let g = f.borrow(py);
                FilledSourceLocation {
                    file_path: self.file_data.borrow(py).file_ids[&g.file].clone_ref(py),
                    line: g.line,
                    column: g.column,
                }
            })
            .map_or(py.None(), |g| {
                g.into_pyobject(py).unwrap().into_any().into()
            })
    }

    pub fn walk_stmt(
        self_: PyRef<'_, Self>,
        stmt: &Spanned<Statement>,
        walker: &Bound<PyAny>,
        py: Python<'_>,
    ) -> PyResult<()> {
        let node = Node::from_statement(py, &stmt.elem, Some(stmt.location));
        Node::walk(node.bind(py), &self_.into_pyobject(py).unwrap(), walker, py)?;
        Ok(())
    }

    pub fn walk_proc(
        self_: &PyRef<'_, Self>,
        node_index: NodeIndex,
        proc_name: String,
        walker: &Bound<PyAny>,
        proc_index: usize,
        py: Python<'_>,
    ) -> PyResult<()> {
        if !&self_.procs_parsed {
            return Err(PyRuntimeError::new_err(
                "parse_procs=True was not included in DME's constructor",
            ));
        }
        let objtree = &self_.objtree;
        let type_def = &objtree[node_index];
        if let Some(proc) = type_def.procs.get(&proc_name) {
            if let Some(ref code) = proc.value[proc_index].code {
                for stmt in code.iter() {
                    Dme::walk_stmt(self_.into_pyobject(py).unwrap().borrow(), stmt, walker, py)?;
                }
            } else {
                return Err(EmptyProcError::new_err(format!(
                    "no code statements found in proc {} on type {}",
                    proc_name, type_def.path
                )));
            }
        } else {
            return Err(MissingProcError::new_err(format!(
                "cannot find proc {} on type {}",
                proc_name, type_def.path
            )));
        }

        Ok(())
    }

    pub fn get_var_decl(
        &self,
        name: String,
        node_index: NodeIndex,
        parents: bool,
        py: Python<'_>,
    ) -> PyResult<PyObject> {
        let objtree = &self.objtree;
        let type_def = &objtree[node_index];

        if let Some(var) = type_def.vars.get(&name) {
            let declared_type = var
                .declaration
                .as_ref()
                .map(|decl| Path::from_tree_path(&decl.var_type.type_path));
            let const_val = var
                .value
                .constant
                .as_ref()
                .map(helpers::constant_to_python_value);
            return Ok(VarDecl {
                name,
                declared_type,
                const_val,
            }
            .into_pyobject(py)
            .expect("building var_decl")
            .into_any()
            .unbind());
        }

        if parents && !type_def.is_root() {
            if let Some(parent_type_index) = type_def.parent_type_index() {
                return self.get_var_decl(name, parent_type_index, parents, py);
            }
        }

        Err(PyRuntimeError::new_err(format!(
            "cannot find value for {}/{}",
            type_def.path, name
        )))
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
        let pathlib = py.import(pyo3::intern!(py, "pathlib"))?;
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
        let dme = Dme {
            objtree: tree,
            filepath: pathlib_path.into(),
            procs_parsed: parse_procs,
            file_data: Py::new(py, FileData::from_file_list(ctx.file_list(), py))
                .expect("passing file list"),
        };
        Ok(dme)
    }

    #[getter]
    fn get_types(self_: PyRef<'_, Self>, py: Python<'_>) -> PyResult<Py<DmeTypeAccessor>> {
        Py::new(
            py,
            DmeTypeAccessor {
                dme: self_.into_pyobject(py)?.unbind(),
            },
        )
    }

    fn type_decl(
        self_: PyRef<'_, Self>,
        path: &Bound<PyAny>,
        py: Python<'_>,
    ) -> PyResult<Py<TypeDecl>> {
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
            Some(type_ref) => {
                let type_ref_index = type_ref.index();
                let dme = self_
                    .into_pyobject(py)
                    .expect("passing dme")
                    .clone()
                    .as_unbound()
                    .clone_ref(py)
                    .into_any();
                Ok(TypeDecl {
                    dme,
                    path: Path::make_trusted(objpath.as_str()),
                    node_index: type_ref_index,
                }
                .into_pyobject(py)
                .expect("building typedecl")
                .into())
            }
            None => Err(PyRuntimeError::new_err(format!(
                "cannot find path {}",
                objpath
            ))),
        }
    }

    fn typesof(&self, prefix: &Bound<PyAny>, py: Python<'_>) -> PyResult<Py<PyList>> {
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

        Ok(PyList::new(py, out)?.unbind().clone_ref(py))
    }

    fn subtypesof(&self, prefix: &Bound<PyAny>, py: Python<'_>) -> PyResult<Py<PyList>> {
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

        Ok(PyList::new(py, out)?.unbind().clone_ref(py))
    }

    fn __repr__(&self, py: Python<'_>) -> PyResult<String> {
        Ok(format!(
            "<DME {}>",
            self.filepath.getattr(py, "name").unwrap()
        ))
    }
}
