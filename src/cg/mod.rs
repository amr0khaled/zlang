use std::{cell::RefCell, rc::Rc};

use crate::{
    analyzer::symbol::{Symbol, SymbolTable},
    cg::opcode::{
        OpCode::{self, *},
        Sym,
    },
    parser::{node::Node, ops::Operator},
};
mod asm;
mod cpu;
mod opcode;

pub struct Memory<'a> {
    opcodes: Vec<OpCode>,
    tables: Vec<Rc<RefCell<SymbolTable<'a>>>>,
}

impl<'a> Memory<'a> {
    pub fn new(tables: Vec<Rc<RefCell<SymbolTable<'a>>>>) -> Self {
        Self {
            opcodes: vec![],
            tables,
        }
    }
    pub fn get_opcodes(&self) -> &Vec<OpCode> {
        &self.opcodes
    }
    fn push(&mut self, op: OpCode) {
        self.opcodes.push(op);
    }
    fn pop(&mut self) -> Option<OpCode> {
        self.opcodes.pop()
    }
    fn peek(&mut self) -> &OpCode {
        &self.opcodes[self.opcodes.len() - 1]
    }
    fn peekn(&mut self, n: usize) -> &OpCode {
        &self.opcodes[self.opcodes.len() - 1 - n]
    }
    pub fn emit(&mut self, node: &Node) {
        match node {
            Node::Ref(_, _) => todo!(),
            Node::IntLit { v, .. } => {
                self.push_uint32(*v as u32);
            }
            Node::FloatLit { v, .. } => {
                self.push_float64(*v as f64);
            }
            Node::BooleanLit { v, .. } => {
                self.push_bool(*v);
            }
            Node::Dynamic => todo!(),
            Node::IdentExpr { r, .. } => {
                if let Some(n) = r {
                    self.load(Sym::from(n));
                } else {
                    panic!("What happened? in IdentExpr")
                }
            }
            Node::UnaryExpr { op, node, ty, .. } => {
                self.emit(node);
                // let op = OpCode::operator_to_opcode(op, ty);
                // self.push(op);
            }
            Node::BinaryExpr {
                op,
                left,
                right,
                ty,
                ..
            } => {
                self.emit(left);
                self.emit(right);
                // let op = OpCode::operator_to_opcode(op, &ty);
                // self.push(op);
            }
            Node::AssignStmt { op, value, r, .. } => {
                if let Some(a) = r {
                    let sym = Sym::from(a);
                    if op == &Operator::AssignNew || op == &Operator::Assign {
                        self.emit(value);
                        self.store(sym);
                        return;
                    }
                    if let Some(v) = self.lookup(sym.id, sym.depth) {
                        let ty = v.borrow().get_type();
                        self.load(sym.clone());
                        self.emit(value);
                        let assign_op = OpCode::assign_to_opcode(op, ty);
                        if assign_op != OpCode::Nop {
                            self.push(assign_op);
                        }
                        self.store(sym);
                        return;
                    }
                    panic!("Symbol not found")
                }
                panic!("Unknown Reference")
            }
            Node::CallExpr { .. } => todo!("Call expr"),
            Node::LetDecl { value, r, .. } => {
                self.emit(value);
                println!("{:#?}", r);
                if let Some(a) = r {
                    self.store(Sym::from(a));
                }
            }
            Node::ConstDecl { value, r, .. } => {
                self.emit(value);
                if let Some(a) = r {
                    self.store(Sym::from(a));
                }
            }
            Node::ExprStmt { node, .. } => {
                self.emit(node);
            }
            Node::ReturnExpr { node, .. } => {
                self.emit(node);
                self.push(Return);
            }
            Node::Block {
                statements,
                is_body,
                depth,
                ..
            } => {
                if *is_body {
                    self.push(Push);
                } else if let Some(d) = depth {
                    self.push(ScopeStart(d.clone()));
                }
                for stmt in statements {
                    self.emit(stmt);
                }
                if *is_body {
                    self.push(Pop);
                } else if let Some(d) = depth {
                    if self.peek() == &OpCode::Return {
                        self.pop();
                    }
                    self.push(ScopeEnd(d.clone()));
                }
            }
            Node::Program { statements, .. } => {
                for stmt in statements {
                    self.emit(stmt);
                }
            }
            Node::Empty => todo!(),
            _ => todo!(),
        }
    }
    fn lookup(&self, id: u32, depth: usize) -> Option<Rc<RefCell<Symbol<'a>>>> {
        self.tables[depth].borrow_mut().get(&id).cloned()
    }
    pub fn get_instructions(&self) -> &Vec<OpCode> {
        &self.opcodes
    }
}

