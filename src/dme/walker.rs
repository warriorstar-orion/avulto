use std::borrow::Borrow;

use dreammaker::ast::{Expression, Ident2, Statement};
use pyo3::{types::PyAnyMethods, Bound, IntoPy, PyAny, PyResult, Python};

use super::{
    convert::{from_expression_to_node, from_statement_to_node},
    nodes::{self, Identifier},
    Dme,
};

impl Dme {
    pub fn walk_ident(
        &self,
        ident: &Ident2,
        walker: &Bound<PyAny>,
        py: Python<'_>,
    ) -> PyResult<()> {
        if walker.hasattr("visit_Constant").unwrap() {
            walker.call_method1(
                "visit_Constant",
                (Identifier {
                    ident: ident.clone().into_py(py),
                },),
            )?;
        }
        Ok(())
    }

    pub fn visit_expr(
        &self,
        expr: &Expression,
        walker: &Bound<PyAny>,
        py: Python<'_>,
    ) -> PyResult<()> {
        let mut visitor_name = "visit_Expr";
        match expr {
            Expression::Base { term, follow } => {
                match &term.elem {
                    dreammaker::ast::Term::Null
                    | dreammaker::ast::Term::Int(_)
                    | dreammaker::ast::Term::Float(_)
                    | dreammaker::ast::Term::Ident(_)
                    | dreammaker::ast::Term::String(_) => {
                        visitor_name = "visit_Constant";
                    }
                    dreammaker::ast::Term::Resource(_) => {
                        visitor_name = "visit_Resource";
                    }
                    dreammaker::ast::Term::Prefab(prefab) => {
                        visitor_name = "visit_Prefab";
                    }
                    dreammaker::ast::Term::InterpString(ident2, _) => {
                        visitor_name = "visit_InterpString"
                    }
                    dreammaker::ast::Term::Call(ident2, _) => {
                        visitor_name = "visit_Call";
                    }
                    dreammaker::ast::Term::SelfCall(_) => {
                        visitor_name = "visit_SelfCall";
                    }
                    dreammaker::ast::Term::ParentCall(_) => {
                        visitor_name = "visit_ParentCall";
                    }
                    dreammaker::ast::Term::NewImplicit { args: _ }
                    | dreammaker::ast::Term::NewPrefab { prefab: _, args: _ }
                    | dreammaker::ast::Term::NewMiniExpr { expr: _, args: _ } => {
                        visitor_name = "visit_New";
                    }
                    _ => {}
                }

                if walker.hasattr(visitor_name).unwrap() {
                    walker.call_method1(visitor_name, (from_expression_to_node(expr, py)?,))?;
                }

                Ok(())
            }
            Expression::BinaryOp { op: _, lhs, rhs } => {
                if walker.hasattr("visit_BinaryOp").unwrap() {
                    walker.call_method1("visit_BinaryOp", (from_expression_to_node(expr, py)?,))?;
                } else {
                    self.visit_expr(lhs, walker, py)?;
                    self.visit_expr(rhs, walker, py)?;
                }
                Ok(())
            }
            Expression::AssignOp { op: _, lhs, rhs } => {
                if walker.hasattr("visit_AssignOp").unwrap() {
                    walker.call_method1("visit_AssignOp", (from_expression_to_node(expr, py)?,))?;
                } else {
                    self.visit_expr(lhs, walker, py)?;
                    self.visit_expr(rhs, walker, py)?;
                }
                Ok(())
            }
            Expression::TernaryOp { cond, if_, else_ } => {
                if walker.hasattr("visit_TernaryOp").unwrap() {
                    walker
                        .call_method1("visit_TernaryOp", (from_expression_to_node(expr, py)?,))?;
                } else {
                    self.visit_expr(cond, walker, py)?;
                    self.visit_expr(if_, walker, py)?;
                    self.visit_expr(else_, walker, py)?;
                }
                Ok(())
            }
        }
    }

