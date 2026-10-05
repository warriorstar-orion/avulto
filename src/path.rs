use std::fmt;

use dreammaker::ast::TreePath;
use itertools::Itertools;
use pyo3::{
    Bound, PyAny, PyResult, pymethods,
    types::{PyAnyMethods, PyString, PyStringMethods},
};
use pyo3::{
    IntoPyObject, Python,
    exceptions::{PyException, PyTypeError},
    pyclass,
};
use pyo3_stub_gen::{
    create_exception,
    derive::{gen_stub_pyclass, gen_stub_pymethods},
};
use regex::Regex;

create_exception!(avulto.exceptions, PathError, PyException);

const OBJ_PREFIX: &[&str] = &["datum", "atom", "movable", "obj"];
const MOB_PREFIX: &[&str] = &["datum", "atom", "movable", "mob"];
const AREA_PREFIX: &[&str] = &["datum", "atom", "area"];
const TURF_PREFIX: &[&str] = &["datum", "atom", "turf"];
const ATOM_PREFIX: &[&str] = &["datum", "atom"];

const CORE_TYPES: &[&str] = &[
    "datum",
    "image",
    "mutable_appearance",
    "sound",
    "icon",
    "matrix",
    "database",
    "exception",
    "regex",
    "dm_filter",
    "generator",
    "particles",
];

/// A DM typepath.
#[gen_stub_pyclass]
#[pyclass(module = "avulto")]
#[derive(Clone, Eq, Hash, PartialOrd, Ord, PartialEq)]
pub struct Path {
    // We can either do a bunch of munging when displaying paths, which happens a lot,
    // or do a bunch of munging when operating on paths, which happens a lot,
    // or we can just keep both around, because memory is cheap.
    /// Returns the *absolute* representation of the path, rooted at `/datum`.
    #[pyo3(get)]
    pub abs: String,
    /// Returns the *relative* or *declared* representation of the path.
    #[pyo3(get)]
    pub rel: String,
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.abs)
    }
}

impl From<Path> for String {
    fn from(val: Path) -> Self {
        val.abs
    }
}

lazy_static! {
    static ref VALID_PATH_PART_RE: Regex = Regex::new(r"^[A-Za-z_][A-Za-z0-9_]*$").unwrap();
}

fn invalid_path_part(part: &&str) -> bool {
    VALID_PATH_PART_RE.captures(part).is_none()
}

impl Path {
    pub fn root() -> Path {
        Path {
            abs: String::from("/"),
            rel: String::from("/"),
        }
    }

    /// Used where we know the path is coming from a source guaranteed
    /// to emit valid paths, such as DMEs themselves.
    pub fn make_trusted(value: &str) -> Path {
        let rel = to_relative_path(value);
        let abs = to_absolute_path(rel.as_str());
        Path { abs, rel }
    }

    pub fn from_tree_path(tree_path: &TreePath) -> Self {
        Self::make_trusted(
            ("/".to_string() + tree_path.iter().map(|f| f.as_str()).join("/").as_str()).as_str(),
        )
    }

    pub fn to_tree_path(&self) -> TreePath {
        let parts = self.rel.split('/').map(String::from).collect();
        parts
    }

    pub fn make_untrusted(value: &str) -> Result<Path, String> {
        let trimmed = value.trim();
        if trimmed.eq("/") {
            return Ok(Path::root());
        }
        if trimmed.is_empty() {
            return Err(String::from("empty path"));
        }
        if !trimmed.starts_with('/') {
            return Err(String::from("invalid path"));
        }
        let parts: Vec<&str> = trimmed.split('/').collect();
        if parts.iter().filter(|&x| x.is_empty()).count() > 1 {
            return Err(String::from("path contains empty parts"));
        }
        if parts.iter().filter(|&x| invalid_path_part(x)).count() > 1 {
            return Err(String::from("path contains invalid parts"));
        }
        let rel = to_relative_path(trimmed);
        let abs = to_absolute_path(rel.as_str());
        Ok(Path { abs, rel })
    }

