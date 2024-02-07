use std::path::Path;

use dreammaker::dmi::{Metadata, StateIndex};
use lodepng::RGBA;
use pyo3::exceptions::PyRuntimeError;
use pyo3::pyclass::CompareOp;
use pyo3::types::{PyBytes, PyString};
use pyo3::{
    pyclass, pymethods, types::PyList, IntoPy, Py, PyAny, PyCell, PyObject, PyRef, PyRefMut,
    PyResult, Python,
};

use crate::helpers::to_dmm_dir;
use crate::helpers::Dir;

extern crate dreammaker;

#[pyclass(module = "avulto", name = "DMI")]
pub struct Dmi {
    metadata: Metadata,
    bitmap: lodepng::Bitmap<RGBA>,
    #[pyo3(get)]
    filepath: Py<PyAny>,   
}

#[pyclass(module = "avulto")]
pub struct IconState {
    dmi: Py<PyAny>,
    state_idx: StateIndex,
}

#[pyclass(module = "avulto")]
#[derive(Clone)]
pub struct Rect {
    #[pyo3(get)]
    left: u32,
    #[pyo3(get)]
    top: u32,
    #[pyo3(get)]
    width: u32,
    #[pyo3(get)]
    height: u32,
}

#[pyclass(module = "avulto")]
pub struct StateIter {
    inner: std::vec::IntoIter<PyObject>,
}

#[pymethods]
impl Rect {
    #[new]
    pub fn new(left: u32, top: u32, width: u32, height: u32) -> PyResult<Self> {
        Ok(Rect {
            left,
            top,
            width,
            height,
        })
    }

    fn __str__(&self) -> PyResult<String> {
        self.__repr__()
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!(
            "<Rect {}, {}, {}, {}>",
            self.left, self.top, self.width, self.height
        ))
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<bool> {
        if let Ok(rhs) = other.extract::<Self>() {
            let cmp = self.left == rhs.left
                && self.width == rhs.width
                && self.top == rhs.top
                && self.height == rhs.height;
            return match op {
                CompareOp::Eq => Ok(cmp),
                CompareOp::Ne => Ok(!cmp),
                _ => Err(PyRuntimeError::new_err("invalid comparison")),
            };
        }

        Ok(false)
    }
}

#[pymethods]
impl IconState {
    pub fn name(&self, py: Python<'_>) -> String {
        let dmi: &PyCell<Dmi> = self.dmi.downcast(py).unwrap();
        dmi.borrow()
            .metadata
            .get_icon_state(&self.state_idx)
            .unwrap()
            .name
            .clone()
    }

    pub fn movement(&self, py: Python<'_>) -> bool {
        let dmi: &PyCell<Dmi> = self.dmi.downcast(py).unwrap();
        dmi.borrow()
            .metadata
            .get_icon_state(&self.state_idx)
            .unwrap()
            .movement
    }

    pub fn delays(&self, py: Python<'_>) -> PyResult<Py<PyList>> {
        let mut out: Vec<f32> = Vec::new();
        let dmi: &PyCell<Dmi> = self.dmi.downcast(py).unwrap();
        let binding = dmi.borrow();
        let istate = binding.metadata.get_icon_state(&self.state_idx).unwrap();
        let frames = &istate.frames;
        for i in 0..istate.frames.count() {
            out.push(frames.delay(i));
        }

        Ok(PyList::new(py, out).into_py(py))
    }

    pub fn dirs(&self, py: Python<'_>) -> PyResult<Py<PyList>> {
        let dmi: &PyCell<Dmi> = self.dmi.downcast(py).unwrap();
        let dirs = dmi
            .borrow()
            .metadata
            .get_icon_state(&self.state_idx)
            .unwrap()
            .dirs;
        Ok(PyList::new(
            py,
            match dirs {
                dreammaker::dmi::Dirs::One => vec![Dir::South],
                dreammaker::dmi::Dirs::Four => vec![Dir::South, Dir::North, Dir::East, Dir::West],
                dreammaker::dmi::Dirs::Eight => vec![
                    Dir::South,
                    Dir::North,
                    Dir::East,
                    Dir::West,
                    Dir::Southeast,
                    Dir::Southwest,
                    Dir::Northeast,
                    Dir::Northwest,
                ],
            }
            .iter()
            .map(|f| Py::new(py, *f).unwrap()),
        )
        .into_py(py))
    }

    pub fn frames(&self, py: Python<'_>) -> u32 {
        let dmi: &PyCell<Dmi> = self.dmi.downcast(py).unwrap();
        dmi.borrow()
            .metadata
            .get_icon_state(&self.state_idx)
            .unwrap()
            .frames
            .count() as u32
    }

