use core::fmt;
use std::hash::Hash;

use dreammaker::ast::Statement;
use pyo3::{
    pyclass, pymethods, pymodule, types::{PyAnyMethods, PyList, PyModule, PyModuleMethods}, Bound, IntoPy, Py, PyAny, PyObject, PyResult, Python
};

use super::{expression::Expression, operators::SettingMode};

extern crate dreammaker;

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
pub enum Node {
    Unknown(),
    Expression {
        expr: PyObject,
    },
    Crash {
        expr: Option<PyObject>,
    },
    Return {
        retval: Option<PyObject>,
    },
    Throw {
        expr: PyObject,
    },
    Del {
        expr: PyObject,
    },
    Break {
        label: Option<PyObject>,
    },
    While {
        condition: PyObject,
        block: Vec<PyObject>,
    },
    DoWhile {
        condition: PyObject,
        block: Vec<PyObject>,
    },
    If {
        if_arms: Vec<(PyObject, Vec<PyObject>)>,
        else_arm: Option<Vec<PyObject>>,
    },
    ForInfinite {
        block: Vec<PyObject>,
    },
    ForList {
        name: PyObject,
        in_list: Option<PyObject>,
        block: Vec<PyObject>,
    },
    ForLoop {
        init: Option<PyObject>,
        test: Option<PyObject>,
        inc: Option<PyObject>,
        block: Vec<PyObject>,
    },
    ForRange {
        name: PyObject,
        start: PyObject,
        end: PyObject,
        step: Option<PyObject>,
        block: Vec<PyObject>,
    },
    Var {
        name: String,
        value: Option<PyObject>,
    },
    Vars {
        vars: Vec<PyObject>,
    },
    Setting {
        name: PyObject,
        mode: SettingMode,
        value: PyObject,
    },
    Spawn {
        delay: Option<PyObject>,
        block: Vec<PyObject>,
    },
    Continue {
        name: Option<PyObject>,
    },
    Goto {
        label: PyObject,
    },
    Label {
        name: PyObject,
        block: Vec<PyObject>,
    },
    TryCatch {
        try_block: Vec<PyObject>,
        catch_params: Vec<Vec<PyObject>>,
        catch_block: Vec<PyObject>,
    },
    Switch {
        input: PyObject,
        cases: Vec<PyObject>,
        default: Option<Vec<PyObject>>,
    },
}

pub fn visit_constant(_py: Python<'_>, walker: &Bound<PyAny>, constant: Py<PyAny>) -> PyResult<()> {
    if walker.hasattr("visit_Constant").unwrap() {
        walker.call_method1("visit_Constant", (constant,))?;
    }

    Ok(())
}

