use super::node::NodeDetails;
use super::*;

#[allow(dead_code)]
fn clear_info(node: &mut Node<'_>) {
    let blank = NodeDetails::new(0, 0, TokenType::EOL);
    match node {
        // leaves: only their own info
        Node::DataType(_) | Node::Dynamic | Node::Empty => {}
        Node::IntLit { info, .. }
        | Node::FloatLit { info, .. }
        | Node::BooleanLit { info, .. }
        | Node::IdentExpr { info, .. } => *info = blank,

        // children first, then own info
        Node::UnaryExpr { node, info, .. } | Node::ReturnExpr { node, info } => {
            clear_info(node);
            *info = blank;
        }
        Node::BinaryExpr {
            left, right, info, ..
        } => {
            clear_info(left);
            clear_info(right);
            *info = blank;
        }
        Node::CallExpr {
            ident,
            compile_args,
            runtime_args,
            info,
            ..
        } => {
            clear_info(ident);
            for a in compile_args.iter_mut().chain(runtime_args.iter_mut()) {
                clear_info(a);
            }
            *info = blank;
        }
        Node::FunctionDecl {
            decl, block, info, ..
        } => {
            decl.info = blank;
            clear_info(block);
            *info = blank;
        }
        Node::LetDecl {
            decl, value, info, ..
        }
        | Node::ConstDecl {
            decl, value, info, ..
        } => {
            decl.info = blank;
            clear_info(value);
            *info = blank;
        }
        Node::AssignStmt {
            ident, value, info, ..
        } => {
            clear_info(ident);
            clear_info(value);
            *info = blank;
        }
        Node::Block {
            statements, info, ..
        } => {
            for s in statements.iter_mut() {
                clear_info(s);
            }
            *info = blank;
        }
        Node::Program { statements, .. } => {
            for s in statements.iter_mut() {
                clear_info(s);
            }
        }
    }
}
fn assert_node(expected: &Node, actual: &Node) {
    match (expected, actual) {
        (Node::DataType(e), Node::DataType(a)) => assert_eq!(e, a, "DataType"),
        (Node::Dynamic, Node::Dynamic) | (Node::Empty, Node::Empty) => {}

        (
            Node::IntLit {
                v: ev,
                size: es,
                ty: et,
                ..
            },
            Node::IntLit {
                v: av,
                size: asz,
                ty: at,
                ..
            },
        ) => {
            assert_eq!(ev, av, "IntLit value");
            assert_eq!(es, asz, "IntLit size");
            assert_eq!(et, at, "IntLit type");
        }
        (
            Node::FloatLit {
                v: ev,
                size: es,
                ty: et,
                ..
            },
            Node::FloatLit {
                v: av,
                size: asz,
                ty: at,
                ..
            },
        ) => {
            assert_eq!(ev, av, "FloatLit value");
            assert_eq!(es, asz, "FloatLit size");
            assert_eq!(et, at, "FloatLit type");
        }
        (Node::BooleanLit { v: ev, .. }, Node::BooleanLit { v: av, .. }) => {
            assert_eq!(ev, av, "BooleanLit value");
        }

        (
            Node::IdentExpr {
                name: en, ty: et, ..
            },
            Node::IdentExpr {
                name: an, ty: at, ..
            },
        ) => {
            assert_eq!(en, an, "IdentExpr name");
            assert_eq!(et, at, "IdentExpr type");
        }
        (
            Node::UnaryExpr {
                op: eo, node: en, ..
            },
            Node::UnaryExpr {
                op: ao, node: an, ..
            },
        ) => {
            assert_eq!(eo, ao, "UnaryExpr op");
            assert_node(en, an);
        }
        (
            Node::BinaryExpr {
                op: eo,
                left: el,
                right: er,
                ..
            },
            Node::BinaryExpr {
                op: ao,
                left: al,
                right: ar,
                ..
            },
        ) => {
            assert_eq!(eo, ao, "BinaryExpr op");
            assert_node(el, al);
            assert_node(er, ar);
        }
        (
            Node::CallExpr {
                ident: ei,
                compile_args: ec,
                runtime_args: er,
                ..
            },
            Node::CallExpr {
                ident: ai,
                compile_args: ac,
                runtime_args: ar,
                ..
            },
        ) => {
            assert_node(ei, ai);
            assert_list("CallExpr compile_args", ec, ac);
            assert_list("CallExpr runtime_args", er, ar);
        }

        (
            Node::FunctionDecl {
                decl: ed,
                compile_params: ecp,
                runtime_params: erp,
                block: eb,
                ..
            },
            Node::FunctionDecl {
                decl: ad,
                compile_params: acp,
                runtime_params: arp,
                block: ab,
                ..
            },
        ) => {
            assert_bound(ed, ad);
            assert_eq!(ecp.len(), acp.len(), "FunctionDecl compile_params count");
            for (e, a) in ecp.iter().zip(acp) {
                assert_bound(e, a);
            }
            assert_eq!(erp.len(), arp.len(), "FunctionDecl runtime_params count");
            for (e, a) in erp.iter().zip(arp) {
                assert_bound(e, a);
            }
            assert_node(eb, ab);
        }
        (
            Node::LetDecl {
                decl: ed,
                value: ev,
                ..
            },
            Node::LetDecl {
                decl: ad,
                value: av,
                ..
            },
        ) => {
            assert_bound(ed, ad);
            assert_node(ev, av);
        }
        (
            Node::ConstDecl {
                decl: ed,
                value: ev,
                ..
            },
            Node::ConstDecl {
                decl: ad,
                value: av,
                ..
            },
        ) => {
            assert_bound(ed, ad);
            assert_node(ev, av);
        }
        (
            Node::AssignStmt {
                ident: ei,
                op: eo,
                value: ev,
                ..
            },
            Node::AssignStmt {
                ident: ai,
                op: ao,
                value: av,
                ..
            },
        ) => {
            assert_node(ei, ai);
            assert_eq!(eo, ao, "AssignStmt op");
            assert_node(ev, av);
        }
        (Node::ReturnExpr { node: en, .. }, Node::ReturnExpr { node: an, .. }) => {
            assert_node(en, an)
        }
        (
            Node::Block {
                statements: es,
                depth: ed,
                is_body: eb,
                ..
            },
            Node::Block {
                statements: as_,
                depth: ad,
                is_body: ab,
                ..
            },
        ) => {
            assert_eq!(ed, ad, "Block depth");
            assert_eq!(eb, ab, "Block is_body");
            assert_list("Block statements", es, as_);
        }
        (
            Node::Program { statements: es, .. },
            Node::Program {
                statements: as_, ..
            },
        ) => {
            assert_list("Program statements", es, as_);
        }

        _ => core::panic!(
            "different node kinds\nexpected: {:?}\nactual: {:?}",
            expected,
            actual
        ),
    }
}

