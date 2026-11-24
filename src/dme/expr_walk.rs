use pyo3::{Bound, PyAny, PyResult, Python};

use crate::dme::node_walk::try_visit;

use super::{expression::Expression, nodes::visit_constant, prefab::Prefab};

impl Expression {
    pub fn walk(self_: &Bound<Self>, walker: &Bound<PyAny>, py: Python<'_>) -> PyResult<()> {
        let self_expr = self_.get();

        if try_visit(
            walker,
            pyo3::intern!(py, "visit_Expr"),
            (self_, self_expr.source_loc().clone()),
        )? {
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
                try_visit(
                    walker,
                    pyo3::intern!(py, "visit_Identifier"),
                    (self_, source_loc.clone()),
                )?;
                Ok(())
            }
            Expression::BinaryOp {
                op: _,
                lhs,
                rhs,
                source_loc,
            } => {
                if !try_visit(
                    walker,
                    pyo3::intern!(py, "visit_BinaryOp"),
                    (self_, source_loc.clone()),
                )? {
                    Expression::walk(lhs.bind(py), walker, py)?;
                    Expression::walk(rhs.bind(py), walker, py)?;
                }
                Ok(())
            }
            Expression::AssignOp {
                op: _,
                lhs,
                rhs,
                source_loc,
            } => {
                if !try_visit(
                    walker,
                    pyo3::intern!(py, "visit_AssignOp"),
                    (self_, source_loc.clone()),
                )? {
                    Expression::walk(lhs.bind(py), walker, py)?;
                    Expression::walk(rhs.bind(py), walker, py)?;
                }

                Ok(())
            }
            Expression::TernaryOp {
                cond,
                if_expr,
                else_expr,
                source_loc,
            } => {
                if !try_visit(
                    walker,
                    pyo3::intern!(py, "visit_TernaryOp"),
                    (self_, source_loc.clone()),
                )? {
                    Expression::walk(cond.bind(py), walker, py)?;
                    Expression::walk(if_expr.bind(py), walker, py)?;
                    Expression::walk(else_expr.bind(py), walker, py)?;
                }

                Ok(())
            }
            Expression::List { list, source_loc } => {
                if !try_visit(
                    walker,
                    pyo3::intern!(py, "visit_List"),
                    (self_, source_loc.clone()),
                )? {
                    let dmlist = list.borrow(py);
                    for i in 0..dmlist.keys.len() {
                        if let Some(k) = dmlist.keys.get(i) {
                            if let Ok(k_expr) = k.cast_bound::<Expression>(py) {
                                Expression::walk(k_expr, walker, py)?;
                            }
                        }
                        if let Some(v) = dmlist.vals.get(i) {
                            if let Ok(v_expr) = v.cast_bound::<Expression>(py) {
                                Expression::walk(v_expr, walker, py)?;
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
                if !try_visit(
                    walker,
                    pyo3::intern!(py, "visit_InterpString"),
                    (self_, source_loc.clone()),
                )? {
                    visit_constant(first, walker)?;
                    for (expr, str) in token_pairs.iter() {
                        if let Some(tuple_expr) = expr {
                            Expression::walk(tuple_expr.bind(py), walker, py)?;
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
                if !try_visit(
                    walker,
                    pyo3::intern!(py, "visit_Index"),
                    (self_, source_loc.clone()),
                )? {
                    Expression::walk(expr.bind(py), walker, py)?;
                    Expression::walk(index.bind(py), walker, py)?;
                }

                Ok(())
            }
            Expression::Field {
                expr,
                field,
                source_loc,
            } => {
                if !try_visit(
                    walker,
                    pyo3::intern!(py, "visit_Field"),
                    (self_, source_loc.clone()),
                )? {
                    expr.as_ref()
                        .map(|expr| Expression::walk(expr.bind(py), walker, py));
                    Expression::walk(field.bind(py), walker, py)?;
                }

                Ok(())
            }
            Expression::StaticField {
                expr,
                field,
                source_loc,
            } => {
                if !try_visit(
                    walker,
                    pyo3::intern!(py, "visit_StaticField"),
                    (self_, source_loc.clone()),
                )? {
                    Expression::walk(expr.bind(py), walker, py)?;
                    Expression::walk(field.bind(py), walker, py)?;
                }

                Ok(())
            }
            Expression::Call {
                expr,
                name,
                args,
                source_loc,
            } => {
                if !try_visit(
                    walker,
                    pyo3::intern!(py, "visit_Call"),
                    (self_, source_loc.clone()),
                )? {
                    Expression::walk(expr.bind(py), walker, py)?;
                    Expression::walk(name.bind(py), walker, py)?;
                    for arg in args.iter() {
                        Expression::walk(arg.bind(py), walker, py)?;
                    }
                }

                Ok(())
            }
            Expression::SelfCall { args, source_loc } => {
                if !try_visit(
                    walker,
                    pyo3::intern!(py, "visit_SelfCall"),
                    (self_, source_loc.clone()),
                )? {
                    for arg in args.iter() {
                        Expression::walk(arg.bind(py), walker, py)?;
                    }
                }

                Ok(())
            }
            Expression::ParentCall { args, source_loc } => {
                if !try_visit(
                    walker,
                    pyo3::intern!(py, "visit_ParentCall"),
                    (self_, source_loc.clone()),
                )? {
                    for arg in args.iter() {
                        Expression::walk(arg.bind(py), walker, py)?;
                    }
                }

                Ok(())
            }
            Expression::UnaryOp {
                expr,
                unary_op: _,
                source_loc,
            } => {
                if !try_visit(
                    walker,
                    pyo3::intern!(py, "visit_UnaryOp"),
                    (self_, source_loc.clone()),
                )? {
                    Expression::walk(expr.bind(py), walker, py)?;
                }

                Ok(())
            }
            Expression::ProcReference {
                expr,
                name,
                source_loc,
            } => {
                if !try_visit(
                    walker,
                    pyo3::intern!(py, "visit_ProcReference"),
                    (self_, source_loc.clone()),
                )? {
                    Expression::walk(expr.bind(py), walker, py)?;
                    Expression::walk(name.bind(py), walker, py)?;
                }

                Ok(())
            }
            Expression::Locate {
                args,
                in_list,
                source_loc,
            } => {
                if !try_visit(
                    walker,
                    pyo3::intern!(py, "visit_Locate"),
                    (self_, source_loc.clone()),
                )? {
                    for arg in args.iter() {
                        Expression::walk(arg.bind(py), walker, py)?;
                    }
                    if let Some(expr) = in_list {
                        Expression::walk(expr.bind(py), walker, py)?;
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
                if !try_visit(
                    walker,
                    pyo3::intern!(py, "visit_ExternalCall"),
                    (self_, source_loc.clone()),
                )? {
                    Expression::walk(library_name.bind(py), walker, py)?;
                    Expression::walk(function_name.bind(py), walker, py)?;
                    for arg in args.iter() {
                        Expression::walk(arg.bind(py), walker, py)?;
                    }
                }
                Ok(())
            }
            Expression::NewMiniExpr {
                name,
                fields,
                source_loc,
            } => {
                if !try_visit(
                    walker,
                    pyo3::intern!(py, "visit_NewMiniExpr"),
                    (self_, source_loc.clone()),
                )? {
                    Expression::walk(name.bind(py), walker, py)?;
                    for field in fields.iter() {
                        Expression::walk(field.bind(py), walker, py)?;
                    }
                }

                Ok(())
            }
            Expression::NewImplicit { args, source_loc } => {
                if !try_visit(
                    walker,
                    pyo3::intern!(py, "visit_NewImplicit"),
                    (self_, source_loc.clone()),
                )? {
                    if let Some(arg_list) = args {
                        for arg in arg_list.iter() {
                            Expression::walk(arg.bind(py), walker, py)?;
                        }
                    }
                }

                Ok(())
            }
            Expression::NewPrefab {
                prefab,
                args,
                source_loc,
            } => {
                if !try_visit(
                    walker,
                    pyo3::intern!(py, "visit_NewPrefab"),
                    (self_, source_loc.clone()),
                )? {
                    Prefab::walk(prefab.bind(py), walker, py)?;
                    args.as_ref().map(|args| {
                        args.iter()
                            .map(|arg| Expression::walk(arg.bind(py), walker, py))
                    });
                }

                Ok(())
            }
            Expression::DynamicCall {
                lib_name,
                proc_name,
                source_loc,
            } => {
                if !try_visit(
                    walker,
                    pyo3::intern!(py, "visit_DynamicCall"),
                    (self_, source_loc.clone()),
                )? {
                    for lib_name in lib_name.iter() {
                        Expression::walk(lib_name.bind(py), walker, py)?;
                    }
                    for proc_name in proc_name.iter() {
                        Expression::walk(proc_name.bind(py), walker, py)?;
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
                if !try_visit(
                    walker,
                    pyo3::intern!(py, "visit_Input"),
                    (self_, source_loc.clone()),
                )? {
                    for arg in args.iter() {
                        Expression::walk(arg.bind(py), walker, py)?;
                    }
                    if let Some(in_list) = in_list {
                        Expression::walk(in_list.bind(py), walker, py)?;
                    }
                }

                Ok(())
            }
            Expression::Pick { args, source_loc } => {
                if !try_visit(
                    walker,
                    pyo3::intern!(py, "visit_Pick"),
                    (self_, source_loc.clone()),
                )? {
                    for (a, b) in args.iter() {
                        if let Some(a) = a {
                            Expression::walk(a.bind(py), walker, py)?;
                        }
                        Expression::walk(b.bind(py), walker, py)?;
                    }
                }

                Ok(())
            }
        }
    }
}
