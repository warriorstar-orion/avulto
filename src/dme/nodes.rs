use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use dreammaker::ast::AssignOp;
use pyo3::{
    pyclass, pymethods, pymodule,
    types::{PyAnyMethods, PyDict, PyDictMethods, PyList, PyModule, PyModuleMethods, PyString},
    Bound, IntoPy, Py, PyAny, PyClassInitializer, PyObject, PyResult, Python, ToPyObject,
};

use crate::path::Path;

extern crate dreammaker;

#[pymodule]
pub fn ast(_py: Python, m: &Bound<PyModule>) -> PyResult<()> {
    m.add_class::<Node>()?;
    m.add_class::<NodeKind>()?;
    m.add_class::<Prefab>()?;
    m.add_class::<Attribute>()?;
    m.add_class::<Identifier>()?;
    m.add_class::<Call>()?;
    Ok(())
}

#[pyclass(module = "avulto.ast", name = "NodeKind", eq, eq_int)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum NodeKind {
    #[pyo3(name = "UNKNOWN")]
    Unknown,
    #[pyo3(name = "EXPRESSION")]
    Expression,
    #[pyo3(name = "VAR")]
    Var,
    #[pyo3(name = "TERM")]
    Term,
    #[pyo3(name = "ASSIGN")]
    Assign,
    #[pyo3(name = "PREFAB")]
    Prefab,
    #[pyo3(name = "RETURN")]
    Return,
    #[pyo3(name = "IF")]
    If,
    #[pyo3(name = "IF_ARM")]
    IfArm,
    #[pyo3(name = "IF_ELSE")]
    IfElse,
    #[pyo3(name = "ATTRIBUTE")]
    Attribute,
    #[pyo3(name = "TERNARY")]
    Ternary,
    #[pyo3(name = "BINARY_OP")]
    BinaryOp,
    #[pyo3(name = "UNARY_OP")]
    UnaryOp,
    #[pyo3(name = "INDEX")]
    Index,
    #[pyo3(name = "CALL")]
    Call,
    #[pyo3(name = "CRASH")]
    Crash,
    #[pyo3(name = "PARENT_CALL")]
    ParentCall,
    #[pyo3(name = "SELF_CALL")]
    SelfCall,
    #[pyo3(name = "FOR_LOOP")]
    ForLoop,
    #[pyo3(name = "FOR_LIST")]
    ForList,
    #[pyo3(name = "FOR_RANGE")]
    ForRange,
    #[pyo3(name = "NEW_PREFAB")]
    NewPrefab,
    #[pyo3(name = "SETTING")]
    Setting,
    #[pyo3(name = "LABEL")]
    Label,
    #[pyo3(name = "NEW_IMPLICIT")]
    NewImplicit,
    #[pyo3(name = "INTERP_STRING")]
    InterpString,
    #[pyo3(name = "SWITCH_CASE")]
    SwitchCase,
    #[pyo3(name = "SWITCH")]
    Switch,
    #[pyo3(name = "SPAWN")]
    Spawn,
    #[pyo3(name = "WHILE")]
    While,
    #[pyo3(name = "DO_WHILE")]
    DoWhile,
    #[pyo3(name = "PICK")]
    Pick,
    #[pyo3(name = "BREAK")]
    Break,
    #[pyo3(name = "LOCATE")]
    Locate,
    #[pyo3(name = "CONTINUE")]
    Continue,
    #[pyo3(name = "TRY_CATCH")]
    TryCatch,
    #[pyo3(name = "INPUT")]
    Input,
    #[pyo3(name = "NEW_MINI_EXPR")]
    NewMiniExpr,
    #[pyo3(name = "MINI_EXPR")]
    MiniExpr,
    #[pyo3(name = "DYNAMIC_CALL")]
    DynamicCall,
    #[pyo3(name = "EXTERNAL_CALL")]
    ExternalCall,
    #[pyo3(name = "DEL")]
    Del,
    #[pyo3(name = "THROW")]
    Throw,
}

#[pyclass(module = "avulto.ast", name = "Operator", eq, eq_int)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Operator {
    #[pyo3(name = "ASSIGN")]
    Assign,
    #[pyo3(name = "ASSIGN_ADD")]
    AssignAdd,
    #[pyo3(name = "ASSIGN_SUB")]
    AssignSub,
    #[pyo3(name = "ASSIGN_MUL")]
    AssignMul,
    #[pyo3(name = "ASSIGN_DIV")]
    AssignDiv,
    #[pyo3(name = "ASSIGN_MOD")]
    AssignMod,
    #[pyo3(name = "ASSIGN_FLOAT_MOD")]
    AssignFloatMod,
    #[pyo3(name = "ASSIGN_INTO")]
    AssignInto,
    #[pyo3(name = "ASSIGN_BIT_AND")]
    AssignBitAnd,
    #[pyo3(name = "ASSIGN_AND")]
    AssignAnd,
    #[pyo3(name = "ASSIGN_OR")]
    AssignOr,
    #[pyo3(name = "ASSIGN_BIT_OR")]
    AssignBitOr,
    #[pyo3(name = "ASSIGN_BIT_XOR")]
    AssignBitXor,
    #[pyo3(name = "ASSIGN_LSHIFT")]
    AssignLShift,
    #[pyo3(name = "ASSIGN_RSHIFT")]
    AssignRShift,
}

