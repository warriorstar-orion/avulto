use itertools::Itertools;
use pyo3::{
    exceptions::PyRuntimeError,
    pyclass, pymethods,
    types::{PyAnyMethods, PyInt, PyString, PyStringMethods},
    Bound, Py, PyAny, PyRef, PyRefMut, PyResult, Python,
};
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};

use crate::{
    dme::{expression::Expression, nodes::PyExpr, prefab::Prefab},
    path,
};

/// A primitive, read-only representation of a DM list. This is used when
/// returning constant values of lists from the AST walker, and prefab values
/// from the DMM reader. They are not constructable.
/// 
/// Dmlists only support iterating over keys, and indexing with keys.
#[gen_stub_pyclass]
#[pyclass(module = "avulto", name = "Dmlist")]
pub struct DmList {
    // Not PyExprs because we also want to use these for DMM constants
    // and constant value decls, without all the redirection of AST
    // nodes
    pub keys: Vec<Py<PyAny>>,
    pub vals: Vec<Py<PyAny>>,
}

impl DmList {
    pub fn push(mut self, key: Py<PyAny>, value: Py<PyAny>) {
        self.keys.push(key);
        self.vals.push(value);
    }
}

/// A `new` call appearing in a constant-value position.
#[gen_stub_pyclass]
#[pyclass(module = "avulto", name = "NewCall")]
pub struct NewCall {
    /// The type being instantiated, or `None` for an implicit `new()`.
    #[pyo3(get)]
    pub path: Option<path::Path>,
    /// The constructor arguments.
    #[pyo3(get)]
    pub args: Option<Py<DmList>>,
}

#[gen_stub_pymethods]
#[pymethods]
impl NewCall {
    fn __str__(&self) -> PyResult<String> {
        self.__repr__()
    }

    fn __repr__(&self) -> PyResult<String> {
        match &self.path {
            Some(p) => Ok(format!("<NewCall {}>", p.rel)),
            None => Ok("<NewCall>".to_string()),
        }
    }
}

#[pyclass]
struct DmListIter {
    inner: std::iter::Zip<std::vec::IntoIter<PyExpr>, std::vec::IntoIter<PyExpr>>,
}

#[pymethods]
impl DmListIter {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<(PyExpr, PyExpr)> {
        slf.inner.next()
    }
}

#[pyclass]
struct DmListKeyIter {
    list: Vec<Py<PyAny>>,
    index: usize,
}

#[pymethods]
impl DmListKeyIter {
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

#[gen_stub_pymethods]
#[pymethods]
impl DmList {
    /// Iterates over the keys in the list.
    #[gen_stub(override_return_type(type_repr="collections.abc.Iterator[typing.Any]", imports=("collections.abc", "typing")))]
    fn keys(slf: PyRef<'_, Self>, py: Python<'_>) -> PyResult<Py<DmListKeyIter>> {
        let iter = DmListKeyIter {
            list: slf.keys.iter().map(|x| x.clone_ref(py)).collect_vec(),
            index: 0,
        };
        Py::new(slf.py(), iter)
    }

