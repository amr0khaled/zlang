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
    AssignNot,       // !=
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
    Int,
    // Literal
    Ident,
    IntegerLiteral, // 34
    FloatLiteral,   // 4.3
    BoolLiteral,    // true / false
    // Ends
    EOF,       // End of file // -1
    SemiColon, // ;
    EOL,       // end of line
}
}
