use core::panic;

use crate::{
    error,
    lexer::{Token, Tokenizer, types::TokenType},
    parser::{
        node::{DataType, Node},
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
            d => panic!("Line {}: Syntax Error: Expected {:#?}, but got {:#?}", got.line, stringify!($patt), d)
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
    fn parse_arg(&mut self) -> Node<'a> {
        let node = self.parse_primary();
        if self.peek().kind == TokenType::Comma {
            expect!(self, TokenType::Comma);
        }
        node
    }
    // Determine parameters or args between [] or () based on Closing condition token type
    fn parse_params(&mut self, condition_token: TokenType) -> Vec<Box<Node<'a>>> {
        let mut params: Vec<Box<Node<'a>>> = vec![];
        if self.peek().kind != condition_token {
            loop {
                if self.peek().kind == condition_token {
                    break;
                }
                let node = self.parse_arg();
                params.push(Box::new(node));
            }
        }
        expect!(self, condition_token);
        params
    }
    fn parse_primary(&mut self) -> Node<'a> {
        let token = self.next();
        let info = Node::info(&token);
        let result = match token.kind {
            TokenType::I8 | TokenType::I16 | TokenType::I32 | TokenType::I64 => {
                let ty = DataType::from_token(token.kind);
                Node::DataType(ty)
            }
            TokenType::U8 | TokenType::U16 | TokenType::U32 | TokenType::U64 => {
                let ty = DataType::from_token(token.kind);
                Node::DataType(ty)
                //let (v, size) = match ty.get_size() {
                //    8 => (token.pointer.parse::<u8>().unwrap() as u64, 1),
                //    16 => (token.pointer.parse::<u16>().unwrap() as u64, 2),
                //    32 => (token.pointer.parse::<u32>().unwrap() as u64, 4),
                //    64 => (token.pointer.parse::<u64>().unwrap(), 8),
                //    _ => panic!("Line {}: Invalid integer '{}'", token.line, token.pointer),
                //};
            }
            TokenType::IntLiteral => {
                let v = token.pointer.parse::<i64>().unwrap_or_else(|_| {
                    panic!("Line {}: Invalid integer '{}'", token.line, token.pointer)
                });
                Node::IntLit { v, size: 8, info }
            }
            TokenType::FloatLiteral => {
                let v = token.pointer.parse::<f64>().unwrap_or_else(|_| {
                    panic!("Line {}: Invalid integer '{}'", token.line, token.pointer)
                });
                Node::FloatLit { v, size: 8, info }
            }
            TokenType::F32 | TokenType::F64 => {
                let ty = DataType::from_token(token.kind);
                let (v, size) = match ty.get_size() {
                    32 => (token.pointer.parse::<f32>().unwrap() as f64, 4),
                    64 => (token.pointer.parse::<f64>().unwrap(), 8),
                    _ => panic!("Line {}: Invalid Float '{}'", token.line, token.pointer),
                };
                Node::FloatLit { v, size, info }
            }
            TokenType::BoolLiteral => {
                let v = token.pointer.parse::<bool>().unwrap();
                Node::BooleanLit { v, info }
            }
            TokenType::Function => {
                let info = Node::info(&token);
                let decl = Box::new(self.parse_decl(false));
                let mut args: Vec<Box<Node<'a>>> = vec![];
                let mut params: Vec<Box<Node<'a>>> = vec![];
                if self.peek().kind == TokenType::OpenSquareBracket {
                    expect!(self, TokenType::OpenSquareBracket);
                    params = self.parse_params(TokenType::CloseSquareBracket);
                }
                expect!(self, TokenType::OpenParenthese);
                args = self.parse_params(TokenType::CloseParenthese);
                let mut block = Box::new(self.parse_primary());
                if let Node::Block { is_body, .. } = &mut (*block) {
                    *is_body = true;
                }
                Node::FunctionDecl {
                    decl,
                    params,
                    args,
                    block,
                    kind: "FunDecl",
                    info,
                }
            }
            TokenType::Ident => {
                let info = Node::info(&token);
                let ident = Box::new(Node::IdentExpr {
                    name: token.pointer,
                    r: None,
                    ty: None,
                    info,
                });
                if matches!(
                    self.peek().kind,
                    TokenType::OpenParenthese | TokenType::OpenSquareBracket
                ) {
                    let mut params = vec![];
                    let mut args = vec![];
                    if self.peek().kind == TokenType::OpenSquareBracket {
                        expect!(self, TokenType::OpenSquareBracket);
                        params = self.parse_params(TokenType::CloseSquareBracket);
                    }
                    expect!(self, TokenType::OpenParenthese);
                    args = self.parse_params(TokenType::CloseParenthese);
                    Node::CallExpr {
                        ident,
                        params,
                        args,
                        kind: "CallExpr",
                        info,
                        r: None,
                    }
                } else {
                    *ident
                }
            }
            TokenType::OpenParenthese => {
                let inner = self.parse_expr();
                expect!(self, TokenType::CloseParenthese);
                inner
            }
            TokenType::Not => Node::UnaryExpr {
                op: Operator::Not,
                node: Box::new(self.parse_expr()),
                kind: "Inverse Logic",
                ty: None,
                info,
            },
            TokenType::Increment => {
                let op;
                let node;
                if self.peek().kind == TokenType::Ident {
                    let ident = self.next();
                    node = Node::IdentExpr {
                        name: ident.pointer,
                        r: None,
                        ty: None,
                        info,
                    };
                    op = Operator::PreIncrement;
                } else {
                    op = Operator::PostIncrement;
                    node = Node::Empty;
                }
                Node::UnaryExpr {
                    op,
                    node: Box::new(node),
                    kind: get_hint(op),
                    ty: None,
                    info,
                }
            }
            TokenType::Decrement => {
                let op;
                let node;
                if self.peek().kind == TokenType::Ident {
                    let ident = self.next();
                    node = Node::IdentExpr {
                        name: ident.pointer,
                        r: None,
                        ty: None,
                        info,
                    };
                    op = Operator::PreDecrement;
                } else {
                    node = Node::Empty;
                    op = Operator::PostDecrement;
                }
                Node::UnaryExpr {
                    op,
                    node: Box::new(node),
                    kind: get_hint(op),
                    ty: None,
                    info,
                }
            }
            TokenType::OpenBracket => self.parse_block(),
            TokenType::SemiColon => Node::Empty,
            _ => panic!(
                "Line {}:{}: Unexpected token {:#?}",
                token.line, token.col, token.kind
            ),
        };
        result
    }
    fn parse_expr(&mut self) -> Node<'a> {
        #[cfg(test)]
        {
            println!("expr");
            println!("expr: left {:#?}", self.peek().kind);
        }
        println!("SSS");
        let left = self.parse_primary();
        self.parse_expr_remain(left)
    }
    fn parse_expr_remain(&mut self, mut left: Node<'a>) -> Node<'a> {
        #[cfg(test)]
        {
            println!("expr remain");
            println!("expr: right {:#?}", self.peek().kind);
        }
        match self.peek().kind {
            // WARNING: I don't know what is this but be careful
            // TokenType::Ident => loop {},
            _ => loop {
                let op = Operator::from_token(self.peek().kind);
                if op == None {
                    break;
                    // panic!("Unexpected token: {:#?}", token)
                }
                self.next();
                println!("expr: right");
                let right = self.parse_primary();
                left = Node::BinaryExpr {
                    op: op.unwrap(),
                    left: Box::new(left),
                    right: Box::new(right),
                    kind: get_hint(op.unwrap()),
                    ty: None,
                    info: Node::info(&self.peek()),
                };
            },
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
    fn parse_let(&mut self, constant: bool) -> Node<'a> {
        println!("var ");
        let keyword = self.next();
        let decl = Box::new(self.parse_decl(false));
        let (peek_pointer, peek_kind, peek_line) = {
            let peek = self.peek();
            (peek.pointer, peek.kind, peek.line)
        };
        let unintialized_decl = || {
            println!(
                "dynamic ident: '{}' {:?} line {}",
                peek_pointer, peek_kind, peek_line
            );
            let value = Box::new(Node::Dynamic);
            (Operator::AssignNew, value)
        };
        let (_ty, value) = match peek_kind {
            x if is_assign_operator(x) => self.parse_assign_core(),
            TokenType::SemiColon => {
                expect!(self, TokenType::SemiColon);
                unintialized_decl()
            }
            TokenType::EOL | TokenType::EOF | TokenType::Comma => unintialized_decl(),
            _ => {
                let peek = self.peek();
                panic!("Line {}:Unexpected token '{:?}'", peek.line, peek.kind)
            }
        };
        self.eol();
        let info = Node::info(&keyword);
        if constant {
            Node::ConstDecl {
                decl,
                value,
                kind: "ConstDecl",
                info,
                r: None,
            }
        } else {
            Node::LetDecl {
                decl,
                value,
                kind: "LetDecl",
                info,
                r: None,
            }
        }
    }

    fn parse_type(&mut self) -> DataType {
        let mut ty: DataType = DataType::Dynamic;
        if self.peek().kind == TokenType::Colon {
            expect!(self, TokenType::Colon);
            ty = match self.peek() {
                t if is_datatype(t.kind) => DataType::from_token(self.next().kind),
                t => panic!(
                    "{}:{}: Syntax Error: Expected DataType, but got {:?}",
                    t.line, t.col, t.kind
                ),
            };
        }
        ty
    }
    fn parse_decl(&mut self, is_explicit: bool) -> Node<'a> {
        let info = Node::info(&self.peek());
        let ident = expect!(self, TokenType::Ident);
        let ty = self.parse_type();
        if ty == DataType::Dynamic && is_explicit {
            error!(info, "Syntax Error: Expected DataType definition");
        }
        if matches!(
            self.peek().kind,
            TokenType::Comma | TokenType::CloseParenthese | TokenType::CloseSquareBracket
        ) {
            Node::IdentExpr {
                name: ident.pointer,
                r: None,
                ty: None,
                info,
            }
        } else {
            Node::Decl {
                ident: ident.pointer,
                ty: Some(ty),
                kind: "Decl",
                info,
            }
        }
    }
    fn parse_assign_core(&mut self) -> (Operator, Box<Node<'a>>) {
        println!("var ");
        println!("core: before expr");
        let op = match expect!(self, is_assign_operator).kind {
            TokenType::Assign => Operator::Assign,
            TokenType::AssignNew => Operator::AssignNew,
            TokenType::AssignPlus => Operator::AssignPlus,
            TokenType::AssignMinus => Operator::AssignMinus,
            TokenType::AssignMultiply => Operator::AssignMultiply,
            TokenType::AssignPower => Operator::AssignPower,
            TokenType::AssignAnd => Operator::AssignAnd,
            TokenType::AssignOr => Operator::AssignOr,
            TokenType::AssignDivide => Operator::AssignDivide,
            TokenType::AssignModolus => Operator::AssignModolus,
            TokenType::AssignBinaryAnd => Operator::AssignBinaryAnd,
            TokenType::AssignBinaryOr => Operator::AssignBinaryOr,
            TokenType::AssignBinaryXor => Operator::AssignBinaryXor,
            _ => panic!("Assign operator unknown"),
        };

        let value = Box::new(self.parse_expr());
        println!("V: {:?}", value);
        (op, value)
    }
    fn parse_assign(&mut self, ident: &Token<'a>) -> Node<'a> {
        println!("assign core ");
        let info = Node::info(&self.peek());
        let (op, value) = self.parse_assign_core();
        Node::AssignStmt {
            op,
            ident: ident.pointer,
            value,
            info,
            r: None,
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
            is_body: false,
            statements: stmts,
            kind: "LocalScope",
            info,
            depth: None,
        }
    }
    fn parse_call(&mut self, ident_t: &Token<'a>) -> Node<'a> {
        let info = Node::info(&self.peek());
        let ident = Box::new(Node::IdentExpr {
            name: ident_t.pointer,
            r: None,
            ty: None,
            info,
        });
        let mut params = vec![];
        let mut args = vec![];
        if self.peek().kind == TokenType::OpenSquareBracket {
            expect!(self, TokenType::OpenSquareBracket);
            params = self.parse_params(TokenType::CloseSquareBracket);
        }
        expect!(self, TokenType::OpenParenthese);
        args = self.parse_params(TokenType::CloseParenthese);
        Node::CallExpr {
            ident,
            params,
            args,
            kind: "CallExpr",
            info,
            r: None,
        }
    }
    fn parse_statement(&mut self) -> Node<'a> {
        match self.peek().kind {
            TokenType::Let => self.parse_let(false),
            TokenType::Const => self.parse_let(true),
            TokenType::Return => self.parse_return(),
            TokenType::Ident => {
                let ident = self.next();
                let peek = self.peek();
                let info = Node::info(&ident);
                match peek.kind {
                    x if is_assign_operator(x) => self.parse_assign(&ident),
                    x if is_operator(x) => {
                        let left = Node::IdentExpr {
                            name: ident.pointer,
                            r: None,
                            ty: None,
                            info,
                        };
                        self.parse_expr_remain(left)
                    }
                    x if is_unary_operator(x) => {
                        let mut _node = self.parse_primary();
                        if let Node::UnaryExpr { node, .. } = &mut _node {
                            *node = Box::new(Node::IdentExpr {
                                name: ident.pointer,
                                r: None,
                                ty: None,
                                info,
                            })
                        }
                        _node
                    }
                    TokenType::OpenSquareBracket | TokenType::OpenParenthese => {
                        self.parse_call(&ident)
                    }
                    TokenType::EOL | TokenType::EOF | TokenType::CloseBracket => Node::ReturnExpr {
                        node: Box::new(Node::IdentExpr {
                            name: ident.pointer,
                            r: None,
                            ty: None,
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
        TokenType::NotEquality => true,
        TokenType::GreaterThan => true,
        TokenType::LesserThan => true,
        TokenType::GreaterThanOrEqual => true,
        TokenType::LesserThanOrEqual => true,
        _ => false,
    }
}
fn is_unary_operator(ty: TokenType) -> bool {
    match ty {
        TokenType::Not => true,
        TokenType::Increment => true,
        TokenType::Decrement => true,
        _ => false,
    }
}

fn is_literal(ty: TokenType) -> bool {
    match ty {
        TokenType::IntLiteral => true,
        TokenType::FloatLiteral => true,
        TokenType::BoolLiteral => true,
        TokenType::OpenSquareBracket => true,
        _ => false,
    }
}

fn is_datatype(ty: TokenType) -> bool {
    match ty {
        TokenType::U8 => true,
        TokenType::U16 => true,
        TokenType::U32 => true,
        TokenType::U64 => true,
        TokenType::I8 => true,
        TokenType::I16 => true,
        TokenType::I32 => true,
        TokenType::I64 => true,
        TokenType::F32 => true,
        TokenType::F64 => true,
        TokenType::Bool => true,
        _ => false,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::node::NodeDetails;

    mod declaration_tests {
        use super::*;
        const PRIMITIVE_TYPES: &[(&str, DataType)] = &[
            ("u8", DataType::Unsigned8),
            ("u16", DataType::Unsigned16),
            ("u32", DataType::Unsigned32),
            ("u64", DataType::Unsigned64),
            ("i8", DataType::Signed8),
            ("i16", DataType::Signed16),
            ("i32", DataType::Signed32),
            ("i64", DataType::Signed64),
            ("f32", DataType::Float32),
            ("f64", DataType::Float64),
            ("bool", DataType::Boolean),
        ];
        #[test]
        fn test_let() {
            let line = "let x = 5;";
            let mut parser = Parser::new(line);

            let var_info = NodeDetails::new(1, 1);
            let decl_info = NodeDetails::new(1, 5);
            let integer_info = NodeDetails::new(1, 9);
            assert_eq!(
                Node::Program {
                    statements: Vec::from([Box::new(Node::LetDecl {
                        r: None,
                        decl: Box::new(Node::Decl {
                            ident: "x",
                            ty: Some(DataType::Dynamic),
                            kind: "Decl",
                            info: decl_info
                        }),
                        value: Box::new(Node::IntLit {
                            v: 5,
                            size: 8,
                            info: integer_info
                        }),
                        kind: "LetDecl",
                        info: var_info
                    })]),
                    kind: "ProgramNode"
                },
                parser.parse()
            );
        }
        #[test]
        fn test_let_with_type() {
            for &(sym, ref ty) in PRIMITIVE_TYPES {
                let line = format!("let x: {} = 5;", sym);
                let mut parser = Parser::new(&line);

                let var_info = NodeDetails::new(1, 1);
                let decl_info = NodeDetails::new(1, 5);
                let integer_info = NodeDetails::new(1, 9);
                assert_eq!(
                    Node::Program {
                        statements: Vec::from([Box::new(Node::LetDecl {
                            r: None,
                            decl: Box::new(Node::Decl {
                                ident: "x",
                                ty: Some(ty.clone()),
                                kind: "Decl",
                                info: decl_info
                            }),
                            value: Box::new(Node::IntLit {
                                v: 5,
                                size: 8,
                                info: integer_info
                            }),
                            kind: "LetDecl",
                            info: var_info
                        })]),
                        kind: "ProgramNode"
                    },
                    parser.parse()
                );
            }
        }

        #[test]
        fn test_let_block() {
            let line = "let x = {5};";
            let mut parser = Parser::new(line);
            let decl_info = NodeDetails::new(1, 5);
            let var_info = NodeDetails::new(1, 1);
            let block_info = NodeDetails::new(1, 10);
            let integer_info = NodeDetails::new(1, 10);
            let return_info = NodeDetails::new(1, 11);
            assert_eq!(
                Node::Program {
                    statements: Vec::from([Box::new(Node::LetDecl {
                        r: None,
                        decl: Box::new(Node::Decl {
                            ident: "x",
                            ty: Some(DataType::Dynamic),
                            kind: "Decl",
                            info: decl_info
                        }),
                        value: Box::new(Node::Block {
                            is_body: false,
                            depth: None,
                            statements: Vec::from([Box::new(Node::ReturnExpr {
                                node: Box::new(Node::IntLit {
                                    v: 5,
                                    size: 8,
                                    info: integer_info
                                }),
                                info: return_info
                            })]),
                            kind: "LocalScope",
                            info: block_info
                        }),
                        kind: "LetDecl",
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
            let decl_info = NodeDetails::new(1, 5);
            let var_info = NodeDetails::new(1, 1);
            let block_info = NodeDetails::new(1, 10);
            let integer_info = NodeDetails::new(1, 17);
            let return_info = NodeDetails::new(1, 10);
            assert_eq!(
                Node::Program {
                    statements: Vec::from([Box::new(Node::LetDecl {
                        r: None,
                        decl: Box::new(Node::Decl {
                            ident: "x",
                            ty: Some(DataType::Dynamic),
                            kind: "Decl",
                            info: decl_info
                        }),
                        value: Box::new(Node::Block {
                            is_body: false,
                            depth: None,
                            statements: Vec::from([Box::new(Node::ReturnExpr {
                                node: Box::new(Node::IntLit {
                                    v: 5,
                                    size: 8,
                                    info: integer_info
                                }),
                                info: return_info
                            })]),
                            kind: "LocalScope",
                            info: block_info
                        }),
                        kind: "LetDecl",
                        info: var_info
                    })]),
                    kind: "ProgramNode"
                },
                parser.parse()
            );
        }
    }
    mod operations {
        use std::fmt::format;

        use super::*;
        const BINARY_OPS: &[(&str, Operator)] = &[
            ("+", Operator::Add),
            ("-", Operator::Sub),
            ("*", Operator::Mul),
            ("**", Operator::Power),
            ("/", Operator::Div),
            ("&&", Operator::And),
            ("||", Operator::Or),
            ("&", Operator::BinaryAnd),
            ("^", Operator::BinaryXor),
            ("%", Operator::Mod),
            ("==", Operator::Equality),
            ("!=", Operator::NotEquality),
            ("<", Operator::LesserThan),
            ("<=", Operator::LesserThanOrEqual),
            (">", Operator::GreaterThan),
            (">=", Operator::GreaterThanOrEqual),
        ];
        const UNARY_OPS: &[(&str, Operator)] = &[
            ("!", Operator::Not),
            ("++", Operator::PreIncrement),
            ("++", Operator::PostIncrement),
            ("--", Operator::PreDecrement),
            ("--", Operator::PostDecrement),
        ];
        const ASSIGN_OPS: &[(&str, Operator)] = &[
            ("<-", Operator::AssignNew),
            ("+=", Operator::AssignPlus),
            ("-=", Operator::AssignMinus),
            ("*=", Operator::AssignMultiply),
            ("**=", Operator::AssignPower),
            ("/=", Operator::AssignDivide),
            ("%=", Operator::AssignModolus),
            ("&&=", Operator::AssignAnd),
            ("||=", Operator::AssignOr),
            ("&=", Operator::AssignBinaryAnd),
            ("|=", Operator::AssignBinaryOr),
            ("^=", Operator::AssignBinaryXor),
        ];
        #[test]
        fn test_binary_expressions_without_ident() {
            for &(sym, ref op) in BINARY_OPS {
                println!("TEST ({})", sym);
                let line = format!("5 {} 4", sym);
                let mut parser = Parser::new(&line);

                let info = NodeDetails::new(0, 0);
                assert_eq!(
                    Node::Program {
                        statements: Vec::from([Box::new(Node::BinaryExpr {
                            op: op.clone(),
                            left: Box::new(Node::IntLit {
                                v: 5,
                                size: 8,
                                info
                            }),
                            right: Box::new(Node::IntLit {
                                v: 4,
                                size: 8,
                                info
                            }),
                            ty: None,
                            kind: "",
                            info
                        })]),
                        kind: ""
                    },
                    parser.parse()
                );
            }
        }
        #[test]
        fn test_binary_expressions() {
            for &(sym, ref op) in BINARY_OPS {
                let line = format!(
                    "let a = 4;
let b = 10;
a {} b;",
                    sym
                );
                let mut parser = Parser::new(&line);

                let info = NodeDetails::new(0, 0);
                assert_eq!(
                    Node::Program {
                        statements: Vec::from([
                            Box::new(Node::LetDecl {
                                r: None,
                                decl: Box::new(Node::Decl {
                                    ident: "a",
                                    ty: Some(DataType::Dynamic),
                                    kind: "",
                                    info
                                }),
                                value: Box::new(Node::IntLit {
                                    v: 4,
                                    size: 8,
                                    info
                                }),
                                kind: "",
                                info
                            }),
                            Box::new(Node::LetDecl {
                                r: None,
                                decl: Box::new(Node::Decl {
                                    ident: "b",
                                    ty: Some(DataType::Dynamic),
                                    kind: "",
                                    info
                                }),
                                value: Box::new(Node::IntLit {
                                    v: 10,
                                    size: 8,
                                    info
                                }),
                                kind: "",
                                info
                            }),
                            Box::new(Node::BinaryExpr {
                                op: op.clone(),
                                left: Box::new(Node::IdentExpr {
                                    name: "a",
                                    r: None,
                                    ty: None,
                                    info
                                }),
                                right: Box::new(Node::IdentExpr {
                                    name: "b",
                                    r: None,
                                    ty: None,
                                    info
                                }),
                                ty: None,
                                kind: "",
                                info
                            })
                        ]),
                        kind: ""
                    },
                    parser.parse()
                );
            }
        }
        #[test]
        fn test_unary_expresssions() {
            for &(sym, ref op) in UNARY_OPS {
                let pre = {
                    if op == &Operator::PostIncrement || op == &Operator::PostDecrement {
                        false
                    } else {
                        true
                    }
                };
                let line = format!(
                    "let a = 4;
{}",
                    if pre {
                        format!("{}a", sym)
                    } else {
                        format!("a{}", sym)
                    }
                );
                let mut parser = Parser::new(&line);

                let info = NodeDetails::new(0, 0);
                assert_eq!(
                    Node::Program {
                        statements: Vec::from([
                            Box::new(Node::LetDecl {
                                r: None,
                                decl: Box::new(Node::Decl {
                                    ident: "a",
                                    ty: Some(DataType::Dynamic),
                                    kind: "",
                                    info
                                }),
                                value: Box::new(Node::IntLit {
                                    v: 4,
                                    size: 8,
                                    info
                                }),
                                kind: "",
                                info
                            }),
                            Box::new(Node::UnaryExpr {
                                op: op.clone(),
                                node: Box::new(Node::IdentExpr {
                                    name: "a",
                                    r: None,
                                    ty: None,
                                    info
                                }),
                                ty: None,
                                kind: "",
                                info
                            })
                        ]),
                        kind: ""
                    },
                    parser.parse()
                );
            }
        }
        #[test]
        fn test_assign_expressions() {
            for &(sym, ref op) in ASSIGN_OPS {
                let line = format!(
                    "let a = 4;
a {} 4;",
                    sym
                );
                let mut parser = Parser::new(&line);

                let info = NodeDetails::new(0, 0);
                assert_eq!(
                    Node::Program {
                        statements: Vec::from([
                            Box::new(Node::LetDecl {
                                r: None,
                                decl: Box::new(Node::Decl {
                                    ident: "a",
                                    ty: Some(DataType::Dynamic),
                                    kind: "",
                                    info
                                }),
                                value: Box::new(Node::IntLit {
                                    v: 4,
                                    size: 8,
                                    info
                                }),
                                kind: "",
                                info
                            }),
                            Box::new(Node::AssignStmt {
                                ident: "a",
                                op: op.clone(),
                                value: Box::new(Node::IntLit {
                                    v: 4,
                                    size: 8,
                                    info
                                }),
                                info,
                                r: None
                            })
                        ]),
                        kind: ""
                    },
                    parser.parse()
                );
            }
        }
    }
    mod functions {
        use super::*;

        #[test]
        fn test_function() {
            let line = "fn name[](){let x = {return 5;};}";
            let mut parser = Parser::new(line);
            let decl_info = NodeDetails::new(1, 17);
            let var_info = NodeDetails::new(1, 13);
            let block_info = NodeDetails::new(1, 22);
            let integer_info = NodeDetails::new(1, 29);
            let return_info = NodeDetails::new(1, 22);
            let info = NodeDetails::new(0, 0);
            assert_eq!(
                Node::Program {
                    statements: Vec::from([Box::new(Node::FunctionDecl {
                        decl: Box::new(Node::Decl {
                            ident: "name",
                            ty: Some(DataType::Dynamic),
                            kind: "",
                            info
                        }),
                        params: vec![],
                        args: vec![],
                        block: Box::new(Node::Block {
                            statements: vec![Box::new(Node::LetDecl {
                                r: None,
                                decl: Box::new(Node::Decl {
                                    ident: "x",
                                    ty: Some(DataType::Dynamic),
                                    kind: "Decl",
                                    info: decl_info
                                }),
                                value: Box::new(Node::Block {
                                    is_body: false,
                                    depth: None,
                                    statements: Vec::from([Box::new(Node::ReturnExpr {
                                        node: Box::new(Node::IntLit {
                                            v: 5,
                                            size: 8,
                                            info: integer_info
                                        }),
                                        info: return_info
                                    })]),
                                    kind: "LocalScope",
                                    info: block_info
                                }),
                                kind: "LetDecl",
                                info: var_info
                            })],
                            kind: "",
                            info,
                            depth: Some(1),
                            is_body: true
                        }),
                        kind: "",
                        info
                    }),]),
                    kind: "ProgramNode"
                },
                parser.parse()
            );
        }
        #[test]
        fn test_function2() {
            let line = "fn name(){let x = {return 5;};}";
            let mut parser = Parser::new(line);
            let info = NodeDetails::new(0, 0);
            assert_eq!(
                Node::Program {
                    statements: Vec::from([Box::new(Node::FunctionDecl {
                        decl: Box::new(Node::Decl {
                            ident: "name",
                            ty: Some(DataType::Dynamic),
                            kind: "",
                            info
                        }),
                        params: vec![],
                        args: vec![],
                        block: Box::new(Node::Block {
                            statements: vec![Box::new(Node::LetDecl {
                                r: None,
                                decl: Box::new(Node::Decl {
                                    ident: "x",
                                    ty: Some(DataType::Dynamic),
                                    kind: "Decl",
                                    info
                                }),
                                value: Box::new(Node::Block {
                                    is_body: false,
                                    depth: None,
                                    statements: Vec::from([Box::new(Node::ReturnExpr {
                                        node: Box::new(Node::IntLit {
                                            v: 5,
                                            size: 8,
                                            info
                                        }),
                                        info
                                    })]),
                                    kind: "LocalScope",
                                    info
                                }),
                                kind: "LetDecl",
                                info
                            })],
                            kind: "",
                            info,
                            depth: Some(1),
                            is_body: true
                        }),
                        kind: "",
                        info
                    }),]),
                    kind: "ProgramNode"
                },
                parser.parse()
            );
        }
        #[test]
        #[should_panic]
        fn test_function3() {
            let line = "fn name{let x = {return 5;};}";
            let mut parser = Parser::new(line);
            let info = NodeDetails::new(0, 0);
            assert_eq!(
                Node::Program {
                    statements: Vec::from([Box::new(Node::FunctionDecl {
                        decl: Box::new(Node::Decl {
                            ident: "name",
                            ty: Some(DataType::Dynamic),
                            kind: "",
                            info
                        }),
                        params: vec![],
                        args: vec![],
                        block: Box::new(Node::Block {
                            statements: vec![Box::new(Node::LetDecl {
                                r: None,
                                decl: Box::new(Node::Decl {
                                    ident: "x",
                                    ty: Some(DataType::Dynamic),
                                    kind: "Decl",
                                    info
                                }),
                                value: Box::new(Node::Block {
                                    is_body: false,
                                    depth: None,
                                    statements: Vec::from([Box::new(Node::ReturnExpr {
                                        node: Box::new(Node::IntLit {
                                            v: 5,
                                            size: 8,
                                            info
                                        }),
                                        info
                                    })]),
                                    kind: "LocalScope",
                                    info
                                }),
                                kind: "LetDecl",
                                info
                            })],
                            kind: "",
                            info,
                            depth: Some(1),
                            is_body: true
                        }),
                        kind: "",
                        info
                    }),]),
                    kind: "ProgramNode"
                },
                parser.parse()
            );
        }

        #[test]
        fn test_function_call() {
            let line = "name()";
            let mut parser = Parser::new(line);
            let info = NodeDetails::new(0, 0);
            assert_eq!(
                Node::Program {
                    statements: Vec::from([Box::new(Node::CallExpr {
                        ident: Box::new(Node::IdentExpr {
                            name: "name",
                            r: None,
                            ty: None,
                            info
                        }),
                        params: vec![],
                        args: vec![],
                        kind: "",
                        info,
                        r: None,
                    }),]),
                    kind: "ProgramNode"
                },
                parser.parse()
            );
        }
        #[test]
        fn test_function_call2() {
            let line = "name[]()";
            let mut parser = Parser::new(line);
            let info = NodeDetails::new(0, 0);
            assert_eq!(
                Node::Program {
                    statements: Vec::from([Box::new(Node::CallExpr {
                        ident: Box::new(Node::IdentExpr {
                            name: "name",
                            r: None,
                            ty: None,
                            info
                        }),
                        params: vec![],
                        args: vec![],
                        kind: "",
                        info,
                        r: None,
                    }),]),
                    kind: "ProgramNode"
                },
                parser.parse()
            );
        }
        #[test]
        #[should_panic]
        fn test_function_call3() {
            let line = "name[)";
            let mut parser = Parser::new(line);
            let info = NodeDetails::new(0, 0);
            assert_eq!(
                Node::Program {
                    statements: Vec::from([Box::new(Node::CallExpr {
                        ident: Box::new(Node::IdentExpr {
                            name: "name",
                            r: None,
                            ty: None,
                            info
                        }),
                        params: vec![],
                        args: vec![],
                        kind: "",
                        info,
                        r: None,
                    }),]),
                    kind: "ProgramNode"
                },
                parser.parse()
            );
        }
        #[test]
        fn test_function_call4() {
            let line = "name[1, 2, 3](1, 2, 3)";
            let mut parser = Parser::new(line);
            let info = NodeDetails::new(0, 0);
            assert_eq!(
                Node::Program {
                    statements: Vec::from([Box::new(Node::CallExpr {
                        ident: Box::new(Node::IdentExpr {
                            name: "name",
                            r: None,
                            ty: None,
                            info
                        }),
                        params: vec![
                            Box::new(Node::IntLit {
                                v: 1,
                                size: 8,
                                info
                            }),
                            Box::new(Node::IntLit {
                                v: 2,
                                size: 8,
                                info
                            }),
                            Box::new(Node::IntLit {
                                v: 3,
                                size: 8,
                                info
                            }),
                        ],
                        args: vec![
                            Box::new(Node::IntLit {
                                v: 1,
                                size: 8,
                                info
                            }),
                            Box::new(Node::IntLit {
                                v: 2,
                                size: 8,
                                info
                            }),
                            Box::new(Node::IntLit {
                                v: 3,
                                size: 8,
                                info
                            }),
                        ],
                        kind: "",
                        info,
                        r: None,
                    }),]),
                    kind: "ProgramNode"
                },
                parser.parse()
            );
        }
    }
}
