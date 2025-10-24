use std::borrow::Borrow;

use dreammaker::constants::{Constant, Pop};
use pyo3::{
    exceptions::PyRuntimeError,
    pyclass, pyfunction, pymethods,
    types::{PyAnyMethods, PyBool, PyDict, PyFloat, PyInt, PyList, PyString},
    Bound, IntoPyObject, IntoPyObjectExt, Py, PyAny, PyResult, Python,
};

use dmm_tools::dmi::Dir as SDir;

use crate::{dme::prefab::Prefab, dmlist::DmList, path::Path};

#[pyclass(eq, eq_int, ord)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum Dir {
    #[pyo3(name = "NORTH")]
    North = 1,
    #[pyo3(name = "SOUTH")]
    South = 2,
    #[pyo3(name = "EAST")]
    East = 4,
    #[pyo3(name = "WEST")]
    West = 8,
    #[pyo3(name = "NORTHEAST")]
    Northeast = 5,
    #[pyo3(name = "NORTHWEST")]
    Northwest = 9,
    #[pyo3(name = "SOUTHEAST")]
    Southeast = 6,
    #[pyo3(name = "SOUTHWEST")]
    Southwest = 10,
}

#[pymethods]
impl Dir {
    fn __hash__(&self) -> PyResult<u32> {
        Ok(*self as u32)
    }
}

impl From<i32> for Dir {
    fn from(i: i32) -> Self {
        // TODO(wso): There has to be a less dumb way
        match i {
            1 => Dir::North,
            2 => Dir::South,
            4 => Dir::East,
            8 => Dir::West,
            5 => Dir::Northeast,
            9 => Dir::Northwest,
            6 => Dir::Southeast,
            10 => Dir::Southwest,
            _ => panic!("bad dir {}", i),
        }
    }
}

pub fn to_dmm_dir(d: Dir) -> SDir {
    match d {
        Dir::North => SDir::North,
        Dir::South => SDir::South,
        Dir::East => SDir::East,
        Dir::West => SDir::West,
        Dir::Northeast => SDir::Northeast,
        Dir::Northwest => SDir::Northwest,
        Dir::Southeast => SDir::Southeast,
        Dir::Southwest => SDir::Southwest,
    }
}

#[pyfunction]
pub fn as_dir(c: i32) -> PyResult<Dir> {
    match c {
        1 => Ok(Dir::North),
        2 => Ok(Dir::South),
        4 => Ok(Dir::East),
        8 => Ok(Dir::West),
        5 => Ok(Dir::Northeast),
        9 => Ok(Dir::Northwest),
        6 => Ok(Dir::Southeast),
        10 => Ok(Dir::Southwest),
        _ => Err(PyRuntimeError::new_err(format!("no dir {}", c))),
    }
}

pub fn python_value_to_constant(val: &Bound<PyAny>) -> Option<Constant> {
    Python::attach(|py| {
        if val.is_instance_of::<PyBool>() {
            let val = val.extract::<bool>().unwrap();
            Some(Constant::Float(if val { 1.0 } else { 0.0 }))
        } else if let Ok(int) = val.cast::<PyInt>() {
            Some(Constant::Float(
                int.extract::<f32>().expect("could not cast float"),
            ))
        } else if let Ok(float) = val.cast::<PyFloat>() {
            Some(Constant::Float(
                float.extract::<f32>().expect("could not cast float"),
            ))
        } else if let Ok(pystr) = val.cast::<PyString>() {
            Some(Constant::String(pystr.to_string().into()))
        } else if let Ok(pydmlist) = val.cast::<DmList>() {
            let mut r: Vec<(Constant, Option<Constant>)> = vec![];
            let borrowed = pydmlist.borrow();
            for (idx, key) in borrowed.keys.iter().enumerate() {
                r.push((
                    python_value_to_constant(key.bind(py)).unwrap(),
                    python_value_to_constant(borrowed.vals[idx].bind(py)),
                ));
            }
            let boxed_slice = r.into_boxed_slice();
            Some(Constant::List(boxed_slice))
        } else if let Ok(pypth) = val.cast::<Path>() {
            Some(Constant::Prefab(Box::new(Pop {
                path: pypth.borrow().to_tree_path(),
                vars: Default::default(),
            })))
        } else if val.is_none() {
            Some(Constant::Null(None))
        } else {
            None
        }
    })
}

pub fn constant_to_python_value(c: &dreammaker::constants::Constant) -> Py<PyAny> {
    Python::attach(|py| match c {
        Constant::Null(_) => py.None(),
        Constant::New { .. } => py.None(),
        Constant::List(l) => {
            let mut keys: Vec<Py<PyAny>> = vec![];
            let mut vals: Vec<Py<PyAny>> = vec![];

            for args in l.iter() {
                keys.push(constant_to_python_value(&args.0).clone_ref(py));
                vals.push(
                    constant_to_python_value(
                        &args
                            .1
                            .borrow()
                            .clone()
                            .unwrap_or(dreammaker::constants::Constant::Null(Option::None)),
                    )
                    .clone_ref(py),
                );
            }

            Py::new(py, DmList { keys, vals })
                .expect("constant to dmlist")
                .into_any()
        }
        // TODO: How the fuck do I represent these in plain old Python
        Constant::Call(_, _) => py.None(),
        Constant::Prefab(p) => {
            if p.vars.is_empty() {
                Path::from_tree_path(&p.path)
                    .into_pyobject(py)
                    .expect("constant to prefab")
                    .into_any()
                    .unbind()
            } else {
                let mut out: Vec<Bound<PyDict>> = Vec::new();

                for (k, v) in p.vars.iter() {
                    let var = PyDict::new(py);
                    var.set_item(k.as_str(), constant_to_python_value(v))
                        .expect("setting prefab var item");
                    out.push(var);
                }
                Prefab {
                    path: Path::from_tree_path(&p.path),
                    vars: PyList::new(py, out)
                        .expect("building prefab vars list")
                        .into_any()
                        .unbind(),
                }
                .into_pyobject(py)
                .expect("prefab to pyobject")
                .into_any()
                .unbind()
            }
        }
        Constant::String(s) => s.into_py_any(py).unwrap(),
        Constant::Resource(s) => s.into_py_any(py).unwrap(),
        Constant::Float(f) => {
            if f.fract() == 0.0 {
                (*f as i32).into_py_any(py).unwrap()
            } else {
                f.into_py_any(py).unwrap()
            }
        }
    })
}
