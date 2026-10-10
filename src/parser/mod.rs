use core::panic;
use std::vec;

use crate::{
    lexer::{
        Token, Tokenizer,
        types::TokenType::{self, SemiColon},
    },
    parser::{
        node::{BoundVar, DataType, Node, NodeDetails},
        ops::{Operator, get_pred},
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
    pub fn parse(&mut self) -> Node<'a> {
        let mut stmts: Vec<Box<Node<'a>>> = vec![];
        while self.peek().kind != TokenType::EOF {
            let (stmt, _, _) = self.parse_statement();
            if !matches!(stmt, Node::Empty) {
                stmts.push(Box::new(stmt));
            }
        }
        Node::Program {
            statements: stmts,
            kind: "ProgramNode",
        }
    }
    pub fn parse_statement(&mut self) -> (Node<'a>, TokenType, Option<NodeDetails>) {
        let info = Node::info(self.peek());
        let stmt = match self.peek().kind {
            // ##### KEYWORDS
            TokenType::Let => self.parse_let(),
            TokenType::Const => self.parse_const(),
            TokenType::Function => self.parse_func(),
            TokenType::Return => self.parse_return(),
            // ##### EXPRESSIONS
            TokenType::Ident => self.parse_expr_ident(None),
            TokenType::OpenBracket => self.parse_block(),
            TokenType::OpenParenthese
            | TokenType::OpenSquareBracket
            | TokenType::IntLiteral
            | TokenType::FloatLiteral
            | TokenType::BoolLiteral => self.parse_expr(),
            x if is_unary_operator(x) => self.parse_expr(),
            TokenType::EOL => {
                self.next();
                Node::Empty
            }
            t => syntax_error!("Unexpected token '{:?}'", t),
        };
        let mut _info = None;
        let needs_terminator = !matches!(
            stmt,
            Node::FunctionDecl { .. } | Node::Block { .. } | Node::Empty
        );
        let term = if !needs_terminator {
            TokenType::EOL
        } else if self.peek().kind == TokenType::CloseBracket {
            _info = Some(Node::info(self.peek()));
            TokenType::CloseBracket
        } else {
            println!("WHAT?: {:?}", self.peek().kind);
            self.expect_one_of(&[TokenType::EOL, TokenType::SemiColon, TokenType::EOF])
                .unwrap()
                .kind
        };
        (stmt, term, Some(info))
    }
    fn parse_let(&mut self) -> Node<'a> {
        let tok = expect!(self, TokenType::Let);
        let info = Node::info(&tok);
        let mut bound = self.parse_ident();
        let mut ty = None;
        if self.peek().kind == TokenType::Colon {
            ty = Some(self.parse_definition());
        }
        bound.ty = ty;
        let value;
        if self.peek().kind == TokenType::Assign {
            self.next();
            value = self.parse_expr();
        } else {
            value = Node::Empty;
        }
        Node::LetDecl {
            decl: bound,
            value: Box::new(value),
            kind: "LetDecl",
            info,
        }
    }
    fn parse_const(&mut self) -> Node<'a> {
        let tok;
        // TODO: Add condition to consume static here
        //let mut is_static = false;
        //if self.peek().kind == TokenType::Static {
        //    tok = expect!(self, TokenType::Static);
        //}
        //
        tok = expect!(self, TokenType::Const);
        let info = Node::info(&tok);
        let mut bound = self.parse_ident();
        bound.ty = Some(self.parse_definition());
        expect!(self, TokenType::Assign);
        let value = self.parse_expr();
        Node::ConstDecl {
            decl: bound,
            value: Box::new(value),
            kind: "ConstDecl",
            info,
        }
    }
    fn parse_func(&mut self) -> Node<'a> {
        let tok = expect!(self, TokenType::Function);
        let info = Node::info(&tok);
        let mut bound = self.parse_ident();
        let mut return_ty = None;
        let mut compile_params = vec![];
        let mut runtime_params = vec![];
        if self.peek().kind == TokenType::Colon {
            return_ty = Some(self.parse_definition());
        }
        bound.ty = return_ty;
        if self.peek().kind == TokenType::OpenSquareBracket {
            self.next();
            compile_params = self.parse_params(TokenType::CloseSquareBracket)
        }
        expect!(self, TokenType::OpenParenthese);
        runtime_params = self.parse_params(TokenType::CloseParenthese);
        println!("PEEK BLOCK: {:?}", self.peek().kind);
        let mut block = self.parse_block();
        if let Node::Block { is_body, .. } = &mut block {
            *is_body = true
        }
        Node::FunctionDecl {
            decl: bound,
            kind: "FunctionDecl",
            compile_params,
            runtime_params,
            block: Box::new(block),
            info,
        }
    }
    fn parse_params(&mut self, condition_token: TokenType) -> Vec<BoundVar<'a>> {
        let mut params: Vec<BoundVar<'a>> = vec![];
        while self.peek().kind != condition_token {
            if self.peek().kind == TokenType::EOF {
                syntax_error!("Unclosed parameter list");
            }
            let mut ident = self.parse_ident();
            let ty = self.parse_definition();
            ident.ty = Some(ty);
            params.push(ident);
            match self.peek().kind {
                TokenType::Comma => {
                    self.next();
                }
                x if x == condition_token => {}
                _ => {
                    let tok = self.next();
                    let info = Node::info(&tok);
                    syntax_error!(info, "Unexpected '{:?}'", tok.kind);
                }
            };
        }
        self.next();
        return params;
    }
    fn parse_args(&mut self, condition_token: TokenType) -> Vec<Box<Node<'a>>> {
        let mut args: Vec<Box<Node<'a>>> = vec![];
        println!("LOOP: {:?} {:?}", self.peek().kind, condition_token);
        while self.peek().kind != condition_token {
            if self.peek().kind == TokenType::EOF {
                syntax_error!("Unclosed argument list");
            }
            let arg = self.parse_expr();
            args.push(Box::new(arg));
            match self.peek().kind {
                TokenType::Comma => {
                    self.next();
                }
                x if x == condition_token => {}
                _ => {
                    let tok = self.next();
                    let info = Node::info(&tok);
                    // if x == TokenType::Colon {
                    //     syntax_error!(info, "Unexpected 'Colon', this argument cannot be casted");
                    // }
                    syntax_error!(info, "Unexpected '{:?}'", tok.kind);
                }
            };
        }
        self.next();
        args
    }
    fn parse_block(&mut self) -> Node<'a> {
        self.next();
        let mut stmts = vec![];
        let info = Node::info(self.peek());
        while self.peek().kind != TokenType::CloseBracket {
            while self.peek().kind == TokenType::EOL {
                self.next();
            }
            if self.peek().kind == TokenType::CloseBracket {
                break;
            }
            let (stmt, end, _info) = self.parse_statement();
            if stmt != Node::Empty {
                if end != TokenType::SemiColon && self.peek().kind == TokenType::CloseBracket {
                    println!("NODEEEE: {:?}", stmt);
                    stmts.push(Box::new(Node::ReturnExpr {
                        node: Box::new(stmt),
                        info: _info.unwrap(),
                    }));
                } else {
                    stmts.push(Box::new(stmt));
                }
            }
        }
        expect!(self, TokenType::CloseBracket);
        Node::Block {
            statements: stmts,
            kind: "BlockScope",
            info,
            depth: None,
            is_body: false,
        }
    }
    fn parse_return(&mut self) -> Node<'a> {
        let tok = self.next();
        let info = Node::info(&tok);
        if self.peek().kind == SemiColon {
            syntax_error!(info, "Unexpected token, Return shall have a expression")
        }
        let expr = self.parse_expr();
        Node::ReturnExpr {
            node: Box::new(expr),
            info,
        }
    }
    fn parse_definition(&mut self) -> DataType {
        expect!(self, TokenType::Colon);
        if !is_datatype(self.peek().kind) {
            let tok = self.next();
            let info = Node::info(&tok);
            syntax_error!(info, "Expected DataType not '{:?}'", tok.kind);
        }
        let ty = DataType::from_token(self.next().kind);
        ty
    }
    fn parse_as(&mut self) -> DataType {
        expect!(self, TokenType::As);
        if !is_datatype(self.peek().kind) {
            let tok = self.next();
            let info = Node::info(&tok);
            syntax_error!(info, "Expected DataType not '{:?}'", tok.kind);
        }
        let ty = DataType::from_token(self.next().kind);
        ty
    }
    fn parse_ident(&mut self) -> BoundVar<'a> {
        let tok = expect!(self, TokenType::Ident);
        let info = Node::info(&tok);
        BoundVar::new(tok.pointer, None, info)
    }
    fn parse_expr_ident(&mut self, mut ident_expr: Option<Node<'a>>) -> Node<'a> {
        if ident_expr.is_none() {
            let ident = self.parse_ident();
            ident_expr = Some(Node::IdentExpr {
                name: ident.ident,
                r: None,
                ty: None,
                info: ident.info,
            });
        }
        match self.peek().kind {
            TokenType::OpenParenthese | TokenType::OpenSquareBracket => {
                let token = self.next();
                let info = Node::info(&token);
                let mut compile_args = vec![];
                if token.kind == TokenType::OpenSquareBracket {
                    compile_args = self.parse_args(TokenType::CloseSquareBracket);
                    expect!(self, TokenType::OpenParenthese);
                }
                let runtime_args = self.parse_args(TokenType::CloseParenthese);
                Node::CallExpr {
                    ident: Box::new(ident_expr.unwrap()),
                    compile_args,
                    runtime_args,
                    kind: "CallExpr",
                    r: None,
                    info,
                }
            }
            x if is_operator(x) => self.parse_expr_from(ident_expr),
            TokenType::SemiColon => ident_expr.unwrap(),
            x => {
                let info = Node::info(self.peek());
                syntax_error!(info, "Unexpected token '{:?}'", x)
            }
        }
    }
    fn parse_expr(&mut self) -> Node<'a> {
        self.parse_expr_from(None)
    }
    fn parse_expr_from(&mut self, lhs: Option<Node<'a>>) -> Node<'a> {
        let mut ops: Vec<(Token<'a>, bool)> = vec![];
        let mut nodes: Vec<Node> = vec![];
        let mut is_ident = false;
        let mut after_op = false;
        if let Some(lhs) = lhs {
            nodes.push(lhs);
        }
        let combine = |nodes: &mut Vec<Node<'a>>, token: Token<'a>, unary: bool, after_op: bool| {
            let info = Node::info(&token);
            let mut op = Operator::from_token(token.kind);
            if unary {
                let node = Box::new(nodes.pop().unwrap());
                if token.kind != TokenType::Not {
                    op = match token.kind {
                        TokenType::Increment => {
                            if after_op {
                                Some(Operator::PreIncrement)
                            } else {
                                Some(Operator::PostIncrement)
                            }
                        }
                        TokenType::Decrement => {
                            if after_op {
                                Some(Operator::PreDecrement)
                            } else {
                                Some(Operator::PostDecrement)
                            }
                        }
                        _ => syntax_error!("Unexpected token '{:?}'", token.kind),
                    }
                }
                nodes.push(Node::UnaryExpr {
                    op: op.unwrap(),
                    node,
                    kind: "UnaryExpr",
                    ty: None,
                    info,
                });
            } else {
                let right = Box::new(nodes.pop().unwrap());
                let left = Box::new(nodes.pop().unwrap());
                let node;
                if is_assign_operator(token.kind) {
                    node = Node::AssignStmt {
                        ident: left,
                        op: op.unwrap(),
                        value: right,
                        info,
                        r: None,
                    }
                } else {
                    node = Node::BinaryExpr {
                        op: op.unwrap(),
                        left,
                        right,
                        kind: "BinaryExpr",
                        ty: None,
                        info,
                    }
                }
                nodes.push(node);
            }
        };

        loop {
            println!("PEEK: {:?}", self.peek().kind);
            match self.peek().kind {
                TokenType::Ident
                | TokenType::IntLiteral
                | TokenType::FloatLiteral
                | TokenType::BoolLiteral => {
                    if nodes.last().is_some() && ops.last().is_none() {
                        panic!("Expected operator, got '{:?}'", self.peek().kind);
                    }
                    if self.peek().kind == TokenType::Ident {
                        is_ident = true;
                    }
                    nodes.push(self.parse_primary())
                }
                x if is_operator(x) => {
                    let tok = self.next();
                    if is_unary_operator(x) {
                        if nodes.last().is_none() {
                            after_op = true;
                        }
                        ops.push((tok, true));
                    } else {
                        after_op = false;
                        is_ident = false;

                        while let Some((top, unary)) = ops.last() {
                            if !*unary || get_pred(top.kind) < get_pred(tok.kind) {
                                break;
                            }
                            let (top, unary) = ops.pop().unwrap();
                            combine(&mut nodes, top, unary, false);
                        }

                        ops.push((tok, false));
                    }
                }
                TokenType::OpenBracket => {
                    let inner = self.parse_block();
                    nodes.push(inner);
                }
                TokenType::OpenParenthese => {
                    if is_ident {
                        is_ident = false;
                        let ident = nodes.pop();
                        nodes.push(self.parse_expr_ident(ident));
                    } else {
                        let inner = self.parse_expr();
                        expect!(self, TokenType::CloseParenthese);
                        nodes.push(inner);
                    }
                }
                TokenType::EOL => {
                    self.next();
                }
                _ => {
                    println!("BREAKING ON: {:?}", self.peek().kind);
                    break;
                }
            };
            println!("NODES: {:#?}", nodes);
            println!("OPS: {:#?}", ops);
        }
        while let Some((tok, unary)) = ops.pop() {
            combine(&mut nodes, tok, unary, after_op);
        }
        let node = nodes.pop().unwrap();
        println!("RETURNED: {:?}", node);
        node
    }
    fn parse_primary(&mut self) -> Node<'a> {
        match self.peek().kind {
            TokenType::IntLiteral | TokenType::FloatLiteral | TokenType::BoolLiteral => {
                self.parse_literal()
            }
            TokenType::Ident => {
                let ident = self.parse_ident();
                Node::IdentExpr {
                    name: ident.ident,
                    r: None,
                    ty: None,
                    info: ident.info,
                }
            }
            /* TODO: Work on this when you support vectors
            TokenType::OpenSquareBracket => {
                self.next();
                let args = self.parse_args(TokenType::CloseSquareBracket);

            }
            */
            _ => syntax_error!("Unexpected token"),
        }
    }
    fn parse_literal(&mut self) -> Node<'a> {
        let token = self.next();
        let info = Node::info(&token);
        match token.kind {
            TokenType::IntLiteral => {
                let v = token.pointer.parse::<i64>().unwrap_or_else(|_| {
                    let info = Node::info(&token);
                    syntax_error!(info, "Invalid integer '{:?}'", token.pointer);
                });
                let mut ty = DataType::Signed32;
                let mut size: usize = 4;
                if self.peek().kind == TokenType::As {
                    ty = self.parse_as();
                    size = ty.get_size();
                }
                Node::IntLit { v, size, ty, info }
            }

            TokenType::FloatLiteral => {
                let v = token.pointer.parse::<f64>().unwrap_or_else(|_| {
                    let info = Node::info(&token);
                    syntax_error!(info, "Invalid integer '{:?}'", token.pointer);
                });
                let mut ty = DataType::Signed32;
                let mut size: usize = 4;
                if self.peek().kind == TokenType::As {
                    ty = self.parse_as();
                    size = ty.get_size();
                }
                Node::FloatLit { v, size, ty, info }
            }
            TokenType::BoolLiteral => {
                let v = token.pointer.parse::<bool>().unwrap();
                Node::BooleanLit { v, info }
            }
            _ => syntax_error!("Unexpected token"),
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

fn is_operator(ty: TokenType) -> bool {
    match ty {
        x if is_assign_operator(x) || is_binary_operator(x) || is_unary_operator(x) => true,
        _ => false,
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
fn is_binary_operator(ty: TokenType) -> bool {
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
        x if is_assign_operator(x) => true,
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

macro_rules! syntax_error {
    ($d:ident, $($arg:tt)*) => {{
        panic!("{} <- Syntax Error: {}", $d.log(), format_args!($($arg)*));
    }};
    ($($arg:tt)*) => {{
        panic!("Syntax Error: {}", format_args!($($arg)*));
    }};
}
pub(crate) use syntax_error;

#[cfg(test)]
pub mod tests;
