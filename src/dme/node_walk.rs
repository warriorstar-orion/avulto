use pyo3::{
    Bound, IntoPyObject, PyAny, PyResult, Python,
    call::PyCallArgs,
    exceptions::PyAttributeError,
    types::{PyAnyMethods, PyString},
};

use crate::dme::SourceLoc;

use super::{Dme, expression::Expression, nodes::Node};

pub(super) fn try_visit<'py>(
    walker: &Bound<'py, PyAny>,
    method_name: &Bound<'py, PyString>,
    args: impl PyCallArgs<'py>,
) -> PyResult<bool> {
    match walker.getattr(method_name) {
        Ok(method) => {
            method.call1(args)?;
            Ok(true)
        }
        Err(err) if err.is_instance_of::<PyAttributeError>(walker.py()) => Ok(false),
        Err(err) => Err(err),
    }
}

impl Node {
    pub fn walk(
        self_: &Bound<Self>,
        dme: &Bound<Dme>,
        walker: &Bound<PyAny>,
        py: Python<'_>,
    ) -> PyResult<()> {
        let node = self_.get();
        match node {
            Node::Unknown() => todo!(),
            Node::Expression { expr, .. } => {
                Expression::walk(expr.bind(py), walker, py)?;
            }
            Node::Return { retval, source_loc } => {
                if !try_visit(
                    walker,
                    pyo3::intern!(py, "visit_Return"),
                    (self_, source_loc.clone()),
                )? {
                    if let Some(some_expr) = retval {
                        Expression::walk(some_expr.bind(py), walker, py)?;
                    }
                }

                return Ok(());
            }
            Node::Throw { expr, source_loc } => {
                if !try_visit(
                    walker,
                    pyo3::intern!(py, "visit_Throw"),
                    (self_, source_loc.clone()),
                )? {
                    Expression::walk(expr.bind(py), walker, py)?;
                }
            }
            Node::While {
                condition,
                block,
                source_loc,
            } => {
                if !try_visit(
                    walker,
                    pyo3::intern!(py, "visit_While"),
                    (self_, source_loc.clone()),
                )? {
                    Expression::walk(condition.bind(py), walker, py)?;
                    for stmt in block.iter() {
                        Node::walk(stmt.bind(py), dme, walker, py)?;
                    }
                }
                return Ok(());
            }
            Node::DoWhile {
                condition,
                block,
                source_loc,
            } => {
                if !try_visit(
                    walker,
                    pyo3::intern!(py, "visit_DoWhile"),
                    (self_, source_loc.clone()),
                )? {
                    Expression::walk(condition.bind(py), walker, py)?;
                    for stmt in block.iter() {
                        Node::walk(stmt.bind(py), dme, walker, py)?;
                    }
                }
            }
            Node::If {
                if_arms,
                else_arm,
                source_loc,
            } => {
                if !try_visit(
                    walker,
                    pyo3::intern!(py, "visit_If"),
                    (self_, source_loc.clone()),
                )? {
                    for (cond, block) in if_arms.iter() {
                        Expression::walk(cond.bind(py), walker, py)?;
                        for stmt in block.iter() {
                            Node::walk(stmt.bind(py), dme, walker, py)?;
                        }
                    }
                    if let Some(else_arm_block) = else_arm {
                        for stmt in else_arm_block.iter() {
                            Node::walk(stmt.bind(py), dme, walker, py)?;
                        }
                    }
                }
                return Ok(());
            }
            Node::ForInfinite { block, source_loc } => {
                if !try_visit(
                    walker,
                    pyo3::intern!(py, "visit_ForInfinite"),
                    (self_, source_loc.clone()),
                )? {
                    for stmt in block.iter() {
                        Node::walk(stmt.bind(py), dme, walker, py)?;
                    }
                }
                return Ok(());
            }
            Node::ForLoop {
                init,
                test,
                inc,
                block,
                source_loc,
            } => {
                if !try_visit(
                    walker,
                    pyo3::intern!(py, "visit_ForLoop"),
                    (self_, source_loc.clone()),
                )? {
                    if let Some(init_node) = init {
                        Node::walk(init_node.bind(py), dme, walker, py)?;
                    }
                    if let Some(test_expr) = test {
                        Expression::walk(test_expr.bind(py), walker, py)?;
                    }
                    if let Some(inc_node) = inc {
                        Node::walk(inc_node.bind(py), dme, walker, py)?;
                    }
                    for stmt in block.iter() {
                        Node::walk(stmt.bind(py), dme, walker, py)?;
                    }
                }
                return Ok(());
            }
            Node::Var {
                name,
                value,
                source_loc,
                ..
            } => {
                if !try_visit(
                    walker,
                    pyo3::intern!(py, "visit_Var"),
                    (self_, source_loc.clone()),
                )? {
                    Expression::walk(name.bind(py), walker, py)?;
                    if let Some(expr) = value {
                        Expression::walk(expr.bind(py), walker, py)?;
                    }
                }
                return Ok(());
            }
            Node::Crash { expr, source_loc } => {
                if !try_visit(
                    walker,
                    pyo3::intern!(py, "visit_Crash"),
                    (self_, source_loc.clone()),
                )? {
                    if let Some(some_expr) = expr {
                        Expression::walk(some_expr.bind(py), walker, py)?;
                    }
                }

                return Ok(());
            }
            Node::ForList {
                name,
                in_list,
                block,
                source_loc,
                var_type,
            } => {
                if !try_visit(
                    walker,
                    pyo3::intern!(py, "visit_ForList"),
                    (self_, source_loc.clone()),
                )? {
                    let loop_decl = Node::Var {
                        name: name.clone_ref(py),
                        value: None,
                        declared_type: var_type.clone(),
                        source_loc: SourceLoc::builtin(),
                    };
                    // I guess we pretend that a variable declaration in a for loop is a var statement of sorts
                    Node::walk(&loop_decl.into_pyobject(py)?, dme, walker, py)?;
                    // Expression::walk(name.bind(py), dme, walker, py)?;
                    if let Some(in_list_expr) = in_list {
                        Expression::walk(in_list_expr.bind(py), walker, py)?;
                        for stmt in block.iter() {
                            Node::walk(stmt.bind(py), dme, walker, py)?;
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
                source_loc,
            } => {
                if !try_visit(
                    walker,
                    pyo3::intern!(py, "visit_ForRange"),
                    (self_, source_loc.clone()),
                )? {
                    Expression::walk(name.bind(py), walker, py)?;
                    Expression::walk(start.bind(py), walker, py)?;
                    Expression::walk(end.bind(py), walker, py)?;
                    if let Some(step) = step {
                        Expression::walk(step.bind(py), walker, py)?;
                    }
                    for stmt in block.iter() {
                        Node::walk(stmt.bind(py), dme, walker, py)?;
                    }
                }
            }
            Node::Vars { vars, .. } => {
                for var in vars.iter() {
                    Node::walk(var.bind(py), dme, walker, py)?;
                }
            }
            Node::Del { expr, source_loc } => {
                if !try_visit(
                    walker,
                    pyo3::intern!(py, "visit_Del"),
                    (self_, source_loc.clone()),
                )? {
                    Expression::walk(expr.bind(py), walker, py)?;
                }
            }
            Node::Break { label, source_loc } => {
                if !try_visit(
                    walker,
                    pyo3::intern!(py, "visit_Break"),
                    (self_, source_loc.clone()),
                )? {
                    if let Some(l) = label {
                        Expression::walk(l.bind(py), walker, py)?;
                    }
                }
            }
            Node::Setting {
                name,
                mode: _,
                value,
                source_loc,
            } => {
                if !try_visit(
                    walker,
                    pyo3::intern!(py, "visit_Setting"),
                    (self_, source_loc.clone()),
                )? {
                    Expression::walk(name.bind(py), walker, py)?;
                    Expression::walk(value.bind(py), walker, py)?;
                }
                return Ok(());
            }
            Node::Spawn {
                delay,
                block,
                source_loc,
            } => {
                if !try_visit(
                    walker,
                    pyo3::intern!(py, "visit_Spawn"),
                    (self_, source_loc.clone()),
                )? {
                    if let Some(delay) = delay {
                        Expression::walk(delay.bind(py), walker, py)?;
                    }
                    for stmt in block.iter() {
                        Node::walk(stmt.bind(py), dme, walker, py)?;
                    }
                }

                return Ok(());
            }
            Node::Continue { name, source_loc } => {
                if !try_visit(
                    walker,
                    pyo3::intern!(py, "visit_Continue"),
                    (self_, source_loc.clone()),
                )? {
                    if let Some(name) = name {
                        Expression::walk(name.bind(py), walker, py)?;
                    }
                }
            }
            Node::Goto { label, source_loc } => {
                if !try_visit(
                    walker,
                    pyo3::intern!(py, "visit_Goto"),
                    (self_, source_loc.clone()),
                )? {
                    Expression::walk(label.bind(py), walker, py)?;
                }
            }
            Node::Label {
                name,
                block,
                source_loc,
            } => {
                if !try_visit(
                    walker,
                    pyo3::intern!(py, "visit_Label"),
                    (self_, source_loc.clone()),
                )? {
                    Expression::walk(name.bind(py), walker, py)?;
                    for stmt in block.iter() {
                        Node::walk(stmt.bind(py), dme, walker, py)?;
                    }
                }
            }
            Node::TryCatch {
                try_block,
                catch_params,
                catch_block,
                source_loc,
            } => {
                if !try_visit(
                    walker,
                    pyo3::intern!(py, "visit_TryCatch"),
                    (self_, source_loc.clone()),
                )? {
                    for stmt in try_block.iter() {
                        Node::walk(stmt.bind(py), dme, walker, py)?;
                    }
                    for catch_params in catch_params.iter() {
                        for catch_param in catch_params.iter() {
                            Expression::walk(catch_param.bind(py), walker, py)?;
                        }
                    }
                    for stmt in catch_block.iter() {
                        Node::walk(stmt.bind(py), dme, walker, py)?;
                    }
                }
            }
            Node::Switch {
                input,
                cases,
                default,
                source_loc,
            } => {
                if !try_visit(
                    walker,
                    pyo3::intern!(py, "visit_Switch"),
                    (self_, source_loc.clone()),
                )? {
                    Expression::walk(input.bind(py), walker, py)?;
                    for case in cases.iter() {
                        case.borrow(py).walk_parts(dme, walker, py)?;
                    }
                    if let Some(default) = default {
                        for stmt in default.iter() {
                            Node::walk(stmt.bind(py), dme, walker, py)?;
                        }
                    }
                }
            }
            Node::ForKeyValue {
                var_type,
                key,
                value,
                in_list,
                block,
                source_loc,
            } => {
                if !try_visit(
                    walker,
                    pyo3::intern!(py, "visit_ForKeyValue"),
                    (self_, source_loc.clone()),
                )? {
                    let keydecl = Node::Var {
                        name: key.clone_ref(py),
                        value: None,
                        declared_type: var_type.clone(),
                        source_loc: SourceLoc::builtin(),
                    };
                    let valuedecl = Node::Var {
                        name: value.clone_ref(py),
                        value: None,
                        declared_type: None,
                        source_loc: SourceLoc::builtin(),
                    };
                    // I guess we pretend that a variable declaration in a for loop is a var statement of sorts
                    Node::walk(&keydecl.into_pyobject(py)?, dme, walker, py)?;
                    Node::walk(&valuedecl.into_pyobject(py)?, dme, walker, py)?;

                    if let Some(in_list_expr) = in_list {
                        Expression::walk(in_list_expr.bind(py), walker, py)?;
                        for stmt in block.iter() {
                            Node::walk(stmt.bind(py), dme, walker, py)?;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