    pub fn rewind(&self, py: Python<'_>) -> bool {
        let dmi: &PyCell<Dmi> = self.dmi.downcast(py).unwrap();
        dmi.borrow()
            .metadata
            .get_icon_state(&self.state_idx)
            .unwrap()
            .rewind
    }

    pub fn rect(&self, dirval: &PyAny, frame: u32, py: Python<'_>) -> PyResult<Py<Rect>> {
        let mut dir = Dir::South;
        if let Ok(i) = dirval.extract::<i32>() {
            dir = Dir::from(i);
        } else if let Ok(d) = dirval.extract::<Dir>() {
            dir = d;
        }
        let dmi: &PyCell<Dmi> = self.dmi.downcast(py).unwrap();
        let rect = dmi.borrow().metadata.rect_of(
            dmi.borrow().bitmap.width as u32,
            &self.state_idx,
            to_dmm_dir(dir),
            frame,
        );
        match rect {
            Some(r) => Py::new(
                py,
                Rect {
                    left: r.0,
                    top: r.1,
                    width: r.2,
                    height: r.3,
                },
            ),
            None => panic!("cannot get rect"),
        }
    }
}

#[pymethods]
impl Dmi {
    #[staticmethod]
    pub fn from_file(filename: &PyAny, py: Python<'_>) -> PyResult<Dmi> {
        let pathlib = py.import(pyo3::intern!(py, "pathlib"))?;
        if let Ok(path) = filename.extract::<std::path::PathBuf>() {
            let pathlib_path = pathlib.call_method1(pyo3::intern!(py, "Path"), (path.clone(),))?;
            let results = Metadata::from_file(&path).unwrap();
            return Ok(Dmi {
                bitmap: results.0,
                metadata: results.1,
                filepath: pathlib_path.into_py(py),
            });
        } else if let Ok(pystr) = filename.downcast::<PyString>() {
            let pathlib_path = pathlib.call_method1(pyo3::intern!(py, "Path"), (pystr,))?;
            let results = Metadata::from_file(Path::new(&pystr.to_string())).unwrap();
            return Ok(Dmi {
                bitmap: results.0,
                metadata: results.1,
                filepath: pathlib_path.into_py(py),
            });
        };

        Err(PyRuntimeError::new_err(format!(
            "invalid filename {}",
            filename
        )))
    }

    pub fn state_names(&self, py: Python<'_>) -> PyResult<Py<PyList>> {
        let keys: Vec<String> = self
            .metadata
            .states
            .iter()
            .map(|x| x.name.clone())
            .collect();
        Ok(PyList::new(py, keys).into_py(py))
    }

    pub fn states(self_: PyRef<'_, Self>, py: Python<'_>) -> PyResult<Py<StateIter>> {
        let mut out: Vec<Py<PyAny>> = Vec::new();
        let self_ = &self_;

        for state in &self_.metadata.states {
            out.push(
                IconState {
                    dmi: self_.into_py(py),
                    state_idx: state.get_state_name_index(),
                }
                .into_py(py),
            );
        }

        Py::new(
            py,
            StateIter {
                inner: out.into_iter(),
            },
        )
    }

    pub fn state(self_: PyRef<'_, Self>, value: String, py: Python<'_>) -> IconState {
        IconState {
            dmi: self_.into_py(py),
            state_idx: StateIndex::from(value),
        }
    }

    pub fn icon_width(&self) -> u32 {
        self.metadata.width
    }

    pub fn icon_height(&self) -> u32 {
        self.metadata.height
    }

    pub fn data_rgba8(&self, rect: Rect, py: Python<'_>) -> PyResult<Py<PyBytes>> {
        let mut buffer = Vec::new();
        for y in rect.top..(rect.top + rect.height) {
            for x in rect.left..(rect.left + rect.width) {
                let c = self.bitmap.buffer[(y * (self.bitmap.width as u32) + x) as usize];
                buffer.push(c.r);
                buffer.push(c.g);
                buffer.push(c.b);
                buffer.push(c.a);
            }
        }

        Ok(PyBytes::new(py, buffer.as_slice()).into())
    }
}

#[pymethods]
impl StateIter {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<Py<PyAny>> {
        if let Some(n) = slf.inner.next() {
            let cell: &PyCell<IconState> = n.downcast(slf.py()).unwrap();
            let state = cell.borrow_mut();
            return Some(state.into_py(slf.py()));
        }

        None
    }
}
