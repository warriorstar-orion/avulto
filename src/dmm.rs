extern crate dmm_tools;

use std::io;
use std::borrow::BorrowMut;
use std::collections::{btree_map, HashMap, HashSet};
use std::collections::btree_map::Keys as BTreeMapKeysIter;
use std::path::{Path, PathBuf};

use dmm_tools::dmm::{Key, Prefab};
use itertools::iproduct;
use pyo3::exceptions::{PyOSError, PyRuntimeError, PyValueError};
use pyo3::types::{PyAnyMethods, PyList, PyString, PyTuple};
use pyo3::{
    pyclass, pymethods, Bound, IntoPyObject, IntoPyObjectExt, Py, PyAny, PyRef, PyRefMut, PyResult, Python
};

use crate::tile::Tile;

#[pyclass(module = "avulto")]
#[derive(Clone, Eq, Hash, PartialOrd, Ord, PartialEq)]
pub struct Coord3 {
    #[pyo3(get)]
    x: i32,
    #[pyo3(get)]
    y: i32,
    #[pyo3(get)]
    z: i32,
}

#[pymethods]
impl Coord3 {
    fn __eq__(&self, other: &Bound<PyAny>) -> bool {
        if let Ok(tuple) = other.cast::<PyTuple>() {
            if tuple.len().unwrap() != 3 {
                return false;
            } else if let Ok((x, y, z)) = tuple.extract::<(i32, i32, i32)>() {
                return self.x == x && self.y == y && self.z == z;
            }
        } else if let Ok(list) = other.cast::<PyList>() {
            if list.len().unwrap() != 3 {
                return false;
            } else if let Ok((x, y, z)) = list.extract::<(i32, i32, i32)>() {
                return self.x == x && self.y == y && self.z == z;
            }
        }
        false
    }
}

#[pyclass(module = "avulto", name = "DMM")]
pub struct Dmm {
    pub(crate) map: dmm_tools::dmm::Map,
    #[pyo3(get)]
    size: Coord3,
    #[pyo3(get)]
    filepath: Py<PyAny>,
}

impl Dmm {
    pub fn lookup_prefab(
        &self,
        key: dmm_tools::dmm::Key,
        idx: usize,
    ) -> Option<&dmm_tools::dmm::Prefab> {
        self.map.dictionary.get(&key)?.get(idx)
    }
}

pub(crate) enum Address {
    Key(dmm_tools::dmm::Key),
    Coords(dmm_tools::dmm::Coord3),
}

type Itertools2DCartesianProductIter =
    itertools::Product<std::ops::RangeInclusive<i32>, std::ops::RangeInclusive<i32>>;
type Itertools3DCartesianProductIter =
    itertools::Product<Itertools2DCartesianProductIter, std::ops::RangeInclusive<i32>>;

#[pyclass(module = "avulto")]
pub struct CoordIterator {
    iter: itertools::ConsTuples<Itertools3DCartesianProductIter>,
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

    fn __next__(mut slf: PyRefMut<'_, Self>, py: Python<'_>) -> Option<Py<PyAny>> {
        slf.iter.next().map(|c| {
            Tile {
                dmm: slf.dmm.bind(py).into_py_any(py).unwrap(),
                addr: Address::Key(*c),
            }
            .into_py_any(py).unwrap()
        })
    }
}

// TODO(wso): double check to see if there's a better iterator available already
#[pymethods]
impl CoordIterator {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }
    fn __next__(mut slf: PyRefMut<'_, Self>, py: Python<'_>) -> Option<Py<PyAny>> {
        slf.iter.next().map(|c| c.into_py_any(py).unwrap())
    }
}

impl Dmm {
    pub fn generate_new_key(&mut self) -> Key {
        let mut key: Key = Default::default();
        while self.map.dictionary.contains_key(&key) {
            key = key.next();
        }

        self.map.dictionary.insert(key, vec![]);
        self.map.adjust_key_length();
        key
    }

