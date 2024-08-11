use pyo3::prelude::*;

pub mod dme;
pub mod dmi;
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

    m.add_class::<helpers::Dir>()?;
    m.add_function(wrap_pyfunction!(helpers::as_dir, m)?)?;

    let submodule = PyModule::new_bound(_py, "paths")?;
    submodule.add("Root", path::Path::new("/").unwrap())?;
    submodule.add("Area", path::Path::new("/area").unwrap())?;
    submodule.add("Turf", path::Path::new("/turf").unwrap())?;
    submodule.add("Obj", path::Path::new("/obj").unwrap())?;
    submodule.add("Mob", path::Path::new("/mob").unwrap())?;
    submodule.add("Datum", path::Path::new("/datum").unwrap())?;

    m.add_submodule(&submodule)?;

    Ok(())
}
