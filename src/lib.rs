use dmlist::DmList;
use pyo3::{prelude::*, types::PyDict, wrap_pymodule};

pub mod dme;
pub mod dmi;
pub mod dmlist;
pub mod dmm;
pub mod helpers;
pub mod path;
pub mod tile;
pub mod typedecl;

#[pymodule]
fn avulto(_py: Python, m: &Bound<PyModule>) -> PyResult<()> {
    m.add_class::<path::Path>()?;

    m.add_class::<dmm::Dmm>()?;
    m.add_class::<dmm::CoordIterator>()?;
    m.add_class::<tile::Tile>()?;

    m.add_class::<dmi::Dmi>()?;
    m.add_class::<dmi::Rect>()?;
    m.add_class::<dmi::IconState>()?;
    m.add_class::<dmi::StateIter>()?;

    m.add_class::<dme::Dme>()?;
    m.add_class::<DmList>()?;

    m.add_class::<helpers::Dir>()?;
    m.add_function(wrap_pyfunction!(helpers::as_dir, m)?)?;

    let pth_submodule = PyModule::new_bound(_py, "paths")?;
    pth_submodule.add("Root", path::Path::new("/").unwrap())?;
    pth_submodule.add("Area", path::Path::new("/area").unwrap())?;
    pth_submodule.add("Turf", path::Path::new("/turf").unwrap())?;
    pth_submodule.add("Obj", path::Path::new("/obj").unwrap())?;
    pth_submodule.add("Mob", path::Path::new("/mob").unwrap())?;
    pth_submodule.add("Datum", path::Path::new("/datum").unwrap())?;

    m.add_submodule(&pth_submodule)?;
    
    m.add_wrapped(wrap_pymodule!(dme::nodes::ast))?;
    let sys = PyModule::import_bound(_py, "sys")?;
    let sys_modules: Bound<'_, PyDict> = sys.getattr("modules")?.downcast_into()?;
    sys_modules.set_item("avulto.ast", m.getattr("ast")?)?;    

    Ok(())
}