fn assert_list(label: &str, expected: &[Box<Node>], actual: &[Box<Node>]) {
    assert_eq!(expected.len(), actual.len(), "{label}: number of nodes");
    for (e, a) in expected.iter().zip(actual) {
        assert_node(e, a);
    }
}

fn assert_bound(expected: &BoundVar, actual: &BoundVar) {
    assert_eq!(expected.ident, actual.ident, "BoundVar name");
    assert_eq!(expected.ty, actual.ty, "BoundVar type");
}

#[cfg(test)]
mod declaration_tests {

    use crate::parser::node::DataType::Signed32;

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

        let info = NodeDetails::new(0, 0, TokenType::EOL);
        assert_node(
            &Node::Program {
                statements: Vec::from([Box::new(Node::LetDecl {
                    decl: BoundVar::new("x", None, info),
                    value: Box::new(Node::IntLit {
                        v: 5,
                        size: 4,
                        ty: Signed32,
                        info,
                    }),
                    kind: "LetDecl",
                    info,
                })]),
                kind: "ProgramNode",
            },
            &parser.parse(),
        );
    }
    #[test]
    fn test_let_with_type() {
        for &(sym, ref ty) in PRIMITIVE_TYPES {
            let line = format!("let x: {} = 5;", sym);
            let mut parser = Parser::new(&line);

            let info = NodeDetails::new(0, 0, TokenType::EOL);
            assert_node(
                &Node::Program {
                    statements: Vec::from([Box::new(Node::LetDecl {
                        decl: BoundVar::new("x", Some(*ty), info),
                        value: Box::new(Node::IntLit {
                            v: 5,
                            size: 4,
                            ty: Signed32,
                            info,
                        }),
                        kind: "LetDecl",
                        info,
                    })]),
                    kind: "ProgramNode",
                },
                &parser.parse(),
            );
        }
    }
    #[test]
    fn test_let_as_type() {
        for &(sym, ref ty) in PRIMITIVE_TYPES {
            println!("{}", sym);
            let line = format!("let x = 5 as {};", sym);
            let mut parser = Parser::new(&line);

            let info = NodeDetails::new(0, 0, TokenType::EOL);
            assert_node(
                &Node::Program {
                    statements: Vec::from([Box::new(Node::LetDecl {
                        decl: BoundVar::new("x", None, info),
                        value: Box::new(Node::IntLit {
                            v: 5,
                            size: ty.get_size(),
                            ty: *ty,
                            info,
                        }),
                        kind: "LetDecl",
                        info,
                    })]),
                    kind: "ProgramNode",
                },
                &parser.parse(),
            );
        }
    }

    #[test]
    fn test_let_block() {
        let line = "let x = {5};";
        let mut parser = Parser::new(line);
        let info = NodeDetails::new(0, 0, TokenType::EOL);
        assert_node(
            &Node::Program {
                statements: Vec::from([Box::new(Node::LetDecl {
                    decl: BoundVar::new("x", None, info),
                    value: Box::new(Node::Block {
                        is_body: false,
                        depth: None,
                        statements: Vec::from([Box::new(Node::ReturnExpr {
                            node: Box::new(Node::IntLit {
                                v: 5,
                                size: 4,
                                ty: Signed32,
                                info,
                            }),
                            info,
                        })]),
                        kind: "LocalScope",
                        info,
                    }),
                    kind: "LetDecl",
                    info,
                })]),
                kind: "ProgramNode",
            },
            &parser.parse(),
        );
    }
    #[test]

    fn test_let_block_with_return() {
        let line = "let x = {return 5;};";
        let mut parser = Parser::new(line);
        let info = NodeDetails::new(0, 0, TokenType::EOL);
        assert_node(
            &Node::Program {
                statements: Vec::from([Box::new(Node::LetDecl {
                    decl: BoundVar::new("x", None, info),
                    value: Box::new(Node::Block {
                        is_body: false,
                        depth: None,
                        statements: Vec::from([Box::new(Node::ReturnExpr {
                            node: Box::new(Node::IntLit {
                                v: 5,
                                size: 4,
                                ty: Signed32,
                                info,
                            }),
                            info,
                        })]),
                        kind: "LocalScope",
                        info,
                    }),
                    kind: "LetDecl",
                    info,
                })]),
                kind: "ProgramNode",
            },
            &parser.parse(),
        );
    }
}
#[cfg(test)]
mod operations {

