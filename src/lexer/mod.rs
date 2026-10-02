use crate::{lexer::types::TokenType, util::Iter};

pub mod types;

#[derive(Debug, PartialEq, Eq)]
pub struct Token<'a> {
    pub kind: TokenType,
    pub pointer: &'a str,
    pub line: usize,
    pub col: usize,
}
impl<'a> Token<'a> {
    pub fn new(token_type: TokenType, v: &'a str, line: usize, col: usize) -> Self {
        Self {
            kind: token_type,
            pointer: v,
            line,
            col,
        }
    }
}

#[derive(Debug)]
pub struct Tokenizer<'a> {
    buffer: &'a str,
    iter: Iter<'a>,
    line: usize,
    col: usize,
}
impl<'a> Tokenizer<'a> {
    pub const fn new(buffer: &'a str) -> Self {
        Self {
            buffer,
            iter: Iter::new(buffer),
            line: 1,
            col: 1,
        }
    }
    fn skip_whitespace(&mut self) {
        loop {
            if self.is_whitespace() {
                self.consume();
                self.col += 1;
            } else {
                break;
            }
        }
    }
    fn current(&mut self) -> char {
        self.iter.current()
    }
    fn peek(&mut self) -> char {
        self.iter.peek()
    }
    fn peekn(&mut self, n: usize) -> char {
        self.iter.peekn(n)
    }
    fn consume(&mut self) -> char {
        let c = self.iter.consume();
        c
    }
    fn advance_by(&mut self, n: usize) {
        self.iter.advance_by(n);
        // self.col += n;
    }
    pub fn next_token(&mut self) -> Token<'a> {
        self.skip_whitespace();
        let mut buf = String::new();
        let symbol_token = self.lookup_symbols();
        if symbol_token.is_some() {
            return symbol_token.unwrap();
        }
        let ch = self.current();
        if ch.is_ascii_digit() {
            return self.lookup_numbers();
        }
        if self.is_identifier() {
            while self.is_identifier() {
                buf.push(self.consume());
            }
        }
        let token = self.lookup_keyword(buf);
        token
    }
    fn lookup_numbers(&mut self) -> Token<'a> {
        let mut buf = String::new();
        let mut has_dot = false;
        while self.current().is_ascii_digit() || self.iter.current() == '.' {
            if self.current() == '.' {
                if has_dot {
                    break;
                }
                has_dot = true;
            }
            buf.push(self.consume())
        }
        let len = buf.len();
        if has_dot {
            self.tokenize(TokenType::FloatLiteral, len)
        } else {
            self.tokenize(TokenType::IntLiteral, len)
        }
    }
    fn lookup_symbols(&mut self) -> Option<Token<'a>> {
        match self.current() {
            '\0' => Some(self.tokenize(TokenType::EOF, 0)),
            '\n' => {
                self.advance_by(1);
                self.line += 1;
                self.col = 1;
                Some(self.tokenize(TokenType::EOL, 1))
            }
            ':' => {
                self.advance_by(1);
                Some(self.tokenize(TokenType::Colon, 1))
            }
            ';' => {
                self.advance_by(1);
                Some(self.tokenize(TokenType::SemiColon, 1))
            }
            '=' => {
                let peeked = self.peek();
                if peeked == '=' {
                    self.advance_by(2);
                    Some(self.tokenize(TokenType::Equality, 2))
                } else {
                    self.advance_by(1);
                    Some(self.tokenize(TokenType::Assign, 1))
                }
            }
            '<' => {
                let peeked = self.peek();
                if peeked == '-' {
                    self.advance_by(2);
                    Some(self.tokenize(TokenType::AssignNew, 2))
                } else if peeked == '=' {
                    self.advance_by(2);
                    Some(self.tokenize(TokenType::GreaterThanOrEqual, 2))
                } else {
                    self.advance_by(1);
                    Some(self.tokenize(TokenType::GreaterThan, 1))
                }
            }
            '>' => {
                let peeked = self.peek();
                if peeked == '=' {
                    self.advance_by(2);
                    Some(self.tokenize(TokenType::LesserThanOrEqual, 2))
                } else {
                    self.advance_by(1);
                    Some(self.tokenize(TokenType::LesserThan, 1))
                }
            }
            '&' => {
                let peeked = self.peek();
                if peeked == '&' {
                    if self.peekn(2) == '=' {
                        self.advance_by(3);
                        Some(self.tokenize(TokenType::AssignAnd, 3))
                    } else {
                        self.advance_by(2);
                        Some(self.tokenize(TokenType::And, 2))
                    }
                } else if peeked == '=' {
                    self.advance_by(2);
                    Some(self.tokenize(TokenType::AssignBinaryAnd, 2))
                } else {
                    self.advance_by(1);
                    Some(self.tokenize(TokenType::BinaryAnd, 1))
                }
            }
            '|' => {
                let peeked = self.peek();
                if peeked == '|' {
                    if self.peekn(2) == '=' {
                        self.advance_by(3);
                        Some(self.tokenize(TokenType::AssignOr, 3))
                    } else {
                        self.advance_by(2);
                        Some(self.tokenize(TokenType::Or, 2))
                    }
                } else if peeked == '=' {
                    self.advance_by(2);
                    Some(self.tokenize(TokenType::AssignBinaryOr, 2))
                } else {
                    self.advance_by(1);
                    Some(self.tokenize(TokenType::BinaryOr, 1))
                }
            }
            '+' => {
                let peeked = self.peek();
                if peeked == '=' {
                    self.advance_by(2);
                    Some(self.tokenize(TokenType::AssignPlus, 2))
                } else if peeked == '+' {
                    self.advance_by(2);
                    Some(self.tokenize(TokenType::Increment, 2))
                } else {
                    self.advance_by(1);
                    Some(self.tokenize(TokenType::Plus, 1))
                }
            }
            '-' => {
                let peeked = self.peek();
                if peeked == '=' {
                    self.advance_by(2);
                    Some(self.tokenize(TokenType::AssignMinus, 2))
                } else if peeked == '-' {
                    self.advance_by(2);
                    Some(self.tokenize(TokenType::Decrement, 2))
                } else {
                    self.advance_by(1);
                    Some(self.tokenize(TokenType::Minus, 1))
                }
            }
            '*' => {
                let peeked = self.peek();
                if peeked == '*' {
                    if self.peekn(2) == '=' {
                        self.advance_by(3);
                        Some(self.tokenize(TokenType::AssignPower, 3))
                    } else {
                        self.advance_by(2);
                        Some(self.tokenize(TokenType::Power, 2))
                    }
                } else if peeked == '=' {
                    self.advance_by(2);
                    Some(self.tokenize(TokenType::AssignMultiply, 2))
                } else {
                    self.advance_by(1);
                    Some(self.tokenize(TokenType::Multiply, 1))
                }
            }
            '/' => {
                let peeked = self.peek();
                if peeked == '=' {
                    self.advance_by(2);
                    Some(self.tokenize(TokenType::AssignDivide, 2))
                } else {
                    self.advance_by(1);
                    Some(self.tokenize(TokenType::Divide, 1))
                }
            }
            '%' => {
                let peeked = self.peek();
                if peeked == '=' {
                    self.advance_by(2);
                    Some(self.tokenize(TokenType::AssignModolus, 2))
                } else {
                    self.advance_by(1);
                    Some(self.tokenize(TokenType::Modolus, 1))
                }
            }
            '!' => {
                let peeked = self.peek();
                if peeked == '=' {
                    self.advance_by(2);
                    Some(self.tokenize(TokenType::NotEquality, 2))
                } else {
                    self.advance_by(1);
                    Some(self.tokenize(TokenType::Not, 1))
                }
            }
            '^' => {
                let peeked = self.peek();
                if peeked == '=' {
                    self.advance_by(2);
                    Some(self.tokenize(TokenType::AssignBinaryXor, 2))
                } else {
                    self.advance_by(1);
                    Some(self.tokenize(TokenType::BinaryXor, 1))
                }
            }
            ',' => {
                self.advance_by(1);
                Some(self.tokenize(TokenType::Comma, 1))
            }
            '(' => {
                self.advance_by(1);
                Some(self.tokenize(TokenType::OpenParenthese, 1))
            }
            ')' => {
                self.advance_by(1);
                Some(self.tokenize(TokenType::CloseParenthese, 1))
            }
            '{' => {
                self.advance_by(1);
                Some(self.tokenize(TokenType::OpenBracket, 1))
            }
            '}' => {
                self.advance_by(1);
                Some(self.tokenize(TokenType::CloseBracket, 1))
            }
            '[' => {
                self.advance_by(1);
                Some(self.tokenize(TokenType::OpenSquareBracket, 1))
            }
            ']' => {
                self.advance_by(1);
                Some(self.tokenize(TokenType::CloseSquareBracket, 1))
            }
            _ => None,
        }
    }
    fn lookup_keyword(&mut self, buf: String) -> Token<'a> {
        let len = buf.len();
        match buf.as_str() {
            // declaration
            "let" => self.tokenize(TokenType::Let, len),
            "fn" => self.tokenize(TokenType::Function, len),
            "const" => self.tokenize(TokenType::Const, len),
            "true" => self.tokenize(TokenType::BoolLiteral, len),
            "false" => self.tokenize(TokenType::BoolLiteral, len),
            "return" => self.tokenize(TokenType::Return, len),
            // Data types
            "u8" => self.tokenize(TokenType::U8, len),
            "u16" => self.tokenize(TokenType::U16, len),
            "u32" => self.tokenize(TokenType::U32, len),
            "u64" => self.tokenize(TokenType::U64, len),
            "i8" => self.tokenize(TokenType::I8, len),
            "i16" => self.tokenize(TokenType::I16, len),
            "i32" => self.tokenize(TokenType::I32, len),
            "i64" => self.tokenize(TokenType::I64, len),
            "f32" => self.tokenize(TokenType::F32, len),
            "f64" => self.tokenize(TokenType::F64, len),
            "bool" => self.tokenize(TokenType::Bool, len),
            _ => self.tokenize(TokenType::Ident, len),
        }
    }
    fn tokenize(&mut self, kind: TokenType, len: usize) -> Token<'a> {
        let token = Token::new(kind, self.get_slice(len), self.line, self.col);
        self.col += len;
        token
    }
    fn is_whitespace(&mut self) -> bool {
        let ch = self.current();
        ch == ' ' || ch == '\t'
    }
    fn is_identifier(&mut self) -> bool {
        let ch = self.current();
        ch.is_alphanumeric() || ch == '_'
    }
    fn get_slice(&self, n: usize) -> &'a str {
        let start = self.iter.current - n;
        let end = self.iter.current;
        &self.buffer[start..end]
    }
}

