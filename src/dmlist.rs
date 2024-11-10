use itertools::Itertools;
use pyo3::{
    exceptions::PyRuntimeError,
    pyclass, pymethods,
    types::{PyAnyMethods, PyInt, PyString, PyStringMethods},
    Bound, Py, PyAny, PyRef, PyRefMut, PyResult, Python,
};

use crate::path;

// use crate::{dme::nodes::Prefab, path};

#[pyclass(module = "avulto", name = "dmlist")]
pub struct DmList {
    pub keys: Vec<Py<PyAny>>,
    pub vals: Vec<Py<PyAny>>,
}

impl DmList {
    pub fn push(mut self, key: Py<PyAny>, value: Py<PyAny>) {
        self.keys.push(key);
        self.vals.push(value);
    }
}

#[pyclass]
struct DmListIter {
    inner: std::iter::Zip<std::vec::IntoIter<Py<PyAny>>, std::vec::IntoIter<Py<PyAny>>>,
}

#[pymethods]
impl DmListIter {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<(Py<PyAny>, Py<PyAny>)> {
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
        let index = slf.index;
        slf.index += 1;
        slf.list.get(index).map(|user| user.clone_ref(slf.py()))
    }
}

#[pymethods]
impl DmList {
    fn keys(slf: PyRef<'_, Self>, py: Python<'_>) -> PyResult<Py<DmListKeyIter>> {
        let iter = DmListKeyIter {
            list: slf.keys.iter().map(|x| x.clone_ref(py)).collect_vec(),
            index: 0,
        };
        Py::new(slf.py(), iter)
    }

    fn __getitem__(&self, item: &Bound<PyAny>, py: Python<'_>) -> PyResult<Py<PyAny>> {
        if let Ok(i) = item.downcast::<PyInt>() {
            let idx = i.extract::<usize>().unwrap();
            if self.keys.len() <= idx {
                return Err(pyo3::exceptions::PyIndexError::new_err(
                    "list index out of range",
                ));
            }
            let key = self.keys.get(idx).unwrap();
            return Ok(key.clone_ref(py));
        } else if let Ok(pystr) = item.downcast::<PyString>() {
            for (idx, key) in self.keys.iter().enumerate() {
                if let Ok(key_str) = key.downcast_bound::<PyString>(py) {
                    if pystr.to_str().unwrap().eq(key_str.to_str().unwrap()) {
                        return Ok(self.vals.get(idx).unwrap().clone_ref(py));
                    }
                }
            }
        } else if let Ok(pypth) = item.extract::<path::Path>() {
            for (idx, key) in self.keys.iter().enumerate() {
                if let Ok(keypth) = key.extract::<path::Path>(py) {
                    if pypth.eq(&keypth) {
                        return Ok(self.vals.get(idx).unwrap().clone_ref(py));
                    }
                }
            }
        // } else if item.is_instance_of::<Prefab>() {
        //     let item_prefab: &Bound<Prefab> = item.downcast()?;
        //     for (idx, key) in self.keys.iter().enumerate() {
        //         if key.bind(py).is_instance_of::<Prefab>() {
        //             // TODO(wso): Hate hate hate hate hate hate hate
        //             let a = key.downcast_bound::<Prefab>(py)?;
        //             let b = a.try_borrow().unwrap();
        //             if item_prefab.borrow().__eq__(&b, py) {
        //                 return Ok(self.vals.get(idx).unwrap().clone_ref(py));
        //             }
        //         }
        //     }
        } else if item.is_instance_of::<DmList>() {
            for (idx, key) in self.keys.iter().enumerate() {
                if key.bind(py).is_instance_of::<DmList>() && item.eq(key).unwrap() {
                    return Ok(self.vals.get(idx).unwrap().clone_ref(py));
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
        out.push_str("dmlist[");
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
