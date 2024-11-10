use dreammaker::ast::AssignOp;
use pyo3::{
    exceptions::PyRuntimeError,
    pyclass, pymethods,
    types::{PyAnyMethods},
    Bound, IntoPy, Py, PyAny, PyObject, PyResult, Python,
};

use crate::{dmlist::DmList};

use super::{
    nodes::{visit_constant, NodeKind},
    operators::{AssignOperator, BinaryOperator, UnaryOperator},
    prefab::Prefab,
};

#[pyclass(eq, eq_int)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Constants {
    Null,
    ProcMacro,
}

#[pyclass]
#[derive(Clone)]
pub enum Constant {
    Null(),
    Int(i32),
    Float(f32),
    String(String),
    Resource(String),
    ProcMacro(),
}

#[pymethods]
impl Constant {
    #[getter]
    pub fn get_val(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        match self {
            Constant::Null() => Ok(Constants::Null.into_py(py)),
            Constant::Int(i) => Ok(i.into_py(py)),
            Constant::Float(f) => Ok(f.into_py(py)),
            Constant::String(s) => Ok(s.into_py(py)),
            Constant::Resource(s) => Ok(s.into_py(py)),
            Constant::ProcMacro() => Ok(Constants::ProcMacro.into_py(py)),
        }
    }
}

#[pyclass]
#[derive(Clone)]
pub enum Expression {
    Constant {
        constant: Constant,
    },
    Identifier {
        name: String,
    },
    List {
        list: PyObject,
    },
    BinaryOp {
        op: BinaryOperator,
        lhs: PyObject,
        rhs: PyObject,
    },
    AssignOp {
        op: AssignOperator,
        lhs: PyObject,
        rhs: PyObject,
    },
    TernaryOp {
        cond: PyObject,
        if_expr: PyObject,
        else_expr: PyObject,
    },
    InterpString {
        ident: PyObject,
        tokens: Vec<Vec<PyObject>>,
    },
    Locate {
        args: Vec<PyObject>,
        in_list: Option<PyObject>,
    },
    Prefab {
        prefab: PyObject,
    },
    Index {
        expr: PyObject,
        index: PyObject,
    },
    Field {
        expr: Option<PyObject>,
        field: PyObject,
    },
    StaticField {
        expr: PyObject,
        field: PyObject,
    },
    Call {
        expr: PyObject,
        name: PyObject,
        args: Vec<PyObject>,
    },
    SelfCall {
        args: Vec<PyObject>,
    },
    ParentCall {
        args: Vec<PyObject>,
    },
    UnaryOp {
        expr: PyObject,
        unary_op: UnaryOperator,
    },
    ProcReference {
        expr: PyObject,
        name: PyObject,
    },
    ExternalCall {
        library_name: PyObject,
        function_name: PyObject,
        args: Vec<PyObject>,
    },
    NewMiniExpr {
        name: PyObject,
        fields: Vec<PyObject>,
    },
    NewImplicit {
        args: Option<Vec<PyObject>>,
    },
    NewPrefab {
        prefab: PyObject,
        args: Option<Vec<PyObject>>,
    },
    DynamicCall {
        lib_name: Vec<PyObject>,
        proc_name: Vec<PyObject>,
    },
    Input {
        args: Vec<PyObject>,
        input_type: Option<PyObject>,
        in_list: Option<PyObject>,
    },
    Pick {
        args: Vec<(Option<PyObject>, PyObject)>,
    },
}

