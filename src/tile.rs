extern crate dmm_tools;

use std::borrow::Borrow;

use dmm_tools::dmm::{Map, Prefab};
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::pyclass::CompareOp;
use pyo3::types::{PyDict, PyList, PyString};
use pyo3::{
    pyclass, pymethods, IntoPy, Py, PyAny, PyCell, PyErr, PyObject, PyRefMut, PyResult, Python,
};

use crate::dmm::{Address, Dmm};
use crate::helpers::{constant_to_python_value, python_value_to_constant};
use crate::path;

#[pyclass(module = "avulto")]
pub struct Tile {
    pub(crate) dmm: Py<PyAny>,
    pub(crate) addr: Address,
}

#[pymethods]
impl Tile {
    pub fn add_path(&mut self, index: i32, entry: &PyAny, py: Python<'_>) -> PyResult<()> {
        let cell: &PyCell<Dmm> = self.dmm.downcast(py).unwrap();
        let key = match self.addr {
            Address::Key(k) => k,
            Address::Coords(c) => cell.borrow_mut().map[c],
        };

        if let Ok(val) = entry.extract::<path::Path>() {
            let prefab = Prefab {
                path: val.0.clone(),
                vars: Default::default(),
            };
            cell.borrow_mut()
                .map
                .dictionary
                .get_mut(&key)
                .unwrap()
                .insert(index as usize, prefab);
            return Ok(());
        } else if let Ok(pystr) = entry.downcast::<PyString>() {
            let prefab = Prefab {
                path: pystr.to_string(),
                vars: Default::default(),
            };
            cell.borrow_mut()
                .map
                .dictionary
                .get_mut(&key)
                .unwrap()
                .insert(index as usize, prefab);
            return Ok(());
        }

        Err(PyRuntimeError::new_err("invalid insertion type"))
    }

    pub fn area_path(&self, py: Python<'_>) -> PyResult<path::Path> {
        let map = &self.dmm.downcast::<PyCell<Dmm>>(py).unwrap().borrow().map;
        let key = match self.addr {
            Address::Key(k) => k,
            Address::Coords(c) => map[c],
        };
        let prefabs = &map.dictionary[&key];
        for p in prefabs.iter() {
            if p.path.starts_with("/area") {
                return path::Path::new(p.path.as_str());
            }
        }

        Err(PyRuntimeError::new_err(format!(
            "no area on tile {}",
            match self.addr {
                Address::Key(k) => map.format_key(k).to_string(),
                Address::Coords(c) => c.to_string(),
            }
        )))
    }

    pub fn convert(&self, py: Python<'_>) -> PyResult<Py<PyList>> {
        let mut out: Vec<&PyDict> = Vec::new();

        let map = &self.dmm.downcast::<PyCell<Dmm>>(py).unwrap().borrow().map;
        let key = match self.addr {
            Address::Key(k) => k,
            Address::Coords(c) => map[c],
        };
        let prefabs = &map.dictionary[&key];

        for prefab in prefabs {
            let d = PyDict::new(py);
            d.set_item("name", prefab.path.clone())?;

            if !prefab.vars.is_empty() {
                let mut vars: Vec<&PyDict> = Vec::new();
                for (name, constant) in prefab.vars.iter() {
                    let var = PyDict::new(py);
                    var.set_item("name", name)?;
                    var.set_item("value", constant_to_python_value(constant))?;
                    vars.push(var);
                }
                d.set_item("vars", vars)?;
            }

            out.push(d);
        }

        Ok(PyList::new(py, out).into_py(py))
    }

    pub fn del_prefab(&self, index: i32, py: Python<'_>) {
        let cell: &PyCell<Dmm> = self.dmm.downcast(py).unwrap();
        let key = match self.addr {
            Address::Key(k) => k,
            Address::Coords(c) => cell.borrow_mut().map[c],
        };

        cell.borrow_mut()
            .map
            .dictionary
            .get_mut(&key)
            .unwrap()
            .remove(index as usize);
    }