impl Node {
    pub fn from_statement(py: Python<'_>, stmt: &Statement) -> PyObject {
        match stmt {
            Statement::Var(v) => Self::Var {
                name: v.name.clone(),
                value: v
                    .value
                    .as_ref()
                    .map(|expr| Expression::from_expression(py, expr).into_py(py)),
            }
            .into_py(py),
            Statement::Expr(expression) => Self::Expression {
                expr: Expression::from_expression(py, expression).into_py(py),
            }
            .into_py(py),
            Statement::Return(expression) => Self::Return {
                retval: expression
                    .as_ref()
                    .map(|expr| Expression::from_expression(py, expr).into_py(py)),
            }
            .into_py(py),
            Statement::Throw(expression) => Self::Throw {
                expr: Expression::from_expression(py, expression).into_py(py),
            }
            .into_py(py),
            Statement::While { condition, block } => Self::While {
                condition: Expression::from_expression(py, condition).into_py(py),
                block: block
                    .iter()
                    .map(|stmt| Node::from_statement(py, &stmt.elem).into_py(py))
                    .collect::<Vec<PyObject>>(),
            }
            .into_py(py),
            Statement::DoWhile { block, condition } => Self::DoWhile {
                condition: Expression::from_expression(py, &condition.elem).into_py(py),
                block: block
                    .iter()
                    .map(|stmt| Node::from_statement(py, &stmt.elem).into_py(py))
                    .collect::<Vec<PyObject>>(),
            }
            .into_py(py),
            Statement::If { arms, else_arm } => {
                let if_arms: Vec<(PyObject, Vec<PyObject>)> = arms
                    .iter()
                    .map(|(cond, stmts)| {
                        let mut stmt_nodes: Vec<Py<PyAny>> = vec![];
                        for stmt in stmts.iter() {
                            stmt_nodes.push(Node::from_statement(py, &stmt.elem).into_py(py));
                        }

                        (
                            Expression::from_expression(py, &cond.elem).into_py(py),
                            stmt_nodes,
                        )
                    })
                    .collect();
                let mut else_arm_nodes: Vec<Py<PyAny>> = vec![];
                if let Some(else_arm_block) = else_arm {
                    for stmt in else_arm_block.iter() {
                        else_arm_nodes.push(Node::from_statement(py, &stmt.elem).into_py(py));
                    }
                }
                Self::If {
                    if_arms,
                    else_arm: if else_arm_nodes.is_empty() {
                        None
                    } else {
                        Some(else_arm_nodes)
                    },
                }
                .into_py(py)
            }
            Statement::ForInfinite { block } => Self::ForInfinite {
                block: block
                    .iter()
                    .map(|stmt| Node::from_statement(py, &stmt.elem).into_py(py))
                    .collect::<Vec<PyObject>>(),
            }
            .into_py(py),
            Statement::ForLoop {
                init,
                test,
                inc,
                block,
            } => Self::ForLoop {
                init: init
                    .as_ref()
                    .map(|stmt| Node::from_statement(py, stmt).into_py(py)),
                test: test
                    .as_ref()
                    .map(|expr| Expression::from_expression(py, expr).into_py(py)),
                inc: inc
                    .as_ref()
                    .map(|stmt| Node::from_statement(py, stmt).into_py(py)),
                block: block
                    .iter()
                    .map(|stmt| Node::from_statement(py, &stmt.elem).into_py(py))
                    .collect::<Vec<PyObject>>(),
            }
            .into_py(py),
            Statement::ForList(for_list_statement) => Self::ForList {
                name: Expression::Identifier {
                    name: for_list_statement.name.to_string(),
                }
                .into_py(py),
                in_list: for_list_statement
                    .in_list
                    .as_ref()
                    .map(|expr| Expression::from_expression(py, expr).into_py(py)),
                block: for_list_statement
                    .block
                    .iter()
                    .map(|stmt| Node::from_statement(py, &stmt.elem).into_py(py))
                    .collect::<Vec<PyObject>>(),
            }
            .into_py(py),
            Statement::ForRange(for_range_statement) => Self::ForRange {
                name: Expression::Identifier {
                    name: for_range_statement.name.to_string(),
                }
                .into_py(py),
                start: Expression::from_expression(py, &for_range_statement.start).into_py(py),
                end: Expression::from_expression(py, &for_range_statement.end).into_py(py),
                block: for_range_statement
                    .block
                    .iter()
                    .map(|stmt| Node::from_statement(py, &stmt.elem).into_py(py))
                    .collect::<Vec<PyObject>>(),
                step: for_range_statement
                    .step
                    .as_ref()
                    .map(|expr| Expression::from_expression(py, expr).into_py(py)),
            }
            .into_py(py),
            Statement::Vars(vec) => Self::Vars {
                vars: vec
                    .iter()
                    .map(|vs| {
                        Self::Var {
                            name: vs.name.clone(),
                            value: vs
                                .value
                                .as_ref()
                                .map(|expr| Expression::from_expression(py, expr).into_py(py)),
                        }
                        .into_py(py)
                    })
                    .collect(),
            }
            .into_py(py),
            Statement::Setting { name, mode, value } => Self::Setting {
                name: Expression::Identifier {
                    name: name.to_string(),
                }
                .into_py(py),
                mode: match mode {
                    dreammaker::ast::SettingMode::Assign => SettingMode::Assign,
                    dreammaker::ast::SettingMode::In => SettingMode::In,
                },
                value: Expression::from_expression(py, value).into_py(py),
            }
            .into_py(py),
            Statement::Spawn { delay, block } => Self::Spawn {
                delay: delay
                    .as_ref()
                    .map(|expr| Expression::from_expression(py, expr).into_py(py)),
                block: block
                    .iter()
                    .map(|stmt| Node::from_statement(py, &stmt.elem).into_py(py))
                    .collect::<Vec<PyObject>>(),
            }
            .into_py(py),
            Statement::Switch {
                input,
                cases,
                default,
            } => {
                let input_expr = Expression::from_expression(py, input).into_py(py);
                let mut case_nodes: Vec<Py<PyAny>> = vec![];
                for (case, block) in cases.iter() {
                    let mut exact_nodes: Vec<Py<PyAny>> = vec![];
                    let mut range_nodes: Vec<Py<PyAny>> = vec![];

                    for case_type in &case.elem {
                        match case_type {
                            dreammaker::ast::Case::Exact(e) => {
                                exact_nodes.push(Expression::from_expression(py, e).into_py(py));
                            }
                            dreammaker::ast::Case::Range(s, e) => {
                                let range_list = PyList::new_bound(
                                    py,
                                    [
                                        Expression::from_expression(py, s).into_py(py),
                                        Expression::from_expression(py, e).into_py(py),
                                    ],
                                )
                                .into_py(py);
                                range_nodes.push(range_list);
                            }
                        }
                    }
                    case_nodes.push(
                        SwitchCase {
                            exact: PyList::new_bound(py, exact_nodes).into_py(py),
                            range: PyList::new_bound(py, range_nodes).into_py(py),
                            block: block
                                .iter()
                                .map(|stmt| Node::from_statement(py, &stmt.elem).into_py(py))
                                .collect::<Vec<PyObject>>(),
                        }
                        .into_py(py),
                    );
                }

                Self::Switch {
                    input: input_expr,
                    cases: case_nodes,
                    default: default.as_ref().map(|stmts| {
                        stmts
                            .iter()
                            .map(|stmt| Node::from_statement(py, &stmt.elem).into_py(py))
                            .collect::<Vec<PyObject>>()
                    }),
                }
                .into_py(py)
            }
            Statement::TryCatch {
                try_block,
                catch_params,
                catch_block,
            } => Self::TryCatch {
                try_block: try_block
                    .iter()
                    .map(|stmt| Node::from_statement(py, &stmt.elem).into_py(py))
                    .collect::<Vec<PyObject>>(),
                catch_block: catch_block
                    .iter()
                    .map(|stmt| Node::from_statement(py, &stmt.elem).into_py(py))
                    .collect::<Vec<PyObject>>(),
                catch_params: catch_params
                    .iter()
                    .map(|tc| {
                        tc.iter()
                            .map(|tcs| Expression::Identifier { name: tcs.clone() }.into_py(py))
                            .collect::<Vec<PyObject>>()
                    })
                    .collect::<Vec<Vec<PyObject>>>(),
            }
            .into_py(py),
            Statement::Continue(name) => Self::Continue {
                name: name
                    .clone()
                    .map(|s| Expression::Identifier { name: s.clone() }.into_py(py)),
            }
            .into_py(py),
            Statement::Break(label) => Self::Break {
                label: label
                    .as_ref()
                    .map(|l| Expression::Identifier { name: l.clone() }.into_py(py)),
            }
            .into_py(py),
            Statement::Goto(label) => Self::Goto {
                label: Expression::Identifier {
                    name: label.clone(),
                }
                .into_py(py),
            }
            .into_py(py),
            Statement::Label { name, block } => Self::Label {
                name: Expression::Identifier { name: name.clone() }.into_py(py),
                block: block
                    .iter()
                    .map(|stmt| Node::from_statement(py, &stmt.elem).into_py(py))
                    .collect::<Vec<PyObject>>(),
            }
            .into_py(py),
            Statement::Del(expression) => Self::Del {
                expr: Expression::from_expression(py, expression).into_py(py),
            }
            .into_py(py),
            Statement::Crash(expression) => Self::Crash {
                expr: expression
                    .as_ref()
                    .map(|expr| Expression::from_expression(py, expr).into_py(py)),
            }
            .into_py(py),
        }
    }

