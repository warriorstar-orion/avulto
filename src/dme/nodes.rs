use core::fmt;
use std::hash::Hash;

use dreammaker::{FileId, Location};
use pyo3::{
    pyclass, pymethods, pymodule,
    types::{PyAnyMethods, PyList, PyModule, PyModuleMethods},
    Bound, IntoPyObject, Py, PyAny, PyObject, PyResult, Python,
};

use crate::path::Path;

use super::{
    expression::{Constant, Expression},
    operators::SettingMode,
    Dme,
};

extern crate dreammaker;

pub type PyCodeBlock = Vec<Py<Node>>;
pub type PyExpr = Py<Expression>;

#[pymodule]
pub fn ast(_py: Python, m: &Bound<PyModule>) -> PyResult<()> {
    m.add_class::<Expression>()?;
    m.add_class::<Node>()?;
    m.add_class::<NodeKind>()?;
    Ok(())
}

#[pyclass(
    module = "avulto.ast",
    name = "NodeKind",
    eq,
    eq_int,
    rename_all = "SCREAMING_SNAKE_CASE"
)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum NodeKind {
    AssignOp,
    Attribute,
    BinaryOp,
    Break,
    Call,
    Constant,
    Continue,
    Crash,
    Del,
    DoWhile,
    DynamicCall,
    Expression,
    ExternalCall,
    Field,
    ForInfinite,
    ForList,
    ForLoop,
    ForRange,
    Goto,
    Identifier,
    If,
    IfArm,
    IfElse,
    Index,
    Input,
    InterpString,
    Label,
    List,
    Locate,
    MiniExpr,
    NewImplicit,
    NewMiniExpr,
    NewPrefab,
    ParentCall,
    Pick,
    Prefab,
    ProcReference,
    Return,
    SelfCall,
    Setting,
    Spawn,
    StaticField,
    Switch,
    SwitchCase,
    Term,
    TernaryOp,
    Throw,
    TryCatch,
    UnaryOp,
    Unknown,
    Var,
    Vars,
    While,
}

impl fmt::Display for NodeKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[pyclass(frozen)]
#[derive(Clone)]
pub struct OriginalSourceLocation {
    /// The index into the file table.
    pub file: FileId,
    /// The line number, starting at 1.
    #[pyo3(get)]
    pub line: u32,
    /// The column number, starting at 1.
    #[pyo3(get)]
    pub column: u16,
}

impl OriginalSourceLocation {
    pub fn from_location(location: &Location) -> Py<Self> {
        Python::with_gil(|py| {
            OriginalSourceLocation {
                file: location.file,
                line: location.line,
                column: location.column,
            }
            .into_pyobject(py)
            .unwrap()
            .unbind()
        })
    }
}

