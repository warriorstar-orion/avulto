extern crate dreammaker;

use std::vec;

use dreammaker::ast::Block;
use pyo3::{
    types::{PyAnyMethods, PyDict, PyList},
    Bound, IntoPy, Py, PyAny, PyResult, Python, ToPyObject,
};

use crate::{dmlist::DmList, path::Path};

use super::nodes::{
    Assignment, Attribute, BinaryOp, Break, Call, Continue, Del, DoWhile, DynamicCall, ExternalCall, ForList, ForLoop, ForRange, Identifier, If, IfArm, Index, Input, InterpString, Label, Locate, MiniExpr, NewImplicit, NewMiniExpr, NewPrefab, ParentCall, Prefab, Return, SelfCall, Setting, Spawn, Switch, SwitchCase, Ternary, Throw, TryCatch, UnaryOp, While
};

pub fn from_block_to_stmt_node_list(block: &Block, py: Python<'_>) -> PyResult<Py<PyAny>> {
    let mut stmt_nodes: Vec<Py<PyAny>> = vec![];
    for stmt in block.iter() {
        stmt_nodes.push(from_statement_to_node(&stmt.elem, py)?);
    }
    Ok(PyList::new_bound(py, stmt_nodes).into_py(py))
}

pub fn from_statement_to_node(
    stmt: &dreammaker::ast::Statement,
    py: Python<'_>,
) -> PyResult<Py<PyAny>> {
    match stmt {
        dreammaker::ast::Statement::Expr(e) => from_expression_to_node(e, py),
        dreammaker::ast::Statement::Return(e) => {
            let retval = if let Some(rexpr) = e {
                from_expression_to_node(rexpr, py)?
            } else {
                py.None()
            };
            Return::make(py, retval)
        }
        dreammaker::ast::Statement::Throw(t) => {
            let throw_expr = from_expression_to_node(t, py)?;
            Throw::make(py, throw_expr)
        },
        dreammaker::ast::Statement::While { condition, block } => {
            let cond_expr = from_expression_to_node(condition, py)?;
            let stmt_list = from_block_to_stmt_node_list(block, py)?;
            While::make(py, cond_expr, stmt_list)
        }
        dreammaker::ast::Statement::DoWhile { block, condition } => {
            let cond_expr = from_expression_to_node(&condition.elem, py)?;
            let stmt_list = from_block_to_stmt_node_list(block, py)?;
            DoWhile::make(py, cond_expr, stmt_list)
        }
        dreammaker::ast::Statement::If { arms, else_arm } => {
            let if_arms: Vec<Py<PyAny>> = arms
                .iter()
                .map(|(cond, stmts)| {
                    let mut stmt_nodes: Vec<Py<PyAny>> = vec![];
                    for stmt in stmts.iter() {
                        stmt_nodes.push(from_statement_to_node(&stmt.elem, py).unwrap());
                    }

                    IfArm::make(
                        py,
                        from_expression_to_node(&cond.elem, py).unwrap(),
                        PyList::new_bound(py, stmt_nodes).into_py(py),
                    )
                    .unwrap()
                })
                .collect();
            let mut else_arm_node = py.None();
            if let Some(else_arm_block) = else_arm {
                let mut else_arm_nodes: Vec<Py<PyAny>> = vec![];
                for stmt in else_arm_block.iter() {
                    else_arm_nodes.push(from_statement_to_node(&stmt.elem, py)?);
                }
                else_arm_node = PyList::new_bound(py, else_arm_nodes).into_py(py);
            }
            If::make(
                py,
                PyList::new_bound(py, if_arms).into_py(py),
                else_arm_node,
            )
        }
        dreammaker::ast::Statement::ForInfinite { .. } => todo!(),
        dreammaker::ast::Statement::ForLoop {
            init,
            test,
            inc,
            block,
        } => {
            let mut init_node = py.None();
            let mut test_node = py.None();
            let mut inc_node = py.None();
            if let Some(init_stmt_boxed) = init {
                init_node = from_statement_to_node(init_stmt_boxed, py)?;
            }
            if let Some(test_expr_boxed) = test {
                test_node = from_expression_to_node(test_expr_boxed, py)?;
            }
            if let Some(inc_stmt_boxed) = inc {
                inc_node = from_statement_to_node(inc_stmt_boxed, py)?;
            }
            let stmt_list = from_block_to_stmt_node_list(block, py)?;
            ForLoop::make(py, init_node, test_node, inc_node, stmt_list)
        }
        dreammaker::ast::Statement::ForList(l) => {
            let mut in_list = py.None();
            if let Some(in_list_expr) = &l.in_list {
                in_list = from_expression_to_node(in_list_expr, py)?;
            }
            let mut stmt_nodes: Vec<Py<PyAny>> = vec![];
            for stmt in l.block.iter() {
                stmt_nodes.push(from_statement_to_node(&stmt.elem, py).unwrap());
            }
            let stmt_list = PyList::new_bound(py, stmt_nodes).into_py(py);
            ForList::make(
                py,
                Identifier {
                    ident: l.name.clone().into_py(py),
                }
                .into_py(py),
                in_list,
                stmt_list,
            )
        }
        dreammaker::ast::Statement::ForRange(f) => {
            let name = Identifier {
                ident: f.name.into_py(py),
            }
            .into_py(py);
            let start_expr = from_expression_to_node(&f.start, py)?;
            let end_expr = from_expression_to_node(&f.end, py)?;
            let mut step_node = py.None();
            if let Some(step_expr) = &f.step {
                step_node = from_expression_to_node(step_expr, py)?;
            }
            let stmt_list = from_block_to_stmt_node_list(&f.block, py)?;
            ForRange::make(py, name, start_expr, end_expr, step_node, stmt_list)
        }
        dreammaker::ast::Statement::Var(_) => todo!(),
        dreammaker::ast::Statement::Vars(_) => todo!(),
        dreammaker::ast::Statement::Setting { name, mode, value } => Setting::make(
            py,
            Identifier {
                ident: name.into_py(py),
            }
            .into_py(py),
            mode,
            from_expression_to_node(value, py)?,
        ),
        dreammaker::ast::Statement::Spawn { delay, block } => {
            let mut delay_node = py.None();
            if let Some(delay_expr) = delay {
                delay_node = from_expression_to_node(delay_expr, py)?;
            }
            let stmt_list = from_block_to_stmt_node_list(block, py)?;
            Spawn::make(py, delay_node, stmt_list)
        }
        dreammaker::ast::Statement::Switch {
            input,
            cases,
            default,
        } => {
            let input_expr = from_expression_to_node(input, py)?;
            let mut case_nodes: Vec<Py<PyAny>> = vec![];
            for (case, block) in cases.iter() {
                let mut exact_nodes: Vec<Py<PyAny>> = vec![];
                let mut range_nodes: Vec<Py<PyAny>> = vec![];

                for case_type in &case.elem {
                    match case_type {
                        dreammaker::ast::Case::Exact(e) => {
                            exact_nodes.push(from_expression_to_node(e, py)?);
                        }
                        dreammaker::ast::Case::Range(s, e) => {
                            let range_list = PyList::new_bound(
                                py,
                                [
                                    from_expression_to_node(s, py)?,
                                    from_expression_to_node(e, py)?,
                                ],
                            )
                            .into_py(py);
                            range_nodes.push(range_list);
                        }
                    }
                }
                let stmt_list = from_block_to_stmt_node_list(block, py)?;
                case_nodes.push(SwitchCase::make(
                    py,
                    PyList::new_bound(py, exact_nodes).into_py(py),
                    PyList::new_bound(py, range_nodes).into_py(py),
                    stmt_list,
                )?);
            }
            let case_list = PyList::new_bound(py, case_nodes).into_py(py);

            let mut default_nodes: Vec<Py<PyAny>> = vec![];
            if let Some(default_block) = default {
                for stmt in default_block.iter() {
                    default_nodes.push(from_statement_to_node(&stmt.elem, py)?);
                }
            }
            let default_list = PyList::new_bound(py, default_nodes).into_py(py);

            Switch::make(py, input_expr, case_list, default_list)
        }
        dreammaker::ast::Statement::TryCatch {
            try_block,
            catch_params,
            catch_block,
        } => {
            let try_stmts = from_block_to_stmt_node_list(try_block, py)?;
            let mut catch_str_lists: Vec<Py<PyAny>> = vec![];
            for catch_str_list in catch_params.iter() {
                let mut catch_str_list_items = vec![];
                for catch_str_list_item in catch_str_list.iter() {
                    catch_str_list_items.push(catch_str_list_item.into_py(py));
                }
                catch_str_lists.push(PyList::new_bound(py, catch_str_list_items).into_py(py));
            }
            let catch_str_pylist = PyList::new_bound(py, catch_str_lists).into_py(py);
            let catch_stmts = from_block_to_stmt_node_list(catch_block, py)?;
            TryCatch::make(py, try_stmts, catch_str_pylist, catch_stmts)
        }
        dreammaker::ast::Statement::Continue(c) => Continue::make(
            py,
            match c {
                Some(s) => s.into_py(py),
                None => py.None(),
            },
        ),
        dreammaker::ast::Statement::Break(b) => Break::make(
            py,
            match b {
                Some(s) => s.into_py(py),
                None => py.None(),
            },
        ),
        dreammaker::ast::Statement::Goto(_) => todo!(),
        dreammaker::ast::Statement::Label { name, block } => {
            let mut stmt_nodes: Vec<Py<PyAny>> = vec![];
            for stmt in block.iter() {
                stmt_nodes.push(from_statement_to_node(&stmt.elem, py).unwrap());
            }
            let stmt_list = PyList::new_bound(py, stmt_nodes).into_py(py);

            Label::make(py, name.into_py(py), stmt_list)
        }
        dreammaker::ast::Statement::Del(expr) => {
            Del::make(py, from_expression_to_node(expr, py)?)
        },
        dreammaker::ast::Statement::Crash(_) => todo!(),
    }
}