    fn __getitem__(&self, item: &Bound<PyAny>, py: Python<'_>) -> PyResult<Py<PyAny>> {
        if let Ok(i) = item.cast::<PyInt>() {
            let idx = i.extract::<usize>().unwrap();
            if self.keys.len() <= idx {
                return Err(pyo3::exceptions::PyIndexError::new_err(
                    "list index out of range",
                ));
            }
            let key = self.keys.get(idx).unwrap();
            return Ok(key.clone_ref(py));
        }

        if let Ok(pystr) = item.cast::<PyString>() {
            for (idx, key) in self.keys.iter().enumerate() {
                if let Ok(key_expr) = key.cast_bound::<Expression>(py) {
                    if let Expression::Constant {
                        constant: crate::dme::expression::Constant::String(s),
                        ..
                    } = key_expr.get()
                    {
                        if pystr.to_str().unwrap().eq(s) {
                            return Ok(self.vals.get(idx).unwrap().clone_ref(py));
                        }
                    };
                } else if key.bind(py).is_instance_of::<PyString>() {
                    let key_str = key.cast_bound::<PyString>(py)?;
                    if pystr.to_str().unwrap().eq(key_str) {
                        return Ok(self.vals.get(idx).unwrap().clone_ref(py));
                    }
                }
            }
        } else if let Ok(pypth) = item.extract::<path::Path>() {
            for (idx, key) in self.keys.iter().enumerate() {
                if let Ok(key_expr) = key.cast_bound::<Expression>(py) {
                    if let Expression::Prefab { prefab, .. } = key_expr.get() {
                        if pypth.eq(&prefab.borrow(py).path) {
                            return Ok(self.vals.get(idx).unwrap().clone_ref(py));
                        }
                    }
                } else if let Ok(keypth) = key.extract::<path::Path>(py) {
                    if pypth.eq(&keypth) {
                        return Ok(self.vals.get(idx).unwrap().clone_ref(py));
                    }
                } else if let Ok(key_prefab) = key.cast_bound::<Prefab>(py) {
                    if pypth.eq(&key_prefab.borrow().path) {
                        return Ok(self.vals.get(idx).unwrap().clone_ref(py));
                    }
                }
            }
        } else if let Ok(item_prefab) = item.cast::<Prefab>() {
            for (idx, key) in self.keys.iter().enumerate() {
                if let Ok(key_prefab) = key.cast_bound::<Prefab>(py) {
                    if item_prefab
                        .borrow()
                        .__eq__(&key_prefab.try_borrow().unwrap(), py)
                    {
                        return Ok(self.vals.get(idx).unwrap().clone_ref(py));
                    }
                } else if let Ok(key_expr) = key.cast_bound::<Expression>(py) {
                    if let Expression::Prefab { prefab, .. } = key_expr.get() {
                        if item_prefab
                            .borrow()
                            .__eq__(&prefab.try_borrow(py).unwrap(), py)
                        {
                            return Ok(self.vals.get(idx).unwrap().clone_ref(py));
                        }
                    }
                }
            }
        } else if let Ok(item_dmlist) = item.cast::<DmList>() {
            for (idx, key) in self.keys.iter().enumerate() {
                if let Ok(key_list) = key.cast_bound::<DmList>(py) {
                    if key_list.eq(item_dmlist).unwrap() {
                        return Ok(self.vals.get(idx).unwrap().clone_ref(py));
                    }
                } else if let Ok(key_expr) = key.cast_bound::<Expression>(py) {
                    if let Expression::List { list, .. } = key_expr.get() {
                        if item_dmlist.eq(list.clone_ref(py)).unwrap() {
                            return Ok(self.vals.get(idx).unwrap().clone_ref(py));
                        }
                    }
                }
            }
        } else {
            for (idx, key) in self.keys.iter().enumerate() {
                if item.eq(key).unwrap() {
                    return Ok(self.vals.get(idx).unwrap().clone_ref(py));
                }
            }
        }

        Err(PyRuntimeError::new_err("index not found"))
    }

    fn __str__(&self, py: Python<'_>) -> PyResult<String> {
        let mut out = String::new();
        out.push_str("Dmlist[");
        for i in 0..self.keys.len() {
            out.push('\n');
            let k = self.keys.get(i).unwrap();
            out.push('\t');
            if k.extract::<Py<DmList>>(py).is_ok() {
                out.push_str("/list");
            } else {
                out.push_str(
                    k.call_method0(py, "__str__")
                        .unwrap()
                        .extract::<String>(py)
                        .unwrap()
                        .as_str(),
                );
            }

            out.push_str(" = ");
            let v = self.vals.get(i).unwrap();
            if v.extract::<Py<DmList>>(py).is_ok() {
                out.push_str("/list");
            } else {
                let str_call = v.call_method0(py, "__str__").unwrap();
                out.push_str(str_call.to_string().as_str());
            }

            out.push(',');
        }

        out.push_str("\n]");
        Ok(out)
    }
}
