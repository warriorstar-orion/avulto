use std::borrow::Borrow;

use dreammaker::constants::Constant;
use pyo3::{
    exceptions::PyRuntimeError, pyclass, pyfunction, pymethods, types::{PyAnyMethods, PyBool, PyDict, PyFloat, PyInt, PyList, PyString}, Bound, PyAny, PyObject, PyResult, Python, ToPyObject
};

use dmm_tools::dmi::Dir as SDir;

#[pyclass(eq, eq_int)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
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

pub fn python_value_to_constant(val: &Bound<PyAny>) -> Option<dreammaker::constants::Constant> {
    if val.is_instance_of::<PyBool>() {
        let val = val.extract::<bool>().unwrap();
        Some(Constant::Float(if val { 1.0 } else { 0.0 }))
    } else if let Ok(int) = val.downcast::<PyInt>() {
        Some(Constant::Float(
            int.extract::<f32>().expect("could not cast float"),
        ))
    } else if let Ok(float) = val.downcast::<PyFloat>() {
        Some(Constant::Float(
            float.extract::<f32>().expect("could not cast float"),
        ))
    } else if let Ok(pystr) = val.downcast::<PyString>() {
        Some(Constant::String(pystr.to_string().into()))
    } else if val.is_none() {
        Some(Constant::Null(None))
    } else {
        None
    }
}

pub fn constant_to_python_value(c: &dreammaker::constants::Constant) -> PyObject {
    Python::with_gil(|py| match c {
        Constant::Null(_) => py.None(),
        Constant::New { type_: _, args: _ } => todo!(),
        Constant::List(l) => {
            let mut out: Vec<Bound<PyDict>> = Vec::new();

            for args in l.iter() {
                let var = PyDict::new_bound(py);
                var.set_item(
                    constant_to_python_value(&args.0),
                    constant_to_python_value(
                        &args
                            .1
                            .borrow()
                            .clone()
                            .unwrap_or(dreammaker::constants::Constant::Null(Option::None)),
                    ),
                );
                out.push(var);
            }

            PyList::new_bound(py, out).to_object(py)
        }
        Constant::Call(_, _) => todo!(),
        Constant::Prefab(p) => p.to_string().to_object(py),
        Constant::String(s) => s.to_object(py),
        Constant::Resource(s) => s.to_object(py),
        Constant::Float(f) => {
            if f.fract() == 0.0 {
                (*f as i32).to_object(py)
            } else {
                f.to_object(py)
            }
        }
    })
}