impl Expression {
    pub fn from_expression(py: Python<'_>, expr: &dreammaker::ast::Expression) -> Self {
        match expr {
            dreammaker::ast::Expression::Base { term, follow } => {
                let mut core = match &term.elem {
                    dreammaker::ast::Term::Ident(i) => Self::Identifier { name: i.clone() },
                    dreammaker::ast::Term::Int(i) => Self::Constant {
                        constant: Constant::Int(*i),
                    },
                    dreammaker::ast::Term::Null => Self::Constant {
                        constant: Constant::Null(),
                    },
                    dreammaker::ast::Term::Float(f) => Self::Constant {
                        constant: Constant::Float(*f),
                    },
                    dreammaker::ast::Term::String(s) => Self::Constant {
                        constant: Constant::String(s.clone()),
                    },
                    dreammaker::ast::Term::Resource(s) => Self::Constant {
                        constant: Constant::Resource(s.clone()),
                    },
                    dreammaker::ast::Term::As(input_type) => todo!(),
                    dreammaker::ast::Term::__PROC__ => Self::Constant {
                        constant: Constant::ProcMacro(),
                    },
                    dreammaker::ast::Term::__TYPE__ => todo!(),
                    dreammaker::ast::Term::__IMPLIED_TYPE__ => todo!(),
                    dreammaker::ast::Term::Expr(expression) => {
                        Expression::from_expression(py, expression)
                    }
                    dreammaker::ast::Term::Prefab(prefab) => Self::Prefab {
                        prefab: Prefab::make(py, prefab).into_py(py),
                    },
                    dreammaker::ast::Term::InterpString(ident2, tokens) => {
                        let mut token_vec: Vec<Vec<PyObject>> = vec![];
                        for (maybe_token_expr, token_str) in tokens.iter() {
                            let mut token_expr_node = py.None();
                            if let Some(token_expr) = maybe_token_expr {
                                token_expr_node =
                                    Expression::from_expression(py, token_expr).into_py(py);
                            }
                            token_vec.push(vec![token_expr_node, token_str.into_py(py)]);
                        }
                        // let tokens = PyList::new_bound(py, token_vec).into_py(py);
                        Self::InterpString {
                            ident: ident2.into_py(py),
                            tokens: token_vec,
                        }
                    }
                    dreammaker::ast::Term::Call(ident2, args) => Self::Call {
                        expr: py.None(),
                        name: Expression::Identifier {
                            name: ident2.to_string(),
                        }
                        .into_py(py),
                        args: args
                            .iter()
                            .map(|e| Expression::from_expression(py, e).into_py(py))
                            .collect(),
                    },
                    dreammaker::ast::Term::SelfCall(args) => Self::SelfCall {
                        args: args
                            .iter()
                            .map(|e| Expression::from_expression(py, e).into_py(py))
                            .collect(),
                    },
                    dreammaker::ast::Term::ParentCall(args) => Self::ParentCall {
                        args: args
                            .iter()
                            .map(|e| Expression::from_expression(py, e).into_py(py))
                            .collect(),
                    },
                    dreammaker::ast::Term::NewImplicit { args } => Self::NewImplicit {
                        args: args.as_ref().map(|args| {
                            args.iter()
                                .map(|arg| Expression::from_expression(py, arg).into_py(py))
                                .collect()
                        }),
                    },
                    dreammaker::ast::Term::NewPrefab { prefab, args } => Self::NewPrefab {
                        prefab: Prefab::make(py, prefab).into_py(py),
                        args: args.as_ref().map(|args| {
                            args.iter()
                                .map(|expr| Expression::from_expression(py, expr).into_py(py))
                                .collect()
                        }),
                    },
                    dreammaker::ast::Term::NewMiniExpr { expr, args } => Self::NewMiniExpr {
                        name: Expression::Identifier {
                            name: expr.ident.to_string(),
                        }
                        .into_py(py),
                        fields: expr
                            .fields
                            .iter()
                            .map(|f| {
                                Expression::Field {
                                    expr: None,
                                    field: Expression::Identifier {
                                        name: f.ident.to_string(),
                                    }
                                    .into_py(py),
                                }
                                .into_py(py)
                            })
                            .collect(),
                    },
                    dreammaker::ast::Term::List(l) => {
                        let mut keys: Vec<Py<PyAny>> = vec![];
                        let mut vals: Vec<Py<PyAny>> = vec![];

                        for args in l.iter() {
                            match args {
                                dreammaker::ast::Expression::Base { .. } => {
                                    keys.push(Expression::from_expression(py, args).into_py(py));
                                    vals.push(py.None());
                                }
                                dreammaker::ast::Expression::AssignOp { op: _, lhs, rhs } => {
                                    keys.push(Expression::from_expression(py, lhs).into_py(py));
                                    vals.push(Expression::from_expression(py, rhs).into_py(py));
                                }
                                dreammaker::ast::Expression::BinaryOp { .. } => {
                                    keys.push(Expression::from_expression(py, args).into_py(py));
                                    vals.push(py.None());
                                }
                                dreammaker::ast::Expression::TernaryOp { cond, if_, else_ } => {
                                    keys.push(
                                        Self::TernaryOp {
                                            cond: Expression::from_expression(py, cond).into_py(py),
                                            if_expr: Expression::from_expression(py, if_)
                                                .into_py(py),
                                            else_expr: Expression::from_expression(py, else_)
                                                .into_py(py),
                                        }
                                        .into_py(py),
                                    );
                                    vals.push(py.None());
                                }
                            }
                        }
                        Self::List {
                            list: DmList { keys, vals }.into_py(py),
                        }
                    }
                    dreammaker::ast::Term::Input {
                        args,
                        input_type,
                        in_list,
                    } => Self::Input {
                        args: args
                            .iter()
                            .map(|expr| Expression::from_expression(py, expr).into_py(py))
                            .collect(),
                        input_type: input_type.as_ref().map(|it| it.bits().into_py(py)),
                        in_list: in_list
                            .as_ref()
                            .map(|in_list| Expression::from_expression(py, &in_list).into_py(py)),
                    },
                    dreammaker::ast::Term::Locate { args, in_list } => Self::Locate {
                        args: args
                            .iter()
                            .map(|expr| Expression::from_expression(py, expr).into_py(py))
                            .collect(),
                        in_list: in_list
                            .as_ref()
                            .map(|expr| Expression::from_expression(py, expr).into_py(py)),
                    },
                    dreammaker::ast::Term::Pick(p) => Self::Pick {
                        args: p
                            .iter()
                            .map(|(a, b)| {
                                (
                                    a.as_ref().map(|expr| {
                                        Expression::from_expression(py, &expr).into_py(py)
                                    }),
                                    Expression::from_expression(py, &b).into_py(py),
                                )
                            })
                            .collect(),
                    },
                    dreammaker::ast::Term::DynamicCall(lib_name, proc_name) => Self::DynamicCall {
                        lib_name: lib_name
                            .iter()
                            .map(|expr| Expression::from_expression(py, expr).into_py(py))
                            .collect(),
                        proc_name: proc_name
                            .iter()
                            .map(|expr| Expression::from_expression(py, expr).into_py(py))
                            .collect(),
                    },
                    dreammaker::ast::Term::ExternalCall {
                        library_name,
                        function_name,
                        args,
                    } => Self::ExternalCall {
                        library_name: Expression::from_expression(py, &library_name).into_py(py),
                        function_name: Expression::from_expression(py, &function_name).into_py(py),
                        args: args
                            .iter()
                            .map(|a| Expression::from_expression(py, a).into_py(py))
                            .collect(),
                    },
                    dreammaker::ast::Term::GlobalIdent(ident2) => todo!(),
                    dreammaker::ast::Term::GlobalCall(ident2, _) => todo!(),
                };

                for f in follow.iter() {
                    match &f.elem {
                        dreammaker::ast::Follow::Index(list_access_kind, expression) => {
                            core = Self::Index {
                                expr: core.into_py(py),
                                index: Expression::from_expression(py, &expression).into_py(py),
                            };
                        }
                        dreammaker::ast::Follow::Field(property_access_kind, ident2) => {
                            core = Self::Field {
                                expr: Some(core.into_py(py)),
                                field: Expression::Identifier {
                                    name: ident2.to_string(),
                                }
                                .into_py(py),
                            };
                        }
                        dreammaker::ast::Follow::Call(property_access_kind, ident2, args) => {
                            core = Self::Call {
                                expr: core.into_py(py),
                                name: Expression::Identifier {
                                    name: ident2.to_string(),
                                }
                                .into_py(py),
                                args: args
                                    .iter()
                                    .map(|e| Expression::from_expression(py, e).into_py(py))
                                    .collect(),
                            }
                        }
                        dreammaker::ast::Follow::Unary(unary_op) => {
                            core = Self::UnaryOp {
                                expr: core.into_py(py),
                                unary_op: match unary_op {
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
                            }
                        }
                        dreammaker::ast::Follow::StaticField(ident2) => {
                            core = Self::StaticField {
                                expr: core.into_py(py),
                                field: Expression::Identifier {
                                    name: ident2.to_string(),
                                }
                                .into_py(py),
                            };
                        }
                        dreammaker::ast::Follow::ProcReference(ident2) => {
                            core = Self::ProcReference {
                                expr: core.into_py(py),
                                name: Expression::Identifier {
                                    name: ident2.to_string(),
                                }
                                .into_py(py),
                            };
                        }
                    }
                }

                core
            }
            dreammaker::ast::Expression::BinaryOp { op, lhs, rhs } => Self::BinaryOp {
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
                lhs: Self::from_expression(py, lhs).into_py(py),
                rhs: Self::from_expression(py, rhs).into_py(py),
            },
            dreammaker::ast::Expression::AssignOp { op, lhs, rhs } => Self::AssignOp {
                op: match op {
                    AssignOp::Assign => AssignOperator::Assign,
                    AssignOp::AddAssign => AssignOperator::AssignAdd,
                    AssignOp::SubAssign => AssignOperator::AssignSub,
                    AssignOp::MulAssign => AssignOperator::AssignMul,
                    AssignOp::DivAssign => AssignOperator::AssignDiv,
                    AssignOp::ModAssign => AssignOperator::AssignMod,
                    AssignOp::FloatModAssign => AssignOperator::AssignFloatMod,
                    AssignOp::AssignInto => AssignOperator::AssignInto,
                    AssignOp::BitAndAssign => AssignOperator::AssignBitAnd,
                    AssignOp::AndAssign => AssignOperator::AssignAnd,
                    AssignOp::BitOrAssign => AssignOperator::AssignBitOr,
                    AssignOp::OrAssign => AssignOperator::AssignOr,
                    AssignOp::BitXorAssign => AssignOperator::AssignBitXor,
                    AssignOp::LShiftAssign => AssignOperator::AssignLShift,
                    AssignOp::RShiftAssign => AssignOperator::AssignRShift,
                },
                lhs: Self::from_expression(py, lhs).into_py(py),
                rhs: Self::from_expression(py, rhs).into_py(py),
            },
            dreammaker::ast::Expression::TernaryOp { cond, if_, else_ } => Self::TernaryOp {
                cond: Self::from_expression(py, cond).into_py(py),
                if_expr: Self::from_expression(py, if_).into_py(py),
                else_expr: Self::from_expression(py, else_).into_py(py),
            },
        }
    }

    pub fn walk(self_: &Bound<Self>, walker: &Bound<PyAny>) -> PyResult<()> {
        Python::with_gil(|py| {
            if walker.hasattr("visit_Expr").unwrap() {
                walker.call_method1("visit_Expr", (self_.into_py(py),))?;
                return Ok(());
            }

            let self_expr = self_.get();
            match self_expr {
                Expression::Constant { constant } => match constant {
                    Constant::Null() => visit_constant(py, walker, Constants::Null.into_py(py)),
                    Constant::Int(i) => visit_constant(py, walker, i.into_py(py)),
                    Constant::Float(f) => visit_constant(py, walker, f.into_py(py)),
                    Constant::String(s) => visit_constant(py, walker, s.into_py(py)),
                    Constant::Resource(s) => visit_constant(py, walker, s.into_py(py)),
                    Constant::ProcMacro() => {
                        visit_constant(py, walker, Constants::ProcMacro.into_py(py))
                    }
                },
                Expression::Identifier { name: _ } => {
                    if walker.hasattr("visit_Identifier").unwrap() {
                        walker.call_method1("visit_Identifier", (self_.into_py(py),))?;
                    }
                    Ok(())
                }
                Expression::BinaryOp { op: _, lhs, rhs } => {
                    if walker.hasattr("visit_BinaryOp").unwrap() {
                        walker.call_method1("visit_BinaryOp", (self_.into_py(py),))?;
                    } else {
                        if let Ok(lhs_expr) = lhs.downcast_bound::<Expression>(py) {
                            Expression::walk(lhs_expr, walker)?;
                        } else {
                            return Err(PyRuntimeError::new_err("failed to walk binary op lhs"));
                        }
                        if let Ok(rhs_expr) = rhs.downcast_bound::<Expression>(py) {
                            Expression::walk(rhs_expr, walker)?;
                        } else {
                            return Err(PyRuntimeError::new_err("failed to walk binary op lhs"));
                        }
                    }

                    Ok(())
                }
                Expression::AssignOp { op: _, lhs, rhs } => {
                    if walker.hasattr("visit_AssignOp").unwrap() {
                        walker.call_method1("visit_AssignOp", (self_.into_py(py),))?;
                    } else {
                        if let Ok(lhs_expr) = lhs.downcast_bound::<Expression>(py) {
                            Expression::walk(lhs_expr, walker)?;
                        } else {
                            return Err(PyRuntimeError::new_err("failed to walk assign op lhs"));
                        }
                        if let Ok(rhs_expr) = rhs.downcast_bound::<Expression>(py) {
                            Expression::walk(rhs_expr, walker)?;
                        } else {
                            return Err(PyRuntimeError::new_err("failed to walk assign op lhs"));
                        }
                    }

                    Ok(())
                }
                Expression::TernaryOp {
                    cond,
                    if_expr,
                    else_expr,
                } => {
                    if walker.hasattr("visit_TernaryOp").unwrap() {
                        walker.call_method1("visit_TernaryOp", (self_.into_py(py),))?;
                    } else {
                        if let Ok(cond_expr) = cond.downcast_bound::<Expression>(py) {
                            Expression::walk(cond_expr, walker)?;
                        } else {
                            return Err(PyRuntimeError::new_err(
                                "failed to walk ternary op condition",
                            ));
                        }
                        if let Ok(if_) = if_expr.downcast_bound::<Expression>(py) {
                            Expression::walk(if_, walker)?;
                        } else {
                            return Err(PyRuntimeError::new_err("failed to walk ternary op if"));
                        }
                        if let Ok(else_) = else_expr.downcast_bound::<Expression>(py) {
                            Expression::walk(else_, walker)?;
                        } else {
                            return Err(PyRuntimeError::new_err("failed to walk ternary op else"));
                        }
                    }

                    Ok(())
                }
                Expression::List { list } => {
                    if walker.hasattr("visit_List").unwrap() {
                        walker.call_method1("visit_List", (self_.into_py(py),))?;
                    } else if let Ok(dmlist) = list.downcast_bound::<DmList>(py) {
                        let borrowed = dmlist.borrow();
                        for i in 0..borrowed.keys.len() {
                            if let Some(k) = borrowed.keys.get(i) {
                                visit_constant(py, walker, k.clone_ref(py))?;
                            }
                            if let Some(v) = borrowed.vals.get(i) {
                                if let Ok(v_) = v.downcast_bound::<Expression>(py) {
                                    Expression::walk(v_, walker)?;
                                }
                            }
                        }
                    }

                    Ok(())
                }
                Expression::InterpString { ident, tokens } => {
                    if walker.hasattr("visit_InterpString").unwrap() {
                        walker.call_method1("visit_InterpString", (self_.into_py(py),))?;
                    } else {
                        visit_constant(py, walker, ident.clone_ref(py))?;
                        for tokens in tokens.iter() {
                            for token in tokens.iter() {
                                if let Ok(bound_token) = token.downcast_bound::<Expression>(py) {
                                    Expression::walk(bound_token, walker)?;
                                }
                            }
                        }
                    }

                    Ok(())
                }
                Expression::Prefab { prefab } => {
                    if let Ok(prefab) = prefab.downcast_bound::<Prefab>(py) {
                        prefab.borrow().walk(walker);
                    }

                    Ok(())
                }
                Expression::Index { expr, index } => {
                    if walker.hasattr("visit_Index").unwrap() {
                        walker.call_method1("visit_Index", (self_.into_py(py),))?;
                    } else {
                        if let Ok(e) = expr.downcast_bound::<Expression>(py) {
                            Expression::walk(e, walker)?;
                        }
                        if let Ok(e) = index.downcast_bound::<Expression>(py) {
                            Expression::walk(e, walker)?;
                        }
                    }

                    Ok(())
                }
                Expression::Field { expr, field } => {
                    if walker.hasattr("visit_Field").unwrap() {
                        walker.call_method1("visit_Field", (self_.into_py(py),))?;
                    } else {
                        if let Some(expr) = expr {
                            if let Ok(bound_expr) = expr.downcast_bound::<Expression>(py) {
                                Expression::walk(bound_expr, walker)?;
                            }
                        }
                        if let Ok(e) = field.downcast_bound::<Expression>(py) {
                            Expression::walk(e, walker)?;
                        }
                    }

                    Ok(())
                }
                Expression::StaticField { expr, field } => {
                    if walker.hasattr("visit_StaticField").unwrap() {
                        walker.call_method1("visit_StaticField", (self_.into_py(py),))?;
                    } else {
                        if let Ok(e) = expr.downcast_bound::<Expression>(py) {
                            Expression::walk(e, walker)?;
                        }
                        visit_constant(py, walker, field.clone_ref(py))?;
                    }

                    Ok(())
                }
                Expression::Call { expr, name, args } => {
                    if walker.hasattr("visit_Call").unwrap() {
                        walker.call_method1("visit_Call", (self_.into_py(py),))?;
                    } else {
                        if let Ok(bound_expr) = expr.downcast_bound::<Expression>(py) {
                            Expression::walk(bound_expr, walker)?;
                        }
                        visit_constant(py, walker, name.clone_ref(py))?;
                        for arg in args.iter() {
                            if let Ok(bound_arg) = arg.downcast_bound::<Expression>(py) {
                                Expression::walk(bound_arg, walker)?;
                            }
                        }
                    }

                    Ok(())
                }
                Expression::SelfCall { args } => {
                    if walker.hasattr("visit_SelfCall").unwrap() {
                        walker.call_method1("visit_SelfCall", (self_.into_py(py),))?;
                    } else {
                        for arg in args.iter() {
                            if let Ok(bound_arg) = arg.downcast_bound::<Expression>(py) {
                                Expression::walk(bound_arg, walker)?;
                            }
                        }
                    }

                    Ok(())
                }
                Expression::ParentCall { args } => {
                    if walker.hasattr("visit_ParentCall").unwrap() {
                        walker.call_method1("visit_ParentCall", (self_.into_py(py),))?;
                    } else {
                        for arg in args.iter() {
                            if let Ok(bound_arg) = arg.downcast_bound::<Expression>(py) {
                                Expression::walk(bound_arg, walker)?;
                            }
                        }
                    }

                    Ok(())
                }
                Expression::UnaryOp { expr, unary_op } => {
                    if walker.hasattr("visit_UnaryOp").unwrap() {
                        walker.call_method1("visit_UnaryOp", (self_.into_py(py),))?;
                    } else if let Ok(e) = expr.downcast_bound::<Expression>(py) {
                        Expression::walk(e, walker)?;
                    }

                    Ok(())
                }
                Expression::ProcReference { expr, name } => {
                    if walker.hasattr("visit_ProcReference").unwrap() {
                        walker.call_method1("visit_ProcReference", (self_.into_py(py),))?;
                    } else {
                        if let Ok(e) = expr.downcast_bound::<Expression>(py) {
                            Expression::walk(e, walker)?;
                        }
                        visit_constant(py, walker, name.clone_ref(py))?;
                    }

                    Ok(())
                }
                Expression::Locate { args, in_list } => {
                    if walker.hasattr("visit_Locate").unwrap() {
                        walker.call_method1("visit_Locate", (self_.into_py(py),))?;
                    } else {
                        for arg in args.iter() {
                            if let Ok(arg_expr) = arg.downcast_bound::<Expression>(py) {
                                Expression::walk(arg_expr, walker)?;
                            }
                        }
                        if let Some(e) = in_list {
                            if let Ok(expr) = e.downcast_bound::<Expression>(py) {
                                Expression::walk(expr, walker)?;
                            }
                        }
                    }

                    Ok(())
                }
                Expression::ExternalCall {
                    library_name,
                    function_name,
                    args,
                } => {
                    if walker.hasattr("visit_ExternalCall").unwrap() {
                        walker.call_method1("visit_ExternalCall", (self_.into_py(py),))?;
                    } else {
                        visit_constant(py, walker, library_name.clone_ref(py))?;
                        visit_constant(py, walker, function_name.clone_ref(py))?;
                        for arg in args.iter() {
                            if let Ok(bound_arg) = arg.downcast_bound::<Expression>(py) {
                                Expression::walk(bound_arg, walker)?;
                            }
                        }
                    }
                    Ok(())
                }
                Expression::NewMiniExpr { name, fields } => {
                    if walker.hasattr("visit_NewMiniExpr").unwrap() {
                        walker.call_method1("visit_NewMiniExpr", (self_.into_py(py),))?;
                    } else {
                        visit_constant(py, walker, name.clone_ref(py))?;
                        for field in fields.iter() {
                            if let Ok(bound_field) = field.downcast_bound::<Expression>(py) {
                                Expression::walk(bound_field, walker)?;
                            }
                        }
                    }

                    Ok(())
                }
                Expression::NewImplicit { args } => {
                    if walker.hasattr("visit_NewImplicit").unwrap() {
                        walker.call_method1("visit_NewImplicit", (self_.into_py(py),))?;
                    } else if let Some(arg_list) = args {
                        for arg in arg_list.iter() {
                            if let Ok(bound_arg) = arg.downcast_bound::<Expression>(py) {
                                Expression::walk(bound_arg, walker)?;
                            }
                        }
                    }

                    Ok(())
                }
                Expression::NewPrefab { prefab, args } => {
                    if walker.hasattr("visit_NewPrefab").unwrap() {
                        walker.call_method1("visit_NewPrefab", (self_.into_py(py),))?;
                    } else {
                        if let Ok(prefab) = prefab.downcast_bound::<Prefab>(py) {
                            prefab.borrow().walk(walker);
                        }
                        if let Some(args) = args {
                            for arg in args.iter() {
                                if let Ok(bound_arg) = arg.downcast_bound::<Expression>(py) {
                                    Expression::walk(bound_arg, walker)?;
                                }
                            }
                        }
                    }

                    Ok(())
                }
                Expression::DynamicCall {
                    lib_name,
                    proc_name,
                } => {
                    if walker.hasattr("visit_DynamicCall").unwrap() {
                        walker.call_method1("visit_DynamicCall", (self_.into_py(py),))?;
                    } else {
                        for lib_name in lib_name.iter() {
                            if let Ok(bound_lib_name) = lib_name.downcast_bound::<Expression>(py) {
                                Expression::walk(bound_lib_name, walker)?;
                            }
                        }
                        for proc_name in proc_name.iter() {
                            if let Ok(bound_proc_name) = proc_name.downcast_bound::<Expression>(py)
                            {
                                Expression::walk(bound_proc_name, walker)?;
                            }
                        }
                    }
                    Ok(())
                }
                Expression::Input {
                    args,
                    input_type,
                    in_list,
                } => {
                    if walker.hasattr("visit_Input").unwrap() {
                        walker.call_method1("visit_Input", (self_.into_py(py),))?;
                    } else {
                        for arg in args.iter() {
                            if let Ok(bound_arg) = arg.downcast_bound::<Expression>(py) {
                                Expression::walk(bound_arg, walker)?;
                            }
                        }
                        if let Some(in_list) = in_list {
                            if let Ok(bound_in_list) = in_list.downcast_bound::<Expression>(py) {
                                Expression::walk(bound_in_list, walker)?;
                            }
                        }
                    }

                    Ok(())
                }
                Expression::Pick { args } => {
                    if walker.hasattr("visit_Pick").unwrap() {
                        walker.call_method1("visit_Pick", (self_.into_py(py),))?;
                    } else {
                        for (a, b) in args.iter() {
                            if let Some(a) = a {
                                if let Ok(bound_a) = a.downcast_bound::<Expression>(py) {
                                    Expression::walk(bound_a, walker)?;
                                }
                            }
                            if let Ok(bound_b) = b.downcast_bound::<Expression>(py) {
                                Expression::walk(bound_b, walker)?;
                            }
                        }
                    }

                    Ok(())
                }
            }
        })?;
        Ok(())
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
        self.__repr__(py)
    }

    fn __repr__(&self, py: Python<'_>) -> PyResult<String> {
        match self {
            Expression::Constant { constant } => Ok(format!("<Constant {}>", constant.get_val(py)?)),
            Expression::Identifier { name } => Ok(format!("<Identifier {}>", name)),
            Expression::List { list } => Ok(format!("{}", list)),
            Expression::BinaryOp { op, lhs, rhs } => Ok(format!("<BinaryOp {} {:?} {}>", lhs, op, rhs)),
            Expression::AssignOp { op, lhs, rhs } => Ok(format!("<AssignOp {} {:?} {}>", lhs, op, rhs)),
            Expression::TernaryOp {
                cond,
                if_expr: _,
                else_expr: _,
            } => Ok(format!("<TernaryOp {} ...>", cond)),
            Expression::InterpString { ident: _, tokens: _ } => Ok("<InterpString ...>".to_string()),
            Expression::Locate { args: _, in_list: _ } => Ok("<Locate ...>".to_string()),
            Expression::Prefab { prefab } => Ok(format!("<Prefab {}>", prefab)),
            Expression::Index { expr, index } => Ok(format!("<Index {}[{}]>", expr, index)),
            Expression::Field { expr, field } => Ok(format!("<Field {}>", field)),
            Expression::StaticField { expr, field } => Ok(format!("<StaticField {}::{}>", expr, field)),
            Expression::Call { expr, name, args } => Ok(format!("<Call {}.{}(...)>", expr, name)),
            Expression::SelfCall { args } => Ok(format!("<SelfCall ...>")),
            Expression::ParentCall { args } => Ok(format!("<ParentCall ...>")),
            Expression::UnaryOp { expr, unary_op } => Ok(format!("<UnaryOp {:?} {}>", unary_op, expr)),
            Expression::ProcReference { expr, name } => Ok(format!("<ProcReference {}.{}>", expr, name)),
            Expression::ExternalCall {
                library_name,
                function_name,
                args,
            } => Ok(format!("<ExternalCall {},{}(...)>", library_name, function_name)),
            Expression::NewMiniExpr { name, fields } => Ok(format!("<NewMiniExpr {} ...>", name)),
            Expression::NewImplicit { args } => Ok(format!("<NewImplicit ...>")),
            Expression::NewPrefab { prefab, args } => Ok(format!("<NewPrefab {} ...>", prefab)),
            Expression::DynamicCall {
                lib_name,
                proc_name,
            } => Ok(format!("<DynamicCall ({:?})({:?})>", lib_name, proc_name)),
            Expression::Input {
                args,
                input_type,
                in_list,
            } => Ok(format!("<Input {:?} ...>", input_type)),
            Expression::Pick { args } => Ok(format!("<Pick ...>")),
        }
    }
}
