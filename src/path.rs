use std::{
    collections::hash_map::DefaultHasher,
    fmt,
    hash::{Hash, Hasher},
};

use pyo3::{
    exceptions::{PyRuntimeError, PyValueError}, pyclass::CompareOp, pymethods, types::{PyAnyMethods, PyString, PyStringMethods}, Bound, PyAny, PyErr, PyResult
};
use pyo3::pyclass;

#[derive(Clone, Eq, Hash, PartialOrd, Ord, PartialEq)]
#[pyclass(module = "avulto")]
pub struct Path(pub String);

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Path> for String {
    fn from(val: Path) -> Self {
        val.0
    }
}

impl Path {
    pub fn internal_child_of_string(&self, rhs: &String, strict: bool) -> bool {
        if self.0.eq(rhs) {
            return !strict;
        }
        if rhs == "/" {
            return true;
        }
        let parts: Vec<&str> = self.0.split('/').collect();
        let oparts: Vec<&str> = rhs.split('/').collect();
        if parts.len() < oparts.len() {
            return false;
        }
        for (a, b) in parts.iter().zip(oparts) {
            if !(*a).eq(b) {
                return false;
            }
        }

        return true;
    }

    pub fn internal_parent_of_string(&self, rhs: &String, strict: bool) -> bool {
        if self.0.eq(rhs){
            return !strict;
        }
        if self.0 == "/" {
            return true;
        }
        let parts: Vec<&str> = self.0.split('/').collect();
        let oparts: Vec<&str> = rhs.split('/').collect();
        if parts.len() > oparts.len() {
            return false;
        }
        for (a, b) in parts.iter().zip(oparts) {
            if !(*a).eq(b) {
                return false;
            }
        }

        return true;
    }
}

#[pymethods]
impl Path {
    #[new]
    pub fn new(value: &str) -> PyResult<Self> {
        if !&value.starts_with('/') {
            return Err(PyErr::new::<PyValueError, &str>("not a valid path"));
        }
        Ok(Path(value.to_string()))
    }

    #[pyo3(signature = (other, strict=false))]
    fn child_of(&self, other: &Bound<PyAny>, strict: bool) -> PyResult<bool> {
        if let Ok(rhs) = other.extract::<Self>() {
            return Ok(self.internal_child_of_string(&rhs.0, strict));
        } else if let Ok(rhs) = other.downcast::<PyString>() {
            return Ok(self.internal_child_of_string(&rhs.to_cow().unwrap().to_string(), strict));
        }

        Err(PyErr::new::<PyRuntimeError, &str>("not a valid path"))
    }

    #[pyo3(signature = (other, strict=false))]
    fn parent_of(&self, other: &Bound<PyAny>, strict: bool) -> PyResult<bool> {
        if let Ok(rhs) = other.extract::<Self>() {
            return Ok(self.internal_parent_of_string(&rhs.0, strict));
        } else if let Ok(rhs) = other.downcast::<PyString>() {
            return Ok(self.internal_parent_of_string(&rhs.to_cow().unwrap().to_string(), strict));
        }

        Err(PyErr::new::<PyRuntimeError, &str>("not a valid path"))
    }

    #[getter]
    fn get_parent(&self) -> PyResult<Self> {
        if self.0 == "/" {
            return Ok(self.clone());
        }
        let mut parts: Vec<&str> = self.0.split('/').collect();
        let _ = parts.split_off(parts.len() - 1);
        let parent = parts.join("/");
        Ok(Path(parent))
    }

    #[getter]
    fn get_stem(&self) -> PyResult<String> {
        let parts: Vec<&str> = self.0.split('/').collect();
        if let Some(last) = parts.last() {
            let l = *last;
            return Ok(l.to_string());
        }

        Ok("".to_string())
    }

    fn __hash__(&self) -> PyResult<isize> {
        let mut s = DefaultHasher::new();
        self.0.hash(&mut s);
        let result = s.finish() as isize;

        Ok(result)
    }

    fn __str__(&self) -> PyResult<String> {
        self.__repr__()
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(self.0.to_string())
    }

    fn __richcmp__(&self, other: &Bound<PyAny>, op: CompareOp) -> PyResult<bool> {
        if let Ok(rhs) = other.extract::<Self>() {
            return match op {
                CompareOp::Eq => Ok(self.0 == rhs.0),
                CompareOp::Ne => Ok(self.0 != rhs.0),
                CompareOp::Lt => Ok(self.0 < rhs.0),
                CompareOp::Gt => Ok(self.0 > rhs.0),
                CompareOp::Le => Ok(self.0 <= rhs.0),
                CompareOp::Ge => Ok(self.0 >= rhs.0),
            };
        } else if let Ok(rhs) = other.downcast::<PyString>() {
            return match op {
                CompareOp::Eq => Ok(self.0 == rhs.to_string()),
                CompareOp::Ne => Ok(self.0 != rhs.to_string()),
                CompareOp::Lt => Ok(self.0 < rhs.to_string()),
                CompareOp::Gt => Ok(self.0 > rhs.to_string()),
                CompareOp::Le => Ok(self.0 <= rhs.to_string()),
                CompareOp::Ge => Ok(self.0 >= rhs.to_string()),
            };
        }

        Ok(false)
    }

    fn __truediv__(&self, other: &Bound<PyAny>) -> PyResult<Self> {
        if let Ok(rhs) = other.extract::<Self>() {
            return Ok(Path(self.0.clone() + "/" + &rhs.0));
        } else if let Ok(rhs) = other.downcast::<PyString>() {
            return Ok(Path(
                self.0.clone()
                    + "/"
                    + rhs
                        .to_string()
                        .strip_prefix('/')
                        .unwrap_or(rhs.to_string().as_str()),
            ));
        }

        Err(PyRuntimeError::new_err("cannot append"))
    }
}
