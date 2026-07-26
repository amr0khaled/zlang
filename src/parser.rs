use core::panic;

use crate::{
    lexer::{Token, Tokenizer, types::TokenType},
    parser::{
        node::Node,
        ops::{Operator, get_hint},
    },
};
pub mod node;
pub mod ops;

macro_rules! expect {
    ($self:ident, $patt:pat $(if $guard:expr)? $(,)?) => {{
        let got = $self.next();
        let kind = got.kind;
        match kind {
            $patt $(if $guard)? => got,
            d => panic!("Line {}: Syntax Error: Expected {:#?}, but got {:?}", got.line, stringify!($patt), d)
        }
    }};
    ($self:ident, $fn:path) => {
        let got = $self.next();
        match got.kind {
            x if $fn(x) => got,
            _ => panic!("Syntax Error: Expected {:#?}, but got {:?}", stringify!($patt), got)
        }
    };
}

pub struct TokenStream<'a> {
    tokenizer: Tokenizer<'a>,
    peeked: Option<Token<'a>>,
}
impl<'a> TokenStream<'a> {
    pub fn new(buf: &'a str) -> Self {
        Self {
            tokenizer: Tokenizer::new(buf),
            peeked: None,
        }
    }
    pub fn peek(&mut self) -> &Token<'a> {
        if self.peeked.is_none() {
            self.peeked = Some(self.tokenizer.next_token())
        }
        self.peeked.as_ref().unwrap()
    }
    pub fn next(&mut self) -> Token<'a> {
        match self.peeked.take() {
            Some(e) => e,
            None => self.tokenizer.next_token(),
        }
    }
}

