use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;

use dmi::icon::Icon;
use pyo3::exceptions::PyRuntimeError;
use pyo3::pyclass::CompareOp;
use pyo3::types::{PyAnyMethods, PyBytes, PyString};
use pyo3::Bound;
use pyo3::{
    pyclass, pymethods, types::PyList, IntoPy, Py, PyAny, PyObject, PyRef, PyRefMut, PyResult,
    Python,
};

use crate::helpers::Dir;

#[pyclass(module = "avulto", name = "DMI")]
pub struct Dmi {
    icon: Icon,
    #[pyo3(get)]
    filepath: Py<PyAny>,
}

#[pyclass(module = "avulto")]
pub struct IconState {
    dmi: Py<PyAny>,
    idx: usize,
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

    fn __richcmp__(&self, other: Bound<PyAny>, op: CompareOp) -> PyResult<bool> {
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
    #[getter]
    pub fn name(&self, py: Python<'_>) -> String {
        let dmi: &Bound<Dmi> = self.dmi.downcast_bound(py).unwrap();
        dmi.borrow().icon.states[self.idx].name.clone()
    }

    #[getter]
    pub fn movement(&self, py: Python<'_>) -> bool {
        let dmi: &Bound<Dmi> = self.dmi.downcast_bound(py).unwrap();
        dmi.borrow().icon.states[self.idx].movement
    }

    #[getter]
    pub fn delays(&self, py: Python<'_>) -> PyResult<Py<PyList>> {
        let mut out: Vec<f32> = Vec::new();
        let dmi = self.dmi.downcast_bound::<Dmi>(py).unwrap();

        let binding = dmi.borrow();
        let state = binding.icon.states.get(self.idx).unwrap();
        if let Some(delays) = &state.delay {
            out.extend(delays);
        }

        Ok(PyList::new_bound(py, out).into())
    }

    #[getter]
    pub fn dirs(&self, py: Python<'_>) -> PyResult<Py<PyList>> {
        let dmi: &Bound<Dmi> = self.dmi.downcast_bound(py).unwrap();
        let dirs = dmi.borrow().icon.states.get(self.idx).unwrap().dirs;
        Ok(PyList::new_bound(
            py,
            match dirs {
                1 => vec![Dir::South],
                4 => vec![Dir::South, Dir::North, Dir::East, Dir::West],
                8 => vec![
                    Dir::South,
                    Dir::North,
                    Dir::East,
                    Dir::West,
                    Dir::Southeast,
                    Dir::Southwest,
                    Dir::Northeast,
                    Dir::Northwest,
                ],
                _ => panic!("invalid number of dirs {}", dirs),
            }
            .iter()
            .map(|f| Py::new(py, *f).unwrap()),
        )
        .into())
    }

    #[getter]
    pub fn frames(&self, py: Python<'_>) -> u32 {
        let dmi: &Bound<Dmi> = self.dmi.downcast_bound(py).unwrap();
        let binding = dmi.borrow();
        let state = binding.icon.states.get(self.idx).unwrap();
        state.frames
    }

    #[getter]
    pub fn rewind(&self, py: Python<'_>) -> bool {
        let dmi: &Bound<Dmi> = self.dmi.downcast_bound(py).unwrap();
        dmi.borrow().icon.states[self.idx].rewind
    }

    pub fn data_rgba8(&self, frame: usize, py: Python<'_>) -> PyResult<Py<PyBytes>> {
        let dmi: &Bound<Dmi> = self.dmi.downcast_bound(py).unwrap();
        let binding = dmi.borrow();
        let state = binding.icon.states.get(self.idx).unwrap();

        let frame_data = &state.images[frame - 1];
        let buffer = Vec::new();
        let mut cursor = std::io::Cursor::new(buffer);
        cursor.write(frame_data.as_bytes());
        let output = cursor.into_inner();
        Ok(PyBytes::new_bound(py, &output).into())
    }

    fn __str__(&self, py: Python<'_>) -> PyResult<String> {
        self.__repr__(py)
    }

    fn __repr__(&self, py: Python<'_>) -> PyResult<String> {
        let dmi: &Bound<Dmi> = self.dmi.downcast_bound(py).unwrap();
        let binding = dmi.borrow();
        let state = binding.icon.states.get(self.idx).unwrap();

        Ok(format!(
            "<IconState '{}' dirs={} frames={}>",
            state.name, state.dirs, state.frames
        ))
    }
}

#[pymethods]
impl Dmi {
    #[staticmethod]
    pub fn from_file(filename: &Bound<PyAny>, py: Python<'_>) -> PyResult<Dmi> {
        let pathlib = py.import_bound(pyo3::intern!(py, "pathlib"))?;
        if let Ok(path) = filename.extract::<std::path::PathBuf>() {
            let pathlib_path = pathlib.call_method1(pyo3::intern!(py, "Path"), (path.clone(),))?;
            let file = match File::open(path) {
                Ok(f) => f,
                Err(err) => panic!("file error: {}", err),
            };
            let icon = match Icon::load(BufReader::new(file)) {
                Ok(i) => i,
                Err(err) => panic!("icon load error: {}", err),
            };
            return Ok(Dmi {
                icon,
                filepath: pathlib_path.into_py(py),
            });
        } else if let Ok(pystr) = filename.downcast::<PyString>() {
            let pathlib_path = pathlib.call_method1(pyo3::intern!(py, "Path"), (pystr,))?;
            let file = match File::open(Path::new(&pystr.to_string())) {
                Ok(f) => f,
                Err(err) => panic!("file error: {}", err),
            };
            let icon = match Icon::load(BufReader::new(file)) {
                Ok(i) => i,
                Err(err) => panic!("icon load error: {}", err),
            };
            return Ok(Dmi {
                icon,
                filepath: pathlib_path.into_py(py),
            });
        };

        Err(PyRuntimeError::new_err(format!(
            "invalid filename {}",
            filename
        )))
    }

    pub fn state_names(&self, py: Python<'_>) -> PyResult<Py<PyList>> {
        let keys: Vec<String> = self.icon.states.iter().map(|s| s.name.clone()).collect();
        Ok(PyList::new_bound(py, keys).into())
    }

    pub fn states(self_: PyRef<'_, Self>, py: Python<'_>) -> PyResult<Py<StateIter>> {
        let mut out: Vec<Py<PyAny>> = Vec::new();
        let self_ = &self_;

        for (idx, _) in (&self_.icon.states).iter().enumerate() {
            out.push(
                IconState {
                    dmi: self_.into_py(py),
                    idx,
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

    pub fn state(self_: PyRef<'_, Self>, value: String, py: Python<'_>) -> PyResult<IconState> {
        for (idx, state) in (&self_.icon.states).iter().enumerate() {
            if state.name == value {
                return Ok(IconState {
                    dmi: self_.into_py(py),
                    idx,
                });
            }
        }
        Err(PyRuntimeError::new_err(format!(
            "invalid state name {}",
            value
        )))
    }

    #[getter]
    pub fn icon_width(&self) -> u32 {
        self.icon.width
    }

    #[getter]
    pub fn icon_height(&self) -> u32 {
        self.icon.height
    }
}

#[pyclass(module = "avulto")]
pub struct StateIter {
    inner: std::vec::IntoIter<PyObject>,
}

#[pymethods]
impl StateIter {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<Py<PyAny>> {
        if let Some(n) = slf.inner.next() {
            let cell = n.downcast_bound::<IconState>(slf.py()).unwrap();
            let state = cell.borrow_mut();
            return Some(state.into_py(slf.py()));
        }

        None
    }
}