#[pyclass(module = "avulto.ast", name = "SettingMode", eq, eq_int)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum SettingMode {
    #[pyo3(name = "ASSIGN")]
    Assign,
    #[pyo3(name = "IN")]
    In,
}

#[pyclass(module = "avulto.ast", name = "BinaryOperator", eq, eq_int)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum BinaryOperator {
    #[pyo3(name = "ADD")]
    Add,
    #[pyo3(name = "SUB")]
    Sub,
    #[pyo3(name = "MUL")]
    Mul,
    #[pyo3(name = "DIV")]
    Div,
    #[pyo3(name = "POW")]
    Pow,
    #[pyo3(name = "MOD")]
    Mod,
    #[pyo3(name = "FLOAT_MOD")]
    FloatMod,
    #[pyo3(name = "EQ")]
    Eq,
    #[pyo3(name = "NOT_EQ")]
    NotEq,
    #[pyo3(name = "LESS")]
    Less,
    #[pyo3(name = "GREATER")]
    Greater,
    #[pyo3(name = "LESS_EQ")]
    LessEq,
    #[pyo3(name = "GREATER_EQ")]
    GreaterEq,
    #[pyo3(name = "EQUIV")]
    Equiv,
    #[pyo3(name = "NOT_EQUIV")]
    NotEquiv,
    #[pyo3(name = "BIT_AND")]
    BitAnd,
    #[pyo3(name = "BIT_XOR")]
    BitXor,
    #[pyo3(name = "BIT_OR")]
    BitOr,
    #[pyo3(name = "LSHIFT")]
    LShift,
    #[pyo3(name = "RSHIFT")]
    RShift,
    #[pyo3(name = "AND")]
    And,
    #[pyo3(name = "OR")]
    Or,
    #[pyo3(name = "IN")]
    In,
    #[pyo3(name = "TO")]
    To,
}

#[pyclass(subclass, module = "avulto.ast")]
pub struct Node {
    #[pyo3(get)]
    kind: NodeKind,
}

#[pymethods]
impl Node {
    #[new]
    fn new(node_type: NodeKind) -> Self {
        Node { kind: node_type }
    }
}

#[pyclass(module = "avulto.ast")]
pub struct Identifier {
    pub ident: Py<PyAny>,
}

#[pymethods]
impl Identifier {
    fn __str__(&self) -> PyResult<String> {
        Ok(format!("{}", self.ident))
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("(identifier) {}", self.ident))
    }
}

#[pyclass(extends = Node, module = "avulto.ast")]
pub struct Expression {}

#[pymethods]
impl Expression {
    #[new]
    fn new() -> (Self, Node) {
        (Expression {}, Node::new(NodeKind::Expression))
    }
}

#[pyclass(extends = Node, module = "avulto.ast")]
pub struct Var {
    #[pyo3(get)]
    name: Py<PyAny>,
    #[pyo3(get)]
    value: Py<PyAny>,
}

impl Var {
    pub fn make(py: Python<'_>, name: String, value: Py<PyAny>) -> PyResult<PyObject> {
        let base = PyClassInitializer::from(Node::new(NodeKind::Var));
        let sub = base.add_subclass(Var {
            name: name.into_py(py),
            value,
        });

        Ok(Py::new(py, sub)?.to_object(py))
    }
}

#[pyclass(extends = Node, module = "avulto.ast")]
pub struct Assignment {
    #[pyo3(get)]
    lhs: Py<PyAny>,
    #[pyo3(get)]
    rhs: Py<PyAny>,
    #[pyo3(get)]
    op: Operator,
}

impl Assignment {
    pub fn make(
        py: Python<'_>,
        lhs: Py<PyAny>,
        rhs: Py<PyAny>,
        op: &AssignOp,
    ) -> PyResult<PyObject> {
        let base = PyClassInitializer::from(Node::new(NodeKind::Assign));
        let sub = base.add_subclass(Assignment {
            lhs,
            rhs,
            op: match op {
                AssignOp::Assign => Operator::Assign,
                AssignOp::AddAssign => Operator::AssignAdd,
                AssignOp::SubAssign => Operator::AssignSub,
                AssignOp::MulAssign => Operator::AssignMul,
                AssignOp::DivAssign => Operator::AssignDiv,
                AssignOp::ModAssign => Operator::AssignMod,
                AssignOp::FloatModAssign => Operator::AssignFloatMod,
                AssignOp::AssignInto => Operator::AssignInto,
                AssignOp::BitAndAssign => Operator::AssignBitAnd,
                AssignOp::AndAssign => Operator::AssignAnd,
                AssignOp::BitOrAssign => Operator::AssignBitOr,
                AssignOp::OrAssign => Operator::AssignOr,
                AssignOp::BitXorAssign => Operator::AssignBitXor,
                AssignOp::LShiftAssign => Operator::AssignLShift,
                AssignOp::RShiftAssign => Operator::AssignRShift,
            },
        });

        Ok(Py::new(py, sub)?.to_object(py))
    }
}