pub struct Parser<'a> {
    stream: TokenStream<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(buffer: &'a str) -> Self {
        Self {
            stream: TokenStream::new(buffer),
        }
    }
    pub fn expect_one_of(&mut self, expected: &[TokenType]) -> Option<Token<'a>> {
        let current = self.peek();

        if expected.iter().any(|&candidate| candidate == current.kind) {
            return Some(self.next());
        }
        None
    }
    pub fn expect_fn_or<F: Fn(TokenType) -> bool>(&mut self, fun: F, ty: TokenType) -> Token<'a> {
        let got = self.next();
        if fun(got.kind) || ty == got.kind {
            got
        } else {
            panic!(
                "Syntax Error: Expected values of function, but got {:?}",
                got
            );
        }
    }
    fn parse_primary(&mut self) -> Node<'a> {
        let token = self.next();
        let info = Node::info(&token);
        let result = match token.kind {
            TokenType::IntegerLiteral => {
                let v = token.pointer.parse::<usize>().unwrap_or_else(|_| {
                    panic!("Line {}: Invalid integer '{}'", token.line, token.pointer)
                });
                Node::IntegerLit { v, info }
            }
            TokenType::FloatLiteral => {
                let v = token.pointer.parse::<f64>().unwrap_or_else(|_| {
                    panic!("Line {}: Invalid float '{}'", token.line, token.pointer)
                });
                Node::FloatLit { v, info }
            }
            TokenType::BoolLiteral => {
                let v = token.pointer.parse::<bool>().unwrap();
                Node::BooleanLit { v, info }
            }
            TokenType::Ident => Node::IdentExpr {
                name: token.pointer,
                info,
            },
            TokenType::OpenParenthese => {
                let inner = self.parse_expr();
                expect!(self, TokenType::CloseParenthese);
                inner
            }
            TokenType::Increment => {
                if self.peek().kind == TokenType::Ident {
                    self.next();
                    let op = Operator::PreIncrement;
                    Node::UnaryExpr {
                        op,
                        node: Box::new(self.parse_primary()),
                        kind: get_hint(op),
                        info,
                    }
                } else {
                    let op = Operator::PostIncrement;
                    Node::UnaryExpr {
                        op,
                        node: Box::new(self.parse_primary()),
                        kind: get_hint(op),
                        info,
                    }
                }
            }
            TokenType::Decrement => {
                if self.peek().kind == TokenType::Ident {
                    self.next();
                    let op = Operator::PreDecrement;
                    Node::UnaryExpr {
                        op,
                        node: Box::new(self.parse_primary()),
                        kind: get_hint(op),
                        info,
                    }
                } else {
                    let op = Operator::PostDecrement;
                    Node::UnaryExpr {
                        op,
                        node: Box::new(self.parse_primary()),
                        kind: get_hint(op),
                        info,
                    }
                }
            }
            TokenType::OpenBracket => self.parse_block(),
            TokenType::SemiColon => Node::Empty,
            _ => panic!("Line {}: Unexpected token {:#?}", token.line, token.kind),
        };
        result
    }
    fn parse_expr(&mut self) -> Node<'a> {
        println!("expr");
        println!("expr: left {:#?}", self.peek().kind);
        let mut left = self.parse_primary();
        loop {
            let op = match self.peek().kind {
                TokenType::Plus => Operator::Add,
                TokenType::Minus => Operator::Sub,
                TokenType::Multiply => Operator::Mul,
                TokenType::Divide => Operator::Div,
                TokenType::Power => Operator::Power,
                _ => break,
            };
            self.next();
            println!("expr: right");
            let right = self.parse_primary();
            left = Node::BinaryExpr {
                op,
                left: Box::new(left),
                right: Box::new(right),
                kind: get_hint(op),
                info: Node::info(&self.peek()),
            };
        }
        left
    }
    fn parse_return(&mut self) -> Node<'a> {
        let info = Node::info(&self.peek());
        expect!(self, TokenType::Return);
        let token = self.peek();
        let node = match token.kind {
            x if is_operator(x) => panic!(
                "Cannot have operator '{}' after return on line {}",
                token.pointer, token.line
            ),
            x if is_assign_operator(x) => panic!(
                "Cannot have assign operator '{}' after return on line {}",
                token.pointer, token.line
            ),
            x if is_literal(x) => {
                println!("return: before expr");
                self.parse_expr()
            }
            TokenType::SemiColon | TokenType::EOL => {
                self.next();
                Node::Empty
            }
            TokenType::OpenBracket => self.parse_block(),
            TokenType::Ident => self.parse_statement(),
            _ => panic!("What had happend?"),
        };
        Node::ReturnExpr {
            node: Box::new(node),
            info,
        }
    }
    fn parse_var_decl(&mut self, constant: bool) -> Node<'a> {
        println!("var ");
        let keyword = self.next();
        let ident = self.next();
        let (peek_pointer, peek_kind, peek_line) = {
            let peek = self.peek();
            (peek.pointer, peek.kind, peek.line)
        };
        let unintialized_decl = || {
            println!(
                "dynamic ident: '{}' {:?} line {}",
                peek_pointer, peek_kind, peek_line
            );
            let ty = TokenType::Uninitialized;
            let value = Box::new(Node::Dynamic);
            (ty, value)
        };
        let (_ty, value) = match peek_kind {
            x if is_assign_operator(x) => self.parse_assign_core(),
            TokenType::SemiColon => {
                expect!(self, TokenType::SemiColon);
                unintialized_decl()
            }
            TokenType::EOL | TokenType::EOF => unintialized_decl(),
            _ => {
                let peek = self.peek();
                panic!("Line {}:Unexpected token '{:?}'", peek.line, peek.kind)
            }
        };
        self.eol();
        let info = Node::info(&keyword);
        if constant {
            Node::ConstDeclStmt {
                ident: ident.pointer,
                value,
                kind: "ConstDecl",
                info,
            }
        } else {
            Node::VarDeclStmt {
                ident: ident.pointer,
                value,
                kind: "VarDecl",
                info,
            }
        }
    }
    fn parse_assign_core(&mut self) -> (TokenType, Box<Node<'a>>) {
        println!("var ");
        println!("core: before expr");
        let ty = expect!(self, is_assign_operator).kind;
        let value = Box::new(self.parse_expr());
        println!("V: {:?}", value);
        (ty, value)
    }
    fn parse_assign(&mut self, ident: &Token<'a>) -> Node<'a> {
        println!("assign core ");
        let info = Node::info(&self.peek());
        let (ty, value) = self.parse_assign_core();
        Node::AssignStmt {
            ident: ident.pointer,
            value,
            kind: ty,
            info,
        }
    }
    fn parse_block(&mut self) -> Node<'a> {
        let mut stmts: Vec<Box<Node<'a>>> = vec![];
        let info = Node::info(self.peek());
        while !matches!(self.peek().kind, TokenType::CloseBracket) {
            println!("block: stmt");
            let stmt = self.parse_statement();
            if !matches!(stmt, Node::Empty) {
                stmts.push(Box::new(stmt));
            }
        }
        expect!(self, TokenType::CloseBracket);

        self.expect_one_of(&[TokenType::SemiColon, TokenType::EOL]);
        Node::Block {
            statements: stmts,
            kind: "LocalScope",
            info,
        }
    }
    fn parse_statement(&mut self) -> Node<'a> {
        match self.peek().kind {
            TokenType::Let => self.parse_var_decl(false),
            TokenType::Const => self.parse_var_decl(true),
            TokenType::Return => self.parse_return(),
            TokenType::Ident => {
                let ident = self.next();
                let peek = self.peek();
                let info = Node::info(&ident);
                match peek.kind {
                    _ if is_assign_operator(peek.kind) => self.parse_assign(&ident),
                    TokenType::EOL | TokenType::EOF | TokenType::CloseBracket => Node::ReturnExpr {
                        node: Box::new(Node::IdentExpr {
                            name: ident.pointer,
                            info: info.clone(),
                        }),
                        info: info.clone(),
                    },
                    TokenType::SemiColon => Node::Empty,
                    _ => {
                        panic!("Not expected '{:?}' on line {}", peek.kind, peek.line)
                    }
                }
            }
            TokenType::OpenBracket => {
                self.next(); // Consume '{'
                self.parse_block()
            }
            TokenType::SemiColon => {
                self.next();
                Node::Empty
            }

            TokenType::EOL => {
                self.next();
                Node::Empty
            }
            _ => {
                println!("stmt: before expr {:#?}", self.peek());
                let expr = self.parse_expr();
                if self.peek().kind == TokenType::CloseBracket {
                    let info = Node::info(self.peek());
                    Node::ReturnExpr {
                        node: Box::new(expr),
                        info,
                    }
                } else {
                    self.expect_one_of(&[TokenType::SemiColon, TokenType::EOL]);
                    expr
                }
            }
        }
    }
    pub fn parse(&mut self) -> Node<'a> {
        let mut stmts: Vec<Box<Node<'a>>> = vec![];
        while self.peek().kind != TokenType::EOF {
            let stmt = self.parse_statement();
            if !matches!(stmt, Node::Empty) {
                stmts.push(Box::new(stmt));
            }
        }
        Node::Program {
            statements: stmts,
            kind: "ProgramNode",
        }
    }
    fn eol(&mut self) {
        if self.peek().kind == TokenType::SemiColon {
            expect!(self, TokenType::SemiColon);
        }
        expect!(self, TokenType::EOL | TokenType::EOF);
    }
    fn peek(&mut self) -> &Token<'a> {
        self.stream.peek()
    }
    fn next(&mut self) -> Token<'a> {
        let token = self.stream.next();
        token
    }
}