#[cfg(test)]
mod tests {
    use crate::lexer::{Token, Tokenizer, types::TokenType};

    #[test]
    fn test_let_tokenizers() {
        let line = "let x = 5.0";
        let mut tokenizer = Tokenizer::new(line);
        assert_eq!(
            Token::new(TokenType::Let, "let", 1, 1),
            tokenizer.next_token()
        );
        assert_eq!(
            Token::new(TokenType::Ident, "x", 1, 5),
            tokenizer.next_token()
        );
        assert_eq!(
            Token::new(TokenType::Assign, "=", 1, 7),
            tokenizer.next_token()
        );
        assert_eq!(
            Token::new(TokenType::FloatLiteral, "5.0", 1, 9),
            tokenizer.next_token()
        );
        assert_ne!(
            Token::new(TokenType::SemiColon, ";", 1, 12),
            tokenizer.next_token()
        );
        assert_eq!(
            Token::new(TokenType::EOF, "", 1, 12),
            tokenizer.next_token()
        );
    }
    #[test]
    fn test_function_decl() {
        let line = "fn name() {}";
        let mut tokenizer = Tokenizer::new(line);
        assert_eq!(
            Token::new(TokenType::Function, "fn", 1, 1),
            tokenizer.next_token()
        );
        assert_eq!(
            Token::new(TokenType::Ident, "name", 1, 4),
            tokenizer.next_token()
        );
        assert_eq!(
            Token::new(TokenType::OpenParenthese, "(", 1, 8),
            tokenizer.next_token()
        );
        assert_eq!(
            Token::new(TokenType::CloseParenthese, ")", 1, 9),
            tokenizer.next_token()
        );
        assert_eq!(
            Token::new(TokenType::OpenBracket, "{", 1, 11),
            tokenizer.next_token()
        );
        assert_eq!(
            Token::new(TokenType::CloseBracket, "}", 1, 12),
            tokenizer.next_token()
        );
        assert_eq!(
            Token::new(TokenType::EOF, "", 1, 13),
            tokenizer.next_token()
        );
    }

