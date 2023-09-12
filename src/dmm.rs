extern crate dmm_tools;

use std::borrow::{Borrow, BorrowMut};
use std::collections::btree_map;
use std::path::Path;

use dmm_tools::dmm::Prefab;
use dreammaker::constants::Constant;
use itertools::iproduct;
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::pyclass::{CompareOp, IterNextOutput};
use pyo3::types::{PyBool, PyDict, PyFloat, PyInt, PyList, PyString};
use pyo3::{
    pyclass, pymethods, IntoPy, Py, PyAny, PyCell, PyErr, PyObject, PyRef, PyRefMut, PyResult,
    Python,
};
use std::collections::btree_map::Keys as BTreeMapKeysIter;

use crate::{helpers, path};

#[pyclass(module = "avulto", name = "DMM")]
pub struct Dmm {
    map: dmm_tools::dmm::Map,
    #[pyo3(get)]
    extents: (i32, i32, i32),
}

enum Address {
    Key(dmm_tools::dmm::Key),
    Coords(dmm_tools::dmm::Coord3),
}

#[pyclass(module = "avulto")]
pub struct Tile {
    dmm: Py<PyAny>,
    addr: Address,
}

type Itertools2DCartesianProductIter =
    itertools::Product<std::ops::RangeInclusive<i32>, std::ops::RangeInclusive<i32>>;
type Itertools3DCartesianProductIter =
    itertools::Product<Itertools2DCartesianProductIter, std::ops::RangeInclusive<i32>>;

#[pyclass(module = "avulto")]
pub struct CoordIterator {
    iter: itertools::ConsTuples<Itertools3DCartesianProductIter, ((i32, i32), i32)>,
}

#[pyclass(module = "avulto")]
pub struct KeyIterator {
    dmm: Py<PyAny>,
    iter: BTreeMapKeysIter<'static, dmm_tools::dmm::Key, Vec<dmm_tools::dmm::Prefab>>,
}

#[pymethods]
impl KeyIterator {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>, py: Python<'_>) -> IterNextOutput<PyObject, ()> {
        match slf.iter.next() {
            Some(c) => IterNextOutput::Yield(
                Tile {
                    dmm: slf.dmm.as_ref(py).into_py(py),
                    addr: Address::Key(*c),
                }
                .into_py(py),
            ),
            None => IterNextOutput::Return(()),
        }
    }
}

// TODO(wso): double check to see if there's a better iterator available already
#[pymethods]
impl CoordIterator {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }
    fn __next__(mut slf: PyRefMut<'_, Self>, py: Python<'_>) -> IterNextOutput<PyObject, ()> {
        match slf.iter.next() {
            Some(c) => IterNextOutput::Yield(c.into_py(py)),
            None => IterNextOutput::Return(()),
        }
    }
}

#[pymethods]
impl Dmm {
    #[staticmethod]
    fn from_file(filename: &PyAny) -> PyResult<Dmm> {
        if let Ok(path) = filename.extract::<std::path::PathBuf>() {
            let map = dmm_tools::dmm::Map::from_file(&path).unwrap();
            let dim = map.dim_xyz();
            return Ok(Dmm {
                map,
                extents: (dim.0 as i32, dim.1 as i32, dim.2 as i32),
            });
        } else if let Ok(pystr) = filename.downcast::<PyString>() {
            let map = dmm_tools::dmm::Map::from_file(Path::new(&pystr.to_string())).unwrap();
            let dim = map.dim_xyz();
            return Ok(Dmm {
                map,
                extents: (dim.0 as i32, dim.1 as i32, dim.2 as i32),
            });
        };

        Err(PyRuntimeError::new_err(format!(
            "invalid filename {}",
            filename
        )))
    }

    fn save_to(&self, filename: &PyAny) -> PyResult<()> {
        if let Ok(path) = filename.extract::<std::path::PathBuf>() {
            if let Ok(()) = self.map.to_file(&path) {
                return Ok(());
            }
        } else if let Ok(pystr) = filename.downcast::<PyString>() {
            if let Ok(()) = self.map.to_file(Path::new(&pystr.to_string())) {
                return Ok(());
            }
        }

        Err(PyRuntimeError::new_err(format!(
            "invalid filename {}",
            filename
        )))
    }

    fn tiledef(self_: PyRef<'_, Self>, x: i32, y: i32, z: i32) -> Tile {
        Python::with_gil(|py| Tile {
            dmm: self_.into_py(py),
            addr: Address::Coords(dmm_tools::dmm::Coord3 { x, y, z }),
        })
    }

    fn coords(&self) -> CoordIterator {
        let coords = self.map.dim_xyz();
        CoordIterator {
            iter: iproduct!(
                1..=(coords.0 as i32),
                1..=(coords.1 as i32),
                1..=(coords.2 as i32)
            ),
        }
    }

    fn tiles(self_: PyRef<'_, Self>, py: Python<'_>) -> PyResult<Py<KeyIterator>> {
        let self_ = &self_;
        let owner = self_.into_py(self_.py());
        let it = KeyIterator {
            dmm: owner,
            // WARNING: According to the Nomicon this is one of the most 'wildly
            // unsafe' things it is possible to do in Rust; I don't fully
            // understand it myself but the gist is that we can't guarantee a
            // limited lifetime of the iterator.
            iter: unsafe {
                std::mem::transmute::<
                    btree_map::Keys<'_, dmm_tools::dmm::Key, Vec<dmm_tools::dmm::Prefab>>,
                    btree_map::Keys<'static, dmm_tools::dmm::Key, Vec<dmm_tools::dmm::Prefab>>,
                >(self_.map.dictionary.keys())
            },
        };

        Py::new(py, it)
    }
}

