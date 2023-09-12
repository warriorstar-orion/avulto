use std::borrow::Borrow;

use dreammaker::constants::Constant;
use pyo3::{
    exceptions::PyRuntimeError,
    pyclass, pyfunction,
    types::{PyDict, PyList},
    PyObject, PyResult, Python, ToPyObject,
};

use dmm_tools::dmi::Dir as SDir;

#[pyclass]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Dir {
    North = 1,
    South = 2,
    East = 4,
    West = 8,
    Northeast = 5,
    Northwest = 9,
    Southeast = 6,
    Southwest = 10,
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

pub fn constant_to_python_value(c: &dreammaker::constants::Constant) -> PyObject {
    // println!("constant_to_python_value c={}", c);
    Python::with_gil(|py| match c {
        Constant::Null(_) => py.None(),
        Constant::New { type_: _, args: _ } => todo!(),
        Constant::List(l) => {
            let mut out: Vec<&PyDict> = Vec::new();

            for args in l.iter() {
                // println!(
                //     "constant_to_python_value l args={}=>{}",
                //     args.0,
                //     args.1
                //         .as_ref()
                //         .unwrap_or(&dreammaker::constants::Constant::Null(Option::None))
                // );
                let var = PyDict::new(py);
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

            PyList::new(py, out).to_object(py)
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
