extern crate dreammaker;

use std::collections::HashSet;

use dreammaker::objtree::NodeIndex;
use pyo3::{
    Py, PyAny, PyResult, Python,
    exceptions::{PyKeyError, PyTypeError, PyValueError},
    prelude::*,
    types::PyList,
};
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};

use crate::{
    dme::{
        Dme, SourceLoc,
        proc_def::{ProcArg, ProcDef},
    },
    path::TypePath,
};

fn type_search_string(objtree: &dreammaker::objtree::ObjectTree, node_index: NodeIndex) -> String {
    let type_def = &objtree[node_index];
    let current_path = TypePath::make_trusted(&type_def.path);
    if current_path.rel.eq("/") {
        "".to_string()
    } else {
        current_path.rel.clone()
    }
}

fn collect_name_sets<F>(
    objtree: &dreammaker::objtree::ObjectTree,
    node_index: NodeIndex,
    mut visit_type: F,
) -> (HashSet<String>, HashSet<String>, HashSet<String>)
where
    F: FnMut(
        &dreammaker::objtree::TypeRef,
        &mut HashSet<String>,
        &mut HashSet<String>,
        &mut HashSet<String>,
    ),
{
    let mut type_ref = objtree.find(&type_search_string(objtree, node_index));
    let mut leaf_declared_names: HashSet<String> = HashSet::new();
    let mut leaf_undeclared_names: HashSet<String> = HashSet::new();
    let mut parent_names: HashSet<String> = HashSet::new();

    while let Some(ty) = type_ref {
        // magic B.S. B)
        visit_type(&ty, &mut leaf_declared_names, &mut leaf_undeclared_names, &mut parent_names);
        type_ref = ty.parent_type_without_root();
    }

    (leaf_declared_names, leaf_undeclared_names, parent_names)
}

#[gen_stub_pyclass]
#[pyclass(module = "avulto")]
pub struct VarHolder {
    pub dme: Py<PyAny>,
    pub node_index: NodeIndex,
}

impl VarHolder {
    fn collect_names(
        &self,
        py: Python<'_>,
        declared: bool,
        modified: bool,
        unmodified: bool,
    ) -> PyResult<Vec<String>> {
        if !declared && !modified && !unmodified {
            return Err(PyValueError::new_err(
                "at least one of `declared`, `modified`, or `unmodified` must be True",
            ));
        }

        let dme = self.dme.cast_bound::<Dme>(py).unwrap();
        let objtree = &dme.borrow().objtree;
        let (leaf_declared_names, leaf_undeclared_names, parent_names) = collect_name_sets(
            objtree,
            self.node_index,
            |ty, leaf_declared_names, leaf_undeclared_names, parent_names| {
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
            },
        );

        let mut out: HashSet<String> = HashSet::new();
        if unmodified {
            out.extend(
                parent_names
                    .difference(&leaf_declared_names)
                    .cloned()
                    .collect::<Vec<_>>(),
            );
        }
        if modified {
            out.extend(leaf_undeclared_names.iter().cloned());
        }
        if declared {
            out.extend(leaf_declared_names.iter().cloned());
        }

        let mut names: Vec<String> = out.into_iter().collect();
        names.sort();
        Ok(names)
    }