#[pyclass(frozen)]
pub enum Node {
    Unknown(),
    Expression {
        expr: PyExpr,
        source_loc: Option<Py<OriginalSourceLocation>>,
    },
    Crash {
        expr: Option<PyExpr>,
        source_loc: Option<Py<OriginalSourceLocation>>,
    },
    Return {
        retval: Option<PyExpr>,
        source_loc: Option<Py<OriginalSourceLocation>>,
    },
    Throw {
        expr: PyExpr,
        source_loc: Option<Py<OriginalSourceLocation>>,
    },
    Del {
        expr: PyExpr,
        source_loc: Option<Py<OriginalSourceLocation>>,
    },
    Break {
        label: Option<PyExpr>,
        source_loc: Option<Py<OriginalSourceLocation>>,
    },
    While {
        condition: PyExpr,
        block: PyCodeBlock,
        source_loc: Option<Py<OriginalSourceLocation>>,
    },
    DoWhile {
        condition: PyExpr,
        block: PyCodeBlock,
        source_loc: Option<Py<OriginalSourceLocation>>,
    },
    If {
        if_arms: Vec<(PyExpr, PyCodeBlock)>,
        else_arm: Option<PyCodeBlock>,
        source_loc: Option<Py<OriginalSourceLocation>>,
    },
    ForInfinite {
        block: PyCodeBlock,
        source_loc: Option<Py<OriginalSourceLocation>>,
    },
    ForList {
        var_type: Option<Path>,
        name: PyExpr,
        in_list: Option<PyExpr>,
        block: PyCodeBlock,
        source_loc: Option<Py<OriginalSourceLocation>>,
    },
    ForLoop {
        init: Option<Py<Node>>,
        test: Option<PyExpr>,
        inc: Option<Py<Node>>,
        block: PyCodeBlock,
        source_loc: Option<Py<OriginalSourceLocation>>,
    },
    ForRange {
        name: PyExpr,
        start: PyExpr,
        end: PyExpr,
        step: Option<PyExpr>,
        block: PyCodeBlock,
        source_loc: Option<Py<OriginalSourceLocation>>,
    },
    Var {
        name: PyExpr,
        value: Option<PyExpr>,
        declared_type: Option<Path>,
        source_loc: Option<Py<OriginalSourceLocation>>,
    },
    Vars {
        vars: Vec<Py<Node>>,
        source_loc: Option<Py<OriginalSourceLocation>>,
    },
    Setting {
        name: PyExpr,
        mode: SettingMode,
        value: PyExpr,
        source_loc: Option<Py<OriginalSourceLocation>>,
    },
    Spawn {
        delay: Option<PyExpr>,
        block: PyCodeBlock,
        source_loc: Option<Py<OriginalSourceLocation>>,
    },
    Continue {
        name: Option<PyExpr>,
        source_loc: Option<Py<OriginalSourceLocation>>,
    },
    Goto {
        label: PyExpr,
        source_loc: Option<Py<OriginalSourceLocation>>,
    },
    Label {
        name: PyExpr,
        block: PyCodeBlock,
        source_loc: Option<Py<OriginalSourceLocation>>,
    },
    TryCatch {
        try_block: PyCodeBlock,
        catch_params: Vec<Vec<PyExpr>>,
        catch_block: PyCodeBlock,
        source_loc: Option<Py<OriginalSourceLocation>>,
    },
    Switch {
        input: PyExpr,
        cases: Vec<Py<SwitchCase>>,
        default: Option<PyCodeBlock>,
        source_loc: Option<Py<OriginalSourceLocation>>,
    },
}

pub fn visit_constant(constant: &Constant, walker: &Bound<PyAny>) -> PyResult<()> {
    if walker.hasattr("visit_Constant").unwrap() {
        walker.call_method1("visit_Constant", (constant.clone(),))?;
    }

    Ok(())
}

#[pymethods]
impl Node {
    #[getter]
    fn get_kind(&self, py: Python<'_>) -> PyResult<PyObject> {
        match self {
            Node::Unknown() => Ok(Py::new(py, NodeKind::Unknown).unwrap().into_any()),
            Node::Expression { expr, .. } => expr.call_method0(py, "kind"),
            Node::Crash { .. } => Ok(Py::new(py, NodeKind::Crash).unwrap().into_any()),
            Node::Return { .. } => Ok(Py::new(py, NodeKind::Return).unwrap().into_any()),
            Node::Throw { .. } => Ok(Py::new(py, NodeKind::Throw).unwrap().into_any()),
            Node::Del { .. } => Ok(Py::new(py, NodeKind::Del).unwrap().into_any()),
            Node::Break { .. } => Ok(Py::new(py, NodeKind::Break).unwrap().into_any()),
            Node::While { .. } => Ok(Py::new(py, NodeKind::While).unwrap().into_any()),
            Node::DoWhile { .. } => Ok(Py::new(py, NodeKind::DoWhile).unwrap().into_any()),
            Node::If { .. } => Ok(Py::new(py, NodeKind::If).unwrap().into_any()),
            Node::ForInfinite { .. } => Ok(Py::new(py, NodeKind::ForInfinite).unwrap().into_any()),
            Node::ForList { .. } => Ok(Py::new(py, NodeKind::ForList).unwrap().into_any()),
            Node::ForLoop { .. } => Ok(Py::new(py, NodeKind::ForLoop).unwrap().into_any()),
            Node::ForRange { .. } => Ok(Py::new(py, NodeKind::ForRange).unwrap().into_any()),
            Node::Var { .. } => Ok(Py::new(py, NodeKind::Var).unwrap().into_any()),
            Node::Vars { .. } => Ok(Py::new(py, NodeKind::Vars).unwrap().into_any()),
            Node::Setting { .. } => Ok(Py::new(py, NodeKind::Setting).unwrap().into_any()),
            Node::Spawn { .. } => Ok(Py::new(py, NodeKind::Spawn).unwrap().into_any()),
            Node::Continue { .. } => Ok(Py::new(py, NodeKind::Continue).unwrap().into_any()),
            Node::Goto { .. } => Ok(Py::new(py, NodeKind::Goto).unwrap().into_any()),
            Node::Label { .. } => Ok(Py::new(py, NodeKind::Label).unwrap().into_any()),
            Node::TryCatch { .. } => Ok(Py::new(py, NodeKind::TryCatch).unwrap().into_any()),
            Node::Switch { .. } => Ok(Py::new(py, NodeKind::Switch).unwrap().into_any()),
        }
    }