trait Ident {
    fn store(&mut self, sym: Sym);
    fn load(&mut self, sym: Sym);
    fn call(&mut self, sym: Sym);
}
trait Literal {
    // uint
    fn push_uint8(&mut self, v: u8);
    fn push_uint16(&mut self, v: u16);
    fn push_uint32(&mut self, v: u32);
    fn push_uint64(&mut self, v: u64);
    fn push_uint128(&mut self, v: u128);
    // int
    fn push_int8(&mut self, v: i8);
    fn push_int16(&mut self, v: i16);
    fn push_int32(&mut self, v: i32);
    fn push_int64(&mut self, v: i64);
    fn push_int128(&mut self, v: i128);
    // float
    fn push_float32(&mut self, v: f32);
    fn push_float64(&mut self, v: f64);
    // bool
    fn push_bool(&mut self, v: bool);
}

impl<'a> Ident for Memory<'a> {
    fn store(&mut self, sym: Sym) {
        self.push(Store(sym));
    }

    fn load(&mut self, sym: Sym) {
        self.push(Load(sym));
    }

    fn call(&mut self, sym: Sym) {
        self.push(Call(sym));
    }
}

impl<'a> Literal for Memory<'a> {
    fn push_uint8(&mut self, v: u8) {
        self.push(PushUint8(v));
    }

    fn push_uint16(&mut self, v: u16) {
        self.push(PushUint16(v));
    }

    fn push_uint32(&mut self, v: u32) {
        self.push(PushUint32(v));
    }

    fn push_uint64(&mut self, v: u64) {
        self.push(PushUint64(v));
    }

    fn push_uint128(&mut self, v: u128) {
        self.push(PushUint128(v));
    }

    fn push_int8(&mut self, v: i8) {
        self.push(PushInt8(v));
    }

    fn push_int16(&mut self, v: i16) {
        self.push(PushInt16(v));
    }

    fn push_int32(&mut self, v: i32) {
        self.push(PushInt32(v));
    }

    fn push_int64(&mut self, v: i64) {
        self.push(PushInt64(v));
    }

    fn push_int128(&mut self, v: i128) {
        self.push(PushInt128(v));
    }

    fn push_float32(&mut self, v: f32) {
        self.push(PushFloat32(v));
    }

    fn push_float64(&mut self, v: f64) {
        self.push(PushFloat64(v));
    }

    fn push_bool(&mut self, v: bool) {
        self.push(PushBool(v));
    }
}

#[cfg(test)]
mod tests {
    #![cfg(any())]
    use crate::{
        analyzer::{Analyzer, gen_ident_id},
        cg::{Memory, OpCode, opcode::Sym},
        parser::{Parser, node::Node},
    };

    fn exec_memory(code: &'static str) -> (Box<Node<'static>>, Memory<'static>) {
        let mut parser: Parser<'static> = Parser::new(code);
        let mut node = Box::new(parser.parse());
        let mut analyzer = Analyzer::new();
        analyzer.scope_check(&mut node);
        (node, Memory::new(analyzer.tables))
    }

