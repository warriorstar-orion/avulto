use dreammaker::Location;
use pyo3::{pyclass, pymethods, IntoPyObject, Py, PyObject, PyResult, Python};

use crate::{dmlist::DmList, path::Path};

use super::{
    nodes::{NodeKind, OriginalSourceLocation, PyExpr},
    operators::{AssignOperator, BinaryOperator, UnaryOperator},
    prefab::Prefab,
};

#[pyclass(frozen)]
#[derive(Clone)]
pub enum Constant {
    Null(),
    Int(i32),
    Float(f32),
    String(String),
    Resource(String),
    Path(Path),
    ProcMacro(),
}

#[pymethods]
impl Constant {
    #[getter]
    pub fn get_val(&self, py: Python<'_>) -> PyResult<PyObject> {
        Ok(match self {
            Constant::Null() => py.None(),
            Constant::Int(i) => i.into_pyobject(py)?.into_any().unbind(),
            Constant::Float(f) => f.into_pyobject(py)?.into_any().unbind(),
            Constant::String(s) => s.into_pyobject(py)?.into_any().unbind(),
            Constant::Resource(s) => s.into_pyobject(py)?.into_any().unbind(),
            Constant::Path(p) => p.clone().into_pyobject(py)?.into_any().unbind(),
            Constant::ProcMacro() => Constant::ProcMacro().into_pyobject(py)?.into_any().unbind(),
        })
    }
}

#[pyclass]
// #[derive(Clone)]
pub enum Expression {
    Constant {
        constant: Constant,
        source_loc: Option<Py<OriginalSourceLocation>>,
    },
    Identifier {
        name: String,
        source_loc: Option<Py<OriginalSourceLocation>>,
    },
    List {
        list: Py<DmList>,
        source_loc: Option<Py<OriginalSourceLocation>>,
    },
    BinaryOp {
        op: BinaryOperator,
        lhs: PyExpr,
        rhs: PyExpr,
        source_loc: Option<Py<OriginalSourceLocation>>,
    },
    AssignOp {
        op: AssignOperator,
        lhs: PyExpr,
        rhs: PyExpr,
        source_loc: Option<Py<OriginalSourceLocation>>,
    },
    TernaryOp {
        cond: PyExpr,
        if_expr: PyExpr,
        else_expr: PyExpr,
        source_loc: Option<Py<OriginalSourceLocation>>,
    },
    InterpString {
        first: Constant,
        token_pairs: Vec<(Option<PyExpr>, Py<Constant>)>,
        source_loc: Option<Py<OriginalSourceLocation>>,
    },
    Locate {
        args: Vec<PyExpr>,
        in_list: Option<PyExpr>,
        source_loc: Option<Py<OriginalSourceLocation>>,
    },
    Prefab {
        prefab: Py<Prefab>,
        source_loc: Option<Py<OriginalSourceLocation>>,
    },
    Index {
        expr: PyExpr,
        index: PyExpr,
        source_loc: Option<Py<OriginalSourceLocation>>,
    },
    Field {
        expr: Option<PyExpr>,
        field: PyExpr,
        source_loc: Option<Py<OriginalSourceLocation>>,
    },
    StaticField {
        expr: PyExpr,
        field: PyExpr,
        source_loc: Option<Py<OriginalSourceLocation>>,
    },
    Call {
        expr: PyExpr,
        name: PyExpr,
        args: Vec<PyExpr>,
        source_loc: Option<Py<OriginalSourceLocation>>,
    },
    SelfCall {
        args: Vec<PyExpr>,
        source_loc: Option<Py<OriginalSourceLocation>>,
    },
    ParentCall {
        args: Vec<PyExpr>,
        source_loc: Option<Py<OriginalSourceLocation>>,
    },
    UnaryOp {
        expr: PyExpr,
        unary_op: UnaryOperator,
        source_loc: Option<Py<OriginalSourceLocation>>,
    },
    ProcReference {
        expr: PyExpr,
        name: PyExpr,
        source_loc: Option<Py<OriginalSourceLocation>>,
    },
    ExternalCall {
        library_name: PyExpr,
        function_name: PyExpr,
        args: Vec<PyExpr>,
        source_loc: Option<Py<OriginalSourceLocation>>,
    },
    NewMiniExpr {
        name: PyExpr,
        fields: Vec<PyExpr>,
        source_loc: Option<Py<OriginalSourceLocation>>,
    },
    NewImplicit {
        args: Option<Vec<PyExpr>>,
        source_loc: Option<Py<OriginalSourceLocation>>,
    },
    NewPrefab {
        prefab: Py<Prefab>,
        args: Option<Vec<PyExpr>>,
        source_loc: Option<Py<OriginalSourceLocation>>,
    },
    DynamicCall {
        lib_name: Vec<PyExpr>,
        proc_name: Vec<PyExpr>,
        source_loc: Option<Py<OriginalSourceLocation>>,
    },
    Input {
        args: Vec<PyExpr>,
        input_type: Option<u32>,
        in_list: Option<PyExpr>,
        source_loc: Option<Py<OriginalSourceLocation>>,
    },
    Pick {
        args: Vec<(Option<PyExpr>, PyExpr)>,
        source_loc: Option<Py<OriginalSourceLocation>>,
    },
}

impl Expression {
    pub fn null(loc: Option<Location>, py: Python<'_>) -> Py<Self> {
        Expression::Constant {
            constant: Constant::Null(),
            source_loc: loc.map(|l| OriginalSourceLocation::from_location(&l)),
        }
        .into_pyobject(py)
        .expect("bad null")
        .into()
    }

