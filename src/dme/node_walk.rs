use pyo3::{types::PyAnyMethods, Bound, IntoPyObject, PyAny, PyResult, Python};

use super::{expression::Expression, nodes::Node, Dme};

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
                Expression::walk(expr.bind(py), dme, walker, py)?;
            }
            Node::Return { retval, source_loc } => {
                if walker.hasattr("visit_Return").unwrap() {
                    walker.call_method1(
                        "visit_Return",
                        (
                            self_.as_ref(),
                            dme.borrow().populate_source_loc(source_loc, py),
                        ),
                    )?;
                } else if let Some(some_expr) = retval {
                    Expression::walk(some_expr.bind(py), dme, walker, py)?;
                }

                return Ok(());
            }
            Node::Throw { expr, source_loc } => {
                if walker.hasattr("visit_Throw").unwrap() {
                    walker.call_method1(
                        "visit_Throw",
                        (
                            self_.as_ref(),
                            dme.borrow().populate_source_loc(source_loc, py),
                        ),
                    )?;
                } else {
                    Expression::walk(expr.bind(py), dme, walker, py)?;
                }
            }
            Node::While {
                condition,
                block,
                source_loc,
            } => {
                if walker.hasattr("visit_While").unwrap() {
                    walker.call_method1(
                        "visit_While",
                        (
                            self_.as_ref(),
                            dme.borrow().populate_source_loc(source_loc, py),
                        ),
                    )?;
                } else {
                    Expression::walk(condition.bind(py), dme, walker, py)?;
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
                if walker.hasattr("visit_DoWhile").unwrap() {
                    walker.call_method1(
                        "visit_DoWhile",
                        (
                            self_.as_ref(),
                            dme.borrow().populate_source_loc(source_loc, py),
                        ),
                    )?;
                } else {
                    Expression::walk(condition.bind(py), dme, walker, py)?;
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
                if walker.hasattr("visit_If").unwrap() {
                    walker.call_method1(
                        "visit_If",
                        (
                            self_.as_ref(),
                            dme.borrow().populate_source_loc(source_loc, py),
                        ),
                    )?;
                } else {
                    for (cond, block) in if_arms.iter() {
                        Expression::walk(cond.bind(py), dme, walker, py)?;
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
                if walker.hasattr("visit_ForInfinite").unwrap() {
                    walker.call_method1(
                        "visit_ForInfinite",
                        (
                            self_.as_ref(),
                            dme.borrow().populate_source_loc(source_loc, py),
                        ),
                    )?;
                } else {
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
                if walker.hasattr("visit_ForLoop").unwrap() {
                    walker.call_method1(
                        "visit_ForLoop",
                        (
                            self_.as_ref(),
                            dme.borrow().populate_source_loc(source_loc, py),
                        ),
                    )?;
                } else {
                    if let Some(init_node) = init {
                        Node::walk(init_node.bind(py), dme, walker, py)?;
                    }
                    if let Some(test_expr) = test {
                        Expression::walk(test_expr.bind(py), dme, walker, py)?;
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
                if walker.hasattr("visit_Var").unwrap() {
                    walker.call_method1(
                        "visit_Var",
                        (
                            self_.as_ref(),
                            dme.borrow().populate_source_loc(source_loc, py),
                        ),
                    )?;
                } else {
                    Expression::walk(name.bind(py), dme, walker, py)?;
                    if let Some(expr) = value {
                        Expression::walk(expr.bind(py), dme, walker, py)?;
                    }
                }
                return Ok(());
            }
            Node::Crash { expr, source_loc } => {
                if walker.hasattr("visit_Crash").unwrap() {
                    walker.call_method1(
                        "visit_Crash",
                        (
                            self_.as_ref(),
                            dme.borrow().populate_source_loc(source_loc, py),
                        ),
                    )?;
                } else if let Some(some_expr) = expr {
                    Expression::walk(some_expr.bind(py), dme, walker, py)?;
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
                if walker.hasattr("visit_ForList").unwrap() {
                    walker.call_method1(
                        "visit_ForList",
                        (
                            self_.as_ref(),
                            dme.borrow().populate_source_loc(source_loc, py),
                        ),
                    )?;
                } else {
                    let loop_decl = Node::Var {
                        name: name.clone_ref(py),
                        value: None,
                        declared_type: var_type.clone(),
                        source_loc: None,
                    };
                    // I guess we pretend that a variable declaration in a for loop is a var statement of sorts
                    Node::walk(&loop_decl.into_pyobject(py)?, dme, walker, py)?;
                    // Expression::walk(name.bind(py), dme, walker, py)?;
                    if let Some(in_list_expr) = in_list {
                        Expression::walk(in_list_expr.bind(py), dme, walker, py)?;
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
                if walker.hasattr("visit_ForRange").unwrap() {
                    walker.call_method1(
                        "visit_ForRange",
                        (
                            self_.as_ref(),
                            dme.borrow().populate_source_loc(source_loc, py),
                        ),
                    )?;
                } else {
                    Expression::walk(name.bind(py), dme, walker, py)?;
                    Expression::walk(start.bind(py), dme, walker, py)?;
                    Expression::walk(end.bind(py), dme, walker, py)?;
                    if let Some(step) = step {
                        Expression::walk(step.bind(py), dme, walker, py)?;
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
                if walker.hasattr("visit_Del").unwrap() {
                    walker.call_method1(
                        "visit_Del",
                        (
                            self_.as_ref(),
                            dme.borrow().populate_source_loc(source_loc, py),
                        ),
                    )?;
                } else {
                    Expression::walk(expr.bind(py), dme, walker, py)?;
                }
            }
            Node::Break { label, source_loc } => {
                if walker.hasattr("visit_Break").unwrap() {
                    walker.call_method1(
                        "visit_Break",
                        (
                            self_.as_ref(),
                            dme.borrow().populate_source_loc(source_loc, py),
                        ),
                    )?;
                } else if let Some(l) = label {
                    Expression::walk(l.bind(py), dme, walker, py)?;
                }
            }
            Node::Setting {
                name,
                mode: _,
                value,
                source_loc,
            } => {
                if walker.hasattr("visit_Setting").unwrap() {
                    walker.call_method1(
                        "visit_Setting",
                        (
                            self_.as_ref(),
                            dme.borrow().populate_source_loc(source_loc, py),
                        ),
                    )?;
                } else {
                    Expression::walk(name.bind(py), dme, walker, py)?;
                    Expression::walk(value.bind(py), dme, walker, py)?;
                }
                return Ok(());
            }
            Node::Spawn {
                delay,
                block,
                source_loc,
            } => {
                if walker.hasattr("visit_Setting").unwrap() {
                    walker.call_method1(
                        "visit_Setting",
                        (
                            self_.as_ref(),
                            dme.borrow().populate_source_loc(source_loc, py),
                        ),
                    )?;
                } else {
                    if let Some(delay) = delay {
                        Expression::walk(delay.bind(py), dme, walker, py)?;
                    }
                    for stmt in block.iter() {
                        Node::walk(stmt.bind(py), dme, walker, py)?;
                    }
                }

                return Ok(());
            }
            Node::Continue { name, source_loc } => {
                if walker.hasattr("visit_Continue").unwrap() {
                    walker.call_method1(
                        "visit_Continue",
                        (
                            self_.as_ref(),
                            dme.borrow().populate_source_loc(source_loc, py),
                        ),
                    )?;
                } else if let Some(name) = name {
                    Expression::walk(name.bind(py), dme, walker, py)?;
                }
            }
            Node::Goto { label, source_loc } => {
                if walker.hasattr("visit_Goto").unwrap() {
                    walker.call_method1(
                        "visit_Goto",
                        (
                            self_.as_ref(),
                            dme.borrow().populate_source_loc(source_loc, py),
                        ),
                    )?;
                } else {
                    Expression::walk(label.bind(py), dme, walker, py)?;
                }
            }
            Node::Label {
                name,
                block,
                source_loc,
            } => {
                if walker.hasattr("visit_Label").unwrap() {
                    walker.call_method1(
                        "visit_Label",
                        (
                            self_.as_ref(),
                            dme.borrow().populate_source_loc(source_loc, py),
                        ),
                    )?;
                } else {
                    Expression::walk(name.bind(py), dme, walker, py)?;
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
                if walker.hasattr("visit_TryCatch").unwrap() {
                    walker.call_method1(
                        "visit_TryCatch",
                        (
                            self_.as_ref(),
                            dme.borrow().populate_source_loc(source_loc, py),
                        ),
                    )?;
                } else {
                    for stmt in try_block.iter() {
                        Node::walk(stmt.bind(py), dme, walker, py)?;
                    }
                    for catch_params in catch_params.iter() {
                        for catch_param in catch_params.iter() {
                            Expression::walk(catch_param.bind(py), dme, walker, py)?;
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
                if walker.hasattr("visit_Switch").unwrap() {
                    walker.call_method1(
                        "visit_Switch",
                        (
                            self_.as_ref(),
                            dme.borrow().populate_source_loc(source_loc, py),
                        ),
                    )?;
                } else {
                    Expression::walk(input.bind(py), dme, walker, py)?;
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
        }

        Ok(())
    }
}
