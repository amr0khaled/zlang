use crate::{lexer::types::TokenType, util::Iter};

pub mod types;

#[derive(Debug, PartialEq, Eq)]
pub struct Token<'a> {
    pub kind: TokenType,
    pub pointer: &'a str,
    pub line: usize,
}
impl<'a> Token<'a> {
    pub fn new(token_type: TokenType, v: &'a str, line: usize) -> Self {
        Self {
            kind: token_type,
            pointer: v,
            line,
        }
    }
}

#[derive(Debug)]
pub struct Tokenizer<'a> {
    buffer: &'a str,
    iter: Iter<'a>,
    line: usize,
}
impl<'a> Tokenizer<'a> {
    pub const fn new(buffer: &'a str) -> Self {
        Self {
            buffer,
            iter: Iter::new(buffer),
            line: 0,
        }
    }
    fn skip_whitespace(&mut self) {
        loop {
            if self.is_whitespace() {
                self.iter.consume();
            } else {
                break;
            }
        }
    }
    pub fn next_token(&mut self) -> Token<'a> {
        self.skip_whitespace();
        let mut buf = String::new();
        let symbol_token = self.lookup_symbols();
        if symbol_token.is_some() {
            return symbol_token.unwrap();
        }
        let ch = self.iter.current();
        if ch.is_ascii_digit() {
            return self.lookup_numbers();
        }
        if self.is_identifier() {
            while self.is_identifier() {
                buf.push(self.iter.consume());
            }
        }
        let token = self.lookup_keyword(buf);
        token
    }
    fn lookup_numbers(&mut self) -> Token<'a> {
        let mut buf = String::new();
        let mut has_dot = false;
        while self.iter.current().is_ascii_digit() || self.iter.current() == '.' {
            if self.iter.current() == '.' {
                if has_dot {
                    break;
                }
                has_dot = true;
            }
            buf.push(self.iter.consume())
        }
        let len = buf.len();
        if has_dot {
            self.tokenize(TokenType::FloatLiteral, len)
        } else {
            self.tokenize(TokenType::IntegerLiteral, len)
        }
    }
    fn lookup_symbols(&mut self) -> Option<Token<'a>> {
        match self.iter.current() {
            '\0' => Some(self.tokenize(TokenType::EOF, 0)),
            '\n' => {
                self.iter.advance_by(1);
                self.line += 1;
                Some(self.tokenize(TokenType::EOL, 1))
            }
            ';' => {
                self.iter.advance_by(1);
                Some(self.tokenize(TokenType::SemiColon, 1))
            }
            '=' => {
                let peeked = self.iter.peek();
                if peeked == '=' {
                    self.iter.advance_by(2);
                    Some(self.tokenize(TokenType::Equality, 2))
                } else {
                    self.iter.advance_by(1);
                    Some(self.tokenize(TokenType::Assign, 1))
                }
            }
            '<' => {
                let peeked = self.iter.peek();
                if peeked == '-' {
                    self.iter.advance_by(2);
                    Some(self.tokenize(TokenType::AssignNew, 2))
                } else if peeked == '=' {
                    self.iter.advance_by(2);
                    Some(self.tokenize(TokenType::GreaterThanOrEqual, 2))
                } else {
                    self.iter.advance_by(1);
                    Some(self.tokenize(TokenType::GreaterThan, 1))
                }
            }
            '>' => {
                let peeked = self.iter.peek();
                if peeked == '=' {
                    self.iter.advance_by(2);
                    Some(self.tokenize(TokenType::LesserThanOrEqual, 2))
                } else {
                    self.iter.advance_by(1);
                    Some(self.tokenize(TokenType::LesserThan, 1))
                }
            }
            '&' => {
                let peeked = self.iter.peek();
                if peeked == '&' {
                    if self.iter.peekn(2) == '=' {
                        self.iter.advance_by(3);
                        Some(self.tokenize(TokenType::AssignAnd, 3))
                    } else {
                        self.iter.advance_by(2);
                        Some(self.tokenize(TokenType::And, 2))
                    }
                } else if peeked == '=' {
                    self.iter.advance_by(2);
                    Some(self.tokenize(TokenType::AssignBinaryAnd, 2))
                } else {
                    self.iter.advance_by(1);
                    Some(self.tokenize(TokenType::BinaryAnd, 1))
                }
            }
            '|' => {
                let peeked = self.iter.peek();
                if peeked == '|' {
                    if self.iter.peekn(2) == '=' {
                        self.iter.advance_by(3);
                        Some(self.tokenize(TokenType::AssignOr, 3))
                    } else {
                        self.iter.advance_by(2);
                        Some(self.tokenize(TokenType::Or, 2))
                    }
                } else if peeked == '=' {
                    self.iter.advance_by(2);
                    Some(self.tokenize(TokenType::AssignBinaryOr, 2))
                } else {
                    self.iter.advance_by(1);
                    Some(self.tokenize(TokenType::BinaryOr, 1))
                }
            }
            '+' => {
                let peeked = self.iter.peek();
                if peeked == '=' {
                    self.iter.advance_by(2);
                    Some(self.tokenize(TokenType::AssignPlus, 2))
                } else if peeked == '+' {
                    self.iter.advance_by(2);
                    Some(self.tokenize(TokenType::Increment, 2))
                } else {
                    self.iter.advance_by(1);
                    Some(self.tokenize(TokenType::Plus, 1))
                }
            }
            '-' => {
                let peeked = self.iter.peek();
                if peeked == '=' {
                    self.iter.advance_by(2);
                    Some(self.tokenize(TokenType::AssignMinus, 2))
                } else if peeked == '-' {
                    self.iter.advance_by(2);
                    Some(self.tokenize(TokenType::Decrement, 2))
                } else {
                    self.iter.advance_by(1);
                    Some(self.tokenize(TokenType::Minus, 1))
                }
            }
            '*' => {
                let peeked = self.iter.peek();
                if peeked == '*' {
                    if self.iter.peekn(2) == '=' {
                        self.iter.advance_by(3);
                        Some(self.tokenize(TokenType::AssignPower, 3))
                    } else {
                        self.iter.advance_by(2);
                        Some(self.tokenize(TokenType::Power, 2))
                    }
                } else if peeked == '=' {
                    self.iter.advance_by(2);
                    Some(self.tokenize(TokenType::AssignMultiply, 2))
                } else {
                    self.iter.advance_by(1);
                    Some(self.tokenize(TokenType::Multiply, 1))
                }
            }
            '/' => {
                let peeked = self.iter.peek();
                if peeked == '=' {
                    self.iter.advance_by(2);
                    Some(self.tokenize(TokenType::AssignDivide, 2))
                } else {
                    self.iter.advance_by(1);
                    Some(self.tokenize(TokenType::Divide, 1))
                }
            }
            '%' => {
                let peeked = self.iter.peek();
                if peeked == '=' {
                    self.iter.advance_by(2);
                    Some(self.tokenize(TokenType::AssignModolus, 2))
                } else {
                    self.iter.advance_by(1);
                    Some(self.tokenize(TokenType::Modolus, 1))
                }
            }
            '!' => {
                let peeked = self.iter.peek();
                if peeked == '=' {
                    self.iter.advance_by(2);
                    Some(self.tokenize(TokenType::AssignNot, 2))
                } else {
                    self.iter.advance_by(1);
                    Some(self.tokenize(TokenType::Not, 1))
                }
            }
            '^' => {
                let peeked = self.iter.peek();
                if peeked == '=' {
                    self.iter.advance_by(2);
                    Some(self.tokenize(TokenType::AssignBinaryXor, 2))
                } else {
                    self.iter.advance_by(1);
                    Some(self.tokenize(TokenType::BinaryXor, 1))
                }
            }
            ',' => {
                self.iter.advance_by(1);
                Some(self.tokenize(TokenType::Comma, 1))
            }
            '(' => {
                self.iter.advance_by(1);
                Some(self.tokenize(TokenType::OpenParenthese, 1))
            }
            ')' => {
                self.iter.advance_by(1);
                Some(self.tokenize(TokenType::CloseParenthese, 1))
            }
            '{' => {
                self.iter.advance_by(1);
                Some(self.tokenize(TokenType::OpenBracket, 1))
            }
            '}' => {
                self.iter.advance_by(1);
                Some(self.tokenize(TokenType::CloseBracket, 1))
            }
            '[' => {
                self.iter.advance_by(1);
                Some(self.tokenize(TokenType::OpenSquareBracket, 1))
            }
            ']' => {
                self.iter.advance_by(1);
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
            "int" => self.tokenize(TokenType::Int, len),
            "const" => self.tokenize(TokenType::Const, len),
            "true" => self.tokenize(TokenType::BoolLiteral, len),
            "false" => self.tokenize(TokenType::BoolLiteral, len),
            _ => self.tokenize(TokenType::Ident, len),
        }
    }
    fn tokenize(&self, kind: TokenType, len: usize) -> Token<'a> {
        Token::new(kind, self.get_slice(len), self.line)
    }
    fn is_whitespace(&mut self) -> bool {
        let ch = self.iter.current();
        ch == ' ' || ch == '\t'
    }
    fn is_identifier(&mut self) -> bool {
        let ch = self.iter.current();
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
    fn test_int_tokenizers() {
        let line = "int x = 5;";
        let mut tokenizer = Tokenizer::new(line);
        assert_eq!(Token::new(TokenType::Int, "int", 0), tokenizer.next_token());
        assert_eq!(Token::new(TokenType::Ident, "x", 0), tokenizer.next_token());
        assert_eq!(
            Token::new(TokenType::Assign, "=", 0),
            tokenizer.next_token()
        );
        assert_eq!(
            Token::new(TokenType::IntegerLiteral, "5", 0),
            tokenizer.next_token()
        );
        assert_eq!(
            Token::new(TokenType::SemiColon, ";", 0),
            tokenizer.next_token()
        );
        assert_eq!(Token::new(TokenType::EOF, "", 0), tokenizer.next_token());
    }

    #[test]
    fn test_let_tokenizers() {
        let line = "let x = 5.0";
        let mut tokenizer = Tokenizer::new(line);
        assert_eq!(Token::new(TokenType::Let, "let", 0), tokenizer.next_token());
        assert_eq!(Token::new(TokenType::Ident, "x", 0), tokenizer.next_token());
        assert_eq!(
            Token::new(TokenType::Assign, "=", 0),
            tokenizer.next_token()
        );
        assert_eq!(
            Token::new(TokenType::FloatLiteral, "5.0", 0),
            tokenizer.next_token()
        );
        assert_ne!(
            Token::new(TokenType::SemiColon, ";", 0),
            tokenizer.next_token()
        );
        assert_eq!(Token::new(TokenType::EOF, "", 0), tokenizer.next_token());
    }

    #[test]
    fn test_function_tokenizers() {
        let line = "print(1, 2, 3)";
        let mut tokenizer = Tokenizer::new(line);
        assert_eq!(
            Token::new(TokenType::Ident, "print", 0),
            tokenizer.next_token()
        );
        assert_eq!(
            Token::new(TokenType::OpenParenthese, "(", 0),
            tokenizer.next_token()
        );
        assert_eq!(
            Token::new(TokenType::IntegerLiteral, "1", 0),
            tokenizer.next_token()
        );
        assert_eq!(Token::new(TokenType::Comma, ",", 0), tokenizer.next_token());
        assert_eq!(
            Token::new(TokenType::IntegerLiteral, "2", 0),
            tokenizer.next_token()
        );
        assert_eq!(Token::new(TokenType::Comma, ",", 0), tokenizer.next_token());
        assert_eq!(
            Token::new(TokenType::IntegerLiteral, "3", 0),
            tokenizer.next_token()
        );
        assert_eq!(
            Token::new(TokenType::CloseParenthese, ")", 0),
            tokenizer.next_token()
        );
        assert_eq!(Token::new(TokenType::EOF, "", 0), tokenizer.next_token());
    }
}
