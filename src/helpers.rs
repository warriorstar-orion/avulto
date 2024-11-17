use std::borrow::Borrow;

use dreammaker::constants::{ConstFn, Constant};
use pyo3::{
    exceptions::PyRuntimeError,
    pyclass, pyfunction, pymethods,
    types::{PyAnyMethods, PyBool, PyFloat, PyInt, PyString},
    Bound, IntoPyObject, Py, PyAny, PyObject, PyResult, Python, ToPyObject,
};

use dmm_tools::dmi::Dir as SDir;

use crate::{
    dme::{expression::Expression, nodes::PyExpr},
    dmlist::DmList,
    path::Path,
};

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

pub fn python_value_to_constant(val: &Bound<PyAny>) -> Option<Constant> {
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

fn args_list_to_listexpr(l: &[(Constant, Option<Constant>)]) -> Expression {
    Python::with_gil(|py| {
        let mut keys: Vec<PyExpr> = vec![];
        let mut vals: Vec<PyExpr> = vec![];

        for (key, value) in l.iter() {
            keys.push(
                Py::new(py, constant_to_ast_expression(key)).expect("constant to ast: list key"),
            );
            vals.push(
                Py::new(
                    py,
                    constant_to_ast_expression(&value.as_ref().unwrap_or(&Constant::Null(None))),
                )
                .expect("constant to ast: list value"),
            );
        }

        return Expression::List {
            source_loc: None,
            list: Py::new(
                py,
                DmList {
                    keys: keys.iter().map(|k| k.clone_ref(py).into_any()).collect(),
                    vals: vals.iter().map(|k| k.clone_ref(py).into_any()).collect(),
                },
            )
            .expect("constant to ast: list construction"),
        };
    })
}

fn constant_to_ast_expression(c: &dreammaker::constants::Constant) -> Expression {
    Python::with_gil(|py| match c {
        Constant::Null(_) => Expression::Constant {
            constant: crate::dme::expression::Constant::Null(),
            source_loc: None,
        },
        Constant::New { .. } => todo!("no constant_to_ast_expression for Constant::New"),
        Constant::List(l) => args_list_to_listexpr(l),
        Constant::Call(const_fn, args) => Expression::Call {
            source_loc: None,
            expr: Expression::null(None, py),
            name: match const_fn {
                ConstFn::Icon => Expression::ident("icon".to_string(), None, py),
                ConstFn::Matrix => Expression::ident("matrix".to_string(), None, py),
                ConstFn::Newlist => Expression::ident("list".to_string(), None, py),
                ConstFn::Sound => Expression::ident("sound".to_string(), None, py),
                ConstFn::Filter => Expression::ident("filter".to_string(), None, py),
                ConstFn::File => Expression::ident("file".to_string(), None, py),
                ConstFn::Generator => Expression::ident("generator".to_string(), None, py),
            },
            args: args
                .iter()
                .map(|(k, v)| {
                    Py::new(
                        py,
                        Expression::List {
                            source_loc: None,
                            list: Py::new(
                                py,
                                DmList {
                                    keys: vec![Py::new(py, constant_to_ast_expression(k))
                                        .expect("const call arg key")
                                        .into_any()],
                                    vals: vec![Py::new(
                                        py,
                                        constant_to_ast_expression(
                                            v.as_ref().unwrap_or(&Constant::Null(None)),
                                        ),
                                    )
                                    .expect("const call arg val")
                                    .into_any()],
                                },
                            )
                            .expect("const call arg list"),
                        },
                    )
                    .expect("const call arg wrapper")
                })
                .collect(),
        },
        Constant::Prefab(_) => todo!("no constant_to_ast_expression for Constant::Prefab"),
        Constant::String(ident2) => Expression::Constant {
            source_loc: None,
            constant: crate::dme::expression::Constant::String(ident2.to_string()),
        },
        Constant::Resource(ident2) => Expression::Identifier {
            source_loc: None,
            name: ident2.to_string(),
        },
        Constant::Float(f) => Expression::Constant {
            source_loc: None,
            constant: crate::dme::expression::Constant::Float(*f),
        },
    })
}

pub fn constant_to_python_value(c: &dreammaker::constants::Constant) -> PyObject {
    Python::with_gil(|py| match c {
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
        Constant::Prefab(p) => Path::from_tree_path(&p.path)
            .into_pyobject(py)
            .expect("constant to prefab")
            .into_any()
            .unbind(),
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
