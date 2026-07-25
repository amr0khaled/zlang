use crate::{
    lexer::{Token, Tokenizer, types::TokenType},
    parser::{
        node::Node,
        ops::{Operator, get_hint},
    },
};
mod node;
mod ops;

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
    buffer: &'a str,
    nodes: Vec<Node<'a>>,
}

impl<'a> Parser<'a> {
    pub fn new(buffer: &'a str) -> Self {
        Self {
            stream: TokenStream::new(buffer),
            buffer,
            nodes: vec![],
        }
    }
    pub fn expect(&mut self, expected: TokenType) -> Token<'a> {
        let got = self.next();
        if got.kind == expected {
            got
        } else {
            panic!("Syntax Error: Expected {:?}, but got {:?}", expected, got);
        }
    }
    pub fn expect_one_of(&mut self, expected: &[TokenType]) -> Option<Token<'a>> {
        let current = self.peek();

        if expected.iter().any(|&candidate| candidate == current.kind) {
            return Some(self.next());
        }
        None
    }
    fn parse_primary(&mut self) -> Node<'a> {
        println!("parse_primary");
        let token = self.next();
        let result = match token.kind {
            TokenType::IntegerLiteral => {
                let val = token.pointer.parse::<usize>().unwrap_or_else(|_| {
                    panic!("Line {}: Invalid integer '{}'", token.line, token.pointer)
                });
                Node::IntegerLit(val)
            }
            TokenType::FloatLiteral => {
                let val = token.pointer.parse::<f64>().unwrap_or_else(|_| {
                    panic!("Line {}: Invalid float '{}'", token.line, token.pointer)
                });
                Node::FloatLit(val)
            }
            TokenType::BoolLiteral => {
                let val = token.pointer.parse::<bool>().unwrap();
                Node::BooleanLit(val)
            }
            TokenType::Ident => Node::IdentExpr(token.pointer),
            TokenType::OpenParenthese => {
                let inner = self.parse_expr();
                self.expect(TokenType::CloseParenthese);
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
                    }
                } else {
                    let op = Operator::PostIncrement;
                    Node::UnaryExpr {
                        op,
                        node: Box::new(self.parse_primary()),
                        kind: get_hint(op),
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
                        kind: "PreDecUnaryExper",
                    }
                } else {
                    let op = Operator::PostDecrement;
                    Node::UnaryExpr {
                        op,
                        node: Box::new(self.parse_primary()),
                        kind: "PostDecUnaryExper",
                    }
                }
            }
            TokenType::OpenBracket => self.parse_block(),
            TokenType::SemiColon => Node::Empty,
            _ => panic!("Line {}: Unexpected token {:#?}", token.line, token.kind),
        };
        println!("end parse_primary");
        result
    }
    fn parse_expr(&mut self) -> Node<'a> {
        println!("parse_expr");
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

            let right = self.parse_primary();
            left = Node::BinaryExpr {
                op,
                left: Box::new(left),
                right: Box::new(right),
                kind: get_hint(op),
            };
        }
        println!("end parse_expr");
        left
    }
    fn parse_var_decl(&mut self, constant: bool) -> Node<'a> {
        println!("parse_var_decl");
        self.next();
        let (ident, value) = self.parse_assign_core();
        println!("end parse_var_decl");
        if constant {
            Node::ConstDeclStmt {
                ident,
                value,
                kind: "ConstDecl",
            }
        } else {
            Node::VarDeclStmt {
                ident,
                value,
                kind: "VarDecl",
            }
        }
    }
    fn parse_assign_core(&mut self) -> (&'a str, Box<Node<'a>>) {
        println!("parse_assign_core");
        match self.peek().kind {
            TokenType::Ident => {
                let ident = self.next();
                self.expect(TokenType::Assign);
                let value = Box::new(self.parse_expr());
                self.expect_one_of(&[TokenType::SemiColon, TokenType::EOL]);
                println!("end parse_assign_core");
                return (ident.pointer, value);
            }
            _ => {
                let token = self.peek();
                panic!("Line {}: Not expected '{}'", token.line, token.pointer);
            }
        }
    }
    fn parse_assign(&mut self) -> Node<'a> {
        println!("parse_assign");
        let (ident, value) = self.parse_assign_core();
        println!("end parse_assign");
        Node::AssignStmt {
            ident,
            value,
            kind: "Assign",
        }
    }
    fn parse_block(&mut self) -> Node<'a> {
        println!("parse_block");
        let mut stmts: Vec<Box<Node<'a>>> = vec![];
        while !matches!(self.peek().kind, TokenType::CloseBracket) {
            let stmt = self.parse_statement();
            if !matches!(stmt, Node::Empty) {
                stmts.push(Box::new(stmt));
            }
        }
        self.expect(TokenType::CloseBracket);

        self.expect_one_of(&[TokenType::SemiColon, TokenType::EOL]);
        println!("end parse_block");
        Node::Block {
            statements: stmts,
            kind: "LocalScope",
        }
    }
    fn parse_statement(&mut self) -> Node<'a> {
        println!("parse_statement");
        match self.peek().kind {
            TokenType::Let => self.parse_var_decl(false),
            TokenType::Const => self.parse_var_decl(true),
            TokenType::Ident => self.parse_assign(),
            TokenType::OpenBracket => {
                self.next(); // Consume '{'
                self.parse_block()
            }
            TokenType::SemiColon => {
                self.next();
                Node::Empty
            }
            _ => {
                let expr = self.parse_expr();
                self.expect_one_of(&[TokenType::SemiColon, TokenType::EOL]);

                expr
            }
        }
    }
    pub fn parse(&mut self) -> Node<'a> {
        println!("parse");
        let mut stmts: Vec<Box<Node<'a>>> = vec![];
        while self.peek().kind != TokenType::EOF {
            let stmt = self.parse_statement();
            if !matches!(stmt, Node::Empty) {
                stmts.push(Box::new(stmt));
            }
        }
        Node::Block {
            statements: stmts,
            kind: "ProgramNode",
        }
    }
    fn peek(&mut self) -> &Token<'a> {
        self.stream.peek()
    }
    fn next(&mut self) -> Token<'a> {
        let token = self.stream.next();
        println!("Token: {:#?}", token);
        token
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::{Parser, node::Node};

    #[test]
    fn test_let() {
        let line = "let x = 5;";
        let mut parser = Parser::new(line);

        assert_eq!(
            Node::Block {
                statements: Vec::from([Box::new(Node::VarDeclStmt {
                    ident: "x",
                    value: Box::new(Node::IntegerLit(5)),
                    kind: "VarDecl"
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

        assert_eq!(
            Node::Block {
                statements: Vec::from([Box::new(Node::VarDeclStmt {
                    ident: "x",
                    value: Box::new(Node::Block {
                        statements: Vec::from([Box::new(Node::IntegerLit(5))]),
                        kind: "LocalScope",
                    }),
                    kind: "VarDecl"
                })]),
                kind: "ProgramNode"
            },
            parser.parse()
        );
    }
}
