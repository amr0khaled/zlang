use crate::{
    lexer::{
        enums::TokenEnum,
        keywords::{CONST, INTEGER, Keyword, LET},
    },
    util::Iter,
};

mod enums;
mod keywords;
mod symbols;

pub trait TokenType<'a> {
    fn get(&'a self) -> TokenEnum<'a>;
}
#[derive(Debug)]
pub struct Token<'a> {
    pub kind: TokenEnum<'a>,
    pub pointer: &'a str,
    pub line: usize,
}
impl<'a> Token<'a> {
    pub fn new(token_type: TokenEnum<'a>, v: &'a str, line: usize) -> Self {
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
                return self.tokenize(TokenEnum::FloatLiteral, len);
            } else {
                return self.tokenize(TokenEnum::IntegerLiteral, len);
            }
        }
        if self.is_identifier() {
            while self.is_identifier() {
                buf.push(self.iter.consume());
            }
        }
        let token = self.lookup_keyword(buf);
        token
    }
    fn lookup_symbols(&mut self) -> Option<Token<'a>> {
        match self.iter.current() {
            '\0' => Some(self.tokenize(TokenEnum::EOF, 0)),
            '\n' => {
                self.iter.advance_by(1);
                self.line += 1;
                Some(self.tokenize(TokenEnum::EOL, 1))
            }
            ';' => {
                self.iter.advance_by(1);
                Some(self.tokenize(TokenEnum::SemiColon, 1))
            }
            '=' => {
                let peeked = self.iter.peek();
                if peeked == '=' {
                    self.iter.advance_by(2);
                    Some(self.tokenize(TokenEnum::Equality, 2))
                } else {
                    self.iter.advance_by(1);
                    Some(self.tokenize(TokenEnum::Assign, 1))
                }
            }
            '<' => {
                let peeked = self.iter.peek();
                if peeked == '-' {
                    self.iter.advance_by(2);
                    Some(self.tokenize(TokenEnum::AssignNew, 2))
                } else if peeked == '=' {
                    self.iter.advance_by(2);
                    Some(self.tokenize(TokenEnum::GreaterThanOrEqual, 2))
                } else {
                    self.iter.advance_by(1);
                    Some(self.tokenize(TokenEnum::GreaterThan, 1))
                }
            }
            '>' => {
                let peeked = self.iter.peek();
                if peeked == '=' {
                    self.iter.advance_by(2);
                    Some(self.tokenize(TokenEnum::LesserThanOrEqual, 2))
                } else {
                    self.iter.advance_by(1);
                    Some(self.tokenize(TokenEnum::LesserThan, 1))
                }
            }
            '&' => {
                let peeked = self.iter.peek();
                if peeked == '&' {
                    if self.iter.peekn(2) == '=' {
                        self.iter.advance_by(3);
                        Some(self.tokenize(TokenEnum::AssignAnd, 3))
                    } else {
                        self.iter.advance_by(2);
                        Some(self.tokenize(TokenEnum::And, 2))
                    }
                } else if peeked == '=' {
                    self.iter.advance_by(2);
                    Some(self.tokenize(TokenEnum::AssignBinaryAnd, 2))
                } else {
                    self.iter.advance_by(1);
                    Some(self.tokenize(TokenEnum::BinaryAnd, 1))
                }
            }
            '|' => {
                let peeked = self.iter.peek();
                if peeked == '|' {
                    self.iter.advance_by(2);
                    Some(self.tokenize(TokenEnum::Or, 2))
                } else {
                    self.iter.advance_by(1);
                    Some(self.tokenize(TokenEnum::BinaryOr, 1))
                }
            }
            '+' => {
                let peeked = self.iter.peek();
                if peeked == '=' {
                    self.iter.advance_by(2);
                    Some(self.tokenize(TokenEnum::AssignPlus, 2))
                } else if peeked == '+' {
                    self.iter.advance_by(2);
                    Some(self.tokenize(TokenEnum::Increment, 2))
                } else {
                    self.iter.advance_by(1);
                    Some(self.tokenize(TokenEnum::Plus, 1))
                }
            }
            '-' => {
                let peeked = self.iter.peek();
                if peeked == '=' {
                    self.iter.advance_by(2);
                    Some(self.tokenize(TokenEnum::AssignMinus, 2))
                } else if peeked == '-' {
                    self.iter.advance_by(2);
                    Some(self.tokenize(TokenEnum::Decrement, 2))
                } else {
                    self.iter.advance_by(1);
                    Some(self.tokenize(TokenEnum::Minus, 1))
                }
            }
            '*' => {
                let peeked = self.iter.peek();
                if peeked == '*' {
                    if self.iter.peekn(2) == '=' {
                        self.iter.advance_by(3);
                        Some(self.tokenize(TokenEnum::AssignPower, 3))
                    } else {
                        self.iter.advance_by(2);
                        Some(self.tokenize(TokenEnum::Power, 2))
                    }
                } else if peeked == '=' {
                    self.iter.advance_by(2);
                    Some(self.tokenize(TokenEnum::AssignMultiply, 2))
                } else {
                    self.iter.advance_by(1);
                    Some(self.tokenize(TokenEnum::Multiply, 1))
                }
            }
            '/' => {
                let peeked = self.iter.peek();
                if peeked == '=' {
                    self.iter.advance_by(2);
                    Some(self.tokenize(TokenEnum::AssignDivide, 2))
                } else {
                    self.iter.advance_by(1);
                    Some(self.tokenize(TokenEnum::Divide, 1))
                }
            }
            '%' => {
                let peeked = self.iter.peek();
                if peeked == '=' {
                    self.iter.advance_by(2);
                    Some(self.tokenize(TokenEnum::AssignModolus, 2))
                } else {
                    self.iter.advance_by(1);
                    Some(self.tokenize(TokenEnum::Modolus, 1))
                }
            }
            '!' => {
                let peeked = self.iter.peek();
                if peeked == '=' {
                    self.iter.advance_by(2);
                    Some(self.tokenize(TokenEnum::AssignNot, 2))
                } else {
                    self.iter.advance_by(1);
                    Some(self.tokenize(TokenEnum::Not, 1))
                }
            }
            '^' => {
                let peeked = self.iter.peek();
                if peeked == '=' {
                    self.iter.advance_by(2);
                    Some(self.tokenize(TokenEnum::AssignBinaryXor, 2))
                } else {
                    self.iter.advance_by(1);
                    Some(self.tokenize(TokenEnum::BinaryXor, 1))
                }
            }
            '(' => {
                self.iter.advance_by(1);
                Some(self.tokenize(TokenEnum::OpenParenthese, 1))
            }
            ')' => {
                self.iter.advance_by(1);
                Some(self.tokenize(TokenEnum::CloseParenthese, 1))
            }
            '{' => {
                self.iter.advance_by(1);
                Some(self.tokenize(TokenEnum::OpenBracket, 1))
            }
            '}' => {
                self.iter.advance_by(1);
                Some(self.tokenize(TokenEnum::CloseBracket, 1))
            }
            '[' => {
                self.iter.advance_by(1);
                Some(self.tokenize(TokenEnum::OpenSquareBracket, 1))
            }
            ']' => {
                self.iter.advance_by(1);
                Some(self.tokenize(TokenEnum::CloseSquareBracket, 1))
            }
            _ => None,
        }
    }
    fn lookup_keyword(&mut self, buf: String) -> Token<'a> {
        let len = buf.len();
        match buf.as_str() {
            // declaration
            "let" => self.keyword(&LET, len),
            "int" => self.keyword(&INTEGER, len),
            "const" => self.keyword(&CONST, len),
            _ => self.tokenize(TokenEnum::IdentifierType, len),
        }
    }
    fn tokenize(&self, kind: TokenEnum<'a>, len: usize) -> Token<'a> {
        Token::new(kind, self.get_slice(len), self.line)
    }
    fn keyword(&self, keyword: &'a Keyword, len: usize) -> Token<'a> {
        self.tokenize(TokenEnum::KeywordType(keyword), len)
    }
    fn is_whitespace(&mut self) -> bool {
        let ch = self.iter.current();
        ch == ' ' || ch == '\t'
    }
    fn is_identifier(&mut self) -> bool {
        let ch = self.iter.current();
        ch.is_alphanumeric() || ch == '_'
    }
    fn is_numeric(&mut self) -> bool {
        let ch = self.iter.current();
        ch.is_ascii_digit() || ch == '.'
    }
    fn is_float_dot(&mut self) -> bool {
        self.iter.current() == '.' && self.iter.peek().is_ascii_digit()
    }
    fn get_slice(&self, n: usize) -> &'a str {
        let start = self.iter.current - n;
        let end = self.iter.current;
        &self.buffer[start..end]
    }
}
