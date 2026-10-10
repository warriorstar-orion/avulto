extern crate dreammaker;

use std::collections::HashSet;

use dreammaker::objtree::NodeIndex;
use pyo3::{exceptions::PyValueError, prelude::*, types::PyList};
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};

use crate::{
    dme::{
        Dme, SourceLoc,
        proc_def::{ProcArg, ProcDef},
    },
    path::TypePath,
};

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
    pub fn var_decl(&self, name: String, parents: bool, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let bound = self.dme.cast_bound::<Dme>(py).unwrap();
        let dme = bound.borrow();
        dme.get_var_decl(name, self.node_index, parents, py)
    }

    /// Return a list of proc names for the type declaration.
    #[gen_stub(override_return_type(type_repr="builtins.list[builtins.str]", imports=("builtins")))]
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

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("<Type {}>", self.path.rel))
    }
}
