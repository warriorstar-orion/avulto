use pyo3::pyclass;

#[pyclass(
    module = "avulto.ast",
    name = "UnaryOperator",
    eq,
    eq_int,
    rename_all = "SCREAMING_SNAKE_CASE"
)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum UnaryOperator {
    Neg,
    Not,
    BitNot,
    PreIncr,
    PostIncr,
    PreDecr,
    PostDecr,
    Ref,
    Deref,
}

#[pyclass(
    module = "avulto.ast",
    name = "Operator",
    eq,
    eq_int,
    rename_all = "SCREAMING_SNAKE_CASE"
)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum AssignOperator {
    Assign,
    AssignAdd,
    AssignSub,
    AssignMul,
    AssignDiv,
    AssignMod,
    AssignFloatMod,
    AssignInto,
    AssignBitAnd,
    AssignAnd,
    AssignOr,
    AssignBitOr,
    AssignBitXor,
    AssignLShift,
    AssignRShift,
}

#[pyclass(
    module = "avulto.ast",
    name = "SettingMode",
    eq,
    eq_int,
    rename_all = "SCREAMING_SNAKE_CASE"
)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum SettingMode {
    Assign,
    In,
}

#[pyclass(
    module = "avulto.ast",
    name = "BinaryOperator",
    eq,
    eq_int,
    rename_all = "SCREAMING_SNAKE_CASE"
)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum BinaryOperator {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    Mod,
    FloatMod,
    Eq,
    NotEq,
    Less,
    Greater,
    LessEq,
    GreaterEq,
    Equiv,
    NotEquiv,
    BitAnd,
    BitXor,
    BitOr,
    LShift,
    RShift,
    And,
    Or,
    In,
    To,
}