#[pyclass(extends = Node, module = "avulto.ast")]
pub struct Prefab {
    #[pyo3(get)]
    path: Py<PyAny>,
    #[pyo3(get)]
    vars: Py<PyAny>,
}

impl Prefab {
    pub fn make(py: Python<'_>, path: Py<PyAny>, vars: Py<PyAny>) -> PyResult<PyObject> {
        let base = PyClassInitializer::from(Node::new(NodeKind::Prefab));
        let sub = base.add_subclass(Prefab { path, vars });
        Ok(Py::new(py, sub)?.to_object(py))
    }

    pub fn vars_to_string(&self, py: Python<'_>) -> String {
        if let Ok(vardict) = self.vars.downcast_bound::<PyDict>(py) {
            if vardict.is_empty() {
                return "".to_string();
            }
            let mut out = String::new();

            for k in vardict.items() {
                if let Ok(kl) = k.downcast::<PyList>() {
                    out.push_str(
                        format!("{} = {}", kl.get_item(0).unwrap(), kl.get_item(1).unwrap())
                            .as_str(),
                    );
                }
            }

            return out.to_string();
        }

        "".to_string()
    }
}

#[pymethods]
impl Prefab {
    fn __str__(&self) -> PyResult<String> {
        self.__repr__()
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("{}", self.path))
    }

    pub fn __eq__(&self, other: &Self, py: Python<'_>) -> bool {
        if let Ok(pthstr) = self.path.downcast_bound::<PyString>(py) {
            if let Ok(otherpthstr) = other.path.downcast_bound::<PyString>(py) {
                if !pthstr.to_string().eq(&otherpthstr.to_string()) {
                    return false;
                }
            }
        } else if let Ok(pthpth) = self.path.downcast_bound::<Path>(py) {
            if let Ok(otherpthpth) = other.path.downcast_bound::<Path>(py) {
                if !pthpth.eq(otherpthpth).unwrap() {
                    return false;
                }
            }
        }
        if let Ok(vardict) = self.vars.downcast_bound::<PyDict>(py) {
            if let Ok(othervardict) = other.vars.downcast_bound::<PyDict>(py) {
                if !vardict.eq(othervardict).unwrap() {
                    return false;
                }
            }
        }

        true
    }

    pub fn __hash__(&self, py: Python<'_>) -> PyResult<u64> {
        let mut s = DefaultHasher::new();
        if let Ok(pthstr) = self.path.downcast_bound::<PyString>(py) {
            pthstr.hash()?.hash(&mut s);
        }
        if let Ok(vardict) = self.vars.downcast_bound::<PyDict>(py) {
            vardict.hash()?.hash(&mut s);
        }

        Ok(s.finish())
    }
}

#[pyclass(extends = Node, module = "avulto.ast")]
pub struct Return {
    #[pyo3(get)]
    retval: Py<PyAny>,
}

impl Return {
    pub fn make(py: Python<'_>, retval: Py<PyAny>) -> PyResult<PyObject> {
        let base = PyClassInitializer::from(Node::new(NodeKind::Return));
        let sub = base.add_subclass(Return { retval });
        Ok(Py::new(py, sub)?.to_object(py))
    }
}

#[pyclass(extends = Node, module = "avulto.ast")]
pub struct If {
    #[pyo3(get)]
    arms: Py<PyAny>,
    #[pyo3(get)]
    else_arm: Py<PyAny>,
}

impl If {
    pub fn make(py: Python<'_>, arms: Py<PyAny>, else_arm: Py<PyAny>) -> PyResult<PyObject> {
        let base = PyClassInitializer::from(Node::new(NodeKind::If));
        let sub = base.add_subclass(If { arms, else_arm });
        Ok(Py::new(py, sub)?.to_object(py))
    }
}

#[pyclass(extends = Node, module = "avulto.ast")]
pub struct IfArm {
    #[pyo3(get)]
    cond: Py<PyAny>,
    #[pyo3(get)]
    stmts: Py<PyAny>,
}

impl IfArm {
    pub fn make(py: Python<'_>, cond: Py<PyAny>, stmts: Py<PyAny>) -> PyResult<PyObject> {
        let base = PyClassInitializer::from(Node::new(NodeKind::IfArm));
        let sub = base.add_subclass(IfArm { cond, stmts });
        Ok(Py::new(py, sub)?.to_object(py))
    }
}

#[pyclass(extends = Node, module = "avulto.ast")]
pub struct Attribute {
    #[pyo3(get)]
    expr: Py<PyAny>,
    #[pyo3(get)]
    name: Py<PyAny>,
}