fn is_assign_operator(ty: TokenType) -> bool {
    match ty {
        TokenType::AssignNew => true,
        TokenType::AssignPlus => true,
        TokenType::AssignMinus => true,
        TokenType::AssignMultiply => true,
        TokenType::AssignPower => true,
        TokenType::AssignAnd => true,
        TokenType::AssignOr => true,
        TokenType::AssignNot => true,
        TokenType::AssignDivide => true,
        TokenType::AssignModolus => true,
        TokenType::AssignBinaryAnd => true,
        TokenType::AssignBinaryOr => true,
        TokenType::AssignBinaryXor => true,
        TokenType::Assign => true,
        _ => false,
    }
}
fn is_operator(ty: TokenType) -> bool {
    match ty {
        TokenType::Plus => true,
        TokenType::Minus => true,
        TokenType::Multiply => true,
        TokenType::Power => true,
        TokenType::Divide => true,
        TokenType::Modolus => true,
        TokenType::And => true,
        TokenType::Or => true,
        TokenType::Not => true,
        TokenType::BinaryAnd => true,
        TokenType::BinaryOr => true,
        TokenType::BinaryXor => true,
        TokenType::Equality => true,
        TokenType::GreaterThan => true,
        TokenType::LesserThan => true,
        TokenType::GreaterThanOrEqual => true,
        TokenType::LesserThanOrEqual => true,
        _ => false,
    }
}

