#[macro_use]
extern crate lazy_static;

use dme::{EmptyProcError, MissingProcError, MissingTypeError};
use dmi::IconError;
use dmlist::{DmList, NewCall};
use path::PathError;
use pyo3::{prelude::*, types::PyDict, wrap_pymodule};

pub mod dme;
pub mod dmi;
pub mod dmlist;
pub mod dmm;
pub mod helpers;
pub mod path;
pub mod tile;

use pyo3_stub_gen::{define_stub_info_gatherer, derive::gen_stub_pyfunction};

use crate::dme::{SourceLoc, proc_decl::ProcDecl, type_decl::TypeDecl, var_decl::VarDecl};

#[gen_stub_pyfunction]
#[pyfunction]
fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[pymodule]
fn avulto(_py: Python, m: &Bound<PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(version, m)?)?;

    m.add_class::<path::Path>()?;

    m.add_class::<dmm::Dmm>()?;
    m.add_class::<dmm::CoordIterator>()?;
    m.add_class::<tile::Tile>()?;

    m.add_class::<dmi::Dmi>()?;
    m.add_class::<dmi::iconstate::IconState>()?;
    m.add_class::<dmi::StateIter>()?;

    m.add_class::<dme::Dme>()?;
    m.add_class::<ProcDecl>()?;
    m.add_class::<TypeDecl>()?;
    m.add_class::<VarDecl>()?;
    m.add_class::<DmList>()?;
    m.add_class::<NewCall>()?;
    m.add_class::<SourceLoc>()?;

    m.add_class::<helpers::Dir>()?;
    m.add_function(wrap_pyfunction!(helpers::as_dir, m)?)?;

    m.add_wrapped(wrap_pymodule!(dme::nodes::ast))?;
    let sys = PyModule::import(_py, "sys")?;
    let sys_modules: Bound<'_, PyDict> = sys.getattr("modules")?.cast_into()?;
    sys_modules.set_item("avulto.ast", m.getattr("ast")?)?;

    let err_submodule = PyModule::new(_py, "exceptions")?;

    err_submodule.add("EmptyProcError", _py.get_type::<EmptyProcError>())?;
    err_submodule.add("MissingTypeError", _py.get_type::<MissingTypeError>())?;
    err_submodule.add("MissingProcError", _py.get_type::<MissingProcError>())?;
    err_submodule.add("IconError", _py.get_type::<IconError>())?;
    err_submodule.add("PathError", _py.get_type::<PathError>())?;

    m.add_submodule(&err_submodule)?;
    sys_modules.set_item("avulto.exceptions", m.getattr("exceptions")?)?;

    Ok(())
}

define_stub_info_gatherer!(stub_info);
