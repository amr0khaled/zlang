use crate::{
    iterative_num,
    parser::ops::Operator::{
        EndArithmatic, EndAssign, EndBinary, EndComparison, EndLogical, EndUnaryOp,
    },
};

iterative_num! {
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Operator: u64 {
    // Arithamtic
    Add,     // +
    Sub,    // -
    Mul, // *
    Power,    // **
    Div,   // /
    Mod,  // %
    EndArithmatic,
    // Logical
    And, // &&
    Or,  // ||
    Not, // !
    EndLogical,
    // Binray
    BinaryAnd, // &
    BinaryOr,  // |
    BinaryXor, // ^
    // TODO: Add shifting
    EndBinary,
    // Comparisons
    Equality,           // ==
    NotEquality,       // !=
    GreaterThan,        // >
    LesserThan,         // <
    GreaterThanOrEqual, // >=
    LesserThanOrEqual,  // <=
    EndComparison,
    // Assigning Operators
    Assign,             // =
    AssignNew,       // <-
    AssignPlus,      // +=
    AssignMinus,     // -=
    AssignMultiply,  // *=
    AssignPower,     // **=
    AssignAnd,       // &&=
    AssignOr,        // ||=
    AssignDivide,    // /=
    AssignModolus,   // %=
    AssignBinaryAnd, // &=
    AssignBinaryOr,  // |=
    AssignBinaryXor, // ^=
    EndAssign,
    PreIncrement,       // ++
    PostIncrement,       // ++
    PreDecrement,       // --
    PostDecrement,       // --
    EndUnaryOp,
}
}
pub fn get_hint<'a>(op: Operator) -> &'a str {
    if op < EndArithmatic {
        "ArithamticExpr"
    } else if op < EndLogical {
        "LogicalExpr"
    } else if op < EndBinary {
        "BinrayExpr"
    } else if op < EndComparison {
        "ComparisonExpr"
    } else if op < EndAssign {
        "AssignExpr"
    } else if op < EndUnaryOp {
        "UnaryExpr"
    } else {
        "Unknown"
    }
}