    pub fn ident(ident: String, loc: Option<Location>, py: Python<'_>) -> Py<Self> {
        Expression::Identifier {
            name: ident,
            source_loc: loc.map(|l| OriginalSourceLocation::from_location(&l)),
        }
        .into_pyobject(py)
        .expect("bad identifier")
        .into()
    }

    pub fn string(string: String, loc: Option<Location>, py: Python<'_>) -> Py<Self> {
        Expression::Constant {
            constant: Constant::String(string),
            source_loc: loc.map(|l| OriginalSourceLocation::from_location(&l)),
        }
        .into_pyobject(py)
        .expect("bad string constant")
        .into()
    }
}

#[pymethods]
impl Expression {
    #[getter]
    fn get_kind(&self) -> NodeKind {
        match self {
            Expression::Constant { .. } => NodeKind::Constant,
            Expression::Identifier { .. } => NodeKind::Identifier,
            Expression::List { .. } => NodeKind::List,
            Expression::BinaryOp { .. } => NodeKind::BinaryOp,
            Expression::AssignOp { .. } => NodeKind::AssignOp,
            Expression::TernaryOp { .. } => NodeKind::TernaryOp,
            Expression::InterpString { .. } => NodeKind::InterpString,
            Expression::Locate { .. } => NodeKind::Locate,
            Expression::Prefab { .. } => NodeKind::Prefab,
            Expression::Index { .. } => NodeKind::Index,
            Expression::Field { .. } => NodeKind::Field,
            Expression::StaticField { .. } => NodeKind::StaticField,
            Expression::Call { .. } => NodeKind::Call,
            Expression::SelfCall { .. } => NodeKind::SelfCall,
            Expression::ParentCall { .. } => NodeKind::ParentCall,
            Expression::UnaryOp { .. } => NodeKind::UnaryOp,
            Expression::ProcReference { .. } => NodeKind::ProcReference,
            Expression::ExternalCall { .. } => NodeKind::ExternalCall,
            Expression::NewMiniExpr { .. } => NodeKind::NewMiniExpr,
            Expression::NewImplicit { .. } => NodeKind::NewImplicit,
            Expression::NewPrefab { .. } => NodeKind::NewPrefab,
            Expression::DynamicCall { .. } => NodeKind::DynamicCall,
            Expression::Input { .. } => NodeKind::Input,
            Expression::Pick { .. } => NodeKind::Pick,
        }
    }

    fn __str__(&self, py: Python<'_>) -> PyResult<String> {
        match self {
            Expression::Identifier { name, .. } => Ok(name.clone()),
            _ => self.__repr__(py),
        }
    }

    fn __repr__(&self, py: Python<'_>) -> PyResult<String> {
        match self {
            Expression::Constant { constant, .. } => {
                Ok(format!("<Constant {}>", constant.get_val(py)?))
            }
            Expression::Identifier { name, .. } => Ok(format!("<Identifier {}>", name)),
            Expression::List { list, .. } => Ok(format!("{}", list)),
            Expression::BinaryOp { op, lhs, rhs, .. } => {
                Ok(format!("<BinaryOp {} {:?} {}>", lhs, op, rhs))
            }
            Expression::AssignOp { op, lhs, rhs, .. } => {
                Ok(format!("<AssignOp {} {:?} {}>", lhs, op, rhs))
            }
            Expression::TernaryOp { cond, .. } => Ok(format!("<TernaryOp {} ...>", cond)),
            Expression::InterpString { .. } => Ok("<InterpString ...>".to_string()),
            Expression::Locate { .. } => Ok("<Locate ...>".to_string()),
            Expression::Prefab { prefab, .. } => Ok(format!("<Prefab {}>", prefab)),
            Expression::Index { expr, index, .. } => Ok(format!("<Index {}[{}]>", expr, index)),
            Expression::Field { expr: _, field, .. } => Ok(format!("<Field {}>", field)),
            Expression::StaticField { expr, field, .. } => {
                Ok(format!("<StaticField {}::{}>", expr, field))
            }
            Expression::Call { expr, name, .. } => Ok(format!("<Call {}.{}(...)>", expr, name)),
            Expression::SelfCall { .. } => Ok("<SelfCall ...>".to_string()),
            Expression::ParentCall { .. } => Ok("<ParentCall ...>".to_string()),
            Expression::UnaryOp { expr, unary_op, .. } => {
                Ok(format!("<UnaryOp {:?} {}>", unary_op, expr))
            }
            Expression::ProcReference { expr, name, .. } => {
                Ok(format!("<ProcReference {}.{}>", expr, name))
            }
            Expression::ExternalCall {
                library_name,
                function_name,
                ..
            } => Ok(format!(
                "<ExternalCall {},{}(...)>",
                library_name, function_name
            )),
            Expression::NewMiniExpr { name, .. } => Ok(format!("<NewMiniExpr {} ...>", name)),
            Expression::NewImplicit { .. } => Ok("<NewImplicit ...>".to_string()),
            Expression::NewPrefab { prefab, .. } => Ok(format!("<NewPrefab {} ...>", prefab)),
            Expression::DynamicCall {
                lib_name,
                proc_name,
                ..
            } => Ok(format!("<DynamicCall ({:?})({:?})>", lib_name, proc_name)),
            Expression::Input { input_type, .. } => Ok(format!("<Input {:?} ...>", input_type)),
            Expression::Pick { .. } => Ok("<Pick ...>".to_string()),
        }
    }
}