fn is_literal(ty: TokenType) -> bool {
    match ty {
        TokenType::IntegerLiteral => true,
        TokenType::FloatLiteral => true,
        TokenType::BoolLiteral => true,
        TokenType::OpenParenthese => true,
        TokenType::OpenSquareBracket => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::{
        Parser,
        node::{Node, NodeDetails},
    };

    #[test]
    fn test_let() {
        let line = "let x = 5;";
        let mut parser = Parser::new(line);

        let integer_info = NodeDetails::new(0, 8);
        let var_info = NodeDetails::new(0, 0);
        assert_eq!(
            Node::Program {
                statements: Vec::from([Box::new(Node::VarDeclStmt {
                    ident: "x",
                    value: Box::new(Node::IntegerLit {
                        v: 5,
                        info: integer_info
                    }),
                    kind: "VarDecl",
                    info: var_info
                })]),
                kind: "ProgramNode"
            },
            parser.parse()
        );
    }

    #[test]
    fn test_let_block() {
        let line = "let x = {5};";
        let mut parser = Parser::new(line);
        let var_info = NodeDetails::new(0, 0);
        let block_info = NodeDetails::new(0, 9);
        let integer_info = NodeDetails::new(0, 9);
        let return_info = NodeDetails::new(0, 10);
        assert_eq!(
            Node::Program {
                statements: Vec::from([Box::new(Node::VarDeclStmt {
                    ident: "x",
                    value: Box::new(Node::Block {
                        statements: Vec::from([Box::new(Node::ReturnExpr {
                            node: Box::new(Node::IntegerLit {
                                v: 5,
                                info: integer_info
                            }),
                            info: return_info
                        })]),
                        kind: "LocalScope",
                        info: block_info
                    }),
                    kind: "VarDecl",
                    info: var_info
                })]),
                kind: "ProgramNode"
            },
            parser.parse()
        );
    }
    #[test]

    fn test_let_block_with_return() {
        let line = "let x = {return 5;};";
        let mut parser = Parser::new(line);
        let var_info = NodeDetails::new(0, 0);
        let block_info = NodeDetails::new(0, 9);
        let integer_info = NodeDetails::new(0, 16);
        let return_info = NodeDetails::new(0, 9);
        assert_eq!(
            Node::Program {
                statements: Vec::from([Box::new(Node::VarDeclStmt {
                    ident: "x",
                    value: Box::new(Node::Block {
                        statements: Vec::from([Box::new(Node::ReturnExpr {
                            node: Box::new(Node::IntegerLit {
                                v: 5,
                                info: integer_info
                            }),
                            info: return_info
                        })]),
                        kind: "LocalScope",
                        info: block_info
                    }),
                    kind: "VarDecl",
                    info: var_info
                })]),
                kind: "ProgramNode"
            },
            parser.parse()
        );
    }
}
