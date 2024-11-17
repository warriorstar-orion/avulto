use pyo3::{types::PyAnyMethods, Bound, PyAny, PyResult, Python};

use super::{expression::Expression, nodes::visit_constant, prefab::Prefab, Dme};

impl Expression {
    pub fn walk(
        self_: &Bound<Self>,
        dme: &Bound<Dme>,
        walker: &Bound<PyAny>,
        py: Python<'_>,
    ) -> PyResult<()> {
        if walker.hasattr("visit_Expr").unwrap() {
            walker.call_method1("visit_Expr", (self_, py.None()))?;

            return Ok(());
        }

        let self_expr = self_.get();
        match self_expr {
            Expression::Constant { constant, .. } => {
                visit_constant(constant, walker)?;
                Ok(())
            }
            Expression::Identifier {
                name: _,
                source_loc,
            } => {
                if walker.hasattr("visit_Identifier").unwrap() {
                    walker.call_method1(
                        "visit_Identifier",
                        (self_, dme.borrow().populate_source_loc(source_loc, py)),
                    )?;
                }
                Ok(())
            }
            Expression::BinaryOp {
                op: _,
                lhs,
                rhs,
                source_loc,
            } => {
                if walker.hasattr("visit_BinaryOp").unwrap() {
                    walker
                        .call_method1(
                            "visit_BinaryOp",
                            (self_, dme.borrow().populate_source_loc(source_loc, py)),
                        )
                        .expect("failed to walk binary op");
                } else {
                    Expression::walk(lhs.bind(py), dme, walker, py).expect("bad binary op lhs");
                    Expression::walk(rhs.bind(py), dme, walker, py).expect("bad binary op rhs");
                }

                Ok(())
            }
            Expression::AssignOp {
                op: _,
                lhs,
                rhs,
                source_loc,
            } => {
                if walker.hasattr("visit_AssignOp").unwrap() {
                    walker.call_method1(
                        "visit_AssignOp",
                        (self_, dme.borrow().populate_source_loc(source_loc, py)),
                    )?;
                } else {
                    Expression::walk(lhs.bind(py), dme, walker, py)?;
                    Expression::walk(rhs.bind(py), dme, walker, py)?;
                }

                Ok(())
            }
            Expression::TernaryOp {
                cond,
                if_expr,
                else_expr,
                source_loc,
            } => {
                if walker.hasattr("visit_TernaryOp").unwrap() {
                    walker.call_method1(
                        "visit_TernaryOp",
                        (self_, dme.borrow().populate_source_loc(source_loc, py)),
                    )?;
                } else {
                    Expression::walk(cond.bind(py), dme, walker, py)?;
                    Expression::walk(if_expr.bind(py), dme, walker, py)?;
                    Expression::walk(else_expr.bind(py), dme, walker, py)?;
                }

                Ok(())
            }
            Expression::List { list, source_loc } => {
                if walker.hasattr("visit_List").unwrap() {
                    walker.call_method1(
                        "visit_List",
                        (self_, dme.borrow().populate_source_loc(source_loc, py)),
                    )?;
                } else {
                    let dmlist = list.borrow(py);
                    for i in 0..dmlist.keys.len() {
                        if let Some(k) = dmlist.keys.get(i) {
                            if let Ok(k_expr) = k.downcast_bound::<Expression>(py) {
                                Expression::walk(k_expr, dme, walker, py)?;
                            }
                        }
                        if let Some(v) = dmlist.vals.get(i) {
                            if let Ok(v_expr) = v.downcast_bound::<Expression>(py) {
                                Expression::walk(v_expr, dme, walker, py)?;
                            }
                        }
                    }
                }

                Ok(())
            }
            Expression::InterpString {
                first,
                token_pairs,
                source_loc,
            } => {
                if walker.hasattr("visit_InterpString").unwrap() {
                    walker.call_method1(
                        "visit_InterpString",
                        (self_, dme.borrow().populate_source_loc(source_loc, py)),
                    )?;
                } else {
                    visit_constant(first, walker)?;
                    for (expr, str) in token_pairs.iter() {
                        if let Some(tuple_expr) = expr {
                            Expression::walk(tuple_expr.bind(py), dme, walker, py)?;
                        }
                        visit_constant(str.get(), walker)?;
                    }
                }

                Ok(())
            }
            Expression::Prefab { prefab, .. } => {
                Prefab::walk(prefab.bind(py), walker, py)?;
                Ok(())
            }
            Expression::Index {
                expr,
                index,
                source_loc,
            } => {
                if walker.hasattr("visit_Index").unwrap() {
                    walker.call_method1(
                        "visit_Index",
                        (self_, dme.borrow().populate_source_loc(source_loc, py)),
                    )?;
                } else {
                    Expression::walk(expr.bind(py), dme, walker, py)?;
                    Expression::walk(index.bind(py), dme, walker, py)?;
                }

                Ok(())
            }
            Expression::Field {
                expr,
                field,
                source_loc,
            } => {
                if walker.hasattr("visit_Field").unwrap() {
                    walker.call_method1(
                        "visit_Field",
                        (self_, dme.borrow().populate_source_loc(source_loc, py)),
                    )?;
                } else {
                    expr.as_ref()
                        .map(|expr| Expression::walk(expr.bind(py), dme, walker, py));
                    Expression::walk(field.bind(py), dme, walker, py)?;
                }

                Ok(())
            }
            Expression::StaticField {
                expr,
                field,
                source_loc,
            } => {
                if walker.hasattr("visit_StaticField").unwrap() {
                    walker.call_method1(
                        "visit_StaticField",
                        (self_, dme.borrow().populate_source_loc(source_loc, py)),
                    )?;
                } else {
                    Expression::walk(expr.bind(py), dme, walker, py)?;
                    Expression::walk(field.bind(py), dme, walker, py)?;
                }

                Ok(())
            }
            Expression::Call {
                expr,
                name,
                args,
                source_loc,
            } => {
                if walker.hasattr("visit_Call").unwrap() {
                    walker.call_method1(
                        "visit_Call",
                        (self_, dme.borrow().populate_source_loc(source_loc, py)),
                    )?;
                } else {
                    Expression::walk(expr.bind(py), dme, walker, py)?;
                    Expression::walk(name.bind(py), dme, walker, py)?;
                    for arg in args.iter() {
                        Expression::walk(arg.bind(py), dme, walker, py)?;
                    }
                }

                Ok(())
            }
            Expression::SelfCall { args, source_loc } => {
                if walker.hasattr("visit_SelfCall").unwrap() {
                    walker.call_method1(
                        "visit_SelfCall",
                        (self_, dme.borrow().populate_source_loc(source_loc, py)),
                    )?;
                } else {
                    for arg in args.iter() {
                        Expression::walk(arg.bind(py), dme, walker, py)?;
                    }
                }

                Ok(())
            }
            Expression::ParentCall { args, source_loc } => {
                if walker.hasattr("visit_ParentCall").unwrap() {
                    walker.call_method1(
                        "visit_ParentCall",
                        (self_, dme.borrow().populate_source_loc(source_loc, py)),
                    )?;
                } else {
                    for arg in args.iter() {
                        Expression::walk(arg.bind(py), dme, walker, py)?;
                    }
                }

                Ok(())
            }
            Expression::UnaryOp {
                expr,
                unary_op: _,
                source_loc,
            } => {
                if walker.hasattr("visit_UnaryOp").unwrap() {
                    walker.call_method1(
                        "visit_UnaryOp",
                        (self_, dme.borrow().populate_source_loc(source_loc, py)),
                    )?;
                } else {
                    Expression::walk(expr.bind(py), dme, walker, py)?;
                }

                Ok(())
            }
            Expression::ProcReference {
                expr,
                name,
                source_loc,
            } => {
                if walker.hasattr("visit_ProcReference").unwrap() {
                    walker.call_method1(
                        "visit_ProcReference",
                        (self_, dme.borrow().populate_source_loc(source_loc, py)),
                    )?;
                } else {
                    Expression::walk(expr.bind(py), dme, walker, py)?;
                    Expression::walk(name.bind(py), dme, walker, py)?;
                }

                Ok(())
            }
            Expression::Locate {
                args,
                in_list,
                source_loc,
            } => {
                if walker.hasattr("visit_Locate").unwrap() {
                    walker.call_method1(
                        "visit_Locate",
                        (self_, dme.borrow().populate_source_loc(source_loc, py)),
                    )?;
                } else {
                    for arg in args.iter() {
                        Expression::walk(arg.bind(py), dme, walker, py)?;
                    }
                    if let Some(expr) = in_list {
                        Expression::walk(expr.bind(py), dme, walker, py)?;
                    }
                }

                Ok(())
            }
            Expression::ExternalCall {
                library_name,
                function_name,
                args,
                source_loc,
            } => {
                if walker.hasattr("visit_ExternalCall").unwrap() {
                    walker.call_method1(
                        "visit_ExternalCall",
                        (self_, dme.borrow().populate_source_loc(source_loc, py)),
                    )?;
                } else {
                    Expression::walk(library_name.bind(py), dme, walker, py)?;
                    Expression::walk(function_name.bind(py), dme, walker, py)?;
                    for arg in args.iter() {
                        Expression::walk(arg.bind(py), dme, walker, py)?;
                    }
                }
                Ok(())
            }
            Expression::NewMiniExpr {
                name,
                fields,
                source_loc,
            } => {
                if walker.hasattr("visit_NewMiniExpr").unwrap() {
                    walker.call_method1(
                        "visit_NewMiniExpr",
                        (self_, dme.borrow().populate_source_loc(source_loc, py)),
                    )?;
                } else {
                    Expression::walk(name.bind(py), dme, walker, py)?;
                    for field in fields.iter() {
                        Expression::walk(field.bind(py), dme, walker, py)?;
                    }
                }

                Ok(())
            }
            Expression::NewImplicit { args, source_loc } => {
                if walker.hasattr("visit_NewImplicit").unwrap() {
                    walker.call_method1(
                        "visit_NewImplicit",
                        (self_, dme.borrow().populate_source_loc(source_loc, py)),
                    )?;
                } else if let Some(arg_list) = args {
                    for arg in arg_list.iter() {
                        Expression::walk(arg.bind(py), dme, walker, py)?;
                    }
                }

                Ok(())
            }
            Expression::NewPrefab {
                prefab,
                args,
                source_loc,
            } => {
                if walker.hasattr("visit_NewPrefab").unwrap() {
                    walker.call_method1(
                        "visit_NewPrefab",
                        (self_, dme.borrow().populate_source_loc(source_loc, py)),
                    )?;
                } else {
                    Prefab::walk(prefab.bind(py), walker, py)?;
                    args.as_ref().map(|args| {
                        args.iter()
                            .map(|arg| Expression::walk(arg.bind(py), dme, walker, py))
                    });
                }

                Ok(())
            }
            Expression::DynamicCall {
                lib_name,
                proc_name,
                source_loc,
            } => {
                if walker.hasattr("visit_DynamicCall").unwrap() {
                    walker.call_method1(
                        "visit_DynamicCall",
                        (self_, dme.borrow().populate_source_loc(source_loc, py)),
                    )?;
                } else {
                    for lib_name in lib_name.iter() {
                        Expression::walk(lib_name.bind(py), dme, walker, py)?;
                    }
                    for proc_name in proc_name.iter() {
                        Expression::walk(proc_name.bind(py), dme, walker, py)?;
                    }
                }
                Ok(())
            }
            Expression::Input {
                args,
                input_type: _,
                in_list,
                source_loc,
            } => {
                if walker.hasattr("visit_Input").unwrap() {
                    walker.call_method1(
                        "visit_Input",
                        (self_, dme.borrow().populate_source_loc(source_loc, py)),
                    )?;
                } else {
                    for arg in args.iter() {
                        Expression::walk(arg.bind(py), dme, walker, py)?;
                    }
                    if let Some(in_list) = in_list {
                        Expression::walk(in_list.bind(py), dme, walker, py)?;
                    }
                }

                Ok(())
            }
            Expression::Pick { args, source_loc } => {
                if walker.hasattr("visit_Pick").unwrap() {
                    walker.call_method1(
                        "visit_Pick",
                        (self_, dme.borrow().populate_source_loc(source_loc, py)),
                    )?;
                } else {
                    for (a, b) in args.iter() {
                        if let Some(a) = a {
                            Expression::walk(a.bind(py), dme, walker, py)?;
                        }
                        Expression::walk(b.bind(py), dme, walker, py)?;
                    }
                }

                Ok(())
            }
        }
    }
}