    pub fn del_prefab_var(&self, index: i32, name: String, py: Python<'_>) {
        let cell: &PyCell<Dmm> = self.dmm.downcast(py).unwrap();
        let key = match self.addr {
            Address::Key(k) => k,
            Address::Coords(c) => cell.borrow_mut().map[c],
        };

        cell.borrow_mut().map.dictionary.get_mut(&key).unwrap()[index as usize]
            .vars
            .remove(&name);
    }

    #[pyo3(signature = (prefix, exact=false))]
    pub fn find(&self, prefix: &PyAny, exact: bool, py: Python<'_>) -> PyResult<Vec<i32>> {
        let mut vec = Vec::new();
        let cell: &PyCell<Dmm> = self.dmm.downcast(py).unwrap();
        let mut dmm: PyRefMut<'_, Dmm> = cell.borrow_mut();
        let map: &mut Map = &mut dmm.map;

        let key = match self.addr {
            Address::Key(k) => k,
            Address::Coords(c) => map[c],
        };

        let prefix_str = if let Ok(v) = prefix.extract::<path::Path>() {
            v.0
        } else if let Ok(pystr) = prefix.downcast::<PyString>() {
            pystr.to_string()
        } else {
            return Err(PyErr::new::<PyValueError, &str>("not a valid path"));
        };

        if let Some(p) = map.dictionary.get(&key) {
            for (idx, prefab) in p.iter().enumerate() {
                let matches = (!exact && prefab.path.starts_with(&prefix_str))
                    || (exact && prefab.path.eq(&prefix_str));
                if matches {
                    vec.push(idx as i32);
                }
            }
        }

        Ok(vec)
    }

    pub fn prefab_path(&self, index: i32, py: Python<'_>) -> PyResult<path::Path> {
        let map = &self.dmm.downcast::<PyCell<Dmm>>(py).unwrap().borrow().map;
        let key = match self.addr {
            Address::Key(k) => k,
            Address::Coords(c) => map[c],
        };
        let prefabs = &map.dictionary[&key];

        let binding = prefabs[index as usize].path.clone();
        let s = binding.as_str();
        path::Path::new(s)
    }

    pub fn prefab_var(&self, index: i32, name: String) -> PyObject {
        Python::with_gil(|py| {
            let map = &self.dmm.downcast::<PyCell<Dmm>>(py).unwrap().borrow().map;
            let key = match self.addr {
                Address::Key(k) => k,
                Address::Coords(c) => map[c],
            };
            let prefabs = &map.dictionary[&key];

            constant_to_python_value(prefabs[index as usize].vars.get(&name).unwrap())
        })
    }

    #[pyo3(signature = (index, name, default=None))]
    pub fn get_prefab_var(
        &self,
        index: i32,
        name: String,
        default: Option<&PyAny>,
        py: Python<'_>,
    ) -> PyObject {
        Python::with_gil(|py| {
            let map = &self.dmm.downcast::<PyCell<Dmm>>(py).unwrap().borrow().map;
            let key = match self.addr {
                Address::Key(k) => k,
                Address::Coords(c) => map[c],
            };
            let prefabs = &map.dictionary[&key];
            let vars = &prefabs[index as usize].vars;
            if vars.contains_key(&name) {
                return constant_to_python_value(vars.get(&name).unwrap());
            }

            if let Some(t) = default {
                return t.into_py(py);
            }

            pyo3::types::PyNone::get(py).into()
        })
    }

    pub fn prefab_vars(&self, index: i32) -> Vec<String> {
        Python::with_gil(|py| {
            let mut vec = Vec::new();
            let map = &self.dmm.downcast::<PyCell<Dmm>>(py).unwrap().borrow().map;
            let key = match self.addr {
                Address::Key(k) => k,
                Address::Coords(c) => map[c],
            };
            let prefabs = &map.dictionary[&key];

            prefabs[index as usize].vars.iter().for_each(|(name, _)| {
                vec.push(name.clone());
            });

            vec
        })
    }

    pub fn set_prefab_var(&self, atom_index: i32, name: String, val: &PyAny, py: Python<'_>) {
        let cell: &PyCell<Dmm> = self.dmm.downcast(py).unwrap();
        let key = match self.addr {
            Address::Key(k) => k,
            Address::Coords(c) => cell.borrow_mut().map[c],
        };

        cell.borrow_mut()
            .map
            .dictionary
            .get_mut(&key)
            .unwrap()
            .get_mut(atom_index as usize)
            .unwrap()
            .vars
            .insert(name, python_value_to_constant(val).unwrap());
    }