impl Attribute {
    pub fn make(py: Python<'_>, expr: Py<PyAny>, name: Py<PyAny>) -> PyResult<PyObject> {
        let base = PyClassInitializer::from(Node::new(NodeKind::Attribute));
        let sub = base.add_subclass(Attribute { expr, name });
        Ok(Py::new(py, sub)?.to_object(py))
    }
}

#[pyclass(extends = Node, module = "avulto.ast")]
pub struct Ternary {
    #[pyo3(get)]
    cond: Py<PyAny>,
    #[pyo3(get)]
    if_: Py<PyAny>,
    #[pyo3(get)]
    else_: Py<PyAny>,
}

impl Ternary {
    pub fn make(
        py: Python<'_>,
        cond: Py<PyAny>,
        if_: Py<PyAny>,
        else_: Py<PyAny>,
    ) -> PyResult<PyObject> {
        let base = PyClassInitializer::from(Node::new(NodeKind::Ternary));
        let sub = base.add_subclass(Ternary { cond, if_, else_ });
        Ok(Py::new(py, sub)?.to_object(py))
    }
}

#[pyclass(extends = Node, module = "avulto.ast")]
pub struct BinaryOp {
    #[pyo3(get)]
    lhs: Py<PyAny>,
    #[pyo3(get)]
    rhs: Py<PyAny>,
    #[pyo3(get)]
    op: BinaryOperator,
}

impl BinaryOp {
    pub fn make(
        py: Python<'_>,
        lhs: Py<PyAny>,
        rhs: Py<PyAny>,
        op: &dreammaker::ast::BinaryOp,
    ) -> PyResult<PyObject> {
        let binary_op = BinaryOp {
            lhs,
            rhs,
            op: match op {
                dreammaker::ast::BinaryOp::Add => BinaryOperator::Add,
                dreammaker::ast::BinaryOp::Sub => BinaryOperator::Sub,
                dreammaker::ast::BinaryOp::Mul => BinaryOperator::Mul,
                dreammaker::ast::BinaryOp::Div => BinaryOperator::Div,
                dreammaker::ast::BinaryOp::Pow => BinaryOperator::Pow,
                dreammaker::ast::BinaryOp::Mod => BinaryOperator::Mod,
                dreammaker::ast::BinaryOp::FloatMod => BinaryOperator::FloatMod,
                dreammaker::ast::BinaryOp::Eq => BinaryOperator::Eq,
                dreammaker::ast::BinaryOp::NotEq => BinaryOperator::NotEq,
                dreammaker::ast::BinaryOp::Less => BinaryOperator::Less,
                dreammaker::ast::BinaryOp::Greater => BinaryOperator::Greater,
                dreammaker::ast::BinaryOp::LessEq => BinaryOperator::LessEq,
                dreammaker::ast::BinaryOp::GreaterEq => BinaryOperator::GreaterEq,
                dreammaker::ast::BinaryOp::Equiv => BinaryOperator::Equiv,
                dreammaker::ast::BinaryOp::NotEquiv => BinaryOperator::NotEquiv,
                dreammaker::ast::BinaryOp::BitAnd => BinaryOperator::BitAnd,
                dreammaker::ast::BinaryOp::BitXor => BinaryOperator::BitXor,
                dreammaker::ast::BinaryOp::BitOr => BinaryOperator::BitOr,
                dreammaker::ast::BinaryOp::LShift => BinaryOperator::LShift,
                dreammaker::ast::BinaryOp::RShift => BinaryOperator::RShift,
                dreammaker::ast::BinaryOp::And => BinaryOperator::And,
                dreammaker::ast::BinaryOp::Or => BinaryOperator::Or,
                dreammaker::ast::BinaryOp::In => BinaryOperator::In,
                dreammaker::ast::BinaryOp::To => BinaryOperator::To,
            },
        };

        let base = PyClassInitializer::from(Node::new(NodeKind::BinaryOp));
        let sub = base.add_subclass(binary_op);

        Ok(Py::new(py, sub)?.to_object(py))
    }
}

#[pyclass(module = "avulto.ast", name = "BinaryOperator", eq, eq_int)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum UnaryOperator {
    #[pyo3(name = "NEG")]
    Neg,
    #[pyo3(name = "NOT")]
    Not,
    #[pyo3(name = "BIT_NOT")]
    BitNot,
    #[pyo3(name = "PRE_INCR")]
    PreIncr,
    #[pyo3(name = "POST_INCR")]
    PostIncr,
    #[pyo3(name = "PRE_DECR")]
    PreDecr,
    #[pyo3(name = "POST_DECR")]
    PostDecr,
    #[pyo3(name = "REF")]
    Ref,
    #[pyo3(name = "DEREF")]
    Deref,
}

#[pyclass(extends = Node, module = "avulto.ast")]
pub struct UnaryOp {
    #[pyo3(get)]
    term: Py<PyAny>,
    #[pyo3(get)]
    op: UnaryOperator,
}