    fn __str__(&self, py: Python<'_>) -> PyResult<String> {
        self.__repr__(py)
    }

    fn __repr__(&self, py: Python<'_>) -> PyResult<String> {
        match self {
            Node::Unknown() => todo!(),
            Node::Expression { expr, .. } => Ok(format!("{}", expr)),
            Node::Crash { expr, .. } => Ok(format!("<Crash {:?}>", expr)),
            Node::Return { retval, .. } => Ok(format!(
                "<Return {}>",
                retval
                    .as_ref()
                    .map_or(py.None(), |f| f.clone_ref(py).into_any())
            )),
            Node::Throw { expr, .. } => Ok(format!("<Throw {:?}>", expr)),
            Node::Del { expr, .. } => Ok(format!("<Del {:?}>", expr)),
            Node::Break { label, .. } => Ok(format!("<Break {:?}>", label)),
            Node::While { condition, .. } => Ok(format!("<While {} ...>", condition)),
            Node::DoWhile { condition, .. } => Ok(format!("<DoWhile {} ...>", condition)),
            Node::If { .. } => Ok("<If ...>".to_string()),
            Node::ForInfinite { .. } => Ok("<ForInfinite ...>".to_string()),
            Node::ForList { .. } => Ok("<ForList ...>".to_string()),
            Node::ForLoop { .. } => Ok("<ForLoop ...>".to_string()),
            Node::ForRange { .. } => Ok("<ForRange ...>".to_string()),
            Node::Var {
                name,
                declared_type,
                ..
            } => {
                if let Some(path) = declared_type {
                    return Ok(format!("<Var var{}/{} ...>", path.rel, name));
                }
                Ok(format!("<Var var/{} ...>", name))
            }
            Node::Vars { .. } => Ok("<Vars ...>".to_string()),
            Node::Setting { name, .. } => Ok(format!("<Setting {} ...>", name)),
            Node::Spawn { .. } => Ok("<Spawn ...>".to_string()),
            Node::Continue { name, .. } => Ok(format!("<Continue {:?}>", name)),
            Node::Goto { label, .. } => Ok(format!("<Goto {}>", label)),
            Node::Label { name, .. } => Ok(format!("<Label {} ...>", name)),
            Node::TryCatch { .. } => Ok("<TryCatch ...>".to_string()),
            Node::Switch { input, .. } => Ok(format!("<Switch {} ...>", input)),
        }
    }
}

#[pyclass(module = "avulto.ast")]
pub struct SwitchCase {
    #[pyo3(get)]
    pub(crate) exact: Py<PyList>,
    #[pyo3(get)]
    pub(crate) range: Py<PyList>,
    #[pyo3(get)]
    pub(crate) block: PyCodeBlock,
}

impl SwitchCase {
    pub fn walk_parts(
        &self,
        dme: &Bound<Dme>,
        walker: &Bound<PyAny>,
        py: Python<'_>,
    ) -> PyResult<()> {
        for f in self.exact.bind(py).into_iter() {
            Expression::walk(&f.downcast_into::<Expression>().unwrap(), dme, walker, py)?;
        }
        for f in self.range.bind(py).into_iter() {
            if let Ok(list) = f.downcast::<PyList>() {
                list.try_iter()?.for_each(|x| {
                    if let Ok(range) = x {
                        let _ = Expression::walk(
                            &range.into_any().downcast_into::<Expression>().unwrap(),
                            dme,
                            walker,
                            py,
                        );
                    }
                });
            }
        }

        for stmt in self.block.iter() {
            Node::walk(stmt.bind(py), dme, walker, py)?;
        }

        Ok(())
    }
}
