use core::fmt;
use std::hash::Hash;

use pyo3::{
    Bound, Py, PyAny, PyResult, Python, pyclass, pymethods, pymodule,
    types::{PyAnyMethods, PyList, PyModule, PyModuleMethods},
};
use pyo3_stub_gen::derive::{
    gen_stub_pyclass, gen_stub_pyclass_complex_enum, gen_stub_pyclass_enum, gen_stub_pymethods,
};

use crate::{
    dme::{
        SourceLoc,
        operators::{AssignOperator, BinaryOperator, UnaryOperator},
        prefab::Prefab,
    },
    path::Path,
};

use super::{
    Dme,
    expression::{Constant, Expression},
    operators::SettingMode,
};

extern crate dreammaker;

pub type PyCodeBlock = Vec<Py<Node>>;
pub type PyExpr = Py<Expression>;

#[pymodule]
pub fn ast(_py: Python, m: &Bound<PyModule>) -> PyResult<()> {
    m.add_class::<UnaryOperator>()?;
    m.add_class::<AssignOperator>()?;
    m.add_class::<SettingMode>()?;
    m.add_class::<BinaryOperator>()?;

    m.add_class::<Expression>()?;
    m.add_class::<Node>()?;
    m.add_class::<NodeKind>()?;
    m.add_class::<Prefab>()?;
    Ok(())
}

#[gen_stub_pyclass_enum]
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
    ForKeyValue,
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

#[gen_stub_pyclass_complex_enum]
#[pyclass(frozen, module = "avulto.ast")]
pub enum Node {
    Unknown(),
    Expression {
        expr: PyExpr,
        source_loc: SourceLoc,
    },
    Crash {
        expr: Option<PyExpr>,
        source_loc: SourceLoc,
    },
    Return {
        retval: Option<PyExpr>,
        source_loc: SourceLoc,
    },
    Throw {
        expr: PyExpr,
        source_loc: SourceLoc,
    },
    Del {
        expr: PyExpr,
        source_loc: SourceLoc,
    },
    Break {
        label: Option<PyExpr>,
        source_loc: SourceLoc,
    },
    While {
        condition: PyExpr,
        block: PyCodeBlock,
        source_loc: SourceLoc,
    },
    DoWhile {
        condition: PyExpr,
        block: PyCodeBlock,
        source_loc: SourceLoc,
    },
    If {
        if_arms: Vec<(PyExpr, PyCodeBlock)>,
        else_arm: Option<PyCodeBlock>,
        source_loc: SourceLoc,
    },
    ForInfinite {
        block: PyCodeBlock,
        source_loc: SourceLoc,
    },
    ForList {
        var_type: Option<Path>,
        name: PyExpr,
        in_list: Option<PyExpr>,
        block: PyCodeBlock,
        source_loc: SourceLoc,
    },
    ForLoop {
        init: Option<Py<Node>>,
        test: Option<PyExpr>,
        inc: Option<Py<Node>>,
        block: PyCodeBlock,
        source_loc: SourceLoc,
    },
    ForRange {
        name: PyExpr,
        start: PyExpr,
        end: PyExpr,
        step: Option<PyExpr>,
        block: PyCodeBlock,
        source_loc: SourceLoc,
    },
    Var {
        name: PyExpr,
        value: Option<PyExpr>,
        declared_type: Option<Path>,
        source_loc: SourceLoc,
    },
    Vars {
        vars: Vec<Py<Node>>,
        source_loc: SourceLoc,
    },
    Setting {
        name: PyExpr,
        mode: SettingMode,
        value: PyExpr,
        source_loc: SourceLoc,
    },
    Spawn {
        delay: Option<PyExpr>,
        block: PyCodeBlock,
        source_loc: SourceLoc,
    },
    Continue {
        name: Option<PyExpr>,
        source_loc: SourceLoc,
    },
    Goto {
        label: PyExpr,
        source_loc: SourceLoc,
    },
    Label {
        name: PyExpr,
        block: PyCodeBlock,
        source_loc: SourceLoc,
    },
    TryCatch {
        try_block: PyCodeBlock,
        catch_params: Vec<Vec<PyExpr>>,
        catch_block: PyCodeBlock,
        source_loc: SourceLoc,
    },
    Switch {
        input: PyExpr,
        cases: Vec<Py<SwitchCase>>,
        default: Option<PyCodeBlock>,
        source_loc: SourceLoc,
    },
    ForKeyValue {
        var_type: Option<Path>,
        key: PyExpr,
        value: PyExpr,
        in_list: Option<PyExpr>,
        block: PyCodeBlock,
        source_loc: SourceLoc,
    },
}

pub fn visit_constant(constant: &Constant, walker: &Bound<PyAny>) -> PyResult<()> {
    if walker.hasattr("visit_Constant").unwrap() {
        walker.call_method1("visit_Constant", (constant.clone(),))?;
    }

    Ok(())
}

#[gen_stub_pymethods]
#[pymethods]
impl Node {
    #[getter]
    fn get_kind(&self, py: Python<'_>) -> NodeKind {
        match self {
            Node::Unknown() => NodeKind::Unknown,
            Node::Expression { expr, .. } => expr.bind(py).get().get_kind(),
            Node::Crash { .. } => NodeKind::Crash,
            Node::Return { .. } => NodeKind::Return,
            Node::Throw { .. } => NodeKind::Throw,
            Node::Del { .. } => NodeKind::Del,
            Node::Break { .. } => NodeKind::Break,
            Node::While { .. } => NodeKind::While,
            Node::DoWhile { .. } => NodeKind::DoWhile,
            Node::If { .. } => NodeKind::If,
            Node::ForInfinite { .. } => NodeKind::ForInfinite,
            Node::ForList { .. } => NodeKind::ForList,
            Node::ForLoop { .. } => NodeKind::ForLoop,
            Node::ForRange { .. } => NodeKind::ForRange,
            Node::Var { .. } => NodeKind::Var,
            Node::Vars { .. } => NodeKind::Vars,
            Node::Setting { .. } => NodeKind::Setting,
            Node::Spawn { .. } => NodeKind::Spawn,
            Node::Continue { .. } => NodeKind::Continue,
            Node::Goto { .. } => NodeKind::Goto,
            Node::Label { .. } => NodeKind::Label,
            Node::TryCatch { .. } => NodeKind::TryCatch,
            Node::Switch { .. } => NodeKind::Switch,
            Node::ForKeyValue { .. } => NodeKind::ForKeyValue,
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
            Node::ForKeyValue { .. } => Ok("<ForKeyValue ...>".to_string()),
        }
    }
}

#[gen_stub_pyclass]
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
            Expression::walk(&f.cast_into::<Expression>().unwrap(), walker, py)?;
        }
        for f in self.range.bind(py).into_iter() {
            if let Ok(list) = f.cast::<PyList>() {
                list.try_iter()?.for_each(|x| {
                    if let Ok(range) = x {
                        let _ = Expression::walk(
                            &range.into_any().cast_into::<Expression>().unwrap(),
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
