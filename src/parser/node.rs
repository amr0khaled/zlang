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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataType {
    Unsigned8,
    Signed8,
    Unsigned16,
    Signed16,
    Unsigned32,
    Signed32,
    Unsigned64,
    Signed64,
    Float32,
    Float64,
    Boolean,
    Dynamic,
    Void,
}

impl DataType {
    pub fn from_token(token: TokenType) -> Self {
        match token {
            TokenType::U8 => Self::Unsigned8,
            TokenType::U16 => Self::Unsigned16,
            TokenType::U32 => Self::Unsigned32,
            TokenType::U64 => Self::Unsigned64,
            TokenType::I8 => Self::Signed8,
            TokenType::I16 => Self::Signed16,
            TokenType::I32 => Self::Signed32,
            TokenType::I64 => Self::Signed64,
            TokenType::F32 => Self::Float32,
            TokenType::F64 => Self::Float64,
            TokenType::Bool => Self::Boolean,
            _ => panic!("Syntax error: Expected datatype, got {:?}", token),
        }
    }
    pub fn from_symbol(ty: SymbolType) -> Self {
        match ty {
            SymbolType::Unsigned8 => Self::Unsigned8,
            SymbolType::Unsigned16 => Self::Unsigned16,
            SymbolType::Unsigned32 => Self::Unsigned32,
            SymbolType::Unsigned64 => Self::Unsigned64,
            SymbolType::Signed8 => Self::Signed8,
            SymbolType::Signed16 => Self::Signed16,
            SymbolType::Signed32 => Self::Signed32,
            SymbolType::Signed64 => Self::Signed64,
            SymbolType::Float32 => Self::Float32,
            SymbolType::Float64 => Self::Float64,
            SymbolType::Boolean => Self::Boolean,
            _ => panic!("Syntax error: Expected datatype, got {:?}", ty),
        }
    }
    pub fn get_size(&self) -> usize {
        match self {
            DataType::Unsigned8 | DataType::Signed8 => 8,
            DataType::Unsigned16 | DataType::Signed16 => 16,
            DataType::Unsigned32 | DataType::Signed32 | DataType::Float32 => 32,
            DataType::Unsigned64 | DataType::Signed64 | DataType::Float64 => 64,
            DataType::Boolean => 1,
            DataType::Dynamic => 0,
            DataType::Void => 0,
        }
    }
    pub fn is_unsigned(&self) -> bool {
        match self {
            DataType::Unsigned8 => true,
            DataType::Unsigned16 => true,
            DataType::Unsigned32 => true,
            DataType::Unsigned64 => true,
            _ => false,
        }
    }
    pub fn is_signed(&self) -> bool {
        match self {
            DataType::Signed8 => true,
            DataType::Signed16 => true,
            DataType::Signed32 => true,
            DataType::Signed64 => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Node<'a> {
    // ===References===
    Ref(u32, usize),
    DataType(DataType),
    // ===Literal===
    UintLit {
        v: u64,
        size: usize,
        info: NodeDetails,
    },
    IntLit {
        v: i64,
        size: usize,
        info: NodeDetails,
    },
    FloatLit {
        v: f64,
        size: usize,
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
        ty: Option<DataType>,
        info: NodeDetails,
    },
    UnaryExpr {
        op: Operator,
        node: Box<Node<'a>>,
        kind: &'a str,
        ty: Option<DataType>,
        info: NodeDetails,
    }, // --IDENT
    BinaryExpr {
        op: Operator,
        left: Box<Node<'a>>,
        right: Box<Node<'a>>,
        kind: &'a str,
        ty: Option<DataType>,
        info: NodeDetails,
    }, // x + y | 5 < 2
    CallExpr {
        ident: Box<Node<'a>>,
        params: Vec<Box<Node<'a>>>,
        args: Vec<Box<Node<'a>>>,
        kind: &'a str,
        info: NodeDetails,
        r: Option<Box<Node<'a>>>,
    },
    // ===Statements===
    FunctionDecl {
        decl: Box<Node<'a>>,
        params: Vec<Box<Node<'a>>>,
        args: Vec<Box<Node<'a>>>,
        block: Box<Node<'a>>,
        kind: &'a str,
        info: NodeDetails,
    },
    Decl {
        ident: &'a str,
        ty: Option<DataType>,
        kind: &'a str,
        info: NodeDetails,
    },
    LetDecl {
        decl: Box<Node<'a>>,
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
    ConstDecl {
        decl: Box<Node<'a>>,
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

impl<'a> PartialEq for Node<'a> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Ref(l0, l1), Self::Ref(r0, r1)) => l0 == r0 && l1 == r1,
            (Self::DataType(l0), Self::DataType(r0)) => l0 == r0,
            (
                Self::UintLit {
                    v: l_v,
                    size: l_size,
                    ..
                },
                Self::UintLit {
                    v: r_v,
                    size: r_size,
                    ..
                },
            ) => l_v == r_v && l_size == r_size,
            (
                Self::IntLit {
                    v: l_v,
                    size: l_size,
                    ..
                },
                Self::IntLit {
                    v: r_v,
                    size: r_size,
                    ..
                },
            ) => l_v == r_v && l_size == r_size,
            (
                Self::FloatLit {
                    v: l_v,
                    size: l_size,
                    ..
                },
                Self::FloatLit {
                    v: r_v,
                    size: r_size,
                    ..
                },
            ) => l_v == r_v && l_size == r_size,
            (Self::BooleanLit { v: l_v, .. }, Self::BooleanLit { v: r_v, .. }) => l_v == r_v,
            (
                Self::IdentExpr {
                    name: l_name,
                    r: l_r,
                    ty: l_ty,
                    ..
                },
                Self::IdentExpr {
                    name: r_name,
                    r: r_r,
                    ty: r_ty,
                    ..
                },
            ) => l_name == r_name && l_r == r_r && l_ty == r_ty,
            (
                Self::UnaryExpr {
                    op: l_op,
                    node: l_node,

                    ty: l_ty,
                    ..
                },
                Self::UnaryExpr {
                    op: r_op,
                    node: r_node,

                    ty: r_ty,
                    ..
                },
            ) => l_op == r_op && l_node == r_node && l_ty == r_ty,
            (
                Self::BinaryExpr {
                    op: l_op,
                    left: l_left,
                    right: l_right,

                    ty: l_ty,
                    ..
                },
                Self::BinaryExpr {
                    op: r_op,
                    left: r_left,
                    right: r_right,

                    ty: r_ty,
                    ..
                },
            ) => l_op == r_op && l_left == r_left && l_right == r_right && l_ty == r_ty,
            (
                Self::CallExpr {
                    ident: l_ident,
                    params: l_params,
                    args: l_args,

                    r: l_r,
                    ..
                },
                Self::CallExpr {
                    ident: r_ident,
                    params: r_params,
                    args: r_args,

                    r: r_r,
                    ..
                },
            ) => l_ident == r_ident && l_params == r_params && l_args == r_args && l_r == r_r,
            (
                Self::FunctionDecl {
                    decl: l_decl,
                    params: l_params,
                    args: l_args,
                    ..
                },
                Self::FunctionDecl {
                    decl: r_decl,
                    params: r_params,
                    args: r_args,
                    ..
                },
            ) => l_decl == r_decl && l_params == r_params && l_args == r_args,
            (
                Self::Decl {
                    ident: l_ident,
                    ty: l_ty,
                    ..
                },
                Self::Decl {
                    ident: r_ident,
                    ty: r_ty,
                    ..
                },
            ) => l_ident == r_ident && l_ty == r_ty,
            (
                Self::LetDecl {
                    decl: l_decl,
                    value: l_value,

                    r: l_r,
                    ..
                },
                Self::LetDecl {
                    decl: r_decl,
                    value: r_value,

                    r: r_r,
                    ..
                },
            ) => l_decl == r_decl && l_value == r_value && l_r == r_r,
            (
                Self::AssignStmt {
                    ident: l_ident,
                    op: l_op,
                    value: l_value,
                    r: l_r,
                    ..
                },
                Self::AssignStmt {
                    ident: r_ident,
                    op: r_op,
                    value: r_value,
                    r: r_r,
                    ..
                },
            ) => l_ident == r_ident && l_op == r_op && l_value == r_value && l_r == r_r,
            (
                Self::ConstDecl {
                    decl: l_decl,
                    value: l_value,

                    r: l_r,
                    ..
                },
                Self::ConstDecl {
                    decl: r_decl,
                    value: r_value,

                    r: r_r,
                    ..
                },
            ) => l_decl == r_decl && l_value == r_value && l_r == r_r,
            (Self::ExprStmt { node: l_node, .. }, Self::ExprStmt { node: r_node, .. }) => {
                l_node == r_node
            }
            (Self::ReturnExpr { node: l_node, .. }, Self::ReturnExpr { node: r_node, .. }) => {
                l_node == r_node
            }
            (
                Self::Block {
                    statements: l_statements,

                    depth: l_depth,
                    is_body: l_is_body,
                    ..
                },
                Self::Block {
                    statements: r_statements,

                    depth: r_depth,
                    is_body: r_is_body,
                    ..
                },
            ) => l_statements == r_statements && l_depth == r_depth && l_is_body == r_is_body,
            (
                Self::Program {
                    statements: l_statements,
                    ..
                },
                Self::Program {
                    statements: r_statements,
                    ..
                },
            ) => l_statements == r_statements,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
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
            Node::IntLit { info, .. } => Some(*info),
            Node::FloatLit { info, .. } => Some(*info),
            Node::BooleanLit { info, .. } => Some(*info),
            Node::IdentExpr { info, .. } => Some(*info),
            Node::UnaryExpr { info, .. } => Some(*info),
            Node::BinaryExpr { info, .. } => Some(*info),
            Node::CallExpr { info, .. } => Some(*info),
            Node::LetDecl { info, .. } => Some(*info),
            Node::AssignStmt { info, .. } => Some(*info),
            Node::ConstDecl { info, .. } => Some(*info),
            Node::ExprStmt { info, .. } => Some(*info),
            Node::ReturnExpr { info, .. } => Some(*info),
            Node::Block { info, .. } => Some(*info),
            _ => None,
        }
    }
}
