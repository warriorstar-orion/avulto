extern crate dreammaker;

use std::{collections::HashMap, path::PathBuf};

use dreammaker::{
    FileId, FileList, Location,
    ast::{Spanned, Statement},
    objtree::NodeIndex,
};
use nodes::Node;
use pyo3::{
    Bound, IntoPyObject, IntoPyObjectExt, Py, PyAny, PyRef, PyResult, Python,
    exceptions::{PyException, PyKeyError, PyOSError, PyRuntimeError, PyValueError},
    pyclass, pymethods,
    types::{PyAnyMethods, PyList, PyString, PyStringMethods},
};
use pyo3_stub_gen::{create_exception, derive::*};

use crate::{
    dme::{type_def::TypeDef, var_def::VarDef},
    helpers,
    path::{self, TypePath},
};

pub mod expr_parse;
pub mod expr_walk;
pub mod expression;
pub mod node_parse;
pub mod node_walk;
pub mod nodes;
pub mod operators;
pub mod prefab;
pub mod proc_def;
pub mod type_def;
pub mod var_def;

create_exception!(avulto.exceptions, EmptyProcError, PyException);
create_exception!(avulto.exceptions, MissingTypeError, PyException);
create_exception!(avulto.exceptions, MissingProcError, PyException);

#[pyclass(module = "avulto")]
pub struct DmeTypeAccessor {
    pub dme: Py<Dme>,
}

impl DmeTypeAccessor {
    fn convert_path(&self, path: &Bound<PyAny>) -> Result<(String, String), String> {
        let objpath = if let Ok(patht) = path.extract::<path::TypePath>() {
            patht.rel
        } else if let Ok(pystr) = path.cast::<PyString>() {
            pystr.to_string()
        } else {
            return Err(format!("invalid path {:?}", path));
        };

        // TODO(wso): horrid
        if objpath.as_str().eq("/") {
            Ok((objpath, "".into()))
        } else {
            Ok((objpath.clone(), objpath.clone()))
        }
    }
}

#[pymethods]
impl DmeTypeAccessor {
    fn __getitem__(&self, path: &Bound<PyAny>, py: Python<'_>) -> PyResult<Py<TypeDef>> {
        let dme = self.dme.bind(py).borrow();
        if let Ok((obj_path, search_string)) = self.convert_path(path) {
            match dme.objtree.find(&search_string) {
                Some(type_ref) => {
                    let type_ref_index = type_ref.index();
                    let source_loc = self
                        .dme
                        .borrow(py)
                        .file_data
                        .fill_source_loc(&type_ref.location);
                    let dme = dme
                        .into_pyobject(py)
                        .expect("passing dme")
                        .clone()
                        .as_unbound()
                        .clone_ref(py)
                        .into_any();
                    Ok(TypeDef {
                        dme,
                        path: TypePath::make_trusted(obj_path.as_str()),
                        node_index: type_ref_index,
                        source_loc,
                    }
                    .into_pyobject(py)
                    .expect("building typedecl")
                    .into())
                }
                None => Err(PyKeyError::new_err(format!(
                    "unrecognized path {}",
                    obj_path
                ))),
            }
        } else {
            Err(PyRuntimeError::new_err("error looking up type".to_string()))
        }
    }

    fn __contains__(&self, path: &Bound<PyAny>, py: Python<'_>) -> PyResult<bool> {
        let dme = self.dme.bind(py).borrow();
        if let Ok((_, search_string)) = self.convert_path(path) {
            match dme.objtree.find(&search_string) {
                Some(_) => Ok(true),
                None => Ok(false),
            }
        } else {
            Err(PyRuntimeError::new_err("error looking up type".to_string()))
        }
    }
}

/// A representation of a single Dreammaker environment.
#[gen_stub_pyclass]
#[pyclass(module = "avulto", name = "DME")]
pub struct Dme {
    pub objtree: dreammaker::objtree::ObjectTree,
    /// The original filename of the DME.
    #[pyo3(get)]
    filepath: PathBuf,
    procs_parsed: bool,
    pub(crate) file_data: FileData,
}

pub struct FileData {
    pub(crate) file_ids: HashMap<FileId, PathBuf>,
}

impl FileData {
    pub fn fill_source_loc(&self, source_loc: &Location) -> SourceLoc {
        if self.file_ids.contains_key(&source_loc.file) {
            return SourceLoc {
                file_path: Some(self.file_ids[&source_loc.file].clone()),
                line: source_loc.line,
                column: source_loc.column,
            };
        }
        SourceLoc::builtin()
    }
}

/// Information about the location of a source token in the tree.
#[gen_stub_pyclass]
#[derive(Clone)]
#[pyclass(frozen, module = "avulto")]
pub struct SourceLoc {
    /// The file path of the source location.
    #[pyo3(get)]
    pub file_path: Option<PathBuf>,
    /// The line number, starting at 1.
    #[pyo3(get)]
    pub line: u32,
    /// The column number, starting at 1.
    #[pyo3(get)]
    pub column: u16,
}

