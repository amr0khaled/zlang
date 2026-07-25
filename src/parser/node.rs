use crate::parser::ops::Operator;

#[derive(Debug, PartialEq)]
#[allow(dead_code)]
pub enum Node<'a> {
    // ===Literal===
    IntegerLit(usize),
    FloatLit(f64),
    BooleanLit(bool),
    // ===Expressions===
    IdentExpr(&'a str),
    UnaryExpr {
        op: Operator,
        node: Box<Node<'a>>,
        kind: &'a str,
    }, // --IDENT
    BinaryExpr {
        op: Operator,
        left: Box<Node<'a>>,
        right: Box<Node<'a>>,
        kind: &'a str,
    }, // x + y | 5 < 2
    CallExpr {
        ident: Box<Node<'a>>,
        args: Vec<Box<Node<'a>>>,
        kind: &'a str,
    },
    // ===Statements===
    VarDeclStmt {
        ident: &'a str,
        value: Box<Node<'a>>,
        kind: &'a str,
    },
    AssignStmt {
        ident: &'a str,
        value: Box<Node<'a>>,
        kind: &'a str,
    },
    ConstDeclStmt {
        ident: &'a str,
        value: Box<Node<'a>>,
        kind: &'a str,
    },
    ExprStmt(Box<Node<'a>>),
    Block {
        statements: Vec<Box<Node<'a>>>,
        kind: &'a str,
    },
    Empty,
}