    pub fn internal_child_of_string(&self, rhs: &String, strict: bool) -> bool {
        if self.abs.eq(rhs) {
            return !strict;
        }
        if rhs == "/" {
            return true;
        }
        let parts: Vec<&str> = self.abs.split('/').collect();
        let rhs_abs = to_absolute_path(rhs);
        let oparts: Vec<&str> = rhs_abs.split('/').collect();
        if parts.len() < oparts.len() {
            return false;
        }
        for (a, b) in parts.iter().zip(oparts) {
            if !(*a).eq(b) {
                return false;
            }
        }

        true
    }

    pub fn internal_parent_of_string(&self, rhs: &String, strict: bool) -> bool {
        if self.abs.eq(rhs) {
            return !strict;
        }
        if self.abs == "/" {
            return true;
        }
        let parts: Vec<&str> = self.abs.split('/').collect();
        let rhs_abs = to_absolute_path(rhs);
        let oparts: Vec<&str> = rhs_abs.split('/').collect();
        if parts.len() > oparts.len() {
            return false;
        }
        for (a, b) in parts.iter().zip(oparts) {
            if !(*a).eq(b) {
                return false;
            }
        }

        true
    }

    /// Performs a comparison for either another path or a string.
    fn cmp_other(&self, other: &Bound<PyAny>, op: impl Fn(&str, &str) -> bool) -> bool {
        if let Ok(rhs) = other.extract::<Self>() {
            op(&self.abs, &rhs.abs)
        } else if let Ok(rhs) = other.cast::<PyString>() {
            op(&self.abs, &to_absolute_path(rhs.to_str().unwrap()))
        } else {
            false
        }
    }
}

fn to_relative_path(value: &str) -> String {
    let parts: Vec<&str> = value.split('/').filter(|&x| !x.is_empty()).collect();

    for prefix in [
        OBJ_PREFIX,
        MOB_PREFIX,
        AREA_PREFIX,
        TURF_PREFIX,
        ATOM_PREFIX,
    ] {
        if parts
            .iter()
            .zip(prefix.iter())
            .filter(|&(a, b)| a == b)
            .count()
            == prefix.len()
        {
            return format!("/{}", parts[prefix.len() - 1..].join("/"));
        }
    }

    String::from(value)
}