impl UnaryOp {
    pub fn make(
        py: Python<'_>,
        term: Py<PyAny>,
        op: &dreammaker::ast::UnaryOp,
    ) -> PyResult<PyObject> {
        let unary_op = UnaryOp {
            term,
            op: match op {
                dreammaker::ast::UnaryOp::Neg => UnaryOperator::Neg,
                dreammaker::ast::UnaryOp::Not => UnaryOperator::Not,
                dreammaker::ast::UnaryOp::BitNot => UnaryOperator::BitNot,
                dreammaker::ast::UnaryOp::PreIncr => UnaryOperator::PreIncr,
                dreammaker::ast::UnaryOp::PostIncr => UnaryOperator::PostIncr,
                dreammaker::ast::UnaryOp::PreDecr => UnaryOperator::PreDecr,
                dreammaker::ast::UnaryOp::PostDecr => UnaryOperator::PostDecr,
                dreammaker::ast::UnaryOp::Reference => UnaryOperator::Ref,
                dreammaker::ast::UnaryOp::Dereference => UnaryOperator::Deref,
            },
        };

        let base = PyClassInitializer::from(Node::new(NodeKind::UnaryOp));
        let sub = base.add_subclass(unary_op);

        Ok(Py::new(py, sub)?.to_object(py))
    }
}

#[pyclass(extends = Node, module = "avulto.ast")]
pub struct Index {
    #[pyo3(get)]
    expr: Py<PyAny>,
    #[pyo3(get)]
    index: Py<PyAny>,
}

impl Index {
    pub fn make(py: Python<'_>, expr: Py<PyAny>, index: Py<PyAny>) -> PyResult<PyObject> {
        let base = PyClassInitializer::from(Node::new(NodeKind::Index));
        let sub = base.add_subclass(Index { expr, index });
        Ok(Py::new(py, sub)?.to_object(py))
    }
}

#[pyclass(extends = Node, module = "avulto.ast")]
pub struct Call {
    #[pyo3(get)]
    expr: Py<PyAny>,
    #[pyo3(get)]
    name: Py<PyAny>,
    #[pyo3(get)]
    args: Py<PyAny>,
}

impl Call {
    pub fn make(
        py: Python<'_>,
        expr: Py<PyAny>,
        name: Py<PyAny>,
        args: Py<PyAny>,
    ) -> PyResult<PyObject> {
        let base = PyClassInitializer::from(Node::new(NodeKind::Call));
        let sub = base.add_subclass(Call { expr, name, args });
        Ok(Py::new(py, sub)?.to_object(py))
    }
}

#[pyclass(extends = Node, module = "avulto.ast")]
pub struct ParentCall {
    #[pyo3(get)]
    args: Py<PyAny>,
}

impl ParentCall {
    pub fn make(py: Python<'_>, args: Py<PyAny>) -> PyResult<PyObject> {
        let base = PyClassInitializer::from(Node::new(NodeKind::ParentCall));
        let sub = base.add_subclass(ParentCall { args });
        Ok(Py::new(py, sub)?.to_object(py))
    }
}

#[pyclass(extends = Node, module = "avulto.ast")]
pub struct SelfCall {
    #[pyo3(get)]
    args: Py<PyAny>,
}

impl SelfCall {
    pub fn make(py: Python<'_>, args: Py<PyAny>) -> PyResult<PyObject> {
        let base = PyClassInitializer::from(Node::new(NodeKind::SelfCall));
        let sub = base.add_subclass(SelfCall { args });
        Ok(Py::new(py, sub)?.to_object(py))
    }
}

#[pyclass(extends = Node, module = "avulto.ast")]
pub struct Crash {
    #[pyo3(get)]
    expr: Py<PyAny>,
}

impl Crash {
    pub fn make(py: Python<'_>, expr: Py<PyAny>) -> PyResult<PyObject> {
        let base = PyClassInitializer::from(Node::new(NodeKind::Crash));
        let sub = base.add_subclass(Crash { expr });
        Ok(Py::new(py, sub)?.to_object(py))
    }
}

#[pyclass(extends = Node, module = "avulto.ast")]
pub struct ForLoop {
    #[pyo3(get)]
    init: Py<PyAny>,
    #[pyo3(get)]
    test: Py<PyAny>,
    #[pyo3(get)]
    increment: Py<PyAny>,
    #[pyo3(get)]
    stmts: Py<PyAny>,
}

impl ForLoop {
    pub fn make(
        py: Python<'_>,
        init: Py<PyAny>,
        test: Py<PyAny>,
        increment: Py<PyAny>,
        stmts: Py<PyAny>,
    ) -> PyResult<PyObject> {
        let base = PyClassInitializer::from(Node::new(NodeKind::ForLoop));
        let sub = base.add_subclass(ForLoop {
            init,
            test,
            increment,
            stmts,
        });
        Ok(Py::new(py, sub)?.to_object(py))
    }
}

