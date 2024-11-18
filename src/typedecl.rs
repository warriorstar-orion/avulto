extern crate dreammaker;

use dreammaker::objtree::NodeIndex;
use itertools::Itertools;
use pyo3::{exceptions::PyRuntimeError, prelude::*, types::PyList};

use crate::{
    dme::{Dme, FilledSourceLocation},
    path::Path,
};

#[pyclass(module = "avulto")]
pub struct VarDecl {
    #[pyo3(get)]
    pub name: String,
    #[pyo3(get)]
    pub declared_type: Option<Path>,
    #[pyo3(get)]
    pub const_val: Option<PyObject>,
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
                "<Var {}/{}>",
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
    source_info: PyObject,
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
        let dme = self.dme.downcast_bound::<Dme>(py).unwrap();
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
    pub fn var_names(&self, py: Python<'_>) -> PyResult<Py<PyList>> {
        let mut out: Vec<String> = Vec::new();
        let dme = self.dme.downcast_bound::<Dme>(py).unwrap();
        let objtree = &dme.borrow().objtree;
        let type_def = &objtree[self.node_index];

        for (name, _) in type_def.vars.iter() {
            out.push(name.clone());
        }
        let mut x = out.into_iter().unique().collect::<Vec<String>>();
        x.sort();
        Ok(PyList::new(py, x)
            .expect("passing var names list")
            .unbind()
            .clone_ref(py))
    }

    #[pyo3(signature = (name, parents=true))]
    pub fn var_decl(&self, name: String, parents: bool, py: Python<'_>) -> PyResult<PyObject> {
        let bound = self.dme.downcast_bound::<Dme>(py).unwrap();
        let dme = bound.borrow();
        dme.get_var_decl(name, self.node_index, parents, py)
    }

    pub fn proc_names(&self, py: Python<'_>) -> PyResult<Py<PyList>> {
        let mut out: Vec<String> = Vec::new();
        let bound = self.dme.downcast_bound::<Dme>(py).unwrap();
        for ty in bound.borrow().objtree.iter_types() {
            if ty.path == self.path.rel {
                for (name, _) in ty.procs.iter() {
                    out.push(name.clone());
                }
                let mut x = out.into_iter().unique().collect::<Vec<String>>();
                x.sort();
                return Ok(PyList::new(py, x)
                    .expect("passing proc names list")
                    .unbind()
                    .clone_ref(py));
            }
        }

        Err(PyRuntimeError::new_err(format!(
            "cannot find type {}",
            self.path
        )))
    }

    #[pyo3(signature = (name=None))]
    pub fn proc_decls(&self, name: Option<String>, py: Python<'_>) -> PyResult<PyObject> {
        let dme = self.dme.downcast_bound::<Dme>(py).unwrap();
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
                        source_info: FilledSourceLocation {
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
        Ok(format!("<Type {}>", self.path))
    }
}
