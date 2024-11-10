use std::hash::{DefaultHasher, Hash, Hasher};

use pyo3::{pyclass, pymethods, types::{PyAnyMethods, PyDict, PyDictMethods, PyList, PyString}, Bound, IntoPy, Py, PyAny, PyResult, Python, ToPyObject};

use crate::path::Path;

use super::expression::Expression;

#[pyclass(module = "avulto.ast")]
pub struct Prefab {
    #[pyo3(get)]
    pub path: Py<PyAny>,
    #[pyo3(get)]
    pub vars: Py<PyAny>,
}

impl Prefab {
    pub fn make(py: Python<'_>, prefab: &Box<dreammaker::ast::Prefab>) -> Self {
        let mut path: String = "".to_owned();
        for (op, val) in prefab.path.iter() {
            path.push_str(format!("{}{}", op, val).as_str());
        }
        let pypath = Path::make_trusted(path.as_str());
        let mut out: Vec<Bound<PyDict>> = Vec::new();

        for (k, v) in prefab.vars.iter() {
            let var = PyDict::new_bound(py);
            var.set_item(
                k.as_str(),
                Expression::from_expression(py, v).into_py(py),
            );
            out.push(var);
        }   
        Prefab {
            path: pypath.into_py(py),
            vars: PyList::new_bound(py, out).to_object(py).clone_ref(py),

        }     
    }
    pub fn vars_to_string(&self, py: Python<'_>) -> String {
        if let Ok(vardict) = self.vars.downcast_bound::<PyDict>(py) {
            if vardict.is_empty() {
                return "".to_string();
            }
            let mut out = String::new();

            for k in vardict.items() {
                if let Ok(kl) = k.downcast::<PyList>() {
                    out.push_str(
                        format!("{} = {}", kl.get_item(0).unwrap(), kl.get_item(1).unwrap())
                            .as_str(),
                    );
                }
            }

            return out.to_string();
        }

        "".to_string()
    }

    pub fn walk(&self, walker: &Bound<PyAny>) {

    }
}

#[pymethods]
impl Prefab {
    fn __str__(&self) -> PyResult<String> {
        self.__repr__()
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("{}", self.path))
    }

    pub fn __eq__(&self, other: &Self, py: Python<'_>) -> bool {
        if let Ok(pthstr) = self.path.downcast_bound::<PyString>(py) {
            if let Ok(otherpthstr) = other.path.downcast_bound::<PyString>(py) {
                if !pthstr.to_string().eq(&otherpthstr.to_string()) {
                    return false;
                }
            }
        } else if let Ok(pthpth) = self.path.downcast_bound::<Path>(py) {
            if let Ok(otherpthpth) = other.path.downcast_bound::<Path>(py) {
                if !pthpth.eq(otherpthpth).unwrap() {
                    return false;
                }
            }
        }
        if let Ok(vardict) = self.vars.downcast_bound::<PyDict>(py) {
            if let Ok(othervardict) = other.vars.downcast_bound::<PyDict>(py) {
                if !vardict.eq(othervardict).unwrap() {
                    return false;
                }
            }
        }

        true
    }

    pub fn __hash__(&self, py: Python<'_>) -> PyResult<u64> {
        let mut s = DefaultHasher::new();
        if let Ok(pthstr) = self.path.downcast_bound::<PyString>(py) {
            pthstr.hash()?.hash(&mut s);
        }
        if let Ok(vardict) = self.vars.downcast_bound::<PyDict>(py) {
            vardict.hash()?.hash(&mut s);
        }

        Ok(s.finish())
    }
}
