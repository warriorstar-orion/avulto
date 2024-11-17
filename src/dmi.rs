use std::fs::File;
use std::io::{self, BufReader, Write};
use std::path::PathBuf;

use dmi::icon::Icon;
use pyo3::exceptions::{PyException, PyFileNotFoundError, PyRuntimeError};
use pyo3::pyclass::CompareOp;
use pyo3::types::{PyAnyMethods, PyBytes, PyString, PyTuple};
use pyo3::{create_exception, Bound, BoundObject, IntoPyObject};
use pyo3::{
    pyclass, pymethods, types::PyList, IntoPy, Py, PyAny, PyObject, PyRef, PyRefMut, PyResult,
    Python,
};

use crate::helpers::Dir;

create_exception!(avulto.exceptions, IconError, PyException);

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

        Ok(PyList::new(py, out)?.into())
    }

    #[getter]
    pub fn dirs(&self, py: Python<'_>) -> PyResult<Py<PyList>> {
        let dmi: &Bound<Dmi> = self.dmi.downcast_bound(py).unwrap();
        let dirs = dmi.borrow().icon.states.get(self.idx).unwrap().dirs;
        Ok(PyList::new(
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
        )?
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

    pub fn data_rgba8(
        &self,
        frame: u32,
        dir: &Bound<PyAny>,
        py: Python<'_>,
    ) -> PyResult<Py<PyBytes>> {
        let dmi: &Bound<Dmi> = self.dmi.downcast_bound(py).unwrap();
        let binding = dmi.borrow();
        let state = binding.icon.states.get(self.idx).unwrap();

        if let Ok(diridx) = dir.extract::<Dir>() {
            let diridx = match diridx {
                Dir::North => dmi::dirs::Dirs::NORTH,
                Dir::South => dmi::dirs::Dirs::SOUTH,
                Dir::East => dmi::dirs::Dirs::EAST,
                Dir::West => dmi::dirs::Dirs::WEST,
                Dir::Northeast => dmi::dirs::Dirs::NORTHEAST,
                Dir::Northwest => dmi::dirs::Dirs::NORTHWEST,
                Dir::Southeast => dmi::dirs::Dirs::SOUTHEAST,
                Dir::Southwest => dmi::dirs::Dirs::SOUTHWEST,
            };
            let frame_data = state.get_image(&diridx, frame).unwrap();
            let buffer = Vec::new();
            let mut cursor = std::io::Cursor::new(buffer);
            cursor.write_all(frame_data.as_bytes())?;
            let output = cursor.into_inner();
            Ok(PyBytes::new(py, &output).into())
        } else {
            Err(PyRuntimeError::new_err("invalid direction"))
        }
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
        let pathlib = py.import(pyo3::intern!(py, "pathlib"))?;

        let path = if let Ok(pathbuf) = filename.extract::<std::path::PathBuf>() {
            pathbuf
        } else if let Ok(pystr) = filename.downcast::<PyString>() {
            PathBuf::from(&pystr.to_string())
        } else {
            return Err(PyRuntimeError::new_err(format!(
                "invalid filename {}",
                filename
            )));
        };

        let pathlib_path = pathlib.call_method1(pyo3::intern!(py, "Path"), (path.clone(),))?;
        // TODO: Why am I doing this like this instead of just checking for path.exists?
        let file = match File::open(&path) {
            Ok(f) => f,
            Err(err) => {
                if err.kind() == io::ErrorKind::NotFound {
                    return Err(PyFileNotFoundError::new_err(format!(
                        "Not found: {}",
                        path.to_str().unwrap()
                    )));
                }
                return Err(PyRuntimeError::new_err(format!("Unknown error: {}", err)));
            }
        };

        Icon::load(BufReader::new(file)).map_or_else(
            |err| {
                Err(IconError::new_err(format!(
                    "Error loading icon file: {}",
                    err
                )))
            },
            |icon| {
                Ok(Dmi {
                    icon,
                    filepath: pathlib_path
                        .into_pyobject(py)
                        .expect("setting icon filepath")
                        .unbind(),
                })
            },
        )
    }

    pub fn state_names(&self, py: Python<'_>) -> PyResult<Py<PyList>> {
        let keys: Vec<String> = self.icon.states.iter().map(|s| s.name.clone()).collect();
        Ok(PyList::new(py, keys)?.into())
    }

    pub fn states(self_: PyRef<'_, Self>, py: Python<'_>) -> PyResult<Py<StateIter>> {
        let mut out: Vec<Py<PyAny>> = Vec::new();
        let self_ = &self_;

        for (idx, _) in self_.icon.states.iter().enumerate() {
            out.push(
                IconState {
                    dmi: self_.into_pyobject(py)?.into_any().unbind(),
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
        for (idx, state) in self_.icon.states.iter().enumerate() {
            if state.name == value {
                return Ok(IconState {
                    dmi: self_.into_pyobject(py)?.into_any().unbind(),
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

    #[getter]
    pub fn icon_dims(&self, py: Python<'_>) -> Py<PyTuple> {
        PyTuple::new(py, [self.icon.width, self.icon.height])
            .expect("icon dims")
            .unbind()
    }

    fn __str__(&self, py: Python<'_>) -> PyResult<String> {
        self.__repr__(py)
    }

    fn __repr__(&self, py: Python<'_>) -> PyResult<String> {
        Ok(format!(
            "<DMI {} {}x{}>",
            &self.filepath.getattr(py, "name").unwrap(),
            &self.icon.width,
            &self.icon.height
        ))
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
