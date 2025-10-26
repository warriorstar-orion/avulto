extern crate dreammaker;

use std::collections::HashSet;

use dreammaker::objtree::NodeIndex;
use pyo3::{
    exceptions::PyValueError,
    prelude::*,
    types::PyList,
};

use crate::{
    dme::{Dme, FilledSourceLocation},
    path::Path,
};

#[pyclass(module = "avulto")]
pub struct VarDecl {
    #[pyo3(get)]
    pub name: String,
    #[pyo3(get)]
    pub type_path: Py<PyAny>,
    #[pyo3(get)]
    pub declared_type: Option<Path>,
    #[pyo3(get)]
    pub const_val: Option<Py<PyAny>>,
    #[pyo3(get)]
    pub source_loc: Option<Py<PyAny>>,
}

#[pymethods]
impl VarDecl {
    fn __str__(&self) -> PyResult<String> {
        self.__repr__()
    }

    fn __repr__(&self) -> PyResult<String> {
        match &self.declared_type {
            None => Ok(format!("<Var {}>", self.name)),
            Some(p) => Ok(format!(
                "<Var {}/{}/{}>",
                self.type_path,
                p.rel.strip_prefix('/').unwrap(),
                self.name
            )),
        }
    }
}

#[pyclass(module = "avulto")]
pub struct TypeDecl {
    pub dme: Py<PyAny>,
    pub node_index: NodeIndex,
    #[pyo3(get)]
    pub path: Path,
    #[pyo3(get)]
    pub source_loc: Option<Py<PyAny>>,
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
    pub name: String,
    #[pyo3(get)]
    pub args: Py<PyAny>,

    type_index: NodeIndex,
    proc_index: usize,
    #[pyo3(get)]
    source_loc: Py<PyAny>,
}

#[pymethods]
impl ProcDecl {
    fn __str__(&self) -> PyResult<String> {
        self.__repr__()
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("<Proc {}/proc/{}>", self.type_path, self.name))
    }

    pub fn walk(&self, walker: &Bound<PyAny>, py: Python<'_>) -> PyResult<()> {
        let dme = self.dme.cast_bound::<Dme>(py).unwrap();
        Dme::walk_proc(
            &dme.borrow(),
            self.type_index,
            self.name.clone(),
            walker,
            self.proc_index,
            py,
        )
    }
}

#[pymethods]
impl TypeDecl {
    #[pyo3(signature = (declared=false, modified=false, unmodified=false))]
    pub fn var_names(
        &self,
        declared: bool,
        modified: bool,
        unmodified: bool,
        py: Python<'_>,
    ) -> PyResult<Py<PyList>> {
        if !declared && !modified && !unmodified {
            return Err(PyValueError::new_err(
                "at least one of `declared`, `modified`, or `unmodified` must be True",
            ));
        }

        let dme = self.dme.cast_bound::<Dme>(py).unwrap();
        let objtree = &dme.borrow().objtree;

        let search_string = if self.path.rel.eq("/") { "" } else { self.path.rel.as_str()};
        let mut type_ref = objtree.find(search_string);

        let mut leaf_declared_names: HashSet<String> = HashSet::new();
        let mut leaf_undeclared_names: HashSet<String> = HashSet::new();
        let mut parent_names: HashSet<String> = HashSet::new();

        while let Some(ty) = type_ref {
            for (var_name, type_var) in ty.vars.iter() {
                if ty.index() == self.node_index {
                    if let Some(_decl) = &type_var.declaration {
                        leaf_declared_names.insert(var_name.to_string());
                    } else {
                        leaf_undeclared_names.insert(var_name.to_string());
                    }
                } else {
                    parent_names.insert(var_name.to_string());
                }
            }
            type_ref = ty.parent_type_without_root();
        }

        let mut out: HashSet<&String> = HashSet::new();
        if unmodified {
            out = parent_names.difference(&leaf_declared_names).collect();
        }
        if modified {
            out.extend(&leaf_undeclared_names);
        }
        if declared {
            out.extend(&leaf_declared_names);
        }

        Ok(PyList::new(py, Vec::from_iter(out))?
            .into_pyobject(py)?
            .unbind())
    }

    #[pyo3(signature = (name, parents=true))]
    pub fn var_decl(&self, name: String, parents: bool, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let bound = self.dme.cast_bound::<Dme>(py).unwrap();
        let dme = bound.borrow();
        dme.get_var_decl(name, self.node_index, parents, py)
    }

