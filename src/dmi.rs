use std::fs::File;
use std::io::{self, BufReader};
use std::path::{Path, PathBuf};
use std::vec;

use dmi::dirs::Dirs;
use dmi::icon::{DmiVersion, Icon};
use oxipng::{InFile, OutFile};
use pyo3::exceptions::{PyException, PyFileNotFoundError, PyRuntimeError};
use pyo3::types::{PyAnyMethods, PyInt, PyListMethods, PyString, PyTuple};
use pyo3::{Bound, IntoPyObject, IntoPyObjectExt, PyErr, create_exception};
use pyo3::{Py, PyAny, PyRef, PyRefMut, PyResult, Python, pyclass, pymethods, types::PyList};

use crate::dmi::iconstate::IconState;
use crate::helpers::Dir;

pub mod iconstate;

create_exception!(avulto.exceptions, IconError, PyException);

#[pyclass(module = "avulto", name = "DMI")]
pub struct Dmi {
    #[pyo3(get)]
    states: Py<PyList>,
    #[pyo3(get)]
    filepath: Py<PyAny>,
    #[pyo3(get)]
    icon_width: u32,
    #[pyo3(get)]
    icon_height: u32,
}

fn _get_dir_from(arg: &Bound<PyAny>) -> Result<Dirs, PyErr> {
    if let Ok(diridx) = arg.extract::<Dir>() {
        Ok(match diridx {
            Dir::North => dmi::dirs::Dirs::NORTH,
            Dir::South => dmi::dirs::Dirs::SOUTH,
            Dir::East => dmi::dirs::Dirs::EAST,
            Dir::West => dmi::dirs::Dirs::WEST,
            Dir::Northeast => dmi::dirs::Dirs::NORTHEAST,
            Dir::Northwest => dmi::dirs::Dirs::NORTHWEST,
            Dir::Southeast => dmi::dirs::Dirs::SOUTHEAST,
            Dir::Southwest => dmi::dirs::Dirs::SOUTHWEST,
        })
    } else if let Ok(dirint) = arg.cast::<PyInt>() {
        return match dirint.extract::<u8>().unwrap() {
            1 => Ok(dmi::dirs::Dirs::NORTH),
            2 => Ok(dmi::dirs::Dirs::SOUTH),
            4 => Ok(dmi::dirs::Dirs::EAST),
            8 => Ok(dmi::dirs::Dirs::WEST),
            5 => Ok(dmi::dirs::Dirs::NORTHEAST),
            9 => Ok(dmi::dirs::Dirs::NORTHWEST),
            6 => Ok(dmi::dirs::Dirs::SOUTHEAST),
            10 => Ok(dmi::dirs::Dirs::SOUTHWEST),
            _ => Err(PyRuntimeError::new_err("invalid direction")),
        };
    } else {
        Err(PyRuntimeError::new_err("invalid direction"))
    }
}

#[pymethods]
impl Dmi {
    #[staticmethod]
    pub fn new(dims: (u32, u32), py: Python<'_>) -> Self {
        Dmi {
            states: PyList::empty(py).unbind(),
            filepath: py.None(),
            icon_width: dims.0,
            icon_height: dims.1,
        }
    }

    #[staticmethod]
    pub fn from_file(filename: &Bound<PyAny>, py: Python<'_>) -> PyResult<Dmi> {
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
                let list: Vec<IconState> = icon.states.iter().map(IconState::from_dmi).collect();
                let states = PyList::new(py, list).unwrap().as_unbound().clone_ref(py);
                Ok(Dmi {
                    states,
                    filepath: pathlib_path
                        .into_pyobject(py)
                        .expect("setting icon filepath")
                        .unbind(),
                    icon_width: icon.width,
                    icon_height: icon.height,
                })
            },
        )
    }

    pub fn state_names(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let mut out = vec![];
        for boundstate in self.states.bind(py).iter() {
            out.push(
                boundstate
                    .cast_exact::<IconState>()
                    .unwrap()
                    .borrow()
                    .name
                    .clone(),
            );
        }
        PyList::new(py, out).unwrap().into_py_any(py)
    }

    pub fn state(&self, value: String, py: Python<'_>) -> PyResult<Py<PyAny>> {
        for state in self.states.bind(py).iter() {
            let cast_state = state.cast_exact::<IconState>().unwrap().borrow();
            if cast_state.name == value {
                return Ok(cast_state.into_py_any(py).unwrap());
            }
        }
        Err(PyRuntimeError::new_err(format!(
            "invalid state name {}",
            value
        )))
    }

    #[getter]
    pub fn icon_dims(&self, py: Python<'_>) -> Py<PyTuple> {
        PyTuple::new(py, [self.icon_width, self.icon_height])
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
            &self.icon_width,
            &self.icon_height
        ))
    }

    #[pyo3(signature = (filename, compress=0))]
    fn save_to(&self, filename: &Bound<PyAny>, compress: u8, py: Python<'_>) -> PyResult<()> {
        let path = if let Ok(path) = filename.extract::<std::path::PathBuf>() {
            path
        } else if let Ok(pystr) = filename.cast::<PyString>() {
            Path::new(&pystr.to_string()).to_path_buf()
        } else {
            return Err(PyRuntimeError::new_err(format!(
                "invalid filename {}",
                filename
            )));
        };
        self.write_to_file(path.as_path(), compress, py)
    }
}

impl Dmi {
    fn write_to_file(&self, path: &Path, compress: u8, py: Python<'_>) -> PyResult<()> {
        let icon = dmi::icon::Icon {
            version: DmiVersion::default(),
            height: self.icon_height,
            width: self.icon_width,
            states: self
                .states
                .bind(py)
                .into_iter()
                .map(|i| i.cast_into::<IconState>().unwrap().borrow().to_dmi())
                .collect(),
        };
        let mut f = File::create(path)?;
        match icon.save(&mut f) {
            Ok(_) => {
                if compress > 0 {
                    let opts = oxipng::Options::from_preset(compress);
                    let infile = InFile::Path(path.to_path_buf());
                    let outfile = OutFile::from_path(path.to_path_buf());
                    let _ = oxipng::optimize(&infile, &outfile, &opts);
                }
                Ok(())
            }
            Err(e) => Err(PyRuntimeError::new_err(format!("error writing dmi: {e}",))),
        }
    }
}

#[pyclass(module = "avulto")]
pub struct StateIter {
    inner: std::vec::IntoIter<Py<PyAny>>,
}

#[pymethods]
impl StateIter {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<Py<PyAny>> {
        if let Some(n) = slf.inner.next() {
            let cell = n.cast_bound::<IconState>(slf.py()).unwrap();
            let state = cell.borrow_mut();
            return Some(state.into_py_any(slf.py()).unwrap());
        }

        None
    }
}
