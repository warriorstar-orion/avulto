use dreammaker::{ast::Statement, Location};
use pyo3::{types::PyList, IntoPyObject, Py, Python};

use crate::path::Path;

use super::{
    expression::Expression,
    nodes::{Node, OriginalSourceLocation, PyCodeBlock, PyExpr, SwitchCase},
    operators::SettingMode,
};

impl Node {
    pub fn from_statement(py: Python<'_>, root: &Statement, loc: Option<Location>) -> Py<Self> {
        match &root {
            Statement::Var(v) => Self::Var {
                name: Expression::ident(v.name.to_string(), None, py),
                value: v.value.as_ref().map(|expr| {
                    Expression::parse(py, expr)
                        .into_pyobject(py)
                        .expect("parsing var value")
                        .into()
                }),
                declared_type: if v.var_type.type_path.is_empty() {
                    None
                } else {
                    Some(Path::from_tree_path(&v.var_type.type_path))
                },
                source_loc: loc.map(|l| OriginalSourceLocation::from_location(&l)),
            }
            .into_pyobject(py)
            .expect("parsing var")
            .into(),
            Statement::Expr(expression) => Self::Expression {
                expr: Expression::parse(py, expression)
                    .into_pyobject(py)
                    .expect("parsing statement inner expr")
                    .into(),
                source_loc: loc.map(|l| OriginalSourceLocation::from_location(&l)),
            }
            .into_pyobject(py)
            .expect("parsing statement expr")
            .into(),
            Statement::Return(expression) => Self::Return {
                retval: expression.as_ref().map(|expr| {
                    Expression::parse(py, expr)
                        .into_pyobject(py)
                        .expect("parsing return expr")
                        .into()
                }),
                source_loc: loc.map(|l| OriginalSourceLocation::from_location(&l)),
            }
            .into_pyobject(py)
            .expect("parsing return")
            .into(),
            Statement::Throw(expression) => Self::Throw {
                expr: Expression::parse(py, expression)
                    .into_pyobject(py)
                    .expect("parsing throw expr")
                    .into(),
                source_loc: loc.map(|l| OriginalSourceLocation::from_location(&l)),
            }
            .into_pyobject(py)
            .expect("parsing throw")
            .into(),
            Statement::While { condition, block } => Self::While {
                condition: Expression::parse(py, condition)
                    .into_pyobject(py)
                    .expect("parsing while condition")
                    .into(),
                block: block
                    .iter()
                    .map(|stmt| {
                        Node::from_statement(py, &stmt.elem, Some(stmt.location))
                            .into_pyobject(py)
                            .expect("parsing while block")
                            .into()
                    })
                    .collect(),
                source_loc: loc.map(|l| OriginalSourceLocation::from_location(&l)),
            }
            .into_pyobject(py)
            .expect("parsing while")
            .into(),
            Statement::DoWhile { block, condition } => Self::DoWhile {
                condition: Expression::parse(py, &condition.elem)
                    .into_pyobject(py)
                    .expect("parsing do-while condition")
                    .into(),
                block: block
                    .iter()
                    .map(|stmt| {
                        Node::from_statement(py, &stmt.elem, Some(stmt.location))
                            .into_pyobject(py)
                            .expect("parsing do-while block")
                            .into()
                    })
                    .collect(),
                source_loc: loc.map(|l| OriginalSourceLocation::from_location(&l)),
            }
            .into_pyobject(py)
            .expect("parsing do-while")
            .into(),
            Statement::If { arms, else_arm } => {
                let if_arms: Vec<(PyExpr, PyCodeBlock)> = arms
                    .iter()
                    .map(|(cond, stmts)| {
                        let mut stmt_nodes: PyCodeBlock = vec![];
                        for stmt in stmts.iter() {
                            stmt_nodes.push(
                                Node::from_statement(py, &stmt.elem, Some(stmt.location))
                                    .into_pyobject(py)
                                    .expect("parsing if arm block")
                                    .into(),
                            );
                        }

                        (
                            Expression::parse(py, &cond.elem)
                                .into_pyobject(py)
                                .expect("parsing if condition")
                                .into(),
                            stmt_nodes,
                        )
                    })
                    .collect();
                let mut else_arm_nodes: PyCodeBlock = vec![];
                if let Some(else_arm_block) = else_arm {
                    for stmt in else_arm_block.iter() {
                        else_arm_nodes.push(
                            Node::from_statement(py, &stmt.elem, Some(stmt.location))
                                .into_pyobject(py)
                                .expect("parsing if else block")
                                .into(),
                        );
                    }
                }

                Self::If {
                    if_arms,
                    else_arm: if else_arm_nodes.is_empty() {
                        None
                    } else {
                        Some(else_arm_nodes)
                    },
                    source_loc: loc.map(|l| OriginalSourceLocation::from_location(&l)),
                }
                .into_pyobject(py)
                .expect("parsing if")
                .into()
            }
            Statement::ForInfinite { block } => Self::ForInfinite {
                block: block
                    .iter()
                    .map(|stmt| {
                        Node::from_statement(py, &stmt.elem, Some(stmt.location))
                            .into_pyobject(py)
                            .expect("parsing for infinite block")
                            .into()
                    })
                    .collect(),
                source_loc: loc.map(|l| OriginalSourceLocation::from_location(&l)),
            }
            .into_pyobject(py)
            .expect("parsing for infinite")
            .into(),
            Statement::ForLoop {
                init,
                test,
                inc,
                block,
            } => Self::ForLoop {
                init: init.as_ref().map(|stmt| {
                    Node::from_statement(py, stmt, None)
                        .into_pyobject(py)
                        .expect("parsing for loop init")
                        .into()
                }),
                test: test.as_ref().map(|expr| {
                    Expression::parse(py, expr)
                        .into_pyobject(py)
                        .expect("parsing for loop test")
                        .into()
                }),
                inc: inc.as_ref().map(|stmt| {
                    Node::from_statement(py, stmt, None)
                        .into_pyobject(py)
                        .expect("parsing for loop inc")
                        .into()
                }),
                block: block
                    .iter()
                    .map(|stmt| {
                        Node::from_statement(py, &stmt.elem, Some(stmt.location))
                            .into_pyobject(py)
                            .expect("parsing for loop block")
                            .into()
                    })
                    .collect(),
                source_loc: loc.map(|l| OriginalSourceLocation::from_location(&l)),
            }
            .into_pyobject(py)
            .expect("parsing for loop")
            .into(),
            Statement::ForList(for_list_statement) => {
                let mut var_type_path: Option<Path> = None;
                if let Some(var_type) = &for_list_statement.var_type {
                    if !var_type.type_path.is_empty() {
                        var_type_path = Some(Path::from_tree_path(&var_type.type_path));
                    }
                }
                Self::ForList {
                    name: Expression::ident(for_list_statement.name.to_string(), None, py),
                    in_list: for_list_statement.in_list.as_ref().map(|expr| {
                        Expression::parse(py, expr)
                            .into_pyobject(py)
                            .expect("parsing for list in-list")
                            .into()
                    }),
                    block: for_list_statement
                        .block
                        .iter()
                        .map(|stmt| {
                            Node::from_statement(py, &stmt.elem, Some(stmt.location))
                                .into_pyobject(py)
                                .expect("parsing for list block")
                                .into()
                        })
                        .collect(),
                    source_loc: loc.map(|l| OriginalSourceLocation::from_location(&l)),
                    var_type: var_type_path,
                }
                .into_pyobject(py)
                .expect("parsing for list")
                .into()
            }
            Statement::ForRange(for_range_statement) => Self::ForRange {
                name: Expression::ident(for_range_statement.name.to_string(), None, py),
                start: Expression::parse(py, &for_range_statement.start)
                    .into_pyobject(py)
                    .expect("parsing for range start")
                    .into(),
                end: Expression::parse(py, &for_range_statement.end)
                    .into_pyobject(py)
                    .expect("parsing for range end")
                    .into(),
                block: for_range_statement
                    .block
                    .iter()
                    .map(|stmt| {
                        Node::from_statement(py, &stmt.elem, Some(stmt.location))
                            .into_pyobject(py)
                            .expect("parsing for range block")
                            .into()
                    })
                    .collect(),
                step: for_range_statement.step.as_ref().map(|expr| {
                    Expression::parse(py, expr)
                        .into_pyobject(py)
                        .expect("parsing for range step")
                        .into()
                }),
                source_loc: loc.map(|l| OriginalSourceLocation::from_location(&l)),
            }
            .into_pyobject(py)
            .expect("parsing for range")
            .into(),
            Statement::Vars(vec) => Self::Vars {
                vars: vec
                    .iter()
                    .map(|vs| {
                        Self::Var {
                            name: Expression::ident(vs.name.to_string(), None, py),
                            value: vs.value.as_ref().map(|expr| {
                                Expression::parse(py, expr)
                                    .into_pyobject(py)
                                    .expect("parsing vars stmt value")
                                    .into()
                            }),
                            declared_type: if vs.var_type.type_path.is_empty() {
                                None
                            } else {
                                Some(Path::from_tree_path(&vs.var_type.type_path))
                            },
                            source_loc: loc.map(|l| OriginalSourceLocation::from_location(&l)),
                        }
                        .into_pyobject(py)
                        .expect("parsing vars var statement")
                        .into()
                    })
                    .collect(),
                source_loc: loc.map(|l| OriginalSourceLocation::from_location(&l)),
            }
            .into_pyobject(py)
            .expect("parsing vars")
            .into(),
            Statement::Setting { name, mode, value } => Self::Setting {
                name: Expression::ident(name.to_string(), None, py),
                mode: match mode {
                    dreammaker::ast::SettingMode::Assign => SettingMode::Assign,
                    dreammaker::ast::SettingMode::In => SettingMode::In,
                },
                value: Expression::parse(py, value)
                    .into_pyobject(py)
                    .expect("parsing setting value")
                    .into(),
                source_loc: loc.map(|l| OriginalSourceLocation::from_location(&l)),
            }
            .into_pyobject(py)
            .expect("parsing setting")
            .into(),
            Statement::Spawn { delay, block } => Self::Spawn {
                delay: delay.as_ref().map(|expr| {
                    Expression::parse(py, expr)
                        .into_pyobject(py)
                        .expect("parsing spawn delay")
                        .into()
                }),
                block: block
                    .iter()
                    .map(|stmt| {
                        Node::from_statement(py, &stmt.elem, Some(stmt.location))
                            .into_pyobject(py)
                            .expect("parsing spawn block")
                            .into()
                    })
                    .collect::<PyCodeBlock>(),
                source_loc: loc.map(|l| OriginalSourceLocation::from_location(&l)),
            }
            .into_pyobject(py)
            .expect("parsing spawn")
            .into(),
            Statement::Switch {
                input,
                cases,
                default,
            } => {
                let input_expr = Expression::parse(py, input)
                    .into_pyobject(py)
                    .expect("parsing switch input")
                    .into();
                let mut case_nodes: Vec<Py<SwitchCase>> = vec![];
                for (case, block) in cases.iter() {
                    let mut exact_nodes: Vec<PyExpr> = vec![];
                    let mut range_nodes: Vec<Py<PyList>> = vec![];

                    for case_type in &case.elem {
                        match case_type {
                            dreammaker::ast::Case::Exact(e) => {
                                exact_nodes.push(
                                    Expression::parse(py, e)
                                        .into_pyobject(py)
                                        .expect("parsing switch exact case")
                                        .into(),
                                );
                            }
                            dreammaker::ast::Case::Range(s, e) => {
                                let range_list = PyList::new(
                                    py,
                                    [
                                        Expression::parse(py, s)
                                            .into_pyobject(py)
                                            .expect("parsing switch range case"),
                                        Expression::parse(py, e)
                                            .into_pyobject(py)
                                            .expect("parsing switch range case"),
                                    ],
                                )
                                .expect("parsing switch range case");
                                range_nodes.push(range_list.into());
                            }
                        }
                    }
                    case_nodes.push(
                        SwitchCase {
                            exact: PyList::new(py, exact_nodes)
                                .expect("parsing switch-case exact")
                                .into(),
                            range: PyList::new(py, range_nodes)
                                .expect("parsing switch-case range")
                                .into(),
                            block: block
                                .iter()
                                .map(|stmt| {
                                    Node::from_statement(py, &stmt.elem, Some(stmt.location))
                                        .into_pyobject(py)
                                        .expect("parsing switch-case block")
                                        .into()
                                })
                                .collect(),
                        }
                        .into_pyobject(py)
                        .expect("parsing switch-case node")
                        .into(),
                    );
                }

                Self::Switch {
                    input: input_expr,
                    cases: case_nodes,
                    default: default.as_ref().map(|stmts| {
                        stmts
                            .iter()
                            .map(|stmt| {
                                Node::from_statement(py, &stmt.elem, Some(stmt.location))
                                    .into_pyobject(py)
                                    .expect("parsing switch default")
                                    .into()
                            })
                            .collect()
                    }),
                    source_loc: loc.map(|l| OriginalSourceLocation::from_location(&l)),
                }
                .into_pyobject(py)
                .expect("parsing switch")
                .into()
            }
            Statement::TryCatch {
                try_block,
                catch_params,
                catch_block,
            } => Self::TryCatch {
                try_block: try_block
                    .iter()
                    .map(|stmt| {
                        Node::from_statement(py, &stmt.elem, Some(stmt.location))
                            .into_pyobject(py)
                            .expect("parsing trycatch try block")
                            .into()
                    })
                    .collect(),
                catch_block: catch_block
                    .iter()
                    .map(|stmt| {
                        Node::from_statement(py, &stmt.elem, Some(stmt.location))
                            .into_pyobject(py)
                            .expect("parsing trycatch catch block")
                            .into()
                    })
                    .collect(),
                catch_params: catch_params
                    .iter()
                    .map(|tc| {
                        tc.iter()
                            .map(|tcs| Expression::ident(tcs.to_string(), None, py))
                            .collect()
                    })
                    .collect(),
                source_loc: loc.map(|l| OriginalSourceLocation::from_location(&l)),
            }
            .into_pyobject(py)
            .expect("parsing trycatch")
            .into(),
            Statement::Continue(name) => Self::Continue {
                name: name
                    .as_ref()
                    .map(|s| Expression::ident(s.clone(), None, py)),
                source_loc: loc.map(|l| OriginalSourceLocation::from_location(&l)),
            }
            .into_pyobject(py)
            .expect("parsing continue")
            .into(),
            Statement::Break(label) => Self::Break {
                label: label
                    .as_ref()
                    .map(|l| Expression::ident(l.clone(), None, py)),
                source_loc: loc.map(|l| OriginalSourceLocation::from_location(&l)),
            }
            .into_pyobject(py)
            .expect("parsing break")
            .into(),
            Statement::Goto(label) => Self::Goto {
                label: Expression::ident(label.to_string(), None, py),
                source_loc: loc.map(|l| OriginalSourceLocation::from_location(&l)),
            }
            .into_pyobject(py)
            .expect("parsing goto")
            .into(),
            Statement::Label { name, block } => Self::Label {
                name: Expression::ident(name.to_string(), None, py),
                block: block
                    .iter()
                    .map(|stmt| {
                        Node::from_statement(py, &stmt.elem, Some(stmt.location))
                            .into_pyobject(py)
                            .expect("parsing label block")
                            .into()
                    })
                    .collect(),
                source_loc: loc
                    .map(|l| Py::new(py, OriginalSourceLocation::from_location(&l)).unwrap()),
            }
            .into_pyobject(py)
            .expect("parsing label")
            .into(),
            Statement::Del(expression) => Self::Del {
                expr: Expression::parse(py, expression)
                    .into_pyobject(py)
                    .expect("parsing del expr")
                    .into(),
                source_loc: loc
                    .map(|l| Py::new(py, OriginalSourceLocation::from_location(&l)).unwrap()),
            }
            .into_pyobject(py)
            .expect("parsing del")
            .into(),
            Statement::Crash(expression) => Self::Crash {
                expr: expression.as_ref().map(|expr| {
                    Expression::parse(py, expr)
                        .into_pyobject(py)
                        .expect("parsing crash expr")
                        .into()
                }),
                source_loc: loc
                    .map(|l| Py::new(py, OriginalSourceLocation::from_location(&l)).unwrap()),
            }
            .into_pyobject(py)
            .expect("parsing crash")
            .into(),
        }
    }
}