    pub fn walk_stmt(
        &self,
        stmt: &Statement,
        walker: &Bound<PyAny>,
        py: Python<'_>,
    ) -> PyResult<()> {
        match &stmt {
            dreammaker::ast::Statement::Expr(expr) => {
                self.visit_expr(expr, walker, py)?;
            }
            dreammaker::ast::Statement::Return(_) => {
                if walker.hasattr("visit_Return").unwrap() {
                    walker.call_method1("visit_Return", (from_statement_to_node(stmt, py)?,))?;
                }
            }
            dreammaker::ast::Statement::Throw(t) => {
                if walker.hasattr("visit_Throw").unwrap() {
                    walker.call_method1("visit_Throw", (from_statement_to_node(stmt, py)?,))?;
                } else {
                    self.visit_expr(t, walker, py)?;
                }
            }
            dreammaker::ast::Statement::While { condition, block } => {
                if walker.hasattr("visit_While").unwrap() {
                    walker.call_method1("visit_While", (from_statement_to_node(stmt, py)?,))?;
                } else {
                    self.visit_expr(condition, walker, py)?;
                    for stmt in block.iter() {
                        self.walk_stmt(&stmt.elem, walker, py)?;
                    }
                }
            }
            dreammaker::ast::Statement::DoWhile { block, condition } => {
                if walker.hasattr("visit_DoWhile").unwrap() {
                    walker.call_method1("visit_DoWhile", (from_statement_to_node(stmt, py)?,))?;
                } else {
                    self.visit_expr(&condition.elem, walker, py)?;
                    for stmt in block.iter() {
                        self.walk_stmt(&stmt.elem, walker, py)?;
                    }
                }
            }
            dreammaker::ast::Statement::If { arms, else_arm } => {
                if walker.hasattr("visit_If").unwrap() {
                    let if_node = from_statement_to_node(stmt, py)?;
                    walker.call_method1("visit_If", (if_node,))?;
                } else {
                    for (cond, armcode) in arms {
                        self.visit_expr(&cond.elem, walker, py)?;
                        for arm_stmt in armcode.iter() {
                            self.walk_stmt(&arm_stmt.elem, walker, py)?;
                        }
                        if let Some(else_arm_block) = else_arm {
                            for stmt in else_arm_block.iter() {
                                self.walk_stmt(&stmt.elem, walker, py)?;
                            }
                        }
                    }
                }
            }
            dreammaker::ast::Statement::ForInfinite { block } => {
                for stmt in block.iter() {
                    self.walk_stmt(&stmt.elem, walker, py)?;
                }
            }
            dreammaker::ast::Statement::ForLoop {
                init,
                test,
                inc,
                block,
            } => {
                if walker.hasattr("visit_For").unwrap() {
                    walker.call_method1("visit_For", (from_statement_to_node(stmt, py)?,))?;
                } else {
                    if let Some(init_stmt_boxed) = init {
                        let init_stmt = init_stmt_boxed.as_ref();
                        self.walk_stmt(init_stmt, walker, py)?;
                    }
                    if let Some(test_expr_boxed) = test {
                        let test_expr = test_expr_boxed.as_ref();
                        self.visit_expr(test_expr, walker, py)?;
                    }
                    if let Some(inc_stmt_boxed) = inc {
                        let inc_stmt = inc_stmt_boxed.as_ref();
                        self.walk_stmt(inc_stmt, walker, py)?;
                    }
                    for block_stmt_spanned in block.iter() {
                        self.walk_stmt(&block_stmt_spanned.elem, walker, py)?;
                    }
                }
            }
            dreammaker::ast::Statement::ForList(l) => {
                if walker.hasattr("visit_ForList").unwrap() {
                    walker.call_method1("visit_ForList", (from_statement_to_node(stmt, py)?,))?;
                } else {
                    self.walk_ident(&l.name, walker, py)?;
                    if let Some(in_list_expr) = &l.in_list {
                        self.visit_expr(in_list_expr, walker, py)?;
                    }
                    for stmt in l.block.iter() {
                        self.walk_stmt(&stmt.elem, walker, py)?;
                    }
                }
            }
            dreammaker::ast::Statement::ForRange(f) => {
                if walker.hasattr("visit_ForRange").unwrap() {
                    let node = from_statement_to_node(stmt, py)?;
                    walker.call_method1("visit_ForRange", (node,))?;
                } else {
                    self.walk_ident(&f.name, walker, py)?;
                    self.visit_expr(&f.start, walker, py)?;
                    self.visit_expr(&f.end, walker, py)?;
                    if let Some(step_expr) = &f.step {
                        self.visit_expr(step_expr, walker, py)?;
                    }
                    for stmt in f.block.iter() {
                        self.walk_stmt(&stmt.elem, walker, py)?;
                    }
                }
            }
            dreammaker::ast::Statement::Var(v) => {
                if walker.hasattr("visit_Var").unwrap() {
                    let var_node = nodes::Var::make(
                        py,
                        v.name.clone(),
                        match &v.value {
                            Some(e) => from_expression_to_node(e, py)?,
                            None => py.None(),
                        },
                    )?;
                    walker.call_method1("visit_Var", (var_node,))?;
                } else {
                    if walker.hasattr("visit_Constant").unwrap() {
                        walker.call_method1(
                            "visit_Constant",
                            (Identifier {
                                ident: v.name.clone().into_py(py),
                            },),
                        )?;
                    }
                    if let Some(v_expr) = &v.value {
                        self.visit_expr(v_expr, walker, py)?;
                    }
                }
            }
            dreammaker::ast::Statement::Vars(vs) => {
                if walker.hasattr("visit_Var").unwrap() {
                    for v in vs.iter() {
                        let var_node = nodes::Var::make(
                            py,
                            v.name.clone(),
                            match &v.value {
                                Some(e) => from_expression_to_node(e, py)?,
                                None => py.None(),
                            },
                        )?;
                        walker.call_method1("visit_Var", (var_node,))?;
                    }
                } else {
                    for v in vs.iter() {
                        if walker.hasattr("visit_Constant").unwrap() {
                            walker.call_method1(
                                "visit_Constant",
                                (Identifier {
                                    ident: v.name.clone().into_py(py),
                                },),
                            )?;
                        }
                        if let Some(v_expr) = &v.value {
                            self.visit_expr(v_expr, walker, py)?;
                        }
                    }
                }
            }
            dreammaker::ast::Statement::Setting {
                name,
                mode: _,
                value,
            } => {
                if walker.hasattr("visit_Setting").unwrap() {
                    walker.call_method1("visit_Setting", (from_statement_to_node(stmt, py)?,))?;
                } else {
                    self.walk_ident(name, walker, py)?;
                    self.visit_expr(value, walker, py)?;
                }
            }
            dreammaker::ast::Statement::Spawn { delay, block } => {
                if walker.hasattr("visit_Spawn").unwrap() {
                    walker.call_method1("visit_Spawn", (from_statement_to_node(stmt, py)?,))?;
                } else {
                    if let Some(delay_expr) = delay {
                        self.visit_expr(delay_expr, walker, py)?;
                    }
                    for stmt in block.iter() {
                        self.walk_stmt(&stmt.elem, walker, py)?;
                    }
                }
            }
            dreammaker::ast::Statement::Switch {
                input,
                cases,
                default,
            } => {
                if walker.hasattr("visit_Switch").unwrap() {
                    walker.call_method1("visit_Switch", (from_statement_to_node(stmt, py)?,))?;
                } else {
                    self.visit_expr(input, walker, py)?;
                    for (case_types, block) in cases.iter() {
                        if walker.hasattr("visit_Expr").unwrap() {
                            for case_elem in &case_types.elem {
                                match case_elem {
                                    dreammaker::ast::Case::Exact(e) => {
                                        self.visit_expr(e, walker, py)?;
                                    }
                                    dreammaker::ast::Case::Range(s, e) => {
                                        self.visit_expr(s, walker, py)?;
                                        self.visit_expr(e, walker, py)?;
                                    }
                                }
                            }
                        }
                        for stmt in block.iter() {
                            self.walk_stmt(&stmt.elem, walker, py)?;
                        }
                    }
                    if let Some(default_block) = default {
                        for stmt in default_block.iter() {
                            self.walk_stmt(&stmt.elem, walker, py)?;
                        }
                    }
                }
            }
            dreammaker::ast::Statement::TryCatch {
                try_block,
                catch_params,
                catch_block,
            } => {
                if walker.hasattr("visit_TryCatch").unwrap() {
                    walker.call_method1("visit_TryCatch", (from_statement_to_node(stmt, py)?,))?;
                } else {
                    for stmt in try_block.iter() {
                        self.walk_stmt(&stmt.elem, walker, py)?;
                    }
                    for param in catch_params.iter() {
                        if walker.hasattr("visit_Constant").unwrap() {
                            for s in param.iter() {
                                walker.call_method1(
                                    "visit_Constant",
                                    (Identifier {
                                        ident: s.clone().into_py(py),
                                    },),
                                )?;
                            }
                        }
                    }
                    for stmt in catch_block.iter() {
                        self.walk_stmt(&stmt.elem, walker, py)?;
                    }
                }
            }
            dreammaker::ast::Statement::Continue(c) => {
                if walker.hasattr("visit_Continue").unwrap() {
                    walker.call_method1("visit_Continue", (from_statement_to_node(stmt, py)?,))?;
                } else if walker.hasattr("visit_Constant").unwrap() {
                    if let Some(cs) = c {
                        walker.call_method1(
                            "visit_Constant",
                            (Identifier {
                                ident: cs.clone().into_py(py),
                            },),
                        )?;
                    }
                }
            }
            dreammaker::ast::Statement::Break(b) => {
                if walker.hasattr("visit_Break").unwrap() {
                    walker.call_method1("visit_Break", (from_statement_to_node(stmt, py)?,))?;
                } else if walker.hasattr("visit_Constant").unwrap() {
                    if let Some(bs) = b {
                        walker.call_method1(
                            "visit_Constant",
                            (Identifier {
                                ident: bs.clone().into_py(py),
                            },),
                        )?;
                    }
                }
            }
            dreammaker::ast::Statement::Goto(g) => {
                if walker.hasattr("visit_Goto").unwrap() {
                    walker.call_method1("visit_Goto", (from_statement_to_node(stmt, py)?,))?;
                } else if walker.hasattr("visit_Constant").unwrap() {
                    walker.call_method1(
                        "visit_Constant",
                        (Identifier {
                            ident: g.clone().into_py(py),
                        },),
                    )?;
                }
            }
            dreammaker::ast::Statement::Label { name, block } => {
                if walker.hasattr("visit_Label").unwrap() {
                    walker.call_method1("visit_Label", (from_statement_to_node(stmt, py)?,))?;
                } else {
                    if walker.hasattr("visit_Constant").unwrap() {
                        walker.call_method1(
                            "visit_Constant",
                            (Identifier {
                                ident: name.clone().into_py(py),
                            },),
                        )?;
                    }
                    for block_stmt_spanned in block.iter() {
                        self.walk_stmt(&block_stmt_spanned.elem, walker, py)?;
                    }
                }
            }
            dreammaker::ast::Statement::Del(expr) => {
                if walker.hasattr("visit_Del").unwrap() {
                    let del_node = from_statement_to_node(stmt, py)?;
                    walker.call_method1("visit_Del", (del_node,))?;
                } else {
                    self.visit_expr(expr, walker, py)?;
                }
            }
            dreammaker::ast::Statement::Crash(crash_expr) => {
                if walker.hasattr("visit_Crash").unwrap() {
                    let crashval = if let Some(cexpr) = crash_expr {
                        from_expression_to_node(cexpr, py)?
                    } else {
                        py.None()
                    };

                    let crash_node = nodes::Crash::make(py, crashval)?;
                    walker.call_method1("visit_Crash", (crash_node,))?;
                }
            }
        };

        Ok(())
    }
}