    pub fn coalesce_duplicate_tiles(&mut self) {
        let mut coords_using_keys: HashMap<Key, Vec<dmm_tools::dmm::Coord3>> = HashMap::default();
        let map = &mut self.map;

        // First collect all known keys
        map.iter_levels().for_each(|(z, zlvl)| {
            zlvl.iter_top_down().for_each(|(coord, key)| {
                coords_using_keys.entry(key).or_default().push(coord.z(z));
            });
        });

        // Then find our prefab collisions, moving the collided keys over to the first one we found
        // Then update the coords we know of and move them to the first key
        let mut unused_keys: HashSet<Key> = HashSet::default();
        let mut prefab_collisions: HashMap<&Vec<Prefab>, &Key> = HashMap::default();
        for (key, prefabs) in &map.dictionary {
            if prefab_collisions.contains_key(prefabs) {
                unused_keys.insert(*key);
                if let Some(coords) = coords_using_keys.get(key) {
                    for coord in coords {
                        let dim = map.grid.dim();
                        let raw = (coord.z as usize - 1, dim.1 - coord.y as usize, coord.x as usize - 1);
                        map.grid[raw] = *prefab_collisions[prefabs];
                    }
                }
            } else {
                prefab_collisions.insert(prefabs, key);
            }
        }

        for key in unused_keys {
            map.dictionary.borrow_mut().remove_entry(&key);
        }    

        map.adjust_key_length();
    }

    fn write_to_file(&mut self, path: &Path) -> io::Result<()> {
        self.coalesce_duplicate_tiles();
        self.map.to_file(path)
    }
}

#[pymethods]
impl Dmm {
    #[staticmethod]
    fn from_file(filename: &Bound<PyAny>, py: Python<'_>) -> PyResult<Dmm> {
        let pathlib = py.import(pyo3::intern!(py, "pathlib"))?;
        let path = if let Ok(pathbuf) = filename.extract::<std::path::PathBuf>() {
            pathbuf
        } else if let Ok(pystr) = filename.cast::<PyString>() {
            PathBuf::from(&pystr.to_string())
        } else {
            return Err(PyRuntimeError::new_err(format!(
                "invalid filename {}",
                filename
            )));
        };

        if !path.is_file() {
            return Err(PyOSError::new_err(format!("file not found: {:?}", path)));
        }

        let map = dmm_tools::dmm::Map::from_file(&path).unwrap();
        let dim = map.dim_xyz();
        let pathlib_path = pathlib.call_method1(pyo3::intern!(py, "Path"), (path,))?;
        Ok(Dmm {
            map,
            size: Coord3 {
                x: dim.0 as i32,
                y: dim.1 as i32,
                z: dim.2 as i32,
            },
            filepath: pathlib_path.unbind(),
        })
    }

    fn save_to(&mut self, filename: &Bound<PyAny>) -> PyResult<()> {
        if let Ok(path) = filename.extract::<std::path::PathBuf>() {
            if let Ok(()) = self.write_to_file(&path) {
                return Ok(());
            }
        } else if let Ok(pystr) = filename.cast::<PyString>() {
            if let Ok(()) = self.write_to_file(Path::new(&pystr.to_string())) {
                return Ok(());
            }
        }

        Err(PyValueError::new_err(format!(
            "invalid filename {}",
            filename
        )))
    }

    fn tiledef(self_: PyRef<'_, Self>, x: i32, y: i32, z: i32) -> Tile {
        Python::attach(|py| Tile {
            dmm: self_.into_pyobject(py).unwrap().into_any().unbind(),
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
        let owner = self_.into_py_any(self_.py()).unwrap();
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

    fn __repr__(&self, py: Python<'_>) -> PyResult<String> {
        Ok(format!(
            "<DMM {} {}x{}x{}>",
            self.filepath.getattr(py, "name").unwrap(),
            self.size.x,
            self.size.y,
            self.size.z
        ))
    }
}