#[pyclass(extends = Node, module = "avulto.ast")]
pub struct ForList {
    #[pyo3(get)]
    name: Py<PyAny>,
    #[pyo3(get)]
    in_list: Py<PyAny>,
    #[pyo3(get)]
    stmts: Py<PyAny>,
}

impl ForList {
    pub fn make(
        py: Python<'_>,
        name: Py<PyAny>,
        in_list: Py<PyAny>,
        stmts: Py<PyAny>,
    ) -> PyResult<PyObject> {
        let base = PyClassInitializer::from(Node::new(NodeKind::ForList));
        let sub = base.add_subclass(ForList {
            name,
            in_list,
            stmts,
        });
        Ok(Py::new(py, sub)?.to_object(py))
    }
}

#[pyclass(extends = Node, module = "avulto.ast")]
pub struct ForRange {
    #[pyo3(get)]
    name: Py<PyAny>,
    #[pyo3(get)]
    start: Py<PyAny>,
    #[pyo3(get)]
    end: Py<PyAny>,
    #[pyo3(get)]
    step: Py<PyAny>,
    #[pyo3(get)]
    stmts: Py<PyAny>,
}

impl ForRange {
    pub fn make(
        py: Python<'_>,
        name: Py<PyAny>,
        start: Py<PyAny>,
        end: Py<PyAny>,
        step: Py<PyAny>,
        stmts: Py<PyAny>,
    ) -> PyResult<PyObject> {
        let base = PyClassInitializer::from(Node::new(NodeKind::ForRange));
        let sub = base.add_subclass(ForRange {
            name,
            start,
            end,
            step,
            stmts,
        });
        Ok(Py::new(py, sub)?.to_object(py))
    }
}

#[pyclass(extends = Node, module = "avulto.ast")]
pub struct NewPrefab {
    #[pyo3(get)]
    prefab: Py<PyAny>,
    #[pyo3(get)]
    args: Py<PyAny>,
}

impl NewPrefab {
    pub fn make(py: Python<'_>, prefab: Py<PyAny>, args: Py<PyAny>) -> PyResult<PyObject> {
        let base = PyClassInitializer::from(Node::new(NodeKind::NewPrefab));
        let sub = base.add_subclass(NewPrefab { prefab, args });
        Ok(Py::new(py, sub)?.to_object(py))
    }
}

#[pyclass(extends = Node, module = "avulto.ast")]
pub struct Setting {
    #[pyo3(get)]
    name: Py<PyAny>,
    #[pyo3(get)]
    mode: Py<PyAny>,
    #[pyo3(get)]
    value: Py<PyAny>,
}

impl Setting {
    pub fn make(
        py: Python<'_>,
        name: Py<PyAny>,
        mode: &dreammaker::ast::SettingMode,
        value: Py<PyAny>,
    ) -> PyResult<PyObject> {
        let base = PyClassInitializer::from(Node::new(NodeKind::Setting));
        let sub = base.add_subclass(Setting {
            name,
            mode: match mode {
                dreammaker::ast::SettingMode::Assign => SettingMode::Assign,
                dreammaker::ast::SettingMode::In => SettingMode::In,
            }
            .into_py(py),
            value,
        });
        Ok(Py::new(py, sub)?.to_object(py))
    }
}

#[pyclass(extends = Node, module = "avulto.ast")]
pub struct Label {
    #[pyo3(get)]
    name: Py<PyAny>,
    #[pyo3(get)]
    stmts: Py<PyAny>,
}

impl Label {
    pub fn make(py: Python<'_>, name: Py<PyAny>, stmts: Py<PyAny>) -> PyResult<PyObject> {
        let base = PyClassInitializer::from(Node::new(NodeKind::Label));
        let sub = base.add_subclass(Label { name, stmts });
        Ok(Py::new(py, sub)?.to_object(py))
    }
}

#[pyclass(extends = Node, module = "avulto.ast")]
pub struct NewImplicit {
    #[pyo3(get)]
    args: Py<PyAny>,
}

impl NewImplicit {
    pub fn make(py: Python<'_>, args: Py<PyAny>) -> PyResult<PyObject> {
        let base = PyClassInitializer::from(Node::new(NodeKind::NewImplicit));
        let sub = base.add_subclass(NewImplicit { args });
        Ok(Py::new(py, sub)?.to_object(py))
    }
}

#[pyclass(extends = Node, module = "avulto.ast")]
pub struct InterpString {
    #[pyo3(get)]
    ident: Py<PyAny>,
    #[pyo3(get)]
    tokens: Py<PyAny>,
}

impl InterpString {
    pub fn make(py: Python<'_>, ident: Py<PyAny>, tokens: Py<PyAny>) -> PyResult<PyObject> {
        let base = PyClassInitializer::from(Node::new(NodeKind::InterpString));
        let sub = base.add_subclass(InterpString { ident, tokens });
        Ok(Py::new(py, sub)?.to_object(py))
    }
}