    #[test]
    fn test_function_decl2() {
        let line = "fn name[]() {}";
        let mut tokenizer = Tokenizer::new(line);
        assert_eq!(
            Token::new(TokenType::Function, "fn", 1, 1),
            tokenizer.next_token()
        );
        assert_eq!(
            Token::new(TokenType::Ident, "name", 1, 4),
            tokenizer.next_token()
        );
        assert_eq!(
            Token::new(TokenType::OpenSquareBracket, "[", 1, 8),
            tokenizer.next_token()
        );
        assert_eq!(
            Token::new(TokenType::CloseSquareBracket, "]", 1, 9),
            tokenizer.next_token()
        );
        assert_eq!(
            Token::new(TokenType::OpenParenthese, "(", 1, 10),
            tokenizer.next_token()
        );
        assert_eq!(
            Token::new(TokenType::CloseParenthese, ")", 1, 11),
            tokenizer.next_token()
        );
        assert_eq!(
            Token::new(TokenType::OpenBracket, "{", 1, 13),
            tokenizer.next_token()
        );
        assert_eq!(
            Token::new(TokenType::CloseBracket, "}", 1, 14),
            tokenizer.next_token()
        );
        assert_eq!(
            Token::new(TokenType::EOF, "", 1, 15),
            tokenizer.next_token()
        );
    }
    #[test]
    fn test_function_tokenizers() {
        let line = "print(1, 2, 3)";
        let mut tokenizer = Tokenizer::new(line);
        assert_eq!(
            Token::new(TokenType::Ident, "print", 1, 1),
            tokenizer.next_token()
        );
        assert_eq!(
            Token::new(TokenType::OpenParenthese, "(", 1, 6),
            tokenizer.next_token()
        );
        assert_eq!(
            Token::new(TokenType::IntLiteral, "1", 1, 7),
            tokenizer.next_token()
        );
        assert_eq!(
            Token::new(TokenType::Comma, ",", 1, 8),
            tokenizer.next_token()
        );
        assert_eq!(
            Token::new(TokenType::IntLiteral, "2", 1, 10),
            tokenizer.next_token()
        );
        assert_eq!(
            Token::new(TokenType::Comma, ",", 1, 11),
            tokenizer.next_token()
        );
        assert_eq!(
            Token::new(TokenType::IntLiteral, "3", 1, 13),
            tokenizer.next_token()
        );
        assert_eq!(
            Token::new(TokenType::CloseParenthese, ")", 1, 14),
            tokenizer.next_token()
        );
        assert_eq!(
            Token::new(TokenType::EOF, "", 1, 15),
            tokenizer.next_token()
        );
    }
}
