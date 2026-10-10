use pyo3::{Py, PyAny, PyResult, pyclass, pymethods};
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};

use crate::{dme::SourceLoc, path::Path};

/// A single variable declaration.
#[gen_stub_pyclass]
#[pyclass(module = "avulto")]
pub struct VarDef {
    /// The name of the variable.
    #[pyo3(get)]
    pub name: String,
    /// The type path the proc is declared on.
    #[pyo3(get)]
    pub type_path: Py<PyAny>,
    /// The declared type of the variable, if specified.
    #[pyo3(get)]
    pub declared_type: Option<Path>,
    /// The variable's value, if it can be evaluated as a constant expression.
    #[pyo3(get)]
    pub const_val: Option<Py<PyAny>>,
    /// The location of the variable declaration in the source tree.
    #[pyo3(get)]
    pub source_loc: SourceLoc,
}

#[gen_stub_pymethods]
#[pymethods]
impl VarDef {
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