#[pyclass(extends = Node, module = "avulto.ast")]
pub struct SwitchCase {
    #[pyo3(get)]
    exact: Py<PyAny>,
    #[pyo3(get)]
    range: Py<PyAny>,
    #[pyo3(get)]
    stmts: Py<PyAny>,
}

impl SwitchCase {
    pub fn make(
        py: Python<'_>,
        exact: Py<PyAny>,
        range: Py<PyAny>,
        stmts: Py<PyAny>,
    ) -> PyResult<PyObject> {
        let base = PyClassInitializer::from(Node::new(NodeKind::SwitchCase));
        let sub = base.add_subclass(SwitchCase {
            exact,
            range,
            stmts,
        });
        Ok(Py::new(py, sub)?.to_object(py))
    }
}

#[pyclass(extends = Node, module = "avulto.ast")]
pub struct Switch {
    #[pyo3(get)]
    input: Py<PyAny>,
    #[pyo3(get)]
    cases: Py<PyAny>,
    #[pyo3(get)]
    default: Py<PyAny>,
}

impl Switch {
    pub fn make(
        py: Python<'_>,
        input: Py<PyAny>,
        cases: Py<PyAny>,
        default: Py<PyAny>,
    ) -> PyResult<PyObject> {
        let base = PyClassInitializer::from(Node::new(NodeKind::Switch));
        let sub = base.add_subclass(Switch {
            input,
            cases,
            default,
        });
        Ok(Py::new(py, sub)?.to_object(py))
    }
}

#[pyclass(extends = Node, module = "avulto.ast")]
pub struct Spawn {
    #[pyo3(get)]
    delay: Py<PyAny>,
    #[pyo3(get)]
    stmts: Py<PyAny>,
}

impl Spawn {
    pub fn make(py: Python<'_>, delay: Py<PyAny>, stmts: Py<PyAny>) -> PyResult<PyObject> {
        let base = PyClassInitializer::from(Node::new(NodeKind::Spawn));
        let sub = base.add_subclass(Spawn { delay, stmts });
        Ok(Py::new(py, sub)?.to_object(py))
    }
}

#[pyclass(extends = Node, module = "avulto.ast")]
pub struct While {
    #[pyo3(get)]
    cond: Py<PyAny>,
    #[pyo3(get)]
    stmts: Py<PyAny>,
}

impl While {
    pub fn make(py: Python<'_>, cond: Py<PyAny>, stmts: Py<PyAny>) -> PyResult<PyObject> {
        let base = PyClassInitializer::from(Node::new(NodeKind::While));
        let sub = base.add_subclass(While { cond, stmts });
        Ok(Py::new(py, sub)?.to_object(py))
    }
}

#[pyclass(extends = Node, module = "avulto.ast")]
pub struct DoWhile {
    #[pyo3(get)]
    cond: Py<PyAny>,
    #[pyo3(get)]
    stmts: Py<PyAny>,
}

impl DoWhile {
    pub fn make(py: Python<'_>, cond: Py<PyAny>, stmts: Py<PyAny>) -> PyResult<PyObject> {
        let base = PyClassInitializer::from(Node::new(NodeKind::DoWhile));
        let sub = base.add_subclass(DoWhile { cond, stmts });
        Ok(Py::new(py, sub)?.to_object(py))
    }
}

#[pyclass(extends = Node, module = "avulto.ast")]
pub struct Break {
    #[pyo3(get)]
    ident: Py<PyAny>,
}

impl Break {
    pub fn make(py: Python<'_>, ident: Py<PyAny>) -> PyResult<PyObject> {
        let base = PyClassInitializer::from(Node::new(NodeKind::Break));
        let sub = base.add_subclass(Break { ident });
        Ok(Py::new(py, sub)?.to_object(py))
    }
}

#[pyclass(extends = Node, module = "avulto.ast")]
pub struct Continue {
    #[pyo3(get)]
    ident: Py<PyAny>,
}

impl Continue {
    pub fn make(py: Python<'_>, ident: Py<PyAny>) -> PyResult<PyObject> {
        let base = PyClassInitializer::from(Node::new(NodeKind::Continue));
        let sub = base.add_subclass(Continue { ident });
        Ok(Py::new(py, sub)?.to_object(py))
    }
}

#[pyclass(extends = Node, module = "avulto.ast")]
pub struct Locate {
    #[pyo3(get)]
    args: Py<PyAny>,
    #[pyo3(get)]
    in_list: Py<PyAny>,
}

impl Locate {
    pub fn make(py: Python<'_>, args: Py<PyAny>, in_list: Py<PyAny>) -> PyResult<PyObject> {
        let base = PyClassInitializer::from(Node::new(NodeKind::Locate));
        let sub = base.add_subclass(Locate { args, in_list });
        Ok(Py::new(py, sub)?.to_object(py))
    }
}

#[pyclass(extends = Node, module = "avulto.ast")]
pub struct TryCatch {
    #[pyo3(get)]
    try_block: Py<PyAny>,
    #[pyo3(get)]
    catch_params: Py<PyAny>,
    #[pyo3(get)]
    catch_block: Py<PyAny>,
}

