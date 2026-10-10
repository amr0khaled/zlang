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
    pub fn is_assign_operator(&self) -> bool {
        match self {
            Operator::Assign
            | Operator::AssignNew
            | Operator::AssignPlus
            | Operator::AssignMinus
            | Operator::AssignMultiply
            | Operator::AssignPower
            | Operator::AssignAnd
            | Operator::AssignOr
            | Operator::AssignDivide
            | Operator::AssignModolus
            | Operator::AssignBinaryAnd
            | Operator::AssignBinaryOr
            | Operator::AssignBinaryXor => true,
            _ => false,
        }
    }
    pub fn is_logical_operator(&self) -> bool {
        match self {
            Operator::And | Operator::Or => true,
            _ => false,
        }
    }
    pub fn is_equality_compare(&self) -> bool {
        match self {
            Operator::Equality | Operator::NotEquality => true,
            _ => false,
        }
    }
    pub fn is_compare_operator(&self) -> bool {
        match self {
            Operator::GreaterThan
            | Operator::LesserThan
            | Operator::GreaterThanOrEqual
            | Operator::LesserThanOrEqual => true,
            _ => false,
        }
    }
    pub fn is_prefix_unary(&self) -> bool {
        match self {
            Operator::Not | Operator::PreIncrement | Operator::PreDecrement => true,
            _ => false,
        }
    }
    pub fn is_postfix_unary(&self) -> bool {
        match self {
            Operator::PostIncrement | Operator::PostDecrement => true,
            _ => false,
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

fn get_predence(op: &Operator) -> u8 {
    match op {
        x if x.is_assign_operator() => 1,
        Operator::Or => 2,
        x if x.is_equality_compare() => 4,
        x if x.is_compare_operator() => 5,
        Operator::Sub | Operator::And => 6,
        Operator::Mul | Operator::Div | Operator::Mod => 7,
        Operator::Power => 8,
        x if x.is_prefix_unary() => 9,
        x if x.is_postfix_unary() => 10,
        _ => 0,
    }
}
pub fn get_pred(token: TokenType) -> u8 {
    let op = Operator::from_token(token);
    if op.is_some() {
        return get_predence(&op.unwrap());
    }
    0
}
