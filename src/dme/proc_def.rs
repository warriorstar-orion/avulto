use dreammaker::objtree::NodeIndex;
use pyo3::{Bound, Py, PyAny, PyResult, Python, pyclass, pymethods};
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};

use crate::{
    dme::{Dme, SourceLoc},
    path::TypePath,
};

/// A representation of a proc declaration argument.
#[gen_stub_pyclass]
#[pyclass(module = "avulto")]
#[derive(Clone)]
pub struct ProcArg {
    /// The argument name.
    #[pyo3(get)]
    pub arg_name: String,
    /// The argument type, if available.
    #[pyo3(get)]
    pub arg_type: Option<TypePath>,
}

#[gen_stub_pymethods]
#[pymethods]
impl ProcArg {
    fn __str__(&self) -> PyResult<String> {
        self.__repr__()
    }

    fn __repr__(&self) -> PyResult<String> {
        match &self.arg_type {
            Some(p) => Ok(format!("{}/{}", p, self.arg_name)),
            None => Ok(self.arg_name.to_string()),
        }
    }
}

/// A single proc declaration.
#[gen_stub_pyclass]
#[pyclass(module = "avulto")]
pub struct ProcDef {
    pub dme: Py<PyAny>,
    /// The type path the proc is declared on.
    #[pyo3(get)]
    pub type_path: Py<PyAny>,
    /// The name of the proc.
    #[pyo3(get)]
    pub name: String,
    #[pyo3(get)]
    pub args: Vec<ProcArg>,

    pub(crate) type_index: NodeIndex,
    pub(crate) proc_index: usize,
    /// The source location of the proc declaration.
    #[pyo3(get)]
    pub(crate) source_loc: SourceLoc,
}

#[gen_stub_pymethods]
#[pymethods]
impl ProcDef {
    fn __str__(&self) -> PyResult<String> {
        self.__repr__()
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("<Proc {}/proc/{}>", self.type_path, self.name))
    }

    /// Walks the proc AST with *walker*, calling any `visit_*` method names on
    /// *walker* if they exist for AST node types.
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