impl TryCatch {
    pub fn make(
        py: Python<'_>,
        try_block: Py<PyAny>,
        catch_params: Py<PyAny>,
        catch_block: Py<PyAny>,
    ) -> PyResult<PyObject> {
        let base = PyClassInitializer::from(Node::new(NodeKind::TryCatch));
        let sub = base.add_subclass(TryCatch {
            try_block,
            catch_params,
            catch_block,
        });
        Ok(Py::new(py, sub)?.to_object(py))
    }
}

#[pyclass(extends = Node, module = "avulto.ast")]
pub struct Input {
    #[pyo3(get)]
    args: Py<PyAny>,
    #[pyo3(get)]
    input_type: Py<PyAny>,
    #[pyo3(get)]
    in_list: Py<PyAny>,
}

impl Input {
    pub fn make(
        py: Python<'_>,
        args: Py<PyAny>,
        input_type: Py<PyAny>,
        in_list: Py<PyAny>,
    ) -> PyResult<PyObject> {
        let base = PyClassInitializer::from(Node::new(NodeKind::Input));
        let sub = base.add_subclass(Input {
            args,
            input_type,
            in_list,
        });
        Ok(Py::new(py, sub)?.to_object(py))
    }
}

#[pyclass(extends = Node, module = "avulto.ast")]
pub struct MiniExpr {
    #[pyo3(get)]
    ident: Py<PyAny>,
    #[pyo3(get)]
    fields: Py<PyAny>,
}

impl MiniExpr {
    pub fn make(py: Python<'_>, ident: Py<PyAny>, fields: Py<PyAny>) -> PyResult<PyObject> {
        let base = PyClassInitializer::from(Node::new(NodeKind::MiniExpr));
        let sub = base.add_subclass(MiniExpr { ident, fields });
        Ok(Py::new(py, sub)?.to_object(py))
    }
}

#[pyclass(extends = Node, module = "avulto.ast")]
pub struct NewMiniExpr {
    #[pyo3(get)]
    expr: Py<PyAny>,
    #[pyo3(get)]
    args: Py<PyAny>,
}

impl NewMiniExpr {
    pub fn make(py: Python<'_>, expr: Py<PyAny>, args: Py<PyAny>) -> PyResult<PyObject> {
        let base = PyClassInitializer::from(Node::new(NodeKind::NewMiniExpr));
        let sub = base.add_subclass(NewMiniExpr { expr, args });
        Ok(Py::new(py, sub)?.to_object(py))
    }
}

#[pyclass(extends = Node, module = "avulto.ast")]
pub struct DynamicCall {
    #[pyo3(get)]
    proc_info: Py<PyAny>,
    #[pyo3(get)]
    args: Py<PyAny>,
}

impl DynamicCall {
    pub fn make(py: Python<'_>, proc_info: Py<PyAny>, args: Py<PyAny>) -> PyResult<PyObject> {
        let base = PyClassInitializer::from(Node::new(NodeKind::DynamicCall));
        let sub = base.add_subclass(DynamicCall { proc_info, args });
        Ok(Py::new(py, sub)?.to_object(py))
    }
}

#[pyclass(extends = Node, module = "avulto.ast")]
pub struct ExternalCall {
    #[pyo3(get)]
    library_name: Py<PyAny>,
    #[pyo3(get)]
    function_name: Py<PyAny>,
    #[pyo3(get)]
    args: Py<PyAny>,
}

impl ExternalCall {
    pub fn make(
        py: Python<'_>,
        library_name: Py<PyAny>,
        function_name: Py<PyAny>,
        args: Py<PyAny>,
    ) -> PyResult<PyObject> {
        let base = PyClassInitializer::from(Node::new(NodeKind::ExternalCall));
        let sub = base.add_subclass(ExternalCall {
            library_name,
            function_name,
            args,
        });
        Ok(Py::new(py, sub)?.to_object(py))
    }
}

#[pyclass(extends = Node, module = "avulto.ast")]
pub struct Del {
    #[pyo3(get)]
    expr: Py<PyAny>,
}

impl Del {
    pub fn make(py: Python<'_>, expr: Py<PyAny>) -> PyResult<PyObject> {
        let base = PyClassInitializer::from(Node::new(NodeKind::Del));
        let sub = base.add_subclass(Del { expr });
        Ok(Py::new(py, sub)?.to_object(py))
    }
}

#[pyclass(extends = Node, module = "avulto.ast")]
pub struct Throw {
    #[pyo3(get)]
    expr: Py<PyAny>,
}

impl Throw {
    pub fn make(py: Python<'_>, expr: Py<PyAny>) -> PyResult<PyObject> {
        let base = PyClassInitializer::from(Node::new(NodeKind::Throw));
        let sub = base.add_subclass(Throw { expr });
        Ok(Py::new(py, sub)?.to_object(py))
    }
}
