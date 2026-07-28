use core::panic;

use crate::{
    analyzer::symbol::{Symbol, SymbolTable, SymbolType},
    cg::opcode::OpCode::*,
    parser::{node::Node, ops::Operator},
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sym {
    pub id: u32,
    pub depth: usize,
}

impl Sym {
    pub fn from(r: &Node) -> Self {
        match *r {
            Node::Ref(id, depth) => Self { id, depth },
            _ => panic!("Node {:?} is not a Ref node", r),
        }
    }
}
#[derive(Debug, PartialEq)]
pub enum OpCode {
    // ===Literal===
    // ==Unsigned Integer==
    PushUint8(u8),
    PushUint16(u16),
    PushUint32(u32),
    PushUint64(u64),
    PushUint128(u128),
    // ==Signed Integer==
    PushInt8(i8),
    PushInt16(i16),
    PushInt32(i32),
    PushInt64(i64),
    PushInt128(i128),
    // ==Float==
    PushFloat32(f32),
    PushFloat64(f64),
    // ==Boolean==
    PushBool(bool),
    // ===Operation===
    // ==Arithmatic==
    // =Integer=
    Add,  // +
    Sub,  // -
    Mul,  // *
    Imul, // Signed *
    Div,  // /
    // =Float=
    Fadd,  // +
    Fsub,  // -
    Fmul,  // *
    Fimul, // Signed *
    Fdiv,  // /
    // Common
    Power, // **
    Mod,   // %
    // ==Logical==
    And, // &&
    Or,  // ||
    Not, // !
    // ==Binary==
    BiAnd, // &
    BiOr,  // |
    BiXor, // ^
    // ==Comparisons==
    Equality,           // ==
    NotEquality,        // !=
    GreaterThan,        // >
    LesserThan,         // <
    GreaterThanOrEqual, // >=
    LesserThanOrEqual,  // <=
    // ==Assign==
    Assign,
    // ===Ident===
    // ==Variables==
    Store(Sym), // Store current value to u32 at depth usize
    Load(Sym),  // Load from u32 to be current value
    // ==Function==
    Call(Sym),
    Return,
    Push,
    Pop,
    // ==Scopes==
    ScopeStart(usize),
    ScopeEnd(usize),
    Nop,
}

impl OpCode {
    pub fn operator_to_opcode(op: &Operator, ty: &Option<SymbolType>) -> Self {
        match (op, ty) {
            (Operator::Add, Some(SymbolType::Float)) => Fadd,
            (Operator::Add, _) => Add,
            (Operator::Sub, Some(SymbolType::Float)) => Fsub,
            (Operator::Sub, _) => Sub,
            (Operator::Mul, Some(SymbolType::Float)) => Fimul, // TODO: Need to idenitify imul from mul
            (Operator::Mul, _) => Imul,
            (Operator::Div, Some(SymbolType::Float)) => Fdiv,
            (Operator::Div, _) => Div,
            (Operator::Power, _) => Power,
            (Operator::Mod, _) => Mod,
            (Operator::And, _) => And,
            (Operator::Or, _) => Or,
            (Operator::Not, _) => Not,
            (Operator::BinaryAnd, _) => BiAnd,
            (Operator::BinaryOr, _) => BiOr,
            (Operator::BinaryXor, _) => BiXor,
            (Operator::Equality, _) => Equality,
            (Operator::NotEquality, _) => NotEquality,
            (Operator::GreaterThan, _) => GreaterThan,
            (Operator::LesserThan, _) => LesserThan,
            (Operator::GreaterThanOrEqual, _) => GreaterThanOrEqual,
            (Operator::LesserThanOrEqual, _) => LesserThanOrEqual,
            _ => panic!("What is happning operator_to_opcode"),
        }
    }
    pub fn assign_to_opcode(op: &Operator, ty: SymbolType) -> Self {
        match (op, ty) {
            (Operator::Assign, _) => Nop,
            (Operator::AssignNew, _) => Nop,
            (Operator::AssignPlus, SymbolType::Float) => Fadd,
            (Operator::AssignPlus, _) => Add,
            (Operator::AssignMinus, SymbolType::Float) => Fsub,
            (Operator::AssignMinus, _) => Sub,
            (Operator::AssignMultiply, SymbolType::Float) => Fimul,
            (Operator::AssignMultiply, _) => Imul,
            (Operator::AssignDivide, SymbolType::Float) => Fdiv,
            (Operator::AssignDivide, _) => Div,
            (Operator::AssignPower, _) => Power,
            (Operator::AssignAnd, _) => And,
            (Operator::AssignOr, _) => Or,
            (Operator::AssignModolus, _) => Mod,
            (Operator::AssignBinaryAnd, _) => BiAnd,
            (Operator::AssignBinaryOr, _) => BiOr,
            (Operator::AssignBinaryXor, _) => BiXor,
            _ => panic!("Not supported"),
        }
    }
}