#[pymethods]
impl Tile {
    pub fn add_path(&self, index: i32, path: String, py: Python<'_>) {
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
            .insert(index as usize, Prefab::from_path(path))
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
                    var.set_item("value", helpers::constant_to_python_value(constant))?;
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

    pub fn find(&self, prefix: &PyAny) -> PyResult<Vec<i32>> {
        Python::with_gil(|py| {
            let mut vec = Vec::new();
            let map = &self.dmm.downcast::<PyCell<Dmm>>(py).unwrap().borrow().map;
            let key = match self.addr {
                Address::Key(k) => k,
                Address::Coords(c) => map[c],
            };
            let prefabs = &map.dictionary[&key];

            if let Ok(val) = prefix.extract::<path::Path>() {
                prefabs.iter().enumerate().for_each(|(i, p)| {
                    if p.path.starts_with(&val.0) {
                        vec.push(i as i32);
                    }
                });
            } else if let Ok(pystr) = prefix.downcast::<PyString>() {
                prefabs.iter().enumerate().for_each(|(i, p)| {
                    if p.path.starts_with(&pystr.to_string()) {
                        vec.push(i as i32);
                    }
                });
            } else {
                return Err(PyErr::new::<PyValueError, &str>("not a valid path"));
            }

            Ok(vec)
        })
    }

    pub fn paths(&self, py: Python<'_>) -> PyResult<Py<PyList>> {
        let mut out: Vec<Py<PyAny>> = Vec::new();

        let map = &self.dmm.downcast::<PyCell<Dmm>>(py).unwrap().borrow().map;
        let key = match self.addr {
            Address::Key(k) => k,
            Address::Coords(c) => map[c],
        };
        let prefabs = &map.dictionary[&key];

        for prefab in prefabs {
            out.push(path::Path::new(prefab.path.as_str()).unwrap().into_py(py));
        }

        Ok(PyList::new(py, out).into_py(py))
    }

    pub fn prefab_path(&self, index: i32, py: Python<'_>) -> String {
        let map = &self.dmm.downcast::<PyCell<Dmm>>(py).unwrap().borrow().map;
        let key = match self.addr {
            Address::Key(k) => k,
            Address::Coords(c) => map[c],
        };
        let prefabs = &map.dictionary[&key];

        prefabs[index as usize].path.clone()
    }

    pub fn prefab_var(&self, index: i32, name: String) -> PyObject {
        Python::with_gil(|py| {
            let map = &self.dmm.downcast::<PyCell<Dmm>>(py).unwrap().borrow().map;
            let key = match self.addr {
                Address::Key(k) => k,
                Address::Coords(c) => map[c],
            };
            let prefabs = &map.dictionary[&key];

            helpers::constant_to_python_value(prefabs[index as usize].vars.get(&name).unwrap())
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
            .borrow_mut()
            .as_mut()
            .unwrap()
            .vars
            .insert(
                name,
                if val.is_instance_of::<PyBool>() {
                    let val = val.extract::<bool>().unwrap();
                    Constant::Float(if val { 1.0 } else { 0.0 })
                } else if let Ok(int) = val.downcast::<PyInt>() {
                    Constant::Float(int.extract::<f32>().expect("could not cast float"))
                } else if let Ok(float) = val.downcast::<PyFloat>() {
                    Constant::Float(float.extract::<f32>().expect("could not cast float"))
                } else if let Ok(pystr) = val.downcast::<PyString>() {
                    Constant::String(pystr.to_string().into_boxed_str())
                } else if val.is_none() {
                    Constant::Null(None)
                } else {
                    panic!("cannot use {} as value", val);
                },
            );
    }

    pub fn set_path(&self, index: i32, path: &PyAny, py: Python<'_>) -> PyResult<()> {
        let cell: &PyCell<Dmm> = self.dmm.downcast(py).unwrap();
        let key = match self.addr {
            Address::Key(k) => k,
            Address::Coords(c) => cell.borrow_mut().map[c],
        };

        if let Ok(val) = path.extract::<path::Path>() {
            cell.borrow_mut().map.dictionary.get_mut(&key).unwrap()[index as usize].path = val.0;
            return Ok(())
        } else if let Ok(pystr) = path.downcast::<PyString>() {
            cell.borrow_mut().map.dictionary.get_mut(&key).unwrap()[index as usize].path =
                pystr.to_string();
            return Ok(())
        }

        Err(PyErr::new::<PyValueError, &str>("not a valid path"))
    }

    pub fn turf_path(&self, py: Python<'_>) -> PyResult<Py<PyString>> {
        let map = &self.dmm.downcast::<PyCell<Dmm>>(py).unwrap().borrow().map;
        let key = match self.addr {
            Address::Key(k) => k,
            Address::Coords(c) => map[c],
        };
        let prefabs = &map.dictionary[&key];

        for p in prefabs.iter() {
            if p.path.starts_with("/turf") {
                return Ok(PyString::new(py, p.path.as_str()).into_py(py));
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