    use crate::parser::node::DataType::Signed32;

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

            let info = NodeDetails::new(0, 0, TokenType::EOL);
            assert_node(
                &Node::Program {
                    statements: Vec::from([Box::new(Node::BinaryExpr {
                        op: op.clone(),
                        left: Box::new(Node::IntLit {
                            v: 5,
                            size: 4,
                            ty: Signed32,
                            info,
                        }),
                        right: Box::new(Node::IntLit {
                            v: 4,
                            size: 4,
                            ty: Signed32,
                            info,
                        }),
                        ty: None,
                        kind: "",
                        info,
                    })]),
                    kind: "",
                },
                &parser.parse(),
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

            let info = NodeDetails::new(0, 0, TokenType::EOL);
            assert_node(
                &Node::Program {
                    statements: Vec::from([
                        Box::new(Node::LetDecl {
                            decl: BoundVar::new("a", None, info),
                            value: Box::new(Node::IntLit {
                                v: 4,
                                size: 4,
                                ty: DataType::Signed32,
                                info,
                            }),
                            kind: "",
                            info,
                        }),
                        Box::new(Node::LetDecl {
                            decl: BoundVar::new("b", None, info),
                            value: Box::new(Node::IntLit {
                                v: 10,
                                ty: Signed32,
                                size: 4,
                                info,
                            }),
                            kind: "",
                            info,
                        }),
                        Box::new(Node::BinaryExpr {
                            op: op.clone(),
                            left: Box::new(Node::IdentExpr {
                                name: "a",
                                r: None,
                                ty: None,
                                info,
                            }),
                            right: Box::new(Node::IdentExpr {
                                name: "b",
                                r: None,
                                ty: None,
                                info,
                            }),
                            ty: None,
                            kind: "",
                            info,
                        }),
                    ]),
                    kind: "",
                },
                &parser.parse(),
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
                    println!("POST {}", sym);
                    format!("a{}", sym)
                }
            );
            let mut parser = Parser::new(&line);

