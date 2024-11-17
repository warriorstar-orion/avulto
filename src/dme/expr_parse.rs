use dreammaker::ast::AssignOp;
use pyo3::{IntoPyObject, Py, Python};

use crate::dmlist::DmList;

use super::{
    expression::{Constant, Expression},
    nodes::{OriginalSourceLocation, PyExpr},
    operators::{AssignOperator, BinaryOperator, UnaryOperator},
    prefab::Prefab,
};

impl Expression {
    pub fn parse(py: Python<'_>, expr: &dreammaker::ast::Expression) -> Self {
        match expr {
            dreammaker::ast::Expression::Base { term, follow } => {
                let mut core = match &term.elem {
                    dreammaker::ast::Term::Ident(i) => Self::Identifier {
                        name: i.clone(),
                        source_loc: Some(OriginalSourceLocation::from_location(&term.location)),
                    },
                    dreammaker::ast::Term::Int(i) => Self::Constant {
                        constant: Constant::Int(*i),
                        source_loc: Some(OriginalSourceLocation::from_location(&term.location)),
                    },
                    dreammaker::ast::Term::Null => Self::Constant {
                        constant: Constant::Null(),
                        source_loc: Some(OriginalSourceLocation::from_location(&term.location)),
                    },
                    dreammaker::ast::Term::Float(f) => Self::Constant {
                        constant: Constant::Float(*f),
                        source_loc: Some(OriginalSourceLocation::from_location(&term.location)),
                    },
                    dreammaker::ast::Term::String(s) => Self::Constant {
                        constant: Constant::String(s.clone()),
                        source_loc: Some(OriginalSourceLocation::from_location(&term.location)),
                    },
                    dreammaker::ast::Term::Resource(s) => Self::Constant {
                        constant: Constant::Resource(s.clone()),
                        source_loc: Some(OriginalSourceLocation::from_location(&term.location)),
                    },
                    dreammaker::ast::Term::As(_) => todo!(),
                    dreammaker::ast::Term::__PROC__ => Self::Constant {
                        constant: Constant::ProcMacro(),
                        source_loc: Some(OriginalSourceLocation::from_location(&term.location)),
                    },
                    dreammaker::ast::Term::__TYPE__ => todo!(),
                    dreammaker::ast::Term::__IMPLIED_TYPE__ => todo!(),
                    dreammaker::ast::Term::Expr(expression) => Expression::parse(py, expression),
                    dreammaker::ast::Term::Prefab(prefab) => Self::Prefab {
                        prefab: Prefab::make(py, prefab)
                            .into_pyobject(py)
                            .expect("parsing prefab")
                            .into(),
                        source_loc: Some(OriginalSourceLocation::from_location(&term.location)),
                    },
                    dreammaker::ast::Term::InterpString(ident2, tokens) => {
                        let mut token_vec: Vec<(Option<PyExpr>, Py<Constant>)> = vec![];
                        for (maybe_token_expr, token_str) in tokens.iter() {
                            let token_expr_node = maybe_token_expr.as_ref().map(|token_expr| {
                                Expression::parse(py, token_expr)
                                    .into_pyobject(py)
                                    .expect("parsing interpstring token expr")
                                    .into()
                            });
                            let token_expr_str = Constant::String(token_str.to_string())
                                .into_pyobject(py)
                                .expect("parsing interpstring token str")
                                .into();
                            token_vec.push((token_expr_node, token_expr_str));
                        }
                        Self::InterpString {
                            first: Constant::String(ident2.to_string()),
                            token_pairs: token_vec,
                            source_loc: Some(OriginalSourceLocation::from_location(&term.location)),
                        }
                    }
                    dreammaker::ast::Term::Call(ident2, args) => Self::Call {
                        expr: Expression::null(None, py),
                        name: Expression::ident(ident2.to_string(), None, py),
                        args: args
                            .iter()
                            .map(|e| {
                                Expression::parse(py, e)
                                    .into_pyobject(py)
                                    .expect("parsing call args")
                                    .into()
                            })
                            .collect(),
                        source_loc: Some(OriginalSourceLocation::from_location(&term.location)),
                    },
                    dreammaker::ast::Term::SelfCall(args) => Self::SelfCall {
                        args: args
                            .iter()
                            .map(|e| {
                                Expression::parse(py, e)
                                    .into_pyobject(py)
                                    .expect("parsing selfcall args")
                                    .into()
                            })
                            .collect(),
                        source_loc: Some(OriginalSourceLocation::from_location(&term.location)),
                    },
                    dreammaker::ast::Term::ParentCall(args) => Self::ParentCall {
                        args: args
                            .iter()
                            .map(|e| {
                                Expression::parse(py, e)
                                    .into_pyobject(py)
                                    .expect("parsing parentcall args")
                                    .into()
                            })
                            .collect(),
                        source_loc: Some(OriginalSourceLocation::from_location(&term.location)),
                    },
                    dreammaker::ast::Term::NewImplicit { args } => Self::NewImplicit {
                        args: args.as_ref().map(|args| {
                            args.iter()
                                .map(|arg| {
                                    Expression::parse(py, arg)
                                        .into_pyobject(py)
                                        .expect("parsing new implicit arg")
                                        .into()
                                })
                                .collect()
                        }),
                        source_loc: Some(OriginalSourceLocation::from_location(&term.location)),
                    },
                    dreammaker::ast::Term::NewPrefab { prefab, args } => Self::NewPrefab {
                        prefab: Prefab::make(py, prefab)
                            .into_pyobject(py)
                            .expect("parsing new prefab")
                            .into(),
                        args: args.as_ref().map(|args| {
                            args.iter()
                                .map(|expr| {
                                    Expression::parse(py, expr)
                                        .into_pyobject(py)
                                        .expect("parsing new prefab arg")
                                        .into()
                                })
                                .collect()
                        }),
                        source_loc: Some(OriginalSourceLocation::from_location(&term.location)),
                    },
                    dreammaker::ast::Term::NewMiniExpr { expr, args: _ } => Self::NewMiniExpr {
                        name: Expression::ident(expr.ident.to_string(), None, py),
                        fields: expr
                            .fields
                            .iter()
                            .map(|f| {
                                Expression::Field {
                                    expr: None,
                                    field: Expression::ident(f.ident.to_string(), None, py),
                                    source_loc: None,
                                }
                                .into_pyobject(py)
                                .expect("parsing newminiexpr fields")
                                .into()
                            })
                            .collect(),
                        source_loc: Some(OriginalSourceLocation::from_location(&term.location)),
                    },
                    dreammaker::ast::Term::List(l) => {
                        let mut keys: Vec<PyExpr> = vec![];
                        let mut vals: Vec<PyExpr> = vec![];

                        for args in l.iter() {
                            match args {
                                dreammaker::ast::Expression::Base { .. } => {
                                    keys.push(
                                        Expression::parse(py, args)
                                            .into_pyobject(py)
                                            .expect("parsing base expr")
                                            .into(),
                                    );
                                    vals.push(Expression::null(None, py));
                                }
                                dreammaker::ast::Expression::AssignOp { op: _, lhs, rhs } => {
                                    keys.push(
                                        Expression::parse(py, lhs)
                                            .into_pyobject(py)
                                            .expect("parsing assign op lhs")
                                            .into(),
                                    );
                                    vals.push(
                                        Expression::parse(py, rhs)
                                            .into_pyobject(py)
                                            .expect("parsing assign op rhs")
                                            .into(),
                                    );
                                }
                                dreammaker::ast::Expression::BinaryOp { .. } => {
                                    keys.push(
                                        Expression::parse(py, args)
                                            .into_pyobject(py)
                                            .expect("parsing list binary op key")
                                            .into(),
                                    );
                                    vals.push(Expression::null(None, py));
                                }
                                dreammaker::ast::Expression::TernaryOp { cond, if_, else_ } => {
                                    keys.push(
                                        Self::TernaryOp {
                                            cond: Expression::parse(py, cond)
                                                .into_pyobject(py)
                                                .expect("bad ternary op cond")
                                                .into(),
                                            if_expr: Expression::parse(py, if_)
                                                .into_pyobject(py)
                                                .expect("bad ternary op if_expr")
                                                .into(),
                                            else_expr: Expression::parse(py, else_)
                                                .into_pyobject(py)
                                                .expect("bad ternary op else_expr")
                                                .into(),
                                            source_loc: None,
                                        }
                                        .into_pyobject(py)
                                        .expect("bad ternary op")
                                        .into(),
                                    );
                                    vals.push(Expression::null(None, py));
                                }
                            }
                        }
                        Self::List {
                            list: DmList {
                                keys: keys.iter().map(|k| k.clone_ref(py).into_any()).collect(),
                                vals: vals.iter().map(|k| k.clone_ref(py).into_any()).collect(),
                            }
                            .into_pyobject(py)
                            .expect("bad list")
                            .into(),
                            source_loc: Some(OriginalSourceLocation::from_location(&term.location)),
                        }
                    }
                    dreammaker::ast::Term::Input {
                        args,
                        input_type,
                        in_list,
                    } => Self::Input {
                        args: args
                            .iter()
                            .map(|expr| {
                                Expression::parse(py, expr)
                                    .into_pyobject(py)
                                    .expect("parsing input args")
                                    .into()
                            })
                            .collect(),
                        input_type: input_type.as_ref().map(|it| it.bits()),
                        in_list: in_list.as_ref().map(|in_list| {
                            Expression::parse(py, in_list)
                                .into_pyobject(py)
                                .expect("parsing input in-list")
                                .into()
                        }),
                        source_loc: Some(OriginalSourceLocation::from_location(&term.location)),
                    },
                    dreammaker::ast::Term::Locate { args, in_list } => Self::Locate {
                        args: args
                            .iter()
                            .map(|expr| {
                                Expression::parse(py, expr)
                                    .into_pyobject(py)
                                    .expect("parsing locate args")
                                    .into()
                            })
                            .collect(),
                        in_list: in_list.as_ref().map(|expr| {
                            Expression::parse(py, expr)
                                .into_pyobject(py)
                                .expect("parsing locate in-list")
                                .into()
                        }),
                        source_loc: Some(OriginalSourceLocation::from_location(&term.location)),
                    },
                    dreammaker::ast::Term::Pick(p) => Self::Pick {
                        args: p
                            .iter()
                            .map(|(a, b)| {
                                (
                                    a.as_ref().map(|expr| {
                                        Expression::parse(py, expr)
                                            .into_pyobject(py)
                                            .expect("parsing pick arg key")
                                            .into()
                                    }),
                                    Expression::parse(py, b)
                                        .into_pyobject(py)
                                        .expect("parsing pick arg val")
                                        .into(),
                                )
                            })
                            .collect(),
                        source_loc: Some(OriginalSourceLocation::from_location(&term.location)),
                    },
                    dreammaker::ast::Term::DynamicCall(lib_name, proc_name) => Self::DynamicCall {
                        lib_name: lib_name
                            .iter()
                            .map(|expr| {
                                Expression::parse(py, expr)
                                    .into_pyobject(py)
                                    .expect("parsing dynamic call lib name")
                                    .into()
                            })
                            .collect(),
                        proc_name: proc_name
                            .iter()
                            .map(|expr| {
                                Expression::parse(py, expr)
                                    .into_pyobject(py)
                                    .expect("parsing dynamic call proc name")
                                    .into()
                            })
                            .collect(),
                        source_loc: Some(OriginalSourceLocation::from_location(&term.location)),
                    },
                    dreammaker::ast::Term::ExternalCall {
                        library_name,
                        function_name,
                        args,
                    } => Self::ExternalCall {
                        library_name: Expression::parse(py, library_name)
                            .into_pyobject(py)
                            .expect("parsing external call lib name")
                            .into(),
                        function_name: Expression::parse(py, function_name)
                            .into_pyobject(py)
                            .expect("parsing external call func name")
                            .into(),
                        args: args
                            .iter()
                            .map(|a| {
                                Expression::parse(py, a)
                                    .into_pyobject(py)
                                    .expect("parsing external call arg")
                                    .into()
                            })
                            .collect(),
                        source_loc: Some(OriginalSourceLocation::from_location(&term.location)),
                    },
                    dreammaker::ast::Term::GlobalIdent(_) => todo!(),
                    dreammaker::ast::Term::GlobalCall(_, _) => todo!(),
                };

                for f in follow.iter() {
                    match &f.elem {
                        dreammaker::ast::Follow::Index(_, expression) => {
                            core = Self::Index {
                                expr: core
                                    .into_pyobject(py)
                                    .expect("parsing term follow index expr")
                                    .into(),
                                index: Expression::parse(py, expression)
                                    .into_pyobject(py)
                                    .expect("parsing term follow index")
                                    .into(),
                                source_loc: Some(OriginalSourceLocation::from_location(
                                    &f.location,
                                )),
                            };
                        }
                        dreammaker::ast::Follow::Field(_, ident2) => {
                            core = Self::Field {
                                expr: Some(
                                    core.into_pyobject(py)
                                        .expect("parsing term follow field expr")
                                        .into(),
                                ),
                                field: Expression::ident(ident2.to_string(), None, py),
                                source_loc: Some(OriginalSourceLocation::from_location(
                                    &f.location,
                                )),
                            };
                        }
                        dreammaker::ast::Follow::Call(_, ident2, args) => {
                            core = Self::Call {
                                expr: core
                                    .into_pyobject(py)
                                    .expect("parsing term follow call")
                                    .into(),
                                name: Expression::ident(ident2.to_string(), None, py),
                                args: args
                                    .iter()
                                    .map(|e| {
                                        Expression::parse(py, e)
                                            .into_pyobject(py)
                                            .expect("parsing term follow arg")
                                            .into()
                                    })
                                    .collect(),
                                source_loc: Some(OriginalSourceLocation::from_location(
                                    &f.location,
                                )),
                            }
                        }
                        dreammaker::ast::Follow::Unary(unary_op) => {
                            core = Self::UnaryOp {
                                expr: core
                                    .into_pyobject(py)
                                    .expect("parsing term follow unary op")
                                    .into(),
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
                                source_loc: Some(OriginalSourceLocation::from_location(
                                    &f.location,
                                )),
                            }
                        }
                        dreammaker::ast::Follow::StaticField(ident2) => {
                            core = Self::StaticField {
                                expr: core
                                    .into_pyobject(py)
                                    .expect("parsing term follow static field")
                                    .into(),
                                field: Expression::ident(ident2.to_string(), None, py),
                                source_loc: Some(OriginalSourceLocation::from_location(
                                    &f.location,
                                )),
                            };
                        }
                        dreammaker::ast::Follow::ProcReference(ident2) => {
                            core = Self::ProcReference {
                                expr: core
                                    .into_pyobject(py)
                                    .expect("parsing term follow proc ref expr")
                                    .into(),
                                name: Expression::ident(ident2.to_string(), None, py)
                                    .into_pyobject(py)
                                    .expect("parsing term follow proc ref name")
                                    .into(),
                                source_loc: Some(OriginalSourceLocation::from_location(
                                    &f.location,
                                )),
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
                },
                lhs: Self::parse(py, lhs)
                    .into_pyobject(py)
                    .expect("parsing binary op lhs")
                    .into(),
                rhs: Self::parse(py, rhs)
                    .into_pyobject(py)
                    .expect("parsing binary op rhs")
                    .into(),
                source_loc: None,
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
                lhs: Self::parse(py, lhs)
                    .into_pyobject(py)
                    .expect("parsing assign op lhs")
                    .into(),
                rhs: Self::parse(py, rhs)
                    .into_pyobject(py)
                    .expect("parsing assign op rhs")
                    .into(),
                source_loc: None,
            },
            dreammaker::ast::Expression::TernaryOp { cond, if_, else_ } => Self::TernaryOp {
                cond: Self::parse(py, cond)
                    .into_pyobject(py)
                    .expect("parsing ternary op condition")
                    .into(),
                if_expr: Self::parse(py, if_)
                    .into_pyobject(py)
                    .expect("parsing ternary op if")
                    .into(),
                else_expr: Self::parse(py, else_)
                    .into_pyobject(py)
                    .expect("parsing ternary op else")
                    .into(),
                source_loc: None,
            },
        }
    }
}
