use dreammaker::objtree::NodeIndex;
use pyo3::{Bound, Py, PyAny, PyResult, Python, pyclass, pymethods};

use crate::dme::{Dme, SourceLoc};

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

    pub(crate) type_index: NodeIndex,
    pub(crate) proc_index: usize,
    #[pyo3(get)]
    pub(crate) source_loc: SourceLoc,
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
