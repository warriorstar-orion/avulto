use std::{
    collections::{HashMap, HashSet},
    num::NonZero,
};

use dmi::{
    dirs::Dirs,
    icon::{DIR_ORDERING, Looping, dir_to_dmi_index},
};
use itertools::Itertools;
use pyo3::{
    Bound, Py, PyAny, PyErr, PyResult, Python,
    exceptions::PyRuntimeError,
    pyclass, pymethods,
    types::{PyAnyMethods, PyBytes, PyBytesMethods, PyInt, PyList},
};

use crate::helpers::Dir;

#[pyclass(module = "avulto")]
pub struct IconState {
    pub images: Vec<image::DynamicImage>,

    #[pyo3(get)]
    pub(crate) name: String,
    #[pyo3(get)]
    pub dir_count: u8,
    #[pyo3(get)]
    pub frames: u32,
    #[pyo3(get)]
    pub delay: Vec<f32>,
    #[pyo3(get)]
    pub loop_flag: u32,
    #[pyo3(get)]
    rewind: bool,
    #[pyo3(get)]
    movement: bool,
}

impl IconState {
    pub fn from_dmi(state: &dmi::icon::IconState) -> IconState {
        IconState {
            name: state.name.clone(),
            dir_count: state.dirs,
            frames: state.frames,
            images: state.images.to_vec(),
            delay: state.delay.clone().unwrap_or_default(),
            loop_flag: match state.loop_flag {
                Looping::Indefinitely => 0,
                Looping::NTimes(non_zero) => non_zero.into(),
            },
            rewind: state.rewind,
            movement: state.movement,
        }
    }

    pub(crate) fn to_dmi(&self) -> dmi::icon::IconState {
        dmi::icon::IconState {
            name: self.name.clone(),
            dirs: self.dir_count,
            images: self.images.to_vec(),
            delay: Some(self.delay.clone()),
            frames: self.frames,
            loop_flag: match self.loop_flag {
                0 => Looping::Indefinitely,
                _ => Looping::NTimes(NonZero::new(self.loop_flag).unwrap()),
            },
            hotspot: None,
            rewind: self.rewind,
            movement: self.movement,
            unknown_settings: None,
        }
    }
}

#[pymethods]
impl IconState {
    #[staticmethod]
    #[pyo3(signature=(data, width=32, height=32, name="", delays=None, loops=0, rewind=false, movement=false))]
    pub fn from_data(
        data: HashMap<Dir, Vec<Py<PyBytes>>>,
        width: u32,
        height: u32,
        name: &str,
        delays: Option<Vec<f32>>,
        loops: u32,
        rewind: bool,
        movement: bool,
        py: Python<'_>,
    ) -> PyResult<Self> {
        let data_keys: Vec<&Dir> = data.keys().sorted().collect();
        match data_keys[..] {
            [Dir::South] => {}
            [Dir::North, Dir::South, Dir::East, Dir::West] => {}
            [
                Dir::North,
                Dir::South,
                Dir::East,
                Dir::West,
                Dir::Northeast,
                Dir::Northwest,
                Dir::Southeast,
                Dir::Southwest,
            ] => {}
            _ => {
                return Err(PyRuntimeError::new_err(
                    "directions must match south, cardinal, or all directions".to_string(),
                ));
            }
        }

        let data_counts: HashSet<usize> = HashSet::from_iter(data.values().map(|f| f.len()));
        if data_counts.len() > 1 {
            return Err(PyRuntimeError::new_err(
                "inconsistent number of frames across directions".to_string(),
            ));
        }

        let data_count = data[&Dir::South].len();

        match delays {
            Some(ref d) => {
                if data_count != d.len() {
                    return Err(PyRuntimeError::new_err(
                        "number of frames and delays do not match".to_string(),
                    ));
                }
            }
            None => {
                if data_count != 1 {
                    return Err(PyRuntimeError::new_err(
                        "number of frames and delays do not match".to_string(),
                    ));
                }
            }
        }

        let state_delays = match delays {
            Some(d) => d.clone(),
            None => vec![1.0],
        };

        let mut state = IconState {
            name: name.into(),
            dir_count: data_keys.len() as u8,
            frames: state_delays.len() as u32,
            images: vec![],
            delay: state_delays,
            loop_flag: loops,
            rewind,
            movement,
        };

        for dir in DIR_ORDERING {
            let d = match dir {
                Dirs::SOUTH => Dir::South,
                Dirs::NORTH => Dir::North,
                Dirs::EAST => Dir::East,
                Dirs::WEST => Dir::West,
                Dirs::SOUTHEAST => Dir::Southeast,
                Dirs::SOUTHWEST => Dir::Southwest,
                Dirs::NORTHEAST => Dir::Northeast,
                Dirs::NORTHWEST => Dir::Northwest,
                _ => return Err(PyRuntimeError::new_err(format!("invalid direction {dir}"))),
            };
            if data.contains_key(&d) {
                let dir_data = data.get(&d).unwrap();
                for image_data in dir_data {
                    let image_bytes = image_data.bind(py);
                    if let Some(image) =
                        image::ImageBuffer::from_vec(width, height, image_bytes.as_bytes().to_vec())
                    {
                        state.images.push(image::DynamicImage::ImageRgba8(image));
                    } else {
                        return Err(PyRuntimeError::new_err(format!(
                            "not enough data in argument to fill {}x{} state",
                            width, height
                        )));
                    }
                }
            }
        }
        Ok(state)
    }