    pub fn walk(self_: &Bound<Self>, py: Python<'_>, walker: &Bound<PyAny>) -> PyResult<()> {
        let node = self_.get();
        match node {
            Node::Unknown() => todo!(),
            Node::Expression { expr } => {
                if let Ok(expr_) = expr.downcast_bound::<Expression>(py) {
                    Expression::walk(expr_, walker)?;
                }
            }
            Node::Return { retval } => {
                if walker.hasattr("visit_Return").unwrap() {
                    walker.call_method1("visit_Return", (self_.as_ref(),))?;
                } else if let Some(some_expr) = retval {
                    if let Ok(expr_py) = some_expr.downcast_bound::<Expression>(py) {
                        Expression::walk(expr_py, walker)?;
                    }
                }

                return Ok(());
            }
            Node::Throw { expr } => {
                if walker.hasattr("visit_Throw").unwrap() {
                    walker.call_method1("visit_Throw", (self_.as_ref(),))?;
                } else if let Ok(expr_py) = expr.downcast_bound::<Expression>(py) {
                    Expression::walk(expr_py, walker)?;
                }
            }
            Node::While { condition, block } => {
                if walker.hasattr("visit_While").unwrap() {
                    walker.call_method1("visit_While", (self_.as_ref(),))?;
                } else {
                    if let Ok(cond_expr) = condition.downcast_bound::<Expression>(py) {
                        Expression::walk(cond_expr, walker)?;
                    }
                    for stmt in block.iter() {
                        if let Ok(node) = stmt.downcast_bound::<Node>(py) {
                            Node::walk(node, py, walker)?;
                        }
                    }
                }
                return Ok(());
            }
            Node::DoWhile { condition, block } => {
                if walker.hasattr("visit_DoWhile").unwrap() {
                    walker.call_method1("visit_DoWhile", (self_.as_ref(),))?;
                } else {
                    if let Ok(bound_condition) = condition.downcast_bound::<Expression>(py) {
                        Expression::walk(bound_condition, walker)?;
                    }
                    for stmt in block.iter() {
                        if let Ok(node) = stmt.downcast_bound::<Node>(py) {
                            Node::walk(node, py, walker)?;
                        }
                    }
                }
            }
            Node::If { if_arms, else_arm } => {
                if walker.hasattr("visit_If").unwrap() {
                    walker.call_method1("visit_If", (self_.as_ref(),))?;
                } else {
                    for (cond, block) in if_arms.iter() {
                        if let Ok(cond_expr) = cond.downcast_bound::<Expression>(py) {
                            Expression::walk(cond_expr, walker)?;
                        }
                        for stmt in block.iter() {
                            if let Ok(node_expr) = stmt.downcast_bound::<Node>(py) {
                                Node::walk(node_expr, py, walker)?;
                            }
                        }
                    }
                    if let Some(else_arm_block) = else_arm {
                        for stmt in else_arm_block.iter() {
                            if let Ok(node_expr) = stmt.downcast_bound::<Node>(py) {
                                Node::walk(node_expr, py, walker)?;
                            }
                        }
                    }
                }
                return Ok(());
            }
            Node::ForInfinite { block } => {
                if walker.hasattr("visit_ForInfinite").unwrap() {
                    walker.call_method1("visit_ForInfinite", (self_.as_ref(),))?;
                } else {
                    for stmt in block.iter() {
                        if let Ok(node_expr) = stmt.downcast_bound::<Node>(py) {
                            Node::walk(node_expr, py, walker)?;
                        }
                    }
                }
                return Ok(());
            }
            Node::ForLoop {
                init,
                test,
                inc,
                block,
            } => {
                if walker.hasattr("visit_ForLoop").unwrap() {
                    walker.call_method1("visit_ForLoop", (self_.as_ref(),))?;
                } else {
                    if let Some(init) = init {
                        if let Ok(init_node) = init.downcast_bound::<Node>(py) {
                            Node::walk(init_node, py, walker)?;
                        }
                    }
                    if let Some(test) = test {
                        if let Ok(bound_test) = test.downcast_bound::<Expression>(py) {
                            Expression::walk(bound_test, walker)?;
                        }
                    }
                    if let Some(inc) = inc {
                        if let Ok(inc_node) = inc.downcast_bound::<Node>(py) {
                            Node::walk(inc_node, py, walker)?;
                        }
                    }
                    for stmt in block.iter() {
                        if let Ok(node_expr) = stmt.downcast_bound::<Node>(py) {
                            Node::walk(node_expr, py, walker)?;
                        }
                    }
                }
                return Ok(());
            }
            Node::Var { name, value } => {
                if walker.hasattr("visit_Var").unwrap() {
                    walker.call_method1("visit_Var", (self_.as_ref(),))?;
                } else {
                    visit_constant(py, walker, name.clone().into_py(py))?;
                    if let Some(expr) = value {
                        if let Ok(expr_py) = expr.downcast_bound::<Expression>(py) {
                            Expression::walk(expr_py, walker)?;
                        }
                    }
                }
                return Ok(());
            }
            Node::Crash { expr } => {
                if walker.hasattr("visit_Crash").unwrap() {
                    walker.call_method1("visit_Crash", (self_.as_ref(),))?;
                } else if let Some(some_expr) = expr {
                    if let Ok(expr_py) = some_expr.downcast_bound::<Expression>(py) {
                        Expression::walk(expr_py, walker)?;
                    }
                }

                return Ok(());
            }
            Node::ForList {
                name,
                in_list,
                block,
            } => {
                if walker.hasattr("visit_ForList").unwrap() {
                    walker.call_method1("visit_ForList", (self_.as_ref(),))?;
                } else {
                    visit_constant(py, walker, name.clone_ref(py))?;
                    if let Some(in_list_expr) = in_list {
                        if let Ok(bound_in_list) = in_list_expr.downcast_bound::<Expression>(py) {
                            Expression::walk(bound_in_list, walker)?;
                        }
                        for stmt in block.iter() {
                            if let Ok(node_expr) = stmt.downcast_bound::<Node>(py) {
                                Node::walk(node_expr, py, walker)?;
                            }
                        }
                    }
                }

                return Ok(());
            }
            Node::ForRange {
                name,
                start,
                end,
                step,
                block,
            } => {
                if walker.hasattr("visit_ForRange").unwrap() {
                    walker.call_method1("visit_ForRange", (self_.as_ref(),))?;
                } else {
                    visit_constant(py, walker, name.clone_ref(py))?;
                    if let Ok(bound_start) = start.downcast_bound::<Expression>(py) {
                        Expression::walk(bound_start, walker)?;
                    }
                    if let Ok(bound_end) = end.downcast_bound::<Expression>(py) {
                        Expression::walk(bound_end, walker)?;
                    }
                    if let Some(step) = step {
                        if let Ok(bound_step) = step.downcast_bound::<Expression>(py) {
                            Expression::walk(bound_step, walker)?;
                        }
                    }
                    for stmt in block.iter() {
                        if let Ok(node_expr) = stmt.downcast_bound::<Node>(py) {
                            Node::walk(node_expr, py, walker)?;
                        }
                    }
                }
            }
            Node::Vars { vars } => {
                for var in vars.iter() {
                    if let Ok(bound_var) = var.downcast_bound::<Node>(py) {
                        Node::walk(bound_var, py, walker)?;
                    }
                }
            }
            Node::Del { expr } => {
                if walker.hasattr("visit_Del").unwrap() {
                    walker.call_method1("visit_Del", (self_.as_ref(),))?;
                } else if let Ok(expr_py) = expr.downcast_bound::<Expression>(py) {
                    Expression::walk(expr_py, walker)?;
                }
            }
            Node::Break { label } => {
                if walker.hasattr("visit_Break").unwrap() {
                    walker.call_method1("visit_Break", (self_.as_ref(),))?;
                } else if let Some(l) = label {
                    visit_constant(py, walker, l.clone().into_py(py))?;
                }
            }
            Node::Setting { name, mode: _, value } => {
                if walker.hasattr("visit_Setting").unwrap() {
                    walker.call_method1("visit_Setting", (self_.as_ref(),))?;
                } else {
                    visit_constant(py, walker, name.clone().into_py(py))?;
                    if let Ok(expr_py) = value.downcast_bound::<Expression>(py) {
                        Expression::walk(expr_py, walker)?;
                    }
                }
                return Ok(());
            }
            Node::Spawn { delay, block } => {
                if walker.hasattr("visit_Setting").unwrap() {
                    walker.call_method1("visit_Setting", (self_.as_ref(),))?;
                } else {
                    if let Some(delay) = delay {
                        if let Ok(bound_delay) = delay.downcast_bound::<Expression>(py) {
                            Expression::walk(bound_delay, walker)?;
                        }
                    }
                    for stmt in block.iter() {
                        if let Ok(node_expr) = stmt.downcast_bound::<Node>(py) {
                            Node::walk(node_expr, py, walker)?;
                        }
                    }
                }

                return Ok(());
            }
            Node::Continue { name } => {
                if walker.hasattr("visit_Continue").unwrap() {
                    walker.call_method1("visit_Continue", (self_.as_ref(),))?;
                } else if let Some(name) = name {
                    if let Ok(bound_name) = name.downcast_bound::<Expression>(py) {
                        Expression::walk(bound_name, walker)?;
                    }
                }
            }
            Node::Goto { label } => {
                if walker.hasattr("visit_Goto").unwrap() {
                    walker.call_method1("visit_Goto", (self_.as_ref(),))?;
                } else {
                    visit_constant(py, walker, label.clone_ref(py))?;
                }
            }
            Node::Label { name, block } => {
                if walker.hasattr("visit_Label").unwrap() {
                    walker.call_method1("visit_Label", (self_.as_ref(),))?;
                } else {
                    visit_constant(py, walker, name.clone_ref(py))?;
                    for stmt in block.iter() {
                        if let Ok(node_expr) = stmt.downcast_bound::<Node>(py) {
                            Node::walk(node_expr, py, walker)?;
                        }
                    }
                }
            }
            Node::TryCatch {
                try_block,
                catch_params,
                catch_block,
            } => {
                if walker.hasattr("visit_TryCatch").unwrap() {
                    walker.call_method1("visit_TryCatch", (self_.as_ref(),))?;
                } else {
                    for stmt in try_block.iter() {
                        if let Ok(node_expr) = stmt.downcast_bound::<Node>(py) {
                            Node::walk(node_expr, py, walker)?;
                        }
                    }
                    for catch_params in catch_params.iter() {
                        for catch_param in catch_params.iter() {
                            visit_constant(py, walker, catch_param.clone_ref(py))?;
                        }
                    }
                    for stmt in catch_block.iter() {
                        if let Ok(node_expr) = stmt.downcast_bound::<Node>(py) {
                            Node::walk(node_expr, py, walker)?;
                        }
                    }

                }
            },
            Node::Switch {
                input,
                cases,
                default,
            } => {
                if walker.hasattr("visit_Switch").unwrap() {
                    walker.call_method1("visit_Switch", (self_.as_ref(),))?;
                } else {
                    if let Ok(bound_input) = input.downcast_bound::<Expression>(py) {
                        Expression::walk(bound_input, walker)?;
                    }
                    for case in cases.iter() {
                        if let Ok(switch_case) = case.downcast_bound::<SwitchCase>(py) {
                            switch_case.borrow().walk_parts(py, walker)?;
                        }
                    }
                    if let Some(default) = default {
                        for stmt in default.iter() {
                            if let Ok(node_expr) = stmt.downcast_bound::<Node>(py) {
                                Node::walk(node_expr, py, walker)?;
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

#[pymethods]
impl Node {
    #[getter]
    fn get_kind(&self, py: Python<'_>) -> PyResult<PyObject> {
        match self {
            Node::Unknown() => Ok(NodeKind::Unknown.into_py(py)),
            Node::Expression { expr } => expr.call_method0(py, "kind"),
            Node::Crash { .. } => Ok(NodeKind::Crash.into_py(py)),
            Node::Return { .. } => Ok(NodeKind::Return.into_py(py)),
            Node::Throw { .. } => Ok(NodeKind::Throw.into_py(py)),
            Node::Del { .. } => Ok(NodeKind::Del.into_py(py)),
            Node::Break { .. } => Ok(NodeKind::Break.into_py(py)),
            Node::While { .. } => Ok(NodeKind::While.into_py(py)),
            Node::DoWhile { .. } => Ok(NodeKind::DoWhile.into_py(py)),
            Node::If { .. } => Ok(NodeKind::If.into_py(py)),
            Node::ForInfinite { .. } => Ok(NodeKind::ForInfinite.into_py(py)),
            Node::ForList { .. } => Ok(NodeKind::ForList.into_py(py)),
            Node::ForLoop { .. } => Ok(NodeKind::ForLoop.into_py(py)),
            Node::ForRange { .. } => Ok(NodeKind::ForRange.into_py(py)),
            Node::Var { .. } => Ok(NodeKind::Var.into_py(py)),
            Node::Vars { .. } => Ok(NodeKind::Vars.into_py(py)),
            Node::Setting { .. } => Ok(NodeKind::Setting.into_py(py)),
            Node::Spawn { .. } => Ok(NodeKind::Spawn.into_py(py)),
            Node::Continue { .. } => Ok(NodeKind::Continue.into_py(py)),
            Node::Goto { .. } => Ok(NodeKind::Goto.into_py(py)),
            Node::Label { .. } => Ok(NodeKind::Label.into_py(py)),
            Node::TryCatch { .. } => Ok(NodeKind::TryCatch.into_py(py)),
            Node::Switch { .. } => Ok(NodeKind::Switch.into_py(py)),
        }
    }

    fn __str__(&self, py: Python<'_>) -> PyResult<String> {
        self.__repr__(py)
    }

    fn __repr__(&self, py: Python<'_>) -> PyResult<String> {
        match self {
            Node::Unknown() => todo!(),
            Node::Expression { expr } => Ok(format!("{}", expr)),
            Node::Crash { expr } => Ok(format!("<Crash {:?}>", expr)),
            Node::Return { retval } => Ok(format!("<Return {}>", retval.as_ref().unwrap_or(&"...".to_string().into_py(py)))),
            Node::Throw { expr } => Ok(format!("<Throw {:?}>", expr)),
            Node::Del { expr } => Ok(format!("<Del {:?}>", expr)),
            Node::Break { label } => Ok(format!("<Break {:?}>", label)),
            Node::While { condition, block: _ } => Ok(format!("<While {} ...>", condition)),
            Node::DoWhile { condition, block: _ } => Ok(format!("<DoWhile {} ...>", condition)),
            Node::If { .. } => Ok("<If ...>".to_string()),
            Node::ForInfinite { .. } => Ok("<ForInfinite ...>".to_string()),
            Node::ForList { .. } => Ok("<ForList ...>".to_string()),
            Node::ForLoop { .. } => Ok("<ForLoop ...>".to_string()),
            Node::ForRange { .. } => Ok("<ForRange ...>".to_string()),
            Node::Var { name, value: _ } => Ok(format!("<Var {:?} ...>", name)),
            Node::Vars { .. } => Ok("<Vars ...>".to_string()),
            Node::Setting { name, mode: _, value: _ } => Ok(format!("<Setting {} ...>", name)),
            Node::Spawn { delay: _, block: _ } => Ok("<Spawn ...>".to_string()),
            Node::Continue { name } => Ok(format!("<Continue {:?}>", name)),
            Node::Goto { label } => Ok(format!("<Goto {}>", label)),
            Node::Label { name, block: _ } => Ok(format!("<Label {} ...>", name)),
            Node::TryCatch { .. } => Ok("<TryCatch ...>".to_string()),
            Node::Switch { input, cases: _, default: _ } => Ok(format!("<Switch {} ...>", input)),
        }
    }
}

#[pyclass(module = "avulto.ast")]
pub struct SwitchCase {
    #[pyo3(get)]
    exact: PyObject,
    #[pyo3(get)]
    range: PyObject,
    #[pyo3(get)]
    block: Vec<PyObject>,
}

impl SwitchCase {
    pub fn walk_parts(&self, py: Python<'_>, walker: &Bound<PyAny>) -> PyResult<()> {
        if let Ok(bound_exact) = self.exact.downcast_bound::<PyList>(py) {
            bound_exact.into_iter().for_each(|f| {
                if let Ok(expr) = f.downcast::<Expression>() {
                    Expression::walk(expr, walker);
                }
            });
        }
        if let Ok(bound_range) = self.range.downcast_bound::<PyList>(py) {
            bound_range.into_iter().for_each(|f| {
                if let Ok(expr) = f.downcast::<Expression>() {
                    Expression::walk(expr, walker);
                }
            });
        }
        for stmt in self.block.iter() {
            if let Ok(node_expr) = stmt.downcast_bound::<Node>(py) {
                Node::walk(node_expr, py, walker)?;
            }
        }

        Ok(())
    }
}
