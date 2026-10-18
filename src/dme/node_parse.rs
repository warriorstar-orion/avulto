use dreammaker::ast::{self, Spanned, Statement};
use pyo3::{IntoPyObject, Py, PyResult, Python, types::PyList};

use crate::{
    dme::{FileData, SourceLoc},
    path::TypePath,
};

use super::{
    expression::Expression,
    nodes::{Node, PyCodeBlock, PyExpr, SwitchCase},
    operators::SettingMode,
};

fn parse_expr(py: Python<'_>, expr: &ast::Expression, fd: &FileData) -> PyResult<PyExpr> {
    Ok(Expression::parse(py, expr, fd).into_pyobject(py)?.unbind())
}

fn parse_block(
    py: Python<'_>,
    stmts: &[Spanned<Statement>],
    fd: &FileData,
) -> PyResult<PyCodeBlock> {
    stmts
        .iter()
        .map(|s| Node::from_statement(py, &s.elem, fd.fill_source_loc(&s.location), fd))
        .collect()
}

impl Node {
    pub fn from_statement(
        py: Python<'_>,
        root: &Statement,
        source_loc: SourceLoc,
        file_data: &FileData,
    ) -> PyResult<Py<Node>> {
        let node = match &root {
            Statement::Var(v) => Self::Var {
                name: Expression::ident(v.name.to_string(), SourceLoc::builtin(), py),
                value: v
                    .value
                    .as_ref()
                    .map(|expr| parse_expr(py, expr, file_data))
                    .transpose()?,
                declared_type: if v.var_type.type_path.is_empty() {
                    None
                } else {
                    Some(TypePath::from_tree_path(&v.var_type.type_path))
                },
                source_loc,
            },
            Statement::Expr(expression) => Self::Expression {
                expr: parse_expr(py, expression, file_data)?,
                source_loc,
            },
            Statement::Return(expression) => Self::Return {
                retval: expression
                    .as_ref()
                    .map(|expr| parse_expr(py, expr, file_data))
                    .transpose()?,
                source_loc,
            },
            Statement::Throw(expression) => Self::Throw {
                expr: parse_expr(py, expression, file_data)?,
                source_loc,
            },
            Statement::While { condition, block } => Self::While {
                condition: parse_expr(py, condition, file_data)?,
                block: parse_block(py, block, file_data)?,
                source_loc,
            },
            Statement::DoWhile { block, condition } => Self::DoWhile {
                condition: parse_expr(py, &condition.elem, file_data)?,
                block: parse_block(py, block, file_data)?,
                source_loc,
            },
            Statement::If { arms, else_arm } => {
                let if_arms = arms
                    .iter()
                    .map(|(cond, stmts)| {
                        Ok((
                            parse_expr(py, &cond.elem, file_data)?,
                            parse_block(py, stmts, file_data)?,
                        ))
                    })
                    .collect::<PyResult<Vec<(PyExpr, PyCodeBlock)>>>()?;

                let else_arm = match else_arm {
                    Some(block) => Some(parse_block(py, block, file_data)?),
                    None => None,
                };

                Self::If {
                    if_arms,
                    else_arm,
                    source_loc,
                }
            }
            Statement::ForInfinite { block } => Self::ForInfinite {
                block: parse_block(py, block, file_data)?,
                source_loc,
            },
            Statement::ForLoop {
                init,
                test,
                inc,
                block,
            } => Self::ForLoop {
                init: init
                    .as_ref()
                    .map(|stmt| Node::from_statement(py, stmt, SourceLoc::builtin(), file_data))
                    .transpose()?,
                test: test
                    .as_ref()
                    .map(|expr| parse_expr(py, expr, file_data))
                    .transpose()?,
                inc: inc
                    .as_ref()
                    .map(|stmt| Node::from_statement(py, stmt, SourceLoc::builtin(), file_data))
                    .transpose()?,
                block: parse_block(py, block, file_data)?,
                source_loc,
            },
            Statement::ForList(for_list_statement) => {
                let mut var_type_path: Option<TypePath> = None;
                if let Some(var_type) = &for_list_statement.var_type {
                    if !var_type.type_path.is_empty() {
                        var_type_path = Some(TypePath::from_tree_path(&var_type.type_path));
                    }
                }
                Self::ForList {
                    name: Expression::ident(
                        for_list_statement.name.to_string(),
                        SourceLoc::builtin(),
                        py,
                    ),
                    in_list: for_list_statement
                        .in_list
                        .as_ref()
                        .map(|expr| parse_expr(py, expr, file_data))
                        .transpose()?,
                    block: parse_block(py, &for_list_statement.block, file_data)?,
                    source_loc,
                    var_type: var_type_path,
                }
            }
            Statement::ForRange(for_range_statement) => Self::ForRange {
                name: Expression::ident(
                    for_range_statement.name.to_string(),
                    SourceLoc::builtin(),
                    py,
                ),
                start: parse_expr(py, &for_range_statement.start, file_data)?,
                end: parse_expr(py, &for_range_statement.end, file_data)?,
                block: parse_block(py, &for_range_statement.block, file_data)?,
                step: for_range_statement
                    .step
                    .as_ref()
                    .map(|expr| parse_expr(py, expr, file_data))
                    .transpose()?,
                source_loc,
            },
            Statement::Vars(vec) => Self::Vars {
                vars: vec
                    .iter()
                    .map(|vs| {
                        Ok(Self::Var {
                            name: Expression::ident(vs.name.to_string(), SourceLoc::builtin(), py),
                            value: vs
                                .value
                                .as_ref()
                                .map(|expr| parse_expr(py, expr, file_data))
                                .transpose()?,
                            declared_type: if vs.var_type.type_path.is_empty() {
                                None
                            } else {
                                Some(TypePath::from_tree_path(&vs.var_type.type_path))
                            },
                            source_loc: source_loc.clone(),
                        }
                        .into_pyobject(py)?
                        .unbind())
                    })
                    .collect::<PyResult<_>>()?,
                source_loc,
            },
            Statement::Setting { name, mode, value } => Self::Setting {
                name: Expression::ident(name.to_string(), SourceLoc::builtin(), py),
                mode: match mode {
                    dreammaker::ast::SettingMode::Assign => SettingMode::Assign,
                    dreammaker::ast::SettingMode::In => SettingMode::In,
                },
                value: parse_expr(py, value, file_data)?,
                source_loc,
            },
            Statement::Spawn { delay, block } => Self::Spawn {
                delay: delay
                    .as_ref()
                    .map(|expr| parse_expr(py, expr, file_data))
                    .transpose()?,
                block: parse_block(py, block, file_data)?,
                source_loc,
            },
            Statement::Switch {
                input,
                cases,
                default,
            } => {
                let mut case_nodes: Vec<Py<SwitchCase>> = vec![];
                for (case, block) in cases.iter() {
                    let mut exact_nodes: Vec<PyExpr> = vec![];
                    let mut range_nodes: Vec<Py<PyList>> = vec![];

                    for case_type in &case.elem {
                        match case_type {
                            dreammaker::ast::Case::Exact(e) => {
                                exact_nodes.push(parse_expr(py, e, file_data)?);
                            }
                            dreammaker::ast::Case::Range(s, e) => {
                                let range_list = PyList::new(
                                    py,
                                    [parse_expr(py, s, file_data)?, parse_expr(py, e, file_data)?],
                                )?;
                                range_nodes.push(range_list.into());
                            }
                        }
                    }
                    case_nodes.push(Py::new(
                        py,
                        SwitchCase {
                            exact: PyList::new(py, exact_nodes)?.unbind(),
                            range: PyList::new(py, range_nodes)?.unbind(),
                            block: parse_block(py, block, file_data)?,
                        },
                    )?);
                }

                Self::Switch {
                    input: parse_expr(py, input, file_data)?,
                    cases: case_nodes,
                    default: default
                        .as_ref()
                        .map(|block| parse_block(py, block, file_data))
                        .transpose()?,
                    source_loc,
                }
            }
            Statement::TryCatch {
                try_block,
                catch_params,
                catch_block,
            } => Self::TryCatch {
                try_block: parse_block(py, try_block, file_data)?,
                catch_block: parse_block(py, catch_block, file_data)?,
                catch_params: catch_params
                    .iter()
                    .map(|tc| {
                        tc.iter()
                            .map(|tcs| Expression::ident(tcs.to_string(), SourceLoc::builtin(), py))
                            .collect()
                    })
                    .collect(),
                source_loc,
            },
            Statement::Continue(name) => Self::Continue {
                name: name
                    .as_ref()
                    .map(|s| Expression::ident(s.clone(), SourceLoc::builtin(), py)),
                source_loc,
            },
            Statement::Break(label) => Self::Break {
                label: label
                    .as_ref()
                    .map(|l| Expression::ident(l.clone(), SourceLoc::builtin(), py)),
                source_loc,
            },
            Statement::Goto(label) => Self::Goto {
                label: Expression::ident(label.to_string(), SourceLoc::builtin(), py),
                source_loc,
            },
            Statement::Label { name, block } => Self::Label {
                name: Expression::ident(name.to_string(), SourceLoc::builtin(), py),
                block: parse_block(py, block, file_data)?,
                source_loc,
            },
            Statement::Del(expression) => Self::Del {
                expr: parse_expr(py, expression, file_data)?,
                source_loc,
            },
            Statement::Crash(expression) => Self::Crash {
                expr: expression
                    .as_ref()
                    .map(|expr| parse_expr(py, expr, file_data))
                    .transpose()?,
                source_loc,
            },
            Statement::ForKeyValue(stmt) => {
                let mut var_type_path: Option<TypePath> = None;
                if let Some(var_type) = &stmt.var_type {
                    if !var_type.type_path.is_empty() {
                        var_type_path = Some(TypePath::from_tree_path(&var_type.type_path));
                    }
                }
                Self::ForKeyValue {
                    key: Expression::ident(stmt.key.to_string(), SourceLoc::builtin(), py),
                    value: Expression::ident(stmt.value.to_string(), SourceLoc::builtin(), py),
                    in_list: stmt
                        .in_list
                        .as_ref()
                        .map(|expr| parse_expr(py, expr, file_data))
                        .transpose()?,
                    block: parse_block(py, &stmt.block, file_data)?,
                    var_type: var_type_path,
                    source_loc,
                }
            }
        };

        Ok(node.into_pyobject(py)?.unbind())
    }
}
