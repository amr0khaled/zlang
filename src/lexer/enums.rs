use crate::lexer::keywords::Keyword;
#[derive(Debug)]
pub enum TokenEnum<'a> {
    KeywordType(&'a Keyword),
    IdentifierType,
    IntegerLiteral, // 34
    FloatLiteral,   // 4.3
    BoolLiteral,    // 4.3
    // Ends
    EOF,       // End of file // -1
    SemiColon, // ;
    EOL,       // end of line
    // Operations
    Plus,            // +
    Minus,           // -
    Multiply,        // *
    Power,           // **
    Divide,          // /
    Modolus,         // %
    Assign,          // =
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
    // Logic Operations
    And, // &&
    Or,  // ||
    Not, // !
    // Binary Operations
    BinaryAnd, // &
    BinaryOr,  // |
    BinaryXor, // ^
    // not is the same as logic not
    // Comapres
    Equality,           // ==
    GreaterThan,        // >
    LesserThan,         // <
    GreaterThanOrEqual, // >=
    LesserThanOrEqual,  // <=
    OpenParenthese,     // (
    CloseParenthese,    // )
    OpenBracket,        // {
    CloseBracket,       // }
    OpenSquareBracket,  // [
    CloseSquareBracket, // ]
}
