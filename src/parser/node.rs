use crate::{
    analyzer::symbol::SymbolType,
    lexer::{Token, types::TokenType},
    parser::ops::Operator,
};

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct NodeDetails {
    line: usize,
    col: usize,
}

impl NodeDetails {
    pub fn new(line: usize, col: usize) -> Self {
        Self { line, col }
    }
    pub fn log(&self) -> String {
        format!("Line {}:{}", self.line, self.col)
    }
}

#[derive(Debug, PartialEq)]
#[allow(dead_code)]
pub enum Node<'a> {
    // ===References===
    Ref(u32, usize),
    // ===Literal===
    IntegerLit {
        v: usize,
        info: NodeDetails,
    },
    FloatLit {
        v: f64,
        info: NodeDetails,
    },
    BooleanLit {
        v: bool,
        info: NodeDetails,
    },
    Dynamic,
    // ===Expressions===
    IdentExpr {
        name: &'a str,
        r: Option<Box<Node<'a>>>,
        ty: Option<SymbolType>,
        info: NodeDetails,
    },
    UnaryExpr {
        op: Operator,
        node: Box<Node<'a>>,
        kind: &'a str,
        ty: Option<SymbolType>,
        info: NodeDetails,
    }, // --IDENT
    BinaryExpr {
        op: Operator,
        left: Box<Node<'a>>,
        right: Box<Node<'a>>,
        kind: &'a str,
        ty: Option<SymbolType>,
        info: NodeDetails,
    }, // x + y | 5 < 2
    CallExpr {
        ident: Box<Node<'a>>,
        args: Vec<Box<Node<'a>>>,
        kind: &'a str,
        info: NodeDetails,
        r: Option<Box<Node<'a>>>,
    },
    // ===Statements===
    VarDeclStmt {
        ident: &'a str,
        value: Box<Node<'a>>,
        kind: &'a str,
        info: NodeDetails,
        r: Option<Box<Node<'a>>>,
    },
    AssignStmt {
        ident: &'a str,
        op: Operator,
        value: Box<Node<'a>>,
        info: NodeDetails,
        r: Option<Box<Node<'a>>>,
    },
    ConstDeclStmt {
        ident: &'a str,
        value: Box<Node<'a>>,
        kind: &'a str,
        info: NodeDetails,
        r: Option<Box<Node<'a>>>,
    },
    ExprStmt {
        node: Box<Node<'a>>,
        info: NodeDetails,
    },
    ReturnExpr {
        node: Box<Node<'a>>,
        info: NodeDetails,
    },
    Block {
        statements: Vec<Box<Node<'a>>>,
        kind: &'a str,
        info: NodeDetails,
        depth: Option<usize>,
        is_body: bool,
    },
    Program {
        statements: Vec<Box<Node<'a>>>,
        kind: &'a str,
    },
    Empty,
}

impl<'a> Node<'a> {
    pub fn info(token: &Token) -> NodeDetails {
        NodeDetails {
            line: token.line,
            col: token.col,
        }
    }
    pub fn inf(&self) -> Option<NodeDetails> {
        match self {
            Node::IntegerLit { info, .. } => Some(*info),
            Node::FloatLit { info, .. } => Some(*info),
            Node::BooleanLit { info, .. } => Some(*info),
            Node::IdentExpr { info, .. } => Some(*info),
            Node::UnaryExpr { info, .. } => Some(*info),
            Node::BinaryExpr { info, .. } => Some(*info),
            Node::CallExpr { info, .. } => Some(*info),
            Node::VarDeclStmt { info, .. } => Some(*info),
            Node::AssignStmt { info, .. } => Some(*info),
            Node::ConstDeclStmt { info, .. } => Some(*info),
            Node::ExprStmt { info, .. } => Some(*info),
            Node::ReturnExpr { info, .. } => Some(*info),
            Node::Block { info, .. } => Some(*info),
            _ => None,
        }
    }
}