    #[getter]
    pub fn dirs(&self, py: Python<'_>) -> PyResult<Py<PyList>> {
        Ok(PyList::new(
            py,
            match self.dir_count {
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
                _ => panic!("invalid number of dirs {}", self.dir_count),
            }
            .iter()
            .map(|f| Py::new(py, *f).unwrap()),
        )?
        .into())
    }

    pub fn data_rgba8(
        &self,
        frame: u32,
        dir: &Bound<PyAny>,
        py: Python<'_>,
    ) -> PyResult<Py<PyBytes>> {
        let direction_index = get_dir_arg(dir)?;
        let image_idx = match dir_to_dmi_index(&direction_index) {
            Some(idx) => (idx + 1) * frame as usize - 1,
            None => return Err(PyRuntimeError::new_err("invalid dir")),
        };
        let frame_data = self.images.get(image_idx).unwrap();
        let rgba8_data = frame_data.to_rgba8();
        let rgba8_vec: Vec<u8> = rgba8_data.into_raw();
        Ok(PyBytes::new(py, &rgba8_vec).into())
    }

    fn __str__(&self) -> PyResult<String> {
        self.__repr__()
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!(
            "<IconState '{}' dirs={} frames={}>",
            self.name, self.dir_count, self.frames
        ))
    }
}

fn get_dir_arg(dir: &Bound<PyAny>) -> Result<dmi::dirs::Dirs, PyErr> {
    return if let Ok(diridx) = dir.extract::<Dir>() {
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
    } else if let Ok(dirint) = dir.cast::<PyInt>() {
        match dirint.extract::<u8>().unwrap() {
            1 => Ok(dmi::dirs::Dirs::NORTH),
            2 => Ok(dmi::dirs::Dirs::SOUTH),
            4 => Ok(dmi::dirs::Dirs::EAST),
            8 => Ok(dmi::dirs::Dirs::WEST),
            5 => Ok(dmi::dirs::Dirs::NORTHEAST),
            9 => Ok(dmi::dirs::Dirs::NORTHWEST),
            6 => Ok(dmi::dirs::Dirs::SOUTHEAST),
            10 => Ok(dmi::dirs::Dirs::SOUTHWEST),
            _ => Err(PyRuntimeError::new_err("invalid direction")),
        }
    } else {
        Err(PyRuntimeError::new_err("invalid direction"))
    };
}