            let info = NodeDetails::new(0, 0, TokenType::EOL);
            assert_node(
                &Node::Program {
                    statements: Vec::from([
                        Box::new(Node::LetDecl {
                            decl: BoundVar::new("a", None, info),
                            value: Box::new(Node::IntLit {
                                v: 4,
                                size: 4,
                                info,
                                ty: Signed32,
                            }),
                            kind: "",
                            info,
                        }),
                        Box::new(Node::UnaryExpr {
                            op: op.clone(),
                            node: Box::new(Node::IdentExpr {
                                name: "a",
                                r: None,
                                ty: None,
                                info,
                            }),
                            ty: None,
                            kind: "",
                            info,
                        }),
                    ]),
                    kind: "",
                },
                &parser.parse(),
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

            let info = NodeDetails::new(0, 0, TokenType::EOL);
            assert_node(
                &Node::Program {
                    statements: Vec::from([
                        Box::new(Node::LetDecl {
                            decl: BoundVar::new("a", None, info),
                            value: Box::new(Node::IntLit {
                                v: 4,
                                size: 4,
                                ty: Signed32,
                                info,
                            }),
                            kind: "",
                            info,
                        }),
                        Box::new(Node::AssignStmt {
                            ident: Box::new(Node::IdentExpr {
                                name: "a",
                                r: None,
                                ty: None,
                                info,
                            }),
                            op: op.clone(),
                            value: Box::new(Node::IntLit {
                                v: 4,
                                size: 4,
                                ty: Signed32,
                                info,
                            }),
                            info,
                            r: None,
                        }),
                    ]),
                    kind: "",
                },
                &parser.parse(),
            );
        }
    }
}
#[cfg(test)]
mod functions {
    use crate::parser::node::DataType::Signed32;

    use super::*;

    #[test]
    fn test_function() {
        let line = "fn name[](){let x = {return 5;};}";
        let mut parser = Parser::new(line);
        let info = NodeDetails::new(0, 0, TokenType::EOL);
        assert_node(
            &Node::Program {
                statements: Vec::from([Box::new(Node::FunctionDecl {
                    decl: BoundVar::new("name", None, info),
                    compile_params: vec![],
                    runtime_params: vec![],
                    block: Box::new(Node::Block {
                        statements: vec![Box::new(Node::LetDecl {
                            decl: BoundVar::new("x", None, info),
                            value: Box::new(Node::Block {
                                is_body: false,
                                depth: None,
                                statements: Vec::from([Box::new(Node::ReturnExpr {
                                    node: Box::new(Node::IntLit {
                                        v: 5,
                                        size: 4,
                                        ty: Signed32,
                                        info,
                                    }),
                                    info,
                                })]),
                                kind: "LocalScope",
                                info,
                            }),
                            kind: "LetDecl",
                            info,
                        })],
                        kind: "",
                        info,
                        depth: None,
                        is_body: true,
                    }),
                    kind: "",
                    info,
                })]),
                kind: "ProgramNode",
            },
            &parser.parse(),
        );
    }
    #[test]
    fn test_function2() {
        let line = "fn name(){let x = {return 5;};}";
        let mut parser = Parser::new(line);
        let info = NodeDetails::new(0, 0, TokenType::EOL);
        assert_node(
            &Node::Program {
                statements: Vec::from([Box::new(Node::FunctionDecl {
                    decl: BoundVar::new("name", None, info),
                    compile_params: vec![],
                    runtime_params: vec![],
                    block: Box::new(Node::Block {
                        statements: vec![Box::new(Node::LetDecl {
                            decl: BoundVar::new("x", None, info),
                            value: Box::new(Node::Block {
                                is_body: false,
                                depth: None,
                                statements: Vec::from([Box::new(Node::ReturnExpr {
                                    node: Box::new(Node::IntLit {
                                        v: 5,
                                        size: 4,
                                        ty: Signed32,
                                        info,
                                    }),
                                    info,
                                })]),
                                kind: "LocalScope",
                                info,
                            }),
                            kind: "LetDecl",
                            info,
                        })],
                        kind: "",
                        info,
                        depth: None,
                        is_body: true,
                    }),
                    kind: "",
                    info,
                })]),
                kind: "ProgramNode",
            },
            &parser.parse(),
        );
    }
    #[test]
    #[should_panic]
    fn test_function3() {
        let line = "fn name{let x = {return 5;};}";
        let mut parser = Parser::new(line);
        let info = NodeDetails::new(0, 0, TokenType::EOL);
        assert_node(
            &Node::Program {
                statements: Vec::from([Box::new(Node::FunctionDecl {
                    decl: BoundVar::new("name", None, info),
                    compile_params: vec![],
                    runtime_params: vec![],
                    block: Box::new(Node::Block {
                        statements: vec![Box::new(Node::LetDecl {
                            decl: BoundVar::new("x", None, info),
                            value: Box::new(Node::Block {
                                is_body: false,
                                depth: None,
                                statements: Vec::from([Box::new(Node::ReturnExpr {
                                    node: Box::new(Node::IntLit {
                                        v: 5,
                                        size: 4,
                                        ty: Signed32,
                                        info,
                                    }),
                                    info,
                                })]),
                                kind: "LocalScope",
                                info,
                            }),
                            kind: "LetDecl",
                            info,
                        })],
                        kind: "",
                        info,
                        depth: None,
                        is_body: true,
                    }),
                    kind: "",
                    info,
                })]),
                kind: "ProgramNode",
            },
            &parser.parse(),
        );
    }

