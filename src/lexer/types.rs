use crate::iterative_num;

iterative_num! {
#[derive(Debug, PartialEq, Eq)]
pub enum TokenType : u64 {
    // Operations
    Plus,            // +
    Minus,           // -
    Multiply,        // *
    Power,           // **
    Divide,          // /
    Modolus,         // %
    EndArithmatic,
    // Logic Operations
    And, // &&
    Or,  // ||
    Not, // !
    // Binary Operations
    BinaryAnd, // &
    BinaryOr,  // |
    BinaryXor, // ^
    EndBinary,
    // not is the same as logic not
    // Comapres
    Equality,           // ==
    NotEquality,           // !=
    GreaterThan,        // >
    LesserThan,         // <
    GreaterThanOrEqual, // >=
    LesserThanOrEqual,  // <=
    EndComparison,
    // Assginging
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
    Increment,       // ++
    Decrement,       // --
    EndAssign,
    Assign,          // =
    Comma,              // ,
    OpenParenthese,     // (
    CloseParenthese,    // )
    OpenBracket,        // {
    CloseBracket,       // }
    OpenSquareBracket,  // [
    CloseSquareBracket, // ]
    // Keyword
    Let,
    Const,
    Function,
    Return,
    // Data types
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
    Bool,
    // Literal
    Ident,
    IntLiteral, // 34
    FloatLiteral,   // 4.3
    BoolLiteral,    // true / false
    Uninitialized,
    Colon, // :
    // Ends
    EOF,       // End of file // -1
    SemiColon, // ;
    EOL,       // end of line
}
}
