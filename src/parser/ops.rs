use crate::{
    iterative_num,
    lexer::types::TokenType,
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
impl Operator {
    pub fn from_token(token: TokenType) -> Option<Self> {
        match token {
            TokenType::Plus => Some(Self::Add),
            TokenType::Minus => Some(Self::Sub),
            TokenType::Multiply => Some(Self::Mul),
            TokenType::Power => Some(Self::Power),
            TokenType::Divide => Some(Self::Div),
            TokenType::Modolus => Some(Self::Mod),
            TokenType::And => Some(Self::And),
            TokenType::Or => Some(Self::Or),
            TokenType::Not => Some(Self::Not),
            TokenType::BinaryAnd => Some(Self::BinaryAnd),
            TokenType::BinaryOr => Some(Self::BinaryOr),
            TokenType::BinaryXor => Some(Self::BinaryXor),
            TokenType::Equality => Some(Self::Equality),
            TokenType::NotEquality => Some(Self::NotEquality),
            TokenType::GreaterThan => Some(Self::GreaterThan),
            TokenType::LesserThan => Some(Self::LesserThan),
            TokenType::GreaterThanOrEqual => Some(Self::GreaterThanOrEqual),
            TokenType::LesserThanOrEqual => Some(Self::LesserThanOrEqual),
            TokenType::AssignNew => Some(Self::AssignNew),
            TokenType::AssignPlus => Some(Self::AssignPlus),
            TokenType::AssignMinus => Some(Self::AssignMinus),
            TokenType::AssignMultiply => Some(Self::AssignMultiply),
            TokenType::AssignPower => Some(Self::AssignPower),
            TokenType::AssignAnd => Some(Self::AssignAnd),
            TokenType::AssignOr => Some(Self::AssignOr),
            TokenType::AssignDivide => Some(Self::AssignDivide),
            TokenType::AssignModolus => Some(Self::AssignModolus),
            TokenType::AssignBinaryAnd => Some(Self::AssignBinaryAnd),
            TokenType::AssignBinaryOr => Some(Self::AssignBinaryOr),
            TokenType::AssignBinaryXor => Some(Self::AssignBinaryXor),
            TokenType::Assign => Some(Self::Assign),
            _ => None,
        }
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