pub fn make_prefab_node(p: &dreammaker::ast::Prefab, py: Python<'_>) -> PyResult<Py<PyAny>> {
    let mut path: String = "".to_owned();
    for (op, val) in p.path.iter() {
        path.push_str(format!("{}{}", op, val).as_str());
    }
    let pypath = Path(path);
    let mut out: Vec<Bound<PyDict>> = Vec::new();

    for (k, v) in p.vars.iter() {
        let var = PyDict::new_bound(py);
        var.set_item(k.as_str(), from_expression_to_node(v, py).unwrap())?;
        out.push(var);
    }

    Prefab::make(
        py,
        pypath.into_py(py),
        PyList::new_bound(py, out).to_object(py).clone_ref(py),
    )
}

pub fn from_expression_to_node(
    expr: &dreammaker::ast::Expression,
    py: Python<'_>,
) -> PyResult<Py<PyAny>> {
    match expr {
        dreammaker::ast::Expression::Base { term, follow } => {
            let mut original_term = match &term.elem {
                dreammaker::ast::Term::Null => py.None(),
                dreammaker::ast::Term::Int(i) => i.into_py(py),
                dreammaker::ast::Term::Float(f) => f.into_py(py),
                dreammaker::ast::Term::Ident(i) => Identifier {
                    ident: i.into_py(py),
                }
                .into_py(py),
                dreammaker::ast::Term::String(s) => s.into_py(py),
                dreammaker::ast::Term::Resource(r) => r.into_py(py),
                dreammaker::ast::Term::As(_) => todo!(),
                dreammaker::ast::Term::__PROC__ => todo!(),
                dreammaker::ast::Term::__TYPE__ => todo!(),
                dreammaker::ast::Term::__IMPLIED_TYPE__ => todo!(),
                dreammaker::ast::Term::Expr(t_expr) => from_expression_to_node(t_expr, py)?,
                dreammaker::ast::Term::Prefab(p) => make_prefab_node(p, py)?,
                dreammaker::ast::Term::InterpString(ident, tokens) => {
                    let mut token_vec: Vec<Py<PyAny>> = vec![];
                    for (maybe_token_expr, token_str) in tokens.iter() {
                        let mut token_expr_node = py.None();
                        if let Some(token_expr) = maybe_token_expr {
                            token_expr_node = from_expression_to_node(token_expr, py)?;
                        }
                        token_vec.push(
                            PyList::new_bound(py, [token_expr_node, token_str.into_py(py)])
                                .into_py(py),
                        );
                    }
                    let tokens = PyList::new_bound(py, token_vec).into_py(py);
                    InterpString::make(py, ident.into_py(py), tokens)?
                }
                dreammaker::ast::Term::Call(ident, argexprs) => {
                    let mut args: Vec<Py<PyAny>> = vec![];
                    for argexpr in argexprs.iter() {
                        args.push(from_expression_to_node(argexpr, py)?);
                    }
                    Call::make(
                        py,
                        py.None(),
                        ident.into_py(py),
                        PyList::new_bound(py, args).into_py(py),
                    )?
                }
                dreammaker::ast::Term::SelfCall(argexprs) => {
                    let mut args: Vec<Py<PyAny>> = vec![];
                    for argexpr in argexprs.iter() {
                        args.push(from_expression_to_node(argexpr, py)?);
                    }
                    SelfCall::make(py, PyList::new_bound(py, args).into_py(py))?
                }
                dreammaker::ast::Term::ParentCall(argexprs) => {
                    let mut args: Vec<Py<PyAny>> = vec![];
                    for argexpr in argexprs.iter() {
                        args.push(from_expression_to_node(argexpr, py)?);
                    }
                    ParentCall::make(py, PyList::new_bound(py, args).into_py(py))?
                }
                dreammaker::ast::Term::NewImplicit { args } => {
                    let mut arglist: Vec<Py<PyAny>> = vec![];
                    if let Some(exprs_boxed) = args {
                        for argexpr in exprs_boxed.iter() {
                            arglist.push(from_expression_to_node(argexpr, py)?);
                        }
                    }
                    let arg_pylist = PyList::new_bound(py, arglist).into_py(py);
                    NewImplicit::make(py, arg_pylist)?
                }
                dreammaker::ast::Term::NewPrefab { prefab, args } => {
                    let mut arglist: Vec<Py<PyAny>> = vec![];
                    if let Some(exprs_boxed) = args {
                        for argexpr in exprs_boxed.iter() {
                            arglist.push(from_expression_to_node(argexpr, py)?);
                        }
                    }
                    let arg_pylist = PyList::new_bound(py, arglist).into_py(py);
                    NewPrefab::make(py, make_prefab_node(prefab, py)?, arg_pylist)?
                }
                dreammaker::ast::Term::NewMiniExpr { expr, args } => {
                    let mini_expr_ident = expr.ident.into_py(py);
                    let mut field_nodes: Vec<Py<PyAny>> = vec![];
                    for field in expr.fields.iter() {
                        field_nodes.push(field.ident.into_py(py));
                    }
                    let field_pylist = PyList::new_bound(py, field_nodes).into_py(py);
                    let mut arglist: Vec<Py<PyAny>> = vec![];
                    if let Some(exprs_boxed) = args {
                        for argexpr in exprs_boxed.iter() {
                            arglist.push(from_expression_to_node(argexpr, py)?);
                        }
                    }
                    let arg_pylist = PyList::new_bound(py, arglist).into_py(py);
                    let mini_expr = MiniExpr::make(py, mini_expr_ident, field_pylist)?;
                    NewMiniExpr::make(py, mini_expr, arg_pylist)?
                }
                dreammaker::ast::Term::List(l) => {
                    let mut keys: Vec<Py<PyAny>> = vec![];
                    let mut vals: Vec<Py<PyAny>> = vec![];

                    for args in l.iter() {
                        match args {
                            dreammaker::ast::Expression::Base { .. } => {
                                keys.push(from_expression_to_node(args, py)?.clone_ref(py));
                                vals.push(py.None());
                            }
                            dreammaker::ast::Expression::AssignOp { op: _, lhs, rhs } => {
                                keys.push(from_expression_to_node(lhs, py)?.clone_ref(py));
                                vals.push(from_expression_to_node(rhs, py)?.clone_ref(py));
                            }
                            dreammaker::ast::Expression::BinaryOp { .. } => {
                                keys.push(from_expression_to_node(args, py)?.clone_ref(py));
                                vals.push(py.None());
                            }
                            dreammaker::ast::Expression::TernaryOp { cond, if_, else_ } => {
                                keys.push(Ternary::make(
                                    py,
                                    from_expression_to_node(cond, py)?,
                                    from_expression_to_node(if_, py)?,
                                    from_expression_to_node(else_, py)?,
                                )?);
                                vals.push(py.None());
                            },
                        }
                    }
                    DmList { keys, vals }.into_py(py).clone_ref(py)
                }
                dreammaker::ast::Term::Input {
                    args,
                    input_type,
                    in_list,
                } => {
                    let mut arg_nodes: Vec<Py<PyAny>> = vec![];
                    for expr in args.iter() {
                        arg_nodes.push(from_expression_to_node(expr, py).unwrap());
                    }
                    let arglist = PyList::new_bound(py, arg_nodes).into_py(py);
                    let mut input_type_val = 0;
                    if let Some(input_type_) = input_type {
                        input_type_val = input_type_.bits();
                    }
                    let mut in_list_node = py.None();
                    if let Some(in_list_expr) = in_list {
                        in_list_node = from_expression_to_node(in_list_expr, py)?;
                    }
                    Input::make(py, arglist, input_type_val.into_py(py), in_list_node)?
                }
                dreammaker::ast::Term::Locate { args, in_list } => {
                    let mut arg_nodes: Vec<Py<PyAny>> = vec![];
                    for expr in args.iter() {
                        arg_nodes.push(from_expression_to_node(expr, py).unwrap());
                    }
                    let arglist = PyList::new_bound(py, arg_nodes).into_py(py);
                    let mut in_list_node = py.None();
                    if let Some(in_list_expr) = in_list {
                        in_list_node = from_expression_to_node(in_list_expr, py)?;
                    }
                    Locate::make(py, arglist, in_list_node)?
                }
                dreammaker::ast::Term::Pick(argexprs) => {
                    let mut args: Vec<Bound<PyList>> = vec![];
                    for (weight, val) in argexprs.iter() {
                        let mut weight_node = py.None();
                        if let Some(weight_expr) = weight {
                            weight_node = from_expression_to_node(weight_expr, py)?;
                        }
                        let l: Bound<PyList> =
                            PyList::new_bound(py, [weight_node, from_expression_to_node(val, py)?]);
                        args.push(l);
                    }

                    Call::make(
                        py,
                        py.None(),
                        "pick".into_py(py),
                        PyList::new_bound(py, args).into_py(py),
                    )?
                }
                dreammaker::ast::Term::DynamicCall(proc_exprs, args) => {
                    let mut arglist: Vec<Py<PyAny>> = vec![];
                    for argexpr in args.iter() {
                        arglist.push(from_expression_to_node(argexpr, py)?);
                    }
                    let arg_pylist = PyList::new_bound(py, arglist).into_py(py);
                    let mut proclist: Vec<Py<PyAny>> = vec![];
                    for proc_expr in proc_exprs.iter() {
                        proclist.push(from_expression_to_node(proc_expr, py)?);
                    }
                    let proc_pylist = PyList::new_bound(py, proclist).into_py(py);
                    DynamicCall::make(py, proc_pylist, arg_pylist)?
                }
                dreammaker::ast::Term::ExternalCall {
                    library_name,
                    function_name,
                    args,
                } => {
                    let library_expr = from_expression_to_node(library_name, py)?;
                    let function_expr = from_expression_to_node(function_name, py)?;
                    let mut arglist: Vec<Py<PyAny>> = vec![];
                    for argexpr in args.iter() {
                        arglist.push(from_expression_to_node(argexpr, py)?);
                    }
                    let arg_pylist = PyList::new_bound(py, arglist).into_py(py);
                    ExternalCall::make(py, library_expr, function_expr, arg_pylist)?
                },
                dreammaker::ast::Term::GlobalIdent(_) => todo!(),
                dreammaker::ast::Term::GlobalCall(_, _) => todo!(),
            };

            for f in follow.iter() {
                match &f.elem {
                    dreammaker::ast::Follow::Index(_access_kind, expr) => {
                        original_term =
                            Index::make(py, original_term, from_expression_to_node(expr, py)?)?;
                    }
                    // TODO(wso): Support PropertyAccessKind
                    dreammaker::ast::Follow::Field(_access_kind, ident) => {
                        original_term =
                            Attribute::make(py, original_term, ident.into_py(py)).unwrap();
                    }
                    dreammaker::ast::Follow::Call(_access_kind, ident, argexprs) => {
                        let mut args: Vec<Py<PyAny>> = vec![];
                        for arg in argexprs.iter() {
                            args.push(from_expression_to_node(arg, py)?);
                        }
                        original_term = Call::make(
                            py,
                            original_term,
                            ident.into_py(py),
                            PyList::new_bound(py, args).into_py(py),
                        )?;
                    }
                    dreammaker::ast::Follow::Unary(u) => {
                        original_term = UnaryOp::make(py, original_term, u)?;
                    }
                    dreammaker::ast::Follow::StaticField(_) => todo!(),
                    dreammaker::ast::Follow::ProcReference(_) => todo!(),
                }
            }

            Ok(original_term)
        }
        dreammaker::ast::Expression::BinaryOp { op, lhs, rhs } => {
            let node = BinaryOp::make(
                py,
                from_expression_to_node(lhs, py)?,
                from_expression_to_node(rhs, py)?,
                op,
            )?;
            Ok(node)
        }
        dreammaker::ast::Expression::AssignOp { op, lhs, rhs } => {
            let node = Assignment::make(
                py,
                from_expression_to_node(lhs, py)?,
                from_expression_to_node(rhs, py)?,
                op,
            )
            .unwrap();
            Ok(node)
        }
        dreammaker::ast::Expression::TernaryOp { cond, if_, else_ } => {
            let node = Ternary::make(
                py,
                from_expression_to_node(cond, py)?,
                from_expression_to_node(if_, py)?,
                from_expression_to_node(else_, py)?,
            )?;
            Ok(node)
        }
    }
}
