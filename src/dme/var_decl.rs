use pyo3::{Py, PyAny, PyResult, pyclass, pymethods};

use crate::{dme::SourceLoc, path::Path};

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
    pub source_loc: SourceLoc,
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