    pub fn set_path(&self, index: i32, path: &PyAny, py: Python<'_>) -> PyResult<()> {
        let cell: &PyCell<Dmm> = self.dmm.downcast(py).unwrap();
        let key = match self.addr {
            Address::Key(k) => k,
            Address::Coords(c) => cell.borrow_mut().map[c],
        };

        if let Ok(val) = path.extract::<path::Path>() {
            cell.borrow_mut().map.dictionary.get_mut(&key).unwrap()[index as usize].path = val.0;
            return Ok(());
        } else if let Ok(pystr) = path.downcast::<PyString>() {
            cell.borrow_mut().map.dictionary.get_mut(&key).unwrap()[index as usize].path =
                pystr.to_string();
            return Ok(());
        }

        Err(PyErr::new::<PyValueError, &str>("not a valid path"))
    }

    pub fn turf_path(&self, py: Python<'_>) -> PyResult<path::Path> {
        let map = &self.dmm.downcast::<PyCell<Dmm>>(py).unwrap().borrow().map;
        let key = match self.addr {
            Address::Key(k) => k,
            Address::Coords(c) => map[c],
        };
        let prefabs = &map.dictionary[&key];
        for p in prefabs.iter() {
            if p.path.starts_with("/turf") {
                return path::Path::new(p.path.as_str());
            }
        }

        Err(PyRuntimeError::new_err(format!(
            "no turf on tile {}",
            match self.addr {
                Address::Key(k) => map.format_key(k).to_string(),
                Address::Coords(c) => c.to_string(),
            }
        )))
    }

    fn __repr__(&self, py: Python<'_>) -> PyResult<String> {
        let map = &self.dmm.downcast::<PyCell<Dmm>>(py).unwrap().borrow().map;
        Ok(format!(
            "<Tile {}>",
            match self.addr {
                Address::Key(k) => map.format_key(k).to_string(),
                Address::Coords(c) => c.to_string(),
            }
        ))
    }

    pub fn __richcmp__(&self, other: &PyAny, op: CompareOp, py: Python<'_>) -> PyObject {
        let map = &self.dmm.downcast::<PyCell<Dmm>>(py).unwrap().borrow().map;
        let key = match self.addr {
            Address::Key(k) => k,
            Address::Coords(c) => map[c],
        };
        let prefabs = &map.dictionary[&key];

        match op {
            CompareOp::Eq => {
                if let Ok(other) = other.extract::<Py<Self>>() {
                    let odmm = &other.as_ref(py).borrow();
                    let omap = &odmm
                        .borrow()
                        .dmm
                        .downcast::<PyCell<Dmm>>(py)
                        .unwrap()
                        .borrow()
                        .map;
                    let okey = match odmm.addr {
                        Address::Key(k) => k,
                        Address::Coords(c) => omap[c],
                    };
                    let oprefabs = &omap.dictionary[&okey];

                    for (f, s) in prefabs.iter().zip(oprefabs.iter()) {
                        if !f.eq(s) {
                            return false.into_py(py);
                        }
                    }

                    true.into_py(py)
                } else {
                    println!("failed");
                    false.into_py(py)
                }
            }

            CompareOp::Ne => {
                if let Ok(other) = other.extract::<Py<Self>>() {
                    let odmm = &other.as_ref(py).borrow();
                    let omap = &odmm
                        .borrow()
                        .dmm
                        .downcast::<PyCell<Dmm>>(py)
                        .unwrap()
                        .borrow()
                        .map;
                    let okey = match odmm.addr {
                        Address::Key(k) => k,
                        Address::Coords(c) => omap[c],
                    };
                    let oprefabs = &omap.dictionary[&okey];

                    for (f, s) in prefabs.iter().zip(oprefabs.iter()) {
                        if !f.eq(s) {
                            return true.into_py(py);
                        }
                    }

                    false.into_py(py)
                } else {
                    println!("failed");
                    false.into_py(py)
                }
            }

            _ => py.NotImplemented(),
        }
    }
}