    fn var_defs(
        &self,
        py: Python<'_>,
        names: Vec<String>
    ) -> PyResult<Py<PyList>> {
        let dme = self.dme.cast_bound::<Dme>(py).unwrap();
        let mut out = Vec::new();
        for name in names {
            out.push(dme.borrow().get_var_decl(name, self.node_index, true, py)?);
        }
        Ok(PyList::new(py, out)?.into_pyobject(py)?.unbind())
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl VarHolder {
    /// Return the names of all variables visible from this type path.
    #[gen_stub(override_return_type(type_repr="builtins.list[builtins.str]", imports=("builtins")))]
    pub fn names(&self, py: Python<'_>) -> PyResult<Py<PyList>> {
        let names = self.collect_names(py, true, true, true)?;
        Ok(PyList::new(py, names)?.into_pyobject(py)?.unbind())
    }

    /// Return all variable declarations visible from this type path.
    #[gen_stub(override_return_type(type_repr="builtins.list[VarDef]", imports=("builtins")))]
    pub fn all(&self, py: Python<'_>) -> PyResult<Py<PyList>> {
        let names = self.collect_names(py, true, true, true)?;
        self.var_defs(py, names)
    }

    /// Return variables declared directly on this type path.
    #[gen_stub(override_return_type(type_repr="builtins.list[VarDef]", imports=("builtins")))]
    pub fn declared(&self, py: Python<'_>) -> PyResult<Py<PyList>> {
        let names = self.collect_names(py, true, false, false)?;
        self.var_defs(py, names)
    }

    /// Return variables that override inherited values on this subtype.
    #[gen_stub(override_return_type(type_repr="builtins.list[VarDef]", imports=("builtins")))]
    pub fn modified(&self, py: Python<'_>) -> PyResult<Py<PyList>> {
        let names = self.collect_names(py, false, true, false)?;
        self.var_defs(py, names)
    }

    /// Return inherited variables that were not changed on this subtype.
    #[gen_stub(override_return_type(type_repr="builtins.list[VarDef]", imports=("builtins")))]
    pub fn unmodified(&self, py: Python<'_>) -> PyResult<Py<PyList>> {
        let names = self.collect_names(py, false, false, true)?;
        self.var_defs(py, names)
    }

    #[gen_stub(override_return_type(type_repr="collections.abc.Iterator[VarDef]", imports=("collections.abc")))]
    pub fn __iter__(slf: PyRef<'_, Self>, py: Python<'_>) -> PyResult<Py<PythonIter>> {
        let all_list = slf.all(py)?;
        let list = all_list.bind(py).iter().map(|item| item.unbind()).collect();
        let iter = PythonIter {
            list,
            index: 0,
        };
        Py::new(slf.py(), iter)
    }

    #[gen_stub(override_return_type(type_repr="VarDef"))]
    pub fn __getitem__(&self, key: Bound<'_, PyAny>, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let names = self.collect_names(py, true, true, true)?;

        if let Ok(name) = key.extract::<String>() {
            if !names.iter().any(|item| item == &name) {
                return Err(PyKeyError::new_err(name));
            }
            let dme = self.dme.cast_bound::<Dme>(py).unwrap();
            return dme.borrow().get_var_decl(name, self.node_index, true, py);
        }

        Err(PyTypeError::new_err(
            "VarHolder indices must be integers or strings",
        ))
    }
}

#[pyclass]
pub struct PythonIter {
    list: Vec<Py<PyAny>>,
    index: usize,
}

#[pymethods]
impl PythonIter {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<Py<PyAny>> {
        Python::attach(|py| {
            let index = slf.index;
            slf.index += 1;
            slf.list.get(index).map(|user| user.clone_ref(py))
        })
    }
}

#[gen_stub_pyclass]
#[pyclass(module = "avulto")]
pub struct ProcHolder {
    pub dme: Py<PyAny>,
    pub node_index: NodeIndex,
}

impl ProcHolder {
    fn collect_names(
        &self,
        py: Python<'_>,
        declared: bool,
        modified: bool,
        unmodified: bool,
    ) -> PyResult<Vec<String>> {
        if !declared && !modified && !unmodified {
            return Err(PyValueError::new_err(
                "at least one of `declared`, `modified`, or `unmodified` must be True",
            ));
        }

        let dme = self.dme.cast_bound::<Dme>(py).unwrap();
        let objtree = &dme.borrow().objtree;
        let (leaf_declared_names, leaf_undeclared_names, parent_names) = collect_name_sets(
            objtree,
            self.node_index,
            |ty, leaf_declared_names, leaf_undeclared_names, parent_names| {
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
            },
        );

        let mut out: HashSet<String> = HashSet::new();
        if unmodified {
            out.extend(
                parent_names
                    .difference(&leaf_declared_names)
                    .filter(|name| !leaf_undeclared_names.contains(*name))
                    .cloned()
                    .collect::<Vec<_>>(),
            );
        }
        if modified {
            out.extend(leaf_undeclared_names.iter().cloned());
        }
        if declared {
            out.extend(leaf_declared_names.iter().cloned());
        }

        let mut names: Vec<String> = out.into_iter().collect();
        names.sort();
        Ok(names)
    }

    fn proc_defs(
        &self,
        py: Python<'_>,
        names: Vec<String>,
        parents: bool,
    ) -> PyResult<Py<PyList>> {
        let dme = self.dme.cast_bound::<Dme>(py).unwrap();
        let mut out: Vec<Py<PyAny>> = Vec::new();

        for name in names {
            let proc_decls = dme.borrow().get_proc_decls(
                name,
                self.node_index,
                parents,
                self.dme.clone_ref(py),
                py,
            )?;
            let proc_list = proc_decls.bind(py).extract::<Vec<Py<PyAny>>>()?;
            out.extend(proc_list);
        }

        Ok(PyList::new(py, out)?.into_pyobject(py)?.unbind())
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl ProcHolder {
    /// Return the names of all procs visible from this type path.
    #[gen_stub(override_return_type(type_repr="builtins.list[builtins.str]", imports=("builtins")))]
    pub fn names(&self, py: Python<'_>) -> PyResult<Py<PyList>> {
        let names = self.collect_names(py, true, true, true)?;
        Ok(PyList::new(py, names)?.into_pyobject(py)?.unbind())
    }

    /// Return all proc declarations visible from this type path.
    #[gen_stub(override_return_type(type_repr="builtins.list[ProcDef]", imports=("builtins")))]
    pub fn all(&self, py: Python<'_>) -> PyResult<Py<PyList>> {
        let names = self.collect_names(py, true, true, true)?;
        self.proc_defs(py, names, true)
    }

    /// Return procs declared directly on this type path.
    #[gen_stub(override_return_type(type_repr="builtins.list[ProcDef]", imports=("builtins")))]
    pub fn declared(&self, py: Python<'_>) -> PyResult<Py<PyList>> {
        let names = self.collect_names(py, true, false, false)?;
        self.proc_defs(py, names, false)
    }

    /// Return procs that override inherited values on this subtype.
    #[gen_stub(override_return_type(type_repr="builtins.list[ProcDef]", imports=("builtins")))]
    pub fn modified(&self, py: Python<'_>) -> PyResult<Py<PyList>> {
        let names = self.collect_names(py, false, true, false)?;
        self.proc_defs(py, names, false)
    }

    /// Return inherited procs that were not changed on this subtype.
    #[gen_stub(override_return_type(type_repr="builtins.list[ProcDef]", imports=("builtins")))]
    pub fn unmodified(&self, py: Python<'_>) -> PyResult<Py<PyList>> {
        let names = self.collect_names(py, false, false, true)?;
        self.proc_defs(py, names, true)
    }

    #[gen_stub(override_return_type(type_repr="collections.abc.Iterator[ProcDef]", imports=("collections.abc")))]
    pub fn __iter__(slf: PyRef<'_, Self>, py: Python<'_>) -> PyResult<Py<PythonIter>> {
        let all_list = slf.all(py)?;
        let list = all_list.bind(py).iter().map(|item| item.unbind()).collect();
        let iter = PythonIter {
            list,
            index: 0,
        };
        Py::new(slf.py(), iter)
    }

    #[gen_stub(override_return_type(type_repr="builtins.list[ProcDef]", imports=("builtins")))]
    pub fn __getitem__(&self, key: Bound<'_, PyAny>, py: Python<'_>) -> PyResult<Py<PyAny>> {
        if let Ok(name) = key.extract::<String>() {
            let names = self.collect_names(py, true, true, true)?;
            if !names.iter().any(|item| item == &name) {
                return Err(PyKeyError::new_err(name));
            }

            let dme = self.dme.cast_bound::<Dme>(py).unwrap();
            return dme
                .borrow()
                .get_proc_decls(name, self.node_index, true, self.dme.clone_ref(py), py);
        }

        Err(PyTypeError::new_err(
            "ProcHolder indices must be strings",
        ))
    }
}

/// A single type declaration.
#[gen_stub_pyclass]
#[pyclass(module = "avulto")]
pub struct TypeDef {
    pub dme: Py<PyAny>,
    pub node_index: NodeIndex,
    /// The typepath of the TypeDef.
    #[pyo3(get)]
    pub path: TypePath,
    /// The location of the TypeDef's first declaration in source.
    #[pyo3(get)]
    pub source_loc: SourceLoc,
}

#[gen_stub_pymethods]
#[pymethods]
impl TypeDef {
    /// Return a list of variable names for the type declaration.
    #[gen_stub(override_return_type(type_repr="builtins.list[builtins.str]", imports=("builtins")))]
    #[pyo3(signature = (declared=false, modified=false, unmodified=false))]
    #[deprecated(since = "0.5.0", note = "Use `vars` variable instead")]
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

        let search_string = if self.path.rel.eq("/") {
            ""
        } else {
            self.path.rel.as_str()
        };
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

    /// Return the var declaration for variable *name*. If *parents* is True,
    /// check up type path if this type does not have this variable set.
    #[gen_stub(override_return_type(type_repr = "VarDef"))]
    #[pyo3(signature = (name, parents=true))]
    #[deprecated(since = "0.5.0", note = "Use `vars` variable instead")]
    pub fn var_decl(&self, name: String, parents: bool, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let bound = self.dme.cast_bound::<Dme>(py).unwrap();
        let dme = bound.borrow();
        dme.get_var_decl(name, self.node_index, parents, py)
    }

    /// Return a list of proc names for the type declaration.
    #[gen_stub(override_return_type(type_repr="builtins.list[builtins.str]", imports=("builtins")))]
    #[pyo3(signature = (declared=false, modified=false, unmodified=false))]
    #[deprecated(since = "0.5.0", note = "Use `procs` variable instead")]
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

        let search_string = if self.path.rel.eq("/") {
            ""
        } else {
            self.path.rel.as_str()
        };
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

    /// Return proc declarations for the type. If *name* is set, only return
    /// proc declarations with this name.
    #[gen_stub(override_return_type(type_repr="builtins.list[ProcDef]", imports=("builtins")))]
    #[pyo3(signature = (name=None))]
    #[deprecated(since = "0.5.0", note = "Use `procs` variable instead")]
    pub fn proc_decls(&self, name: Option<String>, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let dme = self.dme.cast_bound::<Dme>(py).unwrap();
        let objtree = &dme.borrow().objtree;
        let mut out: Vec<ProcDef> = Vec::new();

        let type_def = &objtree[self.node_index];
        for (proc_name, proc) in type_def.procs.iter() {
            if name.as_ref().is_some_and(|p| !proc_name.eq(p)) {
                continue;
            }
            for (proc_index, proc_value) in proc.value.iter().enumerate() {
                if !proc_value.location.is_builtins() {
                    let mut args_out: Vec<ProcArg> = Vec::new();
                    for arg in proc_value.parameters.iter() {
                        let arg_typepath: Option<TypePath> = if arg.var_type.type_path.is_empty() {
                            None
                        } else {
                            Some(TypePath::from_tree_path(&arg.var_type.type_path))
                        };
                        args_out.push(ProcArg {
                            arg_name: arg.name.clone(),
                            arg_type: arg_typepath,
                        });
                    }

                    out.push(ProcDef {
                        dme: self.dme.clone_ref(py),
                        name: proc_name.clone(),
                        type_path: self.path.clone().into_pyobject(py)?.into_any().unbind(),
                        args: args_out,
                        proc_index,
                        type_index: self.node_index,
                        source_loc: dme.borrow().file_data.fill_source_loc(&proc_value.location),
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

    /// A class to access variables on this typepath.
    #[getter]
    pub fn vars(&self, py: Python<'_>) -> VarHolder {
        VarHolder {
            dme: self.dme.clone_ref(py),
            node_index: self.node_index,
        }
    }
    /// A class to access procs on this typepath.
    #[getter]
    pub fn procs(&self, py: Python<'_>) -> ProcHolder {
        ProcHolder {
            dme: self.dme.clone_ref(py),
            node_index: self.node_index,
        }
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("<Type {}>", self.path.rel))
    }
}
