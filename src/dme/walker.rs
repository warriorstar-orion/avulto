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

    pub fn walk_expr(
        &self,
        expr: &Expression,
        walker: &Bound<PyAny>,
        py: Python<'_>,
    ) -> PyResult<()> {
        if walker.hasattr("visit_Expr").unwrap() {
            walker.call_method1("visit_Expr", (from_expression_to_node(expr, py)?,))?;
        }

        Ok(())
    }

    pub fn walk_stmt(
        &self,
        stmt: &Statement,
        walker: &Bound<PyAny>,
        py: Python<'_>,
    ) -> PyResult<()> {
        match &stmt {
            dreammaker::ast::Statement::Expr(expr) => {
                let mut visit_name = "visit_Expr";
                let node = from_expression_to_node(expr, py)?;
                if let dreammaker::ast::Expression::Base { term, .. } = expr {
                    match &term.elem {
                        dreammaker::ast::Term::Null => {
                            visit_name = "visit_Constant";
                        }
                        dreammaker::ast::Term::Int(_) => {
                            visit_name = "visit_Constant";
                        }
                        dreammaker::ast::Term::Float(_) => {
                            visit_name = "visit_Constant";
                        }
                        dreammaker::ast::Term::String(_) => {
                            visit_name = "visit_Constant";
                        }
                        dreammaker::ast::Term::Resource(_) => {
                            visit_name = "visit_Resource";
                        }
                        dreammaker::ast::Term::Call(_, _) => {
                            visit_name = "visit_Call";
                        }
                        dreammaker::ast::Term::SelfCall(_) => {
                            visit_name = "visit_SelfCall";
                        }
                        dreammaker::ast::Term::ParentCall(_) => {
                            visit_name = "visit_ParentCall";
                        }
                        _ => {}
                    }
                };

                if walker.hasattr(visit_name).unwrap() {
                    walker.call_method1(visit_name, (node,))?;
                }
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
                    self.walk_expr(t, walker, py)?;
                }
            },
            dreammaker::ast::Statement::While { condition, block } => {
                if walker.hasattr("visit_While").unwrap() {
                    walker.call_method1("visit_While", (from_statement_to_node(stmt, py)?,))?;
                } else {
                    self.walk_expr(condition, walker, py)?;
                    for stmt in block.iter() {
                        self.walk_stmt(&stmt.elem, walker, py)?;
                    }
                }
            }
            dreammaker::ast::Statement::DoWhile { block, condition } => {
                if walker.hasattr("visit_DoWhile").unwrap() {
                    walker.call_method1("visit_DoWhile", (from_statement_to_node(stmt, py)?,))?;
                } else {
                    self.walk_expr(&condition.elem, walker, py)?;
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
                        if walker.hasattr("visit_Expr").unwrap() {
                            walker.call_method1(
                                "visit_Expr",
                                (from_expression_to_node(&cond.elem, py)?,),
                            )?;
                        }
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
            dreammaker::ast::Statement::ForInfinite { .. } => todo!(),
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
                        if walker.hasattr("visit_Expr").unwrap() {
                            walker.call_method1(
                                "visit_Expr",
                                (from_expression_to_node(test_expr, py)?,),
                            )?;
                        }
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
                        self.walk_expr(in_list_expr, walker, py)?;
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
                    self.walk_expr(&f.start, walker, py)?;
                    self.walk_expr(&f.end, walker, py)?;
                    if let Some(step_expr) = &f.step {
                        self.walk_expr(step_expr, walker, py)?;
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
                        from_expression_to_node(v.value.as_ref().unwrap(), py)?,
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
                        self.walk_expr(v_expr, walker, py)?;
                    }
                }
            }
            dreammaker::ast::Statement::Vars(_) => todo!(),
            dreammaker::ast::Statement::Setting { name, mode: _, value } => {
                if walker.hasattr("visit_Setting").unwrap() {
                    walker.call_method1("visitSetting", (from_statement_to_node(stmt, py)?,))?;
                } else {
                    self.walk_ident(name, walker, py)?;
                    self.walk_expr(value, walker, py)?;
                }
            }
            dreammaker::ast::Statement::Spawn { delay, block } => {
                if walker.hasattr("visit_Spawn").unwrap() {
                    walker.call_method1("visit_Spawn", (from_statement_to_node(stmt, py)?,))?;
                } else {
                    if let Some(delay_expr) = delay {
                        self.walk_expr(delay_expr, walker, py)?;
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
                    self.walk_expr(input, walker, py)?;
                    for (case_types, block) in cases.iter() {
                        if walker.hasattr("visit_Expr").unwrap() {
                            for case_elem in &case_types.elem {
                                match case_elem {
                                    dreammaker::ast::Case::Exact(e) => {
                                        walker.call_method1(
                                            "visit_Expr",
                                            (from_expression_to_node(e, py)?,),
                                        )?;
                                    }
                                    dreammaker::ast::Case::Range(s, e) => {
                                        walker.call_method1(
                                            "visit_Expr",
                                            (from_expression_to_node(s, py)?,),
                                        )?;
                                        walker.call_method1(
                                            "visit_Expr",
                                            (from_expression_to_node(e, py)?,),
                                        )?;
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
            dreammaker::ast::Statement::TryCatch { .. } => {
                if walker.hasattr("visit_TryCatch").unwrap() {
                    walker.call_method1("visit_TryCatch", (from_statement_to_node(stmt, py)?,))?;
                }
            },
            dreammaker::ast::Statement::Continue(_) => {
                if walker.hasattr("visit_Continue").unwrap() {
                    walker.call_method1("visit_Continue", (from_statement_to_node(stmt, py)?,))?;
                }
            },
            dreammaker::ast::Statement::Break(_) => {
                if walker.hasattr("visit_Break").unwrap() {
                    walker.call_method1("visit_Break", (from_statement_to_node(stmt, py)?,))?;
                }
            }
            dreammaker::ast::Statement::Goto(_) => todo!(),
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
                    self.walk_expr(expr, walker, py)?;
                }
            },
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