    #[test]
    fn test_function_call() {
        let line = "name()";
        let mut parser = Parser::new(line);
        let info = NodeDetails::new(0, 0, TokenType::EOL);
        assert_node(
            &Node::Program {
                statements: Vec::from([Box::new(Node::CallExpr {
                    ident: Box::new(Node::IdentExpr {
                        name: "name",
                        r: None,
                        ty: None,
                        info,
                    }),
                    compile_args: vec![],
                    runtime_args: vec![],
                    kind: "",
                    info,
                    r: None,
                })]),
                kind: "ProgramNode",
            },
            &parser.parse(),
        );
    }
    #[test]
    fn test_function_call2() {
        let line = "name[]()";
        let mut parser = Parser::new(line);
        let info = NodeDetails::new(0, 0, TokenType::EOL);
        assert_node(
            &Node::Program {
                statements: Vec::from([Box::new(Node::CallExpr {
                    ident: Box::new(Node::IdentExpr {
                        name: "name",
                        r: None,
                        ty: None,
                        info,
                    }),
                    compile_args: vec![],
                    runtime_args: vec![],
                    kind: "",
                    info,
                    r: None,
                })]),
                kind: "ProgramNode",
            },
            &parser.parse(),
        );
    }
    #[test]
    #[should_panic]
    fn test_function_call3() {
        let line = "name[)";
        let mut parser = Parser::new(line);
        let info = NodeDetails::new(0, 0, TokenType::EOL);
        assert_node(
            &Node::Program {
                statements: Vec::from([Box::new(Node::CallExpr {
                    ident: Box::new(Node::IdentExpr {
                        name: "name",
                        r: None,
                        ty: None,
                        info,
                    }),
                    compile_args: vec![],
                    runtime_args: vec![],
                    kind: "",
                    info,
                    r: None,
                })]),
                kind: "ProgramNode",
            },
            &parser.parse(),
        );
    }
    #[test]
    fn test_function_call4() {
        let line = "name[1, 2, 3](1, 2, 3)";
        let mut parser = Parser::new(line);
        let info = NodeDetails::new(0, 0, TokenType::EOL);
        assert_node(
            &Node::Program {
                statements: Vec::from([Box::new(Node::CallExpr {
                    ident: Box::new(Node::IdentExpr {
                        name: "name",
                        r: None,
                        ty: None,
                        info,
                    }),
                    compile_args: vec![
                        Box::new(Node::IntLit {
                            v: 1,
                            size: 4,
                            ty: Signed32,
                            info,
                        }),
                        Box::new(Node::IntLit {
                            v: 2,
                            size: 4,
                            ty: Signed32,
                            info,
                        }),
                        Box::new(Node::IntLit {
                            v: 3,
                            size: 4,
                            ty: Signed32,
                            info,
                        }),
                    ],
                    runtime_args: vec![
                        Box::new(Node::IntLit {
                            v: 1,
                            size: 4,
                            ty: Signed32,
                            info,
                        }),
                        Box::new(Node::IntLit {
                            v: 2,
                            size: 4,
                            ty: Signed32,
                            info,
                        }),
                        Box::new(Node::IntLit {
                            v: 3,
                            size: 4,
                            ty: Signed32,
                            info,
                        }),
                    ],
                    kind: "",
                    info,
                    r: None,
                })]),
                kind: "ProgramNode",
            },
            &parser.parse(),
        );
    }
}