    #[pyo3(signature = (declared=false, modified=false, unmodified=false))]
    pub fn proc_names(
        &self,
        declared: bool,
        modified: bool,
        unmodified: bool,
        py: Python<'_>,
    ) -> PyResult<Py<PyList>> {
        if !declared && !modified && !unmodified {
            return Err(PyValueError::new_err(
                "at least one of declared, modified, or unmodified must be True",
            ));
        }
        let dme = self.dme.cast_bound::<Dme>(py).unwrap();
        let objtree = &dme.borrow().objtree;

        let search_string = if self.path.rel.eq("/") { "" } else { self.path.rel.as_str()};
        let mut type_ref = objtree.find(search_string);

        let mut leaf_declared_names: HashSet<String> = HashSet::new();
        let mut leaf_undeclared_names: HashSet<String> = HashSet::new();
        let mut parent_names: HashSet<String> = HashSet::new();

        while let Some(ty) = type_ref {
            for (proc_name, type_proc) in ty.procs.iter() {
                if ty.index() == self.node_index {
                    if let Some(_decl) = &type_proc.declaration {
                        leaf_declared_names.insert(proc_name.to_string());
                    } else {
                        leaf_undeclared_names.insert(proc_name.to_string());
                    }
                } else {
                    parent_names.insert(proc_name.to_string());
                }
            }
            type_ref = ty.parent_type_without_root();
        }

        let mut out: HashSet<&String> = HashSet::new();
        if unmodified {
            out = parent_names.difference(&leaf_declared_names).collect();
        }
        if modified {
            out.extend(&leaf_undeclared_names);
        }
        if declared {
            out.extend(&leaf_declared_names);
        }

        Ok(PyList::new(py, Vec::from_iter(out))?
            .into_pyobject(py)?
            .unbind())
    }

    #[pyo3(signature = (name=None))]
    pub fn proc_decls(&self, name: Option<String>, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let dme = self.dme.cast_bound::<Dme>(py).unwrap();
        let objtree = &dme.borrow().objtree;
        let mut out: Vec<ProcDecl> = Vec::new();

        let type_def = &objtree[self.node_index];
        for (proc_name, proc) in type_def.procs.iter() {
            if name.as_ref().is_some_and(|p| !proc_name.eq(p)) {
                continue;
            }
            for (proc_index, proc_value) in proc.value.iter().enumerate() {
                if !proc_value.location.is_builtins() {
                    let mut args_out: Vec<ProcArg> = Vec::new();
                    for arg in proc_value.parameters.iter() {
                        let arg_typepath = if arg.var_type.type_path.is_empty() {
                            py.None()
                        } else {
                            Path::from_tree_path(&arg.var_type.type_path)
                                .into_pyobject(py)?
                                .into_any()
                                .unbind()
                        };
                        args_out.push(ProcArg {
                            arg_name: arg.name.clone().into_pyobject(py)?.into(),
                            arg_type: arg_typepath.into_pyobject(py)?.into(),
                        });
                    }

                    out.push(ProcDecl {
                        dme: self.dme.clone_ref(py),
                        name: proc_name.clone(),
                        type_path: self.path.clone().into_pyobject(py)?.into_any().unbind(),

                        args: PyList::new(
                            py,
                            args_out
                                .into_iter()
                                .map(|f| f.into_pyobject(py).unwrap().into_any().unbind())
                                .collect::<Vec<Py<PyAny>>>(),
                        )?
                        .into_pyobject(py)?
                        .into_any()
                        .unbind(),

                        proc_index,
                        type_index: self.node_index,
                        source_loc: FilledSourceLocation {
                            file_path: dme.borrow().file_data.borrow(py).file_ids
                                [&proc_value.location.file]
                                .clone_ref(py),
                            column: proc_value.location.column,
                            line: proc_value.location.line,
                        }
                        .into_pyobject(py)
                        .expect("passing proc decl source info")
                        .into_any()
                        .unbind(),
                    });
                }
            }
        }
        Ok(PyList::new(
            py,
            out.into_iter()
                .map(|f| f.into_pyobject(py).unwrap().into_any().unbind())
                .collect::<Vec<Py<PyAny>>>(),
        )?
        .into_any()
        .unbind())
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("<Type {}>", self.path.rel))
    }
}
