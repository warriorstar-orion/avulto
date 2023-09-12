use std::path::Path;

use dreammaker::dmi::{Metadata, StateIndex};
use lodepng::RGBA;
use pyo3::{
    pyclass, pymethods,
    types::PyList,
    IntoPy, Py, PyAny, PyCell, PyRef, PyResult, Python,
};

use crate::helpers::to_dmm_dir;
use crate::helpers::Dir;

extern crate dreammaker;

#[pyclass(module = "avulto", name = "DMI")]
pub struct Dmi {
    metadata: Metadata,
    bitmap: lodepng::Bitmap<RGBA>,
}

#[pyclass(module = "avulto")]
pub struct IconState {
    dmi: Py<PyAny>,
    state_idx: StateIndex,
}

#[pyclass(module = "avulto")]
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
    fn __str__(&self) -> PyResult<String> {
        self.__repr__()
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!(
            "<Rect {}, {}, {}, {}>",
            self.left, self.top, self.width, self.height
        ))
    }
}

#[pymethods]
impl IconState {
    pub fn name(&self) -> String {
        self.state_idx.to_string()
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
                    Dir::Northeast,
                    Dir::Northwest,
                    Dir::Southeast,
                    Dir::Southwest,
                ],
            }
            .iter()
            .map(|f| Py::new(py, *f).unwrap()),
        )
        .into_py(py))
    }
}

#[pymethods]
impl Dmi {
    #[staticmethod]
    pub fn from_file(filename: String) -> Dmi {
        let results = Metadata::from_file(Path::new(&filename)).unwrap();
        Dmi {
            bitmap: results.0,
            metadata: results.1,
        }
    }

    pub fn rect(
        &self,
        icon_state: String,
        dir: Dir,
        frame: u32,
        py: Python<'_>,
    ) -> PyResult<Py<Rect>> {
        let rect = self.metadata.rect_of(
            self.bitmap.width as u32,
            &StateIndex::from(icon_state),
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

    pub fn state_names(&self, py: Python<'_>) -> PyResult<Py<PyList>> {
        let keys: Vec<String> = self
            .metadata
            .states
            .iter()
            .map(|x| x.name.clone())
            .collect();
        Ok(PyList::new(py, keys).into_py(py))
    }

    pub fn state(self_: PyRef<'_, Self>, name: String, py: Python<'_>) -> IconState {
        IconState {
            dmi: self_.into_py(py),
            state_idx: StateIndex::from(name),
        }
    }
}
