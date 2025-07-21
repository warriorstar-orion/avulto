from typing import Any

from .. import SourceLoc

class UnaryOperator:
    NEG: "UnaryOperator"
    NOT: "UnaryOperator"
    BIT_NOT: "UnaryOperator"
    PRE_INCR: "UnaryOperator"
    POST_INCR: "UnaryOperator"
    PRE_DECR: "UnaryOperator"
    POST_DECR: "UnaryOperator"
    REF: "UnaryOperator"
    DEREF: "UnaryOperator"

class Operator:
    ASSIGN: "Operator"
    ASSIGN_ADD: "Operator"
    ASSIGN_SUB: "Operator"
    ASSIGN_MUL: "Operator"
    ASSIGN_DIV: "Operator"
    ASSIGN_MOD: "Operator"
    ASSIGN_FLOAT_MOD: "Operator"
    ASSIGN_INTO: "Operator"
    ASSIGN_BIT_AND: "Operator"
    ASSIGN_AND: "Operator"
    ASSIGN_OR: "Operator"
    ASSIGN_BIT_OR: "Operator"
    ASSIGN_BIT_XOR: "Operator"
    ASSIGN_L_SHIFT: "Operator"
    ASSIGN_R_SHIFT: "Operator"

class SettingMode:
    ASSIGN: "SettingMode"
    IN: "SettingMode"

class BinaryOperator:
    ADD: "BinaryOperator"
    SUB: "BinaryOperator"
    MUL: "BinaryOperator"
    DIV: "BinaryOperator"
    POW: "BinaryOperator"
    MOD: "BinaryOperator"
    FLOAT_MOD: "BinaryOperator"
    EQ: "BinaryOperator"
    NOT_EQ: "BinaryOperator"
    LESS: "BinaryOperator"
    GREATER: "BinaryOperator"
    LESS_EQ: "BinaryOperator"
    GREATER_EQ: "BinaryOperator"
    EQUIV: "BinaryOperator"
    NOT_EQUIV: "BinaryOperator"
    BIT_AND: "BinaryOperator"
    BIT_XOR: "BinaryOperator"
    BIT_OR: "BinaryOperator"
    L_SHIFT: "BinaryOperator"
    R_SHIFT: "BinaryOperator"
    AND: "BinaryOperator"
    OR: "BinaryOperator"
    IN: "BinaryOperator"
    TO: "BinaryOperator"

class NodeKind:
    ASSIGN_OP: "NodeKind"
    ATTRIBUTE: "NodeKind"
    BINARY_OP: "NodeKind"
    BREAK: "NodeKind"
    CALL: "NodeKind"
    CONSTANT: "NodeKind"
    CONTINUE: "NodeKind"
    CRASH: "NodeKind"
    DEL: "NodeKind"
    DO_WHILE: "NodeKind"
    DYNAMIC_CALL: "NodeKind"
    EXPRESSION: "NodeKind"
    EXTERNAL_CALL: "NodeKind"
    FIELD: "NodeKind"
    FOR_INFINITE: "NodeKind"
    FOR_LIST: "NodeKind"
    FOR_LOOP: "NodeKind"
    FOR_RANGE: "NodeKind"
    GOTO: "NodeKind"
    IDENTIFIER: "NodeKind"
    IF: "NodeKind"
    IF_ARM: "NodeKind"
    IF_ELSE: "NodeKind"
    INDEX: "NodeKind"
    INPUT: "NodeKind"
    INTERP_STRING: "NodeKind"
    LABEL: "NodeKind"
    LIST: "NodeKind"
    LOCATE: "NodeKind"
    MINI_EXPR: "NodeKind"
    NEW_IMPLICIT: "NodeKind"
    NEW_MINI_EXPR: "NodeKind"
    NEW_PREFAB: "NodeKind"
    PARENT_CALL: "NodeKind"
    PICK: "NodeKind"
    PREFAB: "NodeKind"
    PROC_REFERENCE: "NodeKind"
    RETURN: "NodeKind"
    SELF_CALL: "NodeKind"
    SETTING: "NodeKind"
    SPAWN: "NodeKind"
    STATIC_FIELD: "NodeKind"
    SWITCH: "NodeKind"
    SWITCH_CASE: "NodeKind"
    TERM: "NodeKind"
    TERNARY_OP: "NodeKind"
    THROW: "NodeKind"
    TRY_CATCH: "NodeKind"
    UNARY_OP: "NodeKind"
    UNKNOWN: "NodeKind"
    VAR: "NodeKind"
    VARS: "NodeKind"
    WHILE: "NodeKind"

class Expression:
    kind: NodeKind

    # technically not part of the "parent class"
    # but we don't need to care about that
    source_loc: SourceLoc

    class AssignOp(Expression):
        lhs: Any
        rhs: Any
        op: Operator

# etc etc but I'm not writing all this out until I'm 100%
# happy with the structure of the API