    #[test]
    fn test_push() {
        let code: &'static str = "5";
        let (node, mut memory) = exec_memory(code);
        memory.emit(&node);
        assert_eq!(Vec::from([OpCode::PushUint32(5)]), *memory.get_opcodes());
    }
    #[test]
    fn test_push2() {
        let code: &'static str = "5.0";
        let (node, mut memory) = exec_memory(code);
        memory.emit(&node);
        assert_eq!(Vec::from([OpCode::PushFloat64(5.0)]), *memory.get_opcodes());
    }
    #[test]
    fn test_expr() {
        let code: &'static str = "5 + 4";
        let (node, mut memory) = exec_memory(code);
        memory.emit(&node);
        assert_eq!(
            Vec::from([OpCode::PushUint32(5), OpCode::PushUint32(4), OpCode::Add]),
            *memory.get_opcodes()
        );
    }
    #[test]
    #[should_panic]
    fn test_expr2() {
        let code: &'static str = "5 + 4.5";
        let (node, mut memory) = exec_memory(code);
        memory.emit(&node);
    }
    #[test]
    fn test_let() {
        let code: &'static str = "let x = 5 + 4";
        let (node, mut memory) = exec_memory(code);
        memory.emit(&node);
        assert_eq!(
            Vec::from([
                OpCode::PushUint32(5),
                OpCode::PushUint32(4),
                OpCode::Add,
                OpCode::Store(Sym {
                    id: gen_ident_id("x"),
                    depth: 0
                })
            ]),
            *memory.get_opcodes()
        );
    }
    #[test]
    fn test_const() {
        let code: &'static str = "const x = 5 + 4";
        let (node, mut memory) = exec_memory(code);
        memory.emit(&node);
        assert_eq!(
            Vec::from([
                OpCode::PushUint32(5),
                OpCode::PushUint32(4),
                OpCode::Add,
                OpCode::Store(Sym {
                    id: gen_ident_id("x"),
                    depth: 0
                })
            ]),
            *memory.get_opcodes()
        );
    }
    #[test]
    fn test_block() {
        let code: &'static str = "const x = {5 + 4}";
        let (node, mut memory) = exec_memory(code);
        memory.emit(&node);
        assert_eq!(
            Vec::from([
                OpCode::ScopeStart(1),
                OpCode::PushUint32(5),
                OpCode::PushUint32(4),
                OpCode::Add,
                OpCode::ScopeEnd(1),
                OpCode::Store(Sym {
                    id: gen_ident_id("x"),
                    depth: 0
                })
            ]),
            *memory.get_opcodes()
        );
    }
    #[test]
    fn test_assign() {
        // TODO: Modifiy Parser to accept "let x = 5; x += 4"
        let code: &'static str = "let x = 5;
        x += 4";
        let (node, mut memory) = exec_memory(code);
        memory.emit(&node);
        assert_eq!(
            Vec::from([
                OpCode::PushUint32(5),
                OpCode::Store(Sym {
                    id: gen_ident_id("x"),
                    depth: 0
                }),
                OpCode::Load(Sym {
                    id: gen_ident_id("x"),
                    depth: 0
                }),
                OpCode::PushUint32(4),
                OpCode::Add,
                OpCode::Store(Sym {
                    id: gen_ident_id("x"),
                    depth: 0
                })
            ]),
            *memory.get_opcodes()
        );
    }
    #[test]
    fn test_assign2() {
        let code: &'static str = "let x = 5;
        x /= 4";
        let (node, mut memory) = exec_memory(code);
        memory.emit(&node);
        assert_eq!(
            Vec::from([
                OpCode::PushUint32(5),
                OpCode::Store(Sym {
                    id: gen_ident_id("x"),
                    depth: 0
                }),
                OpCode::Load(Sym {
                    id: gen_ident_id("x"),
                    depth: 0
                }),
                OpCode::PushUint32(4),
                OpCode::Div,
                OpCode::Store(Sym {
                    id: gen_ident_id("x"),
                    depth: 0
                })
            ]),
            *memory.get_opcodes()
        );
    }
    #[test]
    fn test_assign_new() {
        let code: &'static str = "x <- 5;
        x /= 4";
        let (node, mut memory) = exec_memory(code);
        memory.emit(&node);
        assert_eq!(
            Vec::from([
                OpCode::PushUint32(5),
                OpCode::Store(Sym {
                    id: gen_ident_id("x"),
                    depth: 0
                }),
                OpCode::Load(Sym {
                    id: gen_ident_id("x"),
                    depth: 0
                }),
                OpCode::PushUint32(4),
                OpCode::Div,
                OpCode::Store(Sym {
                    id: gen_ident_id("x"),
                    depth: 0
                })
            ]),
            *memory.get_opcodes()
        );
    }
    #[test]
    fn test_assign3() {
        let code: &'static str = "x <- 5;
        x ||= 4";
        let (node, mut memory) = exec_memory(code);
        memory.emit(&node);
        assert_eq!(
            Vec::from([
                OpCode::PushUint32(5),
                OpCode::Store(Sym {
                    id: gen_ident_id("x"),
                    depth: 0
                }),
                OpCode::Load(Sym {
                    id: gen_ident_id("x"),
                    depth: 0
                }),
                OpCode::PushUint32(4),
                OpCode::Or,
                OpCode::Store(Sym {
                    id: gen_ident_id("x"),
                    depth: 0
                })
            ]),
            *memory.get_opcodes()
        );
    }
}