impl SourceLoc {
    pub(crate) fn builtin() -> SourceLoc {
        SourceLoc {
            file_path: None,
            line: 1,
            column: 1,
        }
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl SourceLoc {
    fn __str__(&self) -> PyResult<String> {
        self.__repr__()
    }

    fn __repr__(&self) -> PyResult<String> {
        let file_path = match &self.file_path {
            Some(pth) => pth.to_str().unwrap_or("(unknown)"),
            None => "(builtins)",
        };
        Ok(format!("{}:{}:{}", file_path, self.line, self.column))
    }

    fn is_builtin(&self) -> PyResult<bool> {
        Ok(self.file_path.is_none())
    }
}

impl FileData {
    fn from_file_list(file_list: &FileList) -> Self {
        let mut result = FileData {
            file_ids: HashMap::default(),
        };
        file_list.for_each(|path| {
            result.file_ids.insert(
                file_list.get_id(path).unwrap(),
                std::path::Path::new(path).to_path_buf(),
            );
        });

        result
    }
}

impl Dme {
    fn collect_child_paths(&self, needle: &TypePath, strict: bool, out: &mut Vec<TypePath>) {
        for ty in self.objtree.iter_types() {
            // special handling for root
            if ty.path.is_empty() && needle.abs.eq("/") {
                if !strict {
                    out.push(TypePath::root());
                }
                continue;
            }
            let trusted = TypePath::make_trusted(&ty.path.clone());
            if needle.internal_parent_of_string(&trusted.abs, strict) {
                out.push(trusted);
            }
        }

        out.sort();
        out.dedup();
    }

    pub fn walk_stmt(
        self_: PyRef<'_, Self>,
        stmt: &Spanned<Statement>,
        walker: &Bound<PyAny>,
        py: Python<'_>,
    ) -> PyResult<()> {
        let source_loc = self_.file_data.fill_source_loc(&stmt.location);
        let node = Node::from_statement(py, &stmt.elem, source_loc, &self_.file_data);
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
    ) -> PyResult<Py<PyAny>> {
        let objtree = &self.objtree;
        let type_def = &objtree[node_index];

        if let Some(var) = type_def.vars.get(&name) {
            let declared_type = var
                .declaration
                .as_ref()
                .map(|decl| TypePath::from_tree_path(&decl.var_type.type_path));
            let const_val = var
                .value
                .constant
                .as_ref()
                .map(helpers::constant_to_python_value);
            let mut source_loc: SourceLoc = SourceLoc::builtin();
            if !var.value.location.is_builtins() {
                source_loc = self.file_data.fill_source_loc(&var.value.location);
            } else if let Some(decl) = &var.declaration {
                if !decl.location.is_builtins() {
                    source_loc = self.file_data.fill_source_loc(&decl.location);
                }
            }
            return VarDef {
                name,
                type_path: TypePath::make_trusted(&type_def.path).into_py_any(py).unwrap(),
                declared_type,
                const_val,
                source_loc,
            }
            .into_py_any(py);
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

#[gen_stub_pymethods]
#[pymethods]
impl Dme {
    /// Creates a DME from the given `filename`.
    ///
    /// If parse_procs is True, the entire AST of the codebase is traversed.
    /// This is slower than the default but provides more reflection
    /// information.
    #[staticmethod]
    #[pyo3(signature = (filename, parse_procs=false))]
    fn from_file(
        #[gen_stub(override_type(type_repr = "os.PathLike[builtins.str] | builtins.str", imports=("builtins", "os")))]
        filename: &Bound<PyAny>,
        parse_procs: bool,
    ) -> PyResult<Dme> {
        let path = if let Ok(path) = filename.extract::<std::path::PathBuf>() {
            path
        } else if let Ok(pystr) = filename.cast::<PyString>() {
            std::path::Path::new(&pystr.to_string()).to_path_buf()
        } else {
            return Err(PyValueError::new_err(format!(
                "invalid filename {}",
                filename
            )));
        };

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

        let dme = Dme {
            objtree: tree,
            filepath: path,
            procs_parsed: parse_procs,
            file_data: FileData::from_file_list(ctx.file_list()),
        };
        Ok(dme)
    }

    /// A mapping of paths in the DME to their TypeDecls.
    #[gen_stub(override_return_type(type_repr="builtins.dict[Path | builtins.str, TypeDef]", imports=("builtins")))]
    #[getter]
    fn get_types(self_: PyRef<'_, Self>, py: Python<'_>) -> PyResult<Py<DmeTypeAccessor>> {
        Py::new(
            py,
            DmeTypeAccessor {
                dme: self_.into_pyobject(py)?.unbind(),
            },
        )
    }

    /// Returns a list of type paths with the given `prefix`.
    #[gen_stub(override_return_type(type_repr="builtins.list[Path]", imports=("builtins")))]
    fn typesof(
        &self,
        #[gen_stub(override_type(type_repr = "Path | builtins.str", imports=("builtins")))]
        prefix: &Bound<PyAny>,
        py: Python<'_>,
    ) -> PyResult<Py<PyList>> {
        let mut out: Vec<TypePath> = Vec::new();

        let prefix_path = if let Ok(path) = prefix.extract::<path::TypePath>() {
            path
        } else if let Ok(pystr) = prefix.cast::<PyString>() {
            match TypePath::make_untrusted(pystr.to_str()?) {
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

    /// Returns a list of type paths with the given `prefix`, excluding `prefix` itself.
    #[gen_stub(override_return_type(type_repr="builtins.list[Path]", imports=("builtins")))]
    fn subtypesof(
        &self,
        #[gen_stub(override_type(type_repr = "Path | builtins.str", imports=("builtins")))]
        prefix: &Bound<PyAny>,
        py: Python<'_>,
    ) -> PyResult<Py<PyList>> {
        let mut out: Vec<TypePath> = Vec::new();

        let prefix_path = if let Ok(path) = prefix.extract::<path::TypePath>() {
            path
        } else if let Ok(pystr) = prefix.cast::<PyString>() {
            match TypePath::make_untrusted(pystr.to_str()?) {
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

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("<DME {:?}>", self.filepath))
    }
}
