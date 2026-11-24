use dreammaker::ast::AssignOp;
use pyo3::{IntoPyObject, Py, PyResult, Python, exceptions::PyValueError};

use crate::{
    dme::{FileData, SourceLoc},
    dmlist::DmList,
};

use super::{
    expression::{Constant, Expression},
    nodes::PyExpr,
    operators::{AssignOperator, BinaryOperator, UnaryOperator},
    prefab::Prefab,
};

impl Expression {
    pub fn parse_exprs(
        py: Python<'_>,
        arr: &[dreammaker::ast::Expression],
        file_data: &FileData,
    ) -> PyResult<Vec<PyExpr>> {
        arr.iter()
            .map(|expr| Self::parse_py(py, expr, file_data))
            .collect()
    }

    pub fn parse_py(
        py: Python<'_>,
        expr: &dreammaker::ast::Expression,
        file_data: &FileData,
    ) -> PyResult<PyExpr> {
        Ok(Self::parse(py, expr, file_data)?.into_pyobject(py)?.unbind())
    }

    pub fn parse(
        py: Python<'_>,
        expr: &dreammaker::ast::Expression,
        file_data: &FileData,
    ) -> PyResult<Self> {
        Ok(match expr {
            dreammaker::ast::Expression::Base { term, follow } => {
                let mut core = match &term.elem {
                    dreammaker::ast::Term::Ident(i) => Self::Identifier {
                        name: i.clone(),
                        source_loc: file_data.fill_source_loc(&term.location),
                    },
                    dreammaker::ast::Term::Int(i) => Self::Constant {
                        constant: Constant::Int(*i),
                        source_loc: file_data.fill_source_loc(&term.location),
                    },
                    dreammaker::ast::Term::Null => Self::Constant {
                        constant: Constant::Null(),
                        source_loc: file_data.fill_source_loc(&term.location),
                    },
                    dreammaker::ast::Term::Float(f) => Self::Constant {
                        constant: Constant::Float(*f),
                        source_loc: file_data.fill_source_loc(&term.location),
                    },
                    dreammaker::ast::Term::String(s) => Self::Constant {
                        constant: Constant::String(s.clone()),
                        source_loc: file_data.fill_source_loc(&term.location),
                    },
                    dreammaker::ast::Term::Resource(s) => Self::Constant {
                        constant: Constant::Resource(s.clone()),
                        source_loc: file_data.fill_source_loc(&term.location),
                    },
                    dreammaker::ast::Term::As(_) => todo!(),
                    dreammaker::ast::Term::__PROC__ => Self::Constant {
                        constant: Constant::ProcMacro(),
                        source_loc: file_data.fill_source_loc(&term.location),
                    },
                    dreammaker::ast::Term::__TYPE__ => todo!(),
                    dreammaker::ast::Term::__IMPLIED_TYPE__ => todo!(),
                    dreammaker::ast::Term::Expr(expression) => {
                        Self::parse(py, expression, file_data)?
                    }
                    dreammaker::ast::Term::Prefab(prefab) => Self::Prefab {
                        prefab: Py::new(py, Prefab::make(py, prefab, file_data)?)?,
                        source_loc: file_data.fill_source_loc(&term.location),
                    },
                    dreammaker::ast::Term::InterpString(ident2, tokens) => {
                        let mut token_vec: Vec<(Option<PyExpr>, Py<Constant>)> = vec![];
                        for (maybe_token_expr, token_str) in tokens.iter() {
                            let token_expr_node = maybe_token_expr
                                .as_ref()
                                .map(|token_expr| Self::parse_py(py, token_expr, file_data))
                                .transpose()?;
                            let token_expr_str = Constant::String(token_str.to_string())
                                .into_pyobject(py)?
                                .unbind();
                            token_vec.push((token_expr_node, token_expr_str));
                        }
                        Self::InterpString {
                            first: Constant::String(ident2.to_string()),
                            token_pairs: token_vec,
                            source_loc: file_data.fill_source_loc(&term.location),
                        }
                    }
                    dreammaker::ast::Term::Call(ident2, args) => Self::Call {
                        // TODO: Not sure these should be marked as builtins
                        expr: Self::null(file_data.fill_source_loc(&term.location), py)?,
                        name: Self::ident(
                            ident2.to_string(),
                            file_data.fill_source_loc(&term.location),
                            py,
                        )?,
                        args: Self::parse_exprs(py, args, file_data)?,
                        source_loc: file_data.fill_source_loc(&term.location),
                    },
                    dreammaker::ast::Term::SelfCall(args) => Self::SelfCall {
                        args: Self::parse_exprs(py, args, file_data)?,
                        source_loc: file_data.fill_source_loc(&term.location),
                    },
                    dreammaker::ast::Term::ParentCall(args) => Self::ParentCall {
                        args: Self::parse_exprs(py, args, file_data)?,
                        source_loc: file_data.fill_source_loc(&term.location),
                    },
                    dreammaker::ast::Term::NewImplicit { args } => Self::NewImplicit {
                        args: args
                            .as_deref()
                            .map(|a| Self::parse_exprs(py, a, file_data))
                            .transpose()?,
                        source_loc: file_data.fill_source_loc(&term.location),
                    },
                    dreammaker::ast::Term::NewPrefab { prefab, args } => Self::NewPrefab {
                        prefab: Py::new(py, Prefab::make(py, prefab, file_data)?)?,
                        args: args
                            .as_deref()
                            .map(|a| Self::parse_exprs(py, a, file_data))
                            .transpose()?,
                        source_loc: file_data.fill_source_loc(&term.location),
                    },
                    dreammaker::ast::Term::NewMiniExpr { expr, args: _ } => Self::NewMiniExpr {
                        name: Self::ident(expr.ident.to_string(), SourceLoc::builtin(), py)?,
                        fields: expr
                            .fields
                            .iter()
                            .map(|f| {
                                Ok(Self::Field {
                                    expr: None,
                                    field: Self::ident(
                                        f.ident.to_string(),
                                        SourceLoc::builtin(),
                                        py,
                                    )?,
                                    source_loc: SourceLoc::builtin(),
                                }
                                .into_pyobject(py)?
                                .unbind())
                            })
                            .collect::<PyResult<Vec<_>>>()?,
                        source_loc: file_data.fill_source_loc(&term.location),
                    },
                    dreammaker::ast::Term::List(l) => {
                        let mut keys: Vec<PyExpr> = vec![];
                        let mut vals: Vec<PyExpr> = vec![];

                        for args in l.iter() {
                            match args {
                                dreammaker::ast::Expression::Base { .. } => {
                                    keys.push(Self::parse_py(py, args, file_data)?);
                                    vals.push(Self::null(SourceLoc::builtin(), py)?);
                                }
                                dreammaker::ast::Expression::AssignOp { op: _, lhs, rhs } => {
                                    keys.push(Self::parse_py(py, lhs, file_data)?);
                                    vals.push(Self::parse_py(py, rhs, file_data)?);
                                }
                                dreammaker::ast::Expression::BinaryOp { .. } => {
                                    keys.push(Self::parse_py(py, args, file_data)?);
                                    vals.push(Self::null(SourceLoc::builtin(), py)?);
                                }
                                dreammaker::ast::Expression::TernaryOp { cond, if_, else_ } => {
                                    keys.push(
                                        Self::TernaryOp {
                                            cond: Self::parse_py(py, cond, file_data)?,
                                            if_expr: Self::parse_py(py, if_, file_data)?,
                                            else_expr: Self::parse_py(py, else_, file_data)?,
                                            source_loc: SourceLoc::builtin(),
                                        }
                                        .into_pyobject(py)?
                                        .unbind(),
                                    );
                                    vals.push(Self::null(SourceLoc::builtin(), py)?);
                                }
                            }
                        }
                        Self::List {
                            list: Py::new(
                                py,
                                DmList {
                                    keys: keys.iter().map(|k| k.clone_ref(py).into_any()).collect(),
                                    vals: vals.iter().map(|k| k.clone_ref(py).into_any()).collect(),
                                },
                            )?,
                            source_loc: file_data.fill_source_loc(&term.location),
                        }
                    }
                    dreammaker::ast::Term::Input {
                        args,
                        input_type,
                        in_list,
                    } => Self::Input {
                        args: Self::parse_exprs(py, args, file_data)?,
                        input_type: input_type.as_ref().map(|it| it.bits()),
                        in_list: in_list
                            .as_ref()
                            .map(|in_list| Self::parse_py(py, in_list, file_data))
                            .transpose()?,
                        source_loc: file_data.fill_source_loc(&term.location),
                    },
                    dreammaker::ast::Term::Locate { args, in_list } => Self::Locate {
                        args: Self::parse_exprs(py, args, file_data)?,
                        in_list: in_list
                            .as_ref()
                            .map(|expr| Self::parse_py(py, expr, file_data))
                            .transpose()?,
                        source_loc: file_data.fill_source_loc(&term.location),
                    },
                    dreammaker::ast::Term::Pick(p) => Self::Pick {
                        args: p
                            .iter()
                            .map(|(a, b)| {
                                Ok((
                                    a.as_ref()
                                        .map(|expr| Self::parse_py(py, expr, file_data))
                                        .transpose()?,
                                    Self::parse_py(py, b, file_data)?,
                                ))
                            })
                            .collect::<PyResult<Vec<_>>>()?,
                        source_loc: file_data.fill_source_loc(&term.location),
                    },
                    dreammaker::ast::Term::DynamicCall(lib_name, proc_name) => Self::DynamicCall {
                        lib_name: Self::parse_exprs(py, lib_name, file_data)?,
                        proc_name: Self::parse_exprs(py, proc_name, file_data)?,
                        source_loc: file_data.fill_source_loc(&term.location),
                    },
                    dreammaker::ast::Term::ExternalCall {
                        library,
                        function,
                        args,
                    } => Self::ExternalCall {
                        library_name: Self::parse_py(
                            py,
                            library.as_deref().ok_or_else(|| {
                                PyValueError::new_err("missing external call library")
                            })?,
                            file_data,
                        )?,
                        function_name: Self::parse_py(py, function, file_data)?,
                        args: Self::parse_exprs(py, args, file_data)?,
                        source_loc: file_data.fill_source_loc(&term.location),
                    },
                    dreammaker::ast::Term::GlobalIdent(_) => todo!(),
                    dreammaker::ast::Term::GlobalCall(_, _) => todo!(),
                };

                for f in follow.iter() {
                    match &f.elem {
                        dreammaker::ast::Follow::Index(_, expression) => {
                            core = Self::Index {
                                expr: core.into_pyobject(py)?.unbind(),
                                index: Self::parse_py(py, expression, file_data)?,
                                source_loc: file_data.fill_source_loc(&f.location),
                            };
                        }
                        dreammaker::ast::Follow::Field(_, ident2) => {
                            core = Self::Field {
                                expr: Some(core.into_pyobject(py)?.unbind()),
                                field: Self::ident(
                                    ident2.to_string(),
                                    file_data.fill_source_loc(&f.location),
                                    py,
                                )?,
                                source_loc: file_data.fill_source_loc(&f.location),
                            };
                        }
                        dreammaker::ast::Follow::Call(_, ident2, args) => {
                            core = Self::Call {
                                expr: core.into_pyobject(py)?.unbind(),
                                name: Self::ident(
                                    ident2.to_string(),
                                    file_data.fill_source_loc(&f.location),
                                    py,
                                )?,
                                args: Self::parse_exprs(py, args, file_data)?,
                                source_loc: file_data.fill_source_loc(&f.location),
                            }
                        }
                        dreammaker::ast::Follow::Unary(unary_op) => {
                            core = Self::UnaryOp {
                                expr: core.into_pyobject(py)?.unbind(),
                                unary_op: match unary_op {
                                    dreammaker::ast::UnaryOp::Neg => UnaryOperator::Neg,
                                    dreammaker::ast::UnaryOp::Not => UnaryOperator::Not,
                                    dreammaker::ast::UnaryOp::BitNot => UnaryOperator::BitNot,
                                    dreammaker::ast::UnaryOp::PreIncr => UnaryOperator::PreIncr,
                                    dreammaker::ast::UnaryOp::PostIncr => UnaryOperator::PostIncr,
                                    dreammaker::ast::UnaryOp::PreDecr => UnaryOperator::PreDecr,
                                    dreammaker::ast::UnaryOp::PostDecr => UnaryOperator::PostDecr,
                                    dreammaker::ast::UnaryOp::Reference => UnaryOperator::Ref,
                                    dreammaker::ast::UnaryOp::Dereference => UnaryOperator::Deref,
                                },
                                source_loc: file_data.fill_source_loc(&f.location),
                            }
                        }
                        dreammaker::ast::Follow::StaticField(ident2) => {
                            core = Self::StaticField {
                                expr: core.into_pyobject(py)?.unbind(),
                                field: Self::ident(ident2.to_string(), SourceLoc::builtin(), py)?,
                                source_loc: file_data.fill_source_loc(&f.location),
                            };
                        }
                        dreammaker::ast::Follow::ProcReference(ident2) => {
                            core = Self::ProcReference {
                                expr: core.into_pyobject(py)?.unbind(),
                                name: Self::ident(ident2.to_string(), SourceLoc::builtin(), py)?,
                                source_loc: file_data.fill_source_loc(&f.location),
                            };
                        }
                    }
                }

                core
            }
            dreammaker::ast::Expression::BinaryOp { op, lhs, rhs } => Self::BinaryOp {
                op: match op {
                    dreammaker::ast::BinaryOp::Add => BinaryOperator::Add,
                    dreammaker::ast::BinaryOp::Sub => BinaryOperator::Sub,
                    dreammaker::ast::BinaryOp::Mul => BinaryOperator::Mul,
                    dreammaker::ast::BinaryOp::Div => BinaryOperator::Div,
                    dreammaker::ast::BinaryOp::Pow => BinaryOperator::Pow,
                    dreammaker::ast::BinaryOp::Mod => BinaryOperator::Mod,
                    dreammaker::ast::BinaryOp::FloatMod => BinaryOperator::FloatMod,
                    dreammaker::ast::BinaryOp::Eq => BinaryOperator::Eq,
                    dreammaker::ast::BinaryOp::NotEq => BinaryOperator::NotEq,
                    dreammaker::ast::BinaryOp::Less => BinaryOperator::Less,
                    dreammaker::ast::BinaryOp::Greater => BinaryOperator::Greater,
                    dreammaker::ast::BinaryOp::LessEq => BinaryOperator::LessEq,
                    dreammaker::ast::BinaryOp::GreaterEq => BinaryOperator::GreaterEq,
                    dreammaker::ast::BinaryOp::Equiv => BinaryOperator::Equiv,
                    dreammaker::ast::BinaryOp::NotEquiv => BinaryOperator::NotEquiv,
                    dreammaker::ast::BinaryOp::BitAnd => BinaryOperator::BitAnd,
                    dreammaker::ast::BinaryOp::BitXor => BinaryOperator::BitXor,
                    dreammaker::ast::BinaryOp::BitOr => BinaryOperator::BitOr,
                    dreammaker::ast::BinaryOp::LShift => BinaryOperator::LShift,
                    dreammaker::ast::BinaryOp::RShift => BinaryOperator::RShift,
                    dreammaker::ast::BinaryOp::And => BinaryOperator::And,
                    dreammaker::ast::BinaryOp::Or => BinaryOperator::Or,
                    dreammaker::ast::BinaryOp::In => BinaryOperator::In,
                    dreammaker::ast::BinaryOp::To => BinaryOperator::To,
                    dreammaker::ast::BinaryOp::LessOrGreater => BinaryOperator::LessOrGreater,
                },
                lhs: Self::parse_py(py, lhs, file_data)?,
                rhs: Self::parse_py(py, rhs, file_data)?,
                source_loc: SourceLoc::builtin(),
            },
            dreammaker::ast::Expression::AssignOp { op, lhs, rhs } => Self::AssignOp {
                op: match op {
                    AssignOp::Assign => AssignOperator::Assign,
                    AssignOp::AddAssign => AssignOperator::AssignAdd,
                    AssignOp::SubAssign => AssignOperator::AssignSub,
                    AssignOp::MulAssign => AssignOperator::AssignMul,
                    AssignOp::DivAssign => AssignOperator::AssignDiv,
                    AssignOp::ModAssign => AssignOperator::AssignMod,
                    AssignOp::FloatModAssign => AssignOperator::AssignFloatMod,
                    AssignOp::AssignInto => AssignOperator::AssignInto,
                    AssignOp::BitAndAssign => AssignOperator::AssignBitAnd,
                    AssignOp::AndAssign => AssignOperator::AssignAnd,
                    AssignOp::BitOrAssign => AssignOperator::AssignBitOr,
                    AssignOp::OrAssign => AssignOperator::AssignOr,
                    AssignOp::BitXorAssign => AssignOperator::AssignBitXor,
                    AssignOp::LShiftAssign => AssignOperator::AssignLShift,
                    AssignOp::RShiftAssign => AssignOperator::AssignRShift,
                },
                lhs: Self::parse_py(py, lhs, file_data)?,
                rhs: Self::parse_py(py, rhs, file_data)?,
                source_loc: SourceLoc::builtin(),
            },
            dreammaker::ast::Expression::TernaryOp { cond, if_, else_ } => Self::TernaryOp {
                cond: Self::parse_py(py, cond, file_data)?,
                if_expr: Self::parse_py(py, if_, file_data)?,
                else_expr: Self::parse_py(py, else_, file_data)?,
                source_loc: SourceLoc::builtin(),
            },
        })
    }
}