fn to_absolute_path(value: &str) -> String {
    let parts: Vec<&str> = value.split('/').filter(|&x| !x.is_empty()).collect();
    if parts.is_empty() {
        "/".to_string()
    } else if parts[0].eq("area") {
        return "/datum/atom/".to_string() + &parts.join("/");
    } else if parts[0].eq("atom") {
        return "/datum/".to_string() + &parts.join("/");
    } else if parts[0].eq("mob") {
        return "/datum/atom/movable/".to_string() + &parts.join("/");
    } else if parts[0].eq("turf") {
        return "/datum/atom/".to_string() + &parts.join("/");
    } else if parts[0].eq("obj") {
        return "/datum/atom/movable/".to_string() + &parts.join("/");
    } else if CORE_TYPES.contains(&parts[0]) {
        return "/".to_string() + &parts.join("/");
    } else {
        return "/datum/".to_string() + &parts.join("/");
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl Path {
    /// Returns a new path.
    #[new]
    pub fn new(value: &str) -> PyResult<Self> {
        match Path::make_untrusted(value) {
            Ok(p) => Ok(p),
            Err(e) => Err(PathError::new_err(e)),
        }
    }

    /// Returns whether the path is a child of `other`.
    ///
    /// If `strict` is true, the current path will not be considered a child of
    /// itself.
    #[pyo3(signature = (other, strict=false))]
    fn child_of(&self, other: &Bound<PyAny>, strict: bool) -> PyResult<bool> {
        if let Ok(rhs) = other.extract::<Self>() {
            return Ok(self.internal_child_of_string(&rhs.abs, strict));
        } else if let Ok(rhs) = other.cast::<PyString>() {
            return Ok(self.internal_child_of_string(&rhs.to_cow().unwrap().to_string(), strict));
        }

        Err(PyTypeError::new_err("invalid argument type"))
    }

    /// Returns whether the path is a parent of `other`.
    ///
    /// If `strict` is true, the current path will not be considered a parent of
    /// itself.
    #[pyo3(signature = (other, strict=false))]
    fn parent_of(&self, other: &Bound<PyAny>, strict: bool) -> PyResult<bool> {
        if let Ok(rhs) = other.extract::<Self>() {
            return Ok(self.internal_parent_of_string(&rhs.abs, strict));
        } else if let Ok(rhs) = other.cast::<PyString>() {
            return Ok(self.internal_parent_of_string(&rhs.to_cow().unwrap().to_string(), strict));
        }

        Err(PyTypeError::new_err("invalid argument type"))
    }

    /// The parent path.
    #[getter]
    pub fn get_parent(&self) -> PyResult<Path> {
        if self.abs == "/" {
            return Ok(self.clone());
        }
        let mut parts: Vec<&str> = self.abs.split('/').filter(|&x| !x.is_empty()).collect();
        let _ = parts.split_off(parts.len() - 1);
        if parts.is_empty() {
            Ok(Path {
                abs: String::from("/"),
                rel: String::from("/"),
            })
        } else {
            let mut parent = parts.join("/");
            parent.insert(0, '/');
            Path::new(parent.as_str())
        }
    }

    /// The final part of the path.
    #[getter]
    fn get_stem(&self) -> PyResult<String> {
        let parts: Vec<&str> = self.abs.split('/').collect();
        if let Some(last) = parts.last() {
            let l = *last;
            return Ok(l.to_string());
        }

        Ok("".to_string())
    }

    /// Whether or not the path is `/`.
    #[getter]
    fn get_is_root(&self) -> bool {
        self.abs == "/"
    }

    fn __hash__(&self, py: Python<'_>) -> PyResult<isize> {
        // TODO: Still not sure how to handle "/obj/foo" in {p("/obj/foo")} and
        // p("/obj/foo") in {"/obj/foo"} if there's a mismatch between if the
        // string is absolute or relative
        self.abs
            .clone()
            .into_pyobject(py)?
            .into_any()
            .call_method0("__hash__")
            .unwrap()
            .extract::<isize>()
    }

    fn __str__(&self) -> PyResult<String> {
        self.__repr__()
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(self.rel.clone())
    }

    fn __eq__(&self, other: &Bound<PyAny>) -> bool {
        self.cmp_other(other, |lhs, rhs| lhs == rhs)
    }

    fn __ne__(&self, other: &Bound<PyAny>) -> bool {
        self.cmp_other(other, |lhs, rhs| lhs != rhs)
    }

    fn __lt__(&self, other: &Bound<PyAny>) -> bool {
        self.cmp_other(other, |lhs, rhs| lhs < rhs)
    }

    fn __le__(&self, other: &Bound<PyAny>) -> bool {
        self.cmp_other(other, |lhs, rhs| lhs <= rhs)
    }

    fn __gt__(&self, other: &Bound<PyAny>) -> bool {
        self.cmp_other(other, |lhs, rhs| lhs > rhs)
    }

    fn __ge__(&self, other: &Bound<PyAny>) -> bool {
        self.cmp_other(other, |lhs, rhs| lhs >= rhs)
    }

    /// Return the path with the specified suffix.
    fn __truediv__(&self, other: &Bound<PyAny>) -> PyResult<Self> {
        if let Ok(rhs) = other.extract::<Self>() {
            let new_path = self.abs.clone() + &rhs.rel;
            return Path::new(new_path.as_str());
        } else if let Ok(rhs) = other.cast::<PyString>() {
            let new_path = if self.get_is_root() {
                String::from("")
            } else {
                self.abs.clone()
            } + "/"
                + rhs
                    .to_string()
                    .strip_prefix('/')
                    .unwrap_or(rhs.to_string().as_str());
            return Path::new(new_path.as_str());
        }

        Err(PathError::new_err("cannot append"))
    }
}
