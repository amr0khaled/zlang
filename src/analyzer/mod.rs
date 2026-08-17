use core::panic;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    analyzer::{
        scope::{
            Scope, ScopeList,
            ScopeRes::{self},
        },
        symbol::{Symbol, SymbolTable, SymbolType},
    },
    parser::{node::Node, ops::Operator},
};
mod scope;
pub mod symbol;

#[macro_export]
macro_rules! log {
    ($inf:ident, $($arg:tt)*) => {{
        println!("{} <- {:#?}", $inf.log(), format_args!($($arg)*));
    }};
}
#[macro_export]
macro_rules! panic_log {
    ($inf:ident, $($arg:tt)*) => {{
        panic!("{} <- {:#?}", $inf.log(), format_args!($($arg)*));
    }};
}

pub fn gen_ident_id(ident: &str) -> u32 {
    let id: u32 = 0xbeef;
    let mut whole_chunk: u32 = 0;
    for (i, &chunk) in ident.as_bytes().iter().enumerate() {
        match i % 4_usize {
            0 => {
                whole_chunk = chunk as u32;
            }
            1 => {
                whole_chunk |= (chunk as u32) << 8_usize;
            }
            2 => {
                whole_chunk |= (chunk as u32) << 16_usize;
            }
            3 => {
                whole_chunk |= (chunk as u32) << 24_usize;
            }
            _ => panic!("Something wrong happened"),
        }
    }
    id ^ !(whole_chunk)
}

pub struct Analyzer<'a> {
    scopes: ScopeList<'a>,
    pub tables: Vec<Rc<RefCell<SymbolTable<'a>>>>,
}
impl<'a> Analyzer<'a> {
    pub fn new() -> Self {
        let mut tables: Vec<Rc<RefCell<SymbolTable<'a>>>> = Vec::new();
        let global_table = Rc::new(RefCell::new(HashMap::new()));
        tables.push(global_table.clone());

        let global_scope = Scope::base(global_table.clone());
        let scopes = ScopeList::new(Box::new(global_scope));
        Self { scopes, tables }
    }

    pub fn scope_check(&mut self, node: &mut Box<Node<'a>>) -> SymbolType {
        match &mut **node {
            Node::Program { statements, .. } => {
                let mut ty: SymbolType = SymbolType::Void;
                for stmt in statements {
                    ty = self.scope_check(stmt);
                }
                ty
            }
            Node::Block {
                statements, depth, ..
            } => {
                let mut ty: SymbolType = SymbolType::Void;
                let table: Rc<RefCell<SymbolTable<'a>>> = Rc::new(RefCell::new(HashMap::new()));
                self.tables.push(table.clone());
                *depth = Some(self.tables.len() - 1);
                let local_scope = Box::new(Scope::base(table));
                self.scopes.push(local_scope);
                for stmt in statements {
                    ty = self.scope_check(stmt);
                }
                self.scopes.pop();
                self.tables.pop();
                ty
            }
            Node::VarDeclStmt {
                ident,
                info,
                value,
                r,
                ..
            } => {
                let id = gen_ident_id(ident);
                let ty = self.scope_check(value);
                let depth = self.tables.len() - 1;
                let scope = self.scopes.get();
                match scope.lookup(&id) {
                    Some(ScopeRes::Local(_)) => {
                        panic_log!(info, "Redeclaration of '{}'", ident)
                    }
                    _ => {
                        let symbol = Symbol::new(ident, ty, depth);
                        scope
                            .get()
                            .borrow_mut()
                            .insert(id, Rc::new(RefCell::new(symbol)));
                        *r = Some(Box::new(Node::Ref(id, depth)));
                    }
                }
                SymbolType::Void
            }
            Node::ConstDeclStmt {
                ident,
                info,
                value,
                r,
                ..
            } => {
                let id = gen_ident_id(ident);
                let ty = self.scope_check(value);
                if !ty.is_initialized() {
                    panic!("Cannot declare constant '{}' without initial value", ident)
                }
                let scopes = &mut self.scopes;
                let depth = self.tables.len() - 1;
                let scope = scopes.get();
                match scope.lookup(&id) {
                    Some(ScopeRes::Local(_)) => {
                        panic_log!(info, "Redeclaration of '{}'", ident);
                        // println!(" on line {}", ident, info.line)
                    }
                    _ => {
                        let symbol = Symbol::new_const(ident, ty, depth);
                        scope
                            .get()
                            .borrow_mut()
                            .insert(id, Rc::new(RefCell::new(symbol)));
                        *r = Some(Box::new(Node::Ref(id, depth)))
                    }
                }
                SymbolType::Void
            }
            Node::IdentExpr { name, info, r, ty } => {
                let scope = self.scopes.get();
                let id = gen_ident_id(name);
                match scope.lookup(&id) {
                    Some(ScopeRes::Local(a)) | Some(ScopeRes::Outer(a)) => {
                        let sym = a.borrow();
                        println!("Found: {:#?}", sym);
                        *r = Some(Box::new(Node::Ref(id, sym.depth().clone())));
                        let _ty = sym.get_type();
                        *ty = Some(_ty.clone());
                        _ty
                    }
                    None => panic_log!(info, "Undeclared Variable '{}'", name), //panic!(" on line {}", name, info.line),
                }
            }
            Node::IntegerLit { .. } => SymbolType::Integer,
            Node::FloatLit { .. } => SymbolType::Float,
            Node::BooleanLit { .. } => SymbolType::Boolean,
            Node::Dynamic => SymbolType::Uninitialized,
            Node::AssignStmt {
                ident,
                value,
                info,
                r,
                op,
                ..
            } => {
                let id = gen_ident_id(ident);
                let current_depth = self.tables.len() - 1;

                let lookup_res = {
                    let scope = self.scopes.get();
                    match scope.lookup(&id) {
                        Some(ScopeRes::Local(a)) => {
                            let v = a.borrow();
                            Some((true, v.is_const(), v.depth().clone(), v.get_type()))
                        }
                        Some(ScopeRes::Outer(a)) => {
                            let v = a.borrow();
                            Some((false, v.is_const(), v.depth().clone(), v.get_type()))
                        }
                        None => None,
                    }
                };

                match lookup_res {
                    Some((local, is_const, decl_depth, expected_ty)) => {
                        if op == &Operator::AssignNew && local {
                            panic!("Cannot declare an existing variable '{}'", ident);
                        }
                        if is_const {
                            panic!("Cannot reassign a constant '{}' value", ident);
                        }

                        let rhs_ty = self.scope_check(value);
                        if rhs_ty != expected_ty {
                            panic!(
                                "The new value {:?} should have the same type {:?} as the variable",
                                rhs_ty, expected_ty
                            );
                        }

                        *r = Some(Box::new(Node::Ref(id, decl_depth)));
                    }
                    None => {
                        if op == &Operator::AssignNew {
                            let new_ty = self.scope_check(value);

                            // Insert into symbol table
                            let symbol = Symbol::new(ident, new_ty, current_depth);
                            self.scopes
                                .get()
                                .get()
                                .borrow_mut()
                                .insert(id, Rc::new(RefCell::new(symbol)));

                            *r = Some(Box::new(Node::Ref(id, current_depth)));
                        } else {
                            panic_log!(info, "Undeclared Variable '{}'", ident);
                        }
                    }
                }

                SymbolType::Void
            }
            Node::ReturnExpr { node, .. } => self.scope_check(node),

            Node::UnaryExpr { node, info, ty, .. } => {
                let mut node_mut = node;
                match self.scope_check(&mut node_mut) {
                    x if x == SymbolType::Integer
                        || x == SymbolType::Float
                        || x == SymbolType::Boolean =>
                    {
                        *ty = Some(x);
                        x
                    }
                    y => panic!(
                        "This type '{:#?}' is not supported in UnaryExpr\n: Error: {:#?}",
                        y, info
                    ),
                }
            }
            Node::BinaryExpr {
                left,
                right,
                info,
                ty,
                ..
            } => {
                let left_ty = match self.scope_check(left) {
                    x if x == SymbolType::Integer || x == SymbolType::Float => {
                        *ty = Some(x);
                        x
                    }
                    y => panic!(
                        "This type '{:#?}' is not supported in BinaryExpr\n: Error: {:#?}",
                        y, info
                    ),
                };
                let right_ty = match self.scope_check(right) {
                    x if x == SymbolType::Integer || x == SymbolType::Float => x,
                    y => panic!(
                        "This type '{:#?}' is not supported in BinaryExpr\n: Error: {:#?}",
                        y, info
                    ),
                };
                if right_ty != left_ty {
                    panic!(
                        "Both '{:#?}' and '{:#?}' should be the same type",
                        left, right
                    )
                }
                left_ty
            }
            // Node::CallExpr {
            //     ident,
            //     args,
            //     kind,
            //     info,
            // } => ,
            // Node::ExprStmt { node, info } => todo!(),
            _ => SymbolType::Void,
        }
    }
}
