use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    analyzer::{
        scope::{
            Scope, ScopeList,
            ScopeRes::{self},
        },
        symbol::{Symbol, SymbolTable, SymbolType},
    },
    dpanic, dprintln,
    parser::{
        node::{DataType, Node, SymbolRef},
        ops::Operator,
    },
};
mod scope;
pub mod symbol;

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

    pub fn scope_check(&mut self, node: &mut Box<Node<'a>>) -> Option<SymbolType> {
        match &mut **node {
            Node::Program { statements, .. } => {
                let mut ty: Option<SymbolType> = None;
                for stmt in statements {
                    ty = self.scope_check(stmt);
                }
                let scope = self.scopes.get();
                // TODO: Make it more dynamic but with some flags
                let entrypoint_name = "main";
                let ep_id = gen_ident_id(entrypoint_name);
                match scope.lookup(&ep_id) {
                    Some(ScopeRes::Local(_)) => {}
                    _ => {
                        #[cfg(not(test))]
                        panic!("Cannot find entrypoint function '{}'", entrypoint_name);
                    }
                }
                ty
            }
            Node::Block {
                statements,
                depth,
                is_body,
                ..
            } => {
                let mut ty: Option<SymbolType> = None;
                let table: Rc<RefCell<SymbolTable<'a>>>;
                if !*is_body {
                    table = Rc::new(RefCell::new(HashMap::new()));
                    self.tables.push(table.clone());
                    let local_scope = Box::new(Scope::base(table));
                    *depth = Some(self.tables.len() - 1);
                    self.scopes.push(local_scope);
                } else {
                    *depth = Some(self.tables.len() - 1);
                }
                for stmt in statements {
                    ty = self.scope_check(stmt);
                }
                self.scopes.pop();
                self.tables.pop();
                ty
            }
            Node::LetDecl {
                decl, info, value, ..
            } => {
                let immutable = false;
                let ident = decl.ident;
                let id = gen_ident_id(ident);
                let ty = self.scope_check(value);
                let decl_ty = SymbolType::from_datatype(decl.ty);
                if ty.is_some() && decl_ty.is_some() && ty != decl_ty {
                    panic!(
                        "Both '{:#?}' and '{:#?}' should be the same type",
                        ty, decl.ty
                    )
                }
                let depth = self.tables.len() - 1;
                let scope = self.scopes.get();
                match scope.lookup(&id) {
                    Some(ScopeRes::Local(_)) => {
                        dpanic!(info, "Redeclaration of '{:?}'", decl)
                    }
                    _ => {
                        self.scopes.declare(&ident, ty, depth, immutable);
                    }
                }
                None
            }
            Node::ConstDecl {
                decl, info, value, ..
            } => {
                let ident = decl.ident;
                let id = gen_ident_id(ident);
                let ty = &self.scope_check(value);
                let decl_ty = SymbolType::from_datatype(decl.ty);
                if ty.is_none() || !ty.as_ref().unwrap().clone().is_initialized() {
                    panic!("Cannot declare constant '{}' without initial value", ident)
                }
                if *ty != decl_ty {
                    panic!(
                        "Both '{:#?}' and '{:#?}' should be the same type",
                        ty, decl.ty
                    )
                }
                let scopes = &mut self.scopes;
                let depth = self.tables.len() - 1;
                let scope = scopes.get();
                match scope.lookup(&id) {
                    Some(ScopeRes::Local(_)) => {
                        dpanic!(info, "Redeclaration of '{}'", ident);
                    }
                    _ => {
                        scopes.declare(ident, ty.clone(), depth, true);
                    }
                }
                None
            }
            Node::IdentExpr { name, info, r, ty } => {
                let scope = self.scopes.get();
                let id = gen_ident_id(name);
                match scope.lookup(&id) {
                    Some(ScopeRes::Local(a)) | Some(ScopeRes::Outer(a)) => {
                        let sym = a.borrow();
                        dprintln!("Found: {:#?}", sym);
                        *r = Some(SymbolRef::new(id, sym.depth().clone()));
                        let _ty = sym.get_type();
                        *ty = DataType::from_symbol(_ty.clone());
                        _ty.clone()
                    }
                    None => dpanic!(info, "Undeclared Variable '{}'", name), //panic!(" on line {}", name, info.line),
                }
            }
            // TODO: Rewrite this
            Node::IntLit { ty, .. } => SymbolType::from_datatype(Some(*ty)),
            Node::FloatLit { ty, .. } => SymbolType::from_datatype(Some(*ty)),
            Node::BooleanLit { .. } => Some(SymbolType::Boolean),
            Node::Dynamic => Some(SymbolType::Uninitialized),
            Node::AssignStmt {
                ident,
                value,
                info,
                r,
                op,
                ..
            } => {
                let name: &str;
                if let Node::IdentExpr { name: _name, .. } = &**ident {
                    name = _name;
                } else {
                    panic!("Unexpected Error: expected identifier got '{:?}'", ident)
                }
                let id = gen_ident_id(name);

                let current_depth = self.tables.len() - 1;

                let lookup_res = {
                    let scope = self.scopes.get();
                    match scope.lookup(&id) {
                        Some(ScopeRes::Local(a)) => {
                            let v = a.borrow();
                            Some((true, v.is_const(), v.depth().clone(), v.get_type().clone()))
                        }
                        Some(ScopeRes::Outer(a)) => {
                            let v = a.borrow();
                            Some((false, v.is_const(), v.depth().clone(), v.get_type().clone()))
                        }
                        None => None,
                    }
                };

                match lookup_res {
                    Some((local, is_const, decl_depth, expected_ty)) => {
                        if op == &Operator::AssignNew && local {
                            panic!("Cannot declare an existing variable '{}'", name);
                        }
                        if is_const {
                            panic!("Cannot reassign a constant '{}' value", name);
                        }

                        let rhs_ty = self.scope_check(value);
                        if rhs_ty != expected_ty {
                            panic!(
                                "The new value {:?} should have the same type {:?} as the variable",
                                rhs_ty, expected_ty
                            );
                        }

                        *r = Some(SymbolRef::new(id, decl_depth));
                    }
                    None => {
                        if op == &Operator::AssignNew {
                            let new_ty = self.scope_check(value);

                            self.scopes.declare(name, new_ty, current_depth, false);

                            *r = Some(SymbolRef::new(id, current_depth));
                        } else {
                            dpanic!(info, "Undeclared Variable '{}'", name);
                        }
                    }
                }
                None
            }
            Node::ReturnExpr { node, .. } => self.scope_check(node),

            Node::UnaryExpr {
                node, info, ty, op, ..
            } => {
                let mut node_mut = node;
                match self.scope_check(&mut node_mut) {
                    // TODO: Check the types
                    Some(x) if x.is_literal() => {
                        *ty = DataType::from_symbol(Some(x.clone()));
                        if *op == Operator::Not {
                            Some(SymbolType::Boolean)
                        } else {
                            Some(x)
                        }
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
                op,
                ..
            } => {
                let left_ty = match self.scope_check(left) {
                    Some(x) if x.is_literal() => {
                        *ty = DataType::from_symbol(Some(x.clone()));
                        x
                    }
                    y => panic!(
                        "This type '{:#?}' is not supported in BinaryExpr\n: Error: {:#?}",
                        y, info
                    ),
                };
                let right_ty = match self.scope_check(right) {
                    Some(x) if x.is_literal() => x,
                    y => panic!(
                        "This type '{:#?}' is not supported in BinaryExpr\n: Error: {:#?}",
                        if y.is_some() { y } else { None },
                        info
                    ),
                };
                if op.is_logical_operator() && !left_ty.is_bool() && !right_ty.is_bool() {
                    dpanic!(
                        info,
                        "Both '{:#?}' and '{:#?}' should be boolean",
                        left,
                        right
                    );
                }
                if op.is_compare_operator() {
                    return Some(SymbolType::Boolean);
                }
                if right_ty != left_ty {
                    panic!(
                        "Both '{:#?}' and '{:#?}' should be the same type",
                        left, right
                    )
                }
                Some(left_ty)
            }
            Node::DataType(dt) => SymbolType::from_datatype(Some(*dt)),
            // ### Function
            Node::FunctionDecl {
                decl,
                compile_params,
                runtime_params,
                block,
                info,
                ..
            } => {
                let id = gen_ident_id(decl.ident);
                let scope = self.scopes.get();
                match scope.lookup(&id) {
                    Some(ScopeRes::Local(_)) => {
                        dpanic!(info, "Redeclaration of '{:?}' function", decl.ident)
                    }
                    _ => {
                        let table = Rc::new(RefCell::new(HashMap::new()));
                        self.tables.push(table.clone());
                        let depth = self.tables.len();
                        let mut _compiler_params = vec![];
                        let mut _runtime_params = vec![];
                        for param in compile_params {
                            let ty = SymbolType::from_datatype(param.ty).unwrap_or_else(|| {
                                dpanic!(
                                    info,
                                    "Either type wasn't explicitly type or unsupported type"
                                )
                            });
                            _compiler_params.push(ty.clone());
                            let sym = Symbol::new(param.ident, Some(ty), depth);

                            table
                                .borrow_mut()
                                .insert(sym.get_id().clone(), Rc::new(RefCell::new(sym)));
                        }
                        for param in runtime_params {
                            let ty = SymbolType::from_datatype(param.ty).unwrap_or_else(|| {
                                dpanic!(
                                    info,
                                    "Either type wasn't explicitly type or unsupported type"
                                )
                            });
                            _runtime_params.push(ty.clone());
                            let sym = Symbol::new(param.ident, Some(ty), depth);

                            table
                                .borrow_mut()
                                .insert(sym.get_id().clone(), Rc::new(RefCell::new(sym)));
                        }
                        let mut ret_ty = None;
                        if decl.ty.is_some() {
                            ret_ty = SymbolType::from_datatype(decl.ty);
                        }
                        let ty = SymbolType::Fn {
                            compile_params: _compiler_params,
                            runtime_params: _runtime_params,
                            ret_ty: Box::new(ret_ty.clone()),
                        };
                        self.scopes
                            .declare(&decl.ident, Some(ty.clone()), depth - 1, true);

                        let local_scope = Box::new(Scope::base(table.clone()));
                        self.scopes.push(local_scope);
                        let actual_ret_ty = match self.scope_check(block) {
                            Some(x) if x.is_literal() => Some(x),
                            None => None,
                            y => panic!(
                                "This type '{:#?}' is not supported as Function return type\n: Error: {:#?}",
                                y.unwrap(),
                                info
                            ),
                        };
                        if ret_ty.clone() != actual_ret_ty {
                            panic!(
                                "Function return type '{:#?}' should match actual returned type '{:#?}'",
                                ret_ty, actual_ret_ty
                            )
                        }
                        Some(ty)
                    }
                }
            }
            Node::CallExpr {
                ident,
                compile_args,
                runtime_args,

                info,
                r,
                ..
            } => {
                let name: &str;
                if let Node::IdentExpr { name: _name, .. } = &**ident {
                    name = _name;
                } else {
                    dpanic!(
                        info,
                        "Unexpected Error: expected identifier got '{:?}'",
                        ident
                    )
                }
                let id = gen_ident_id(name);
                let scope = self.scopes.get();
                match scope.lookup(&id) {
                    Some(ScopeRes::Local(a)) | Some(ScopeRes::Outer(a)) => {
                        let sym = a.borrow();
                        let depth = self.tables.len() - 1;
                        *r = Some(SymbolRef::new(id, depth));
                        let SymbolType::Fn {
                            compile_params,
                            runtime_params,
                            ret_ty,
                        } = sym.get_type().clone().unwrap()
                        else {
                            dpanic!(info, "'{}' is not a function to be called", name)
                        };
                        if compile_params.len() != compile_args.len() {
                            dpanic!(
                                info,
                                "Compile-time arguments should match paramters in length"
                            );
                        } else if runtime_params.len() != runtime_args.len() {
                            dpanic!(info, "Runtime arguments should match paramters in length");
                        }
                        for i in 0..compile_args.len() {
                            let arg = &mut compile_args[i];
                            let arg_ty = self.scope_check(arg);
                            let param_ty = Some(compile_params[i].clone());
                            if arg_ty != param_ty {
                                dpanic!(
                                    info,
                                    "Compile-time argument '${}' type should match paramter ${}",
                                    i,
                                    i
                                );
                            }
                        }
                        for i in 0..runtime_args.len() {
                            let arg = &mut runtime_args[i];
                            let arg_ty = self.scope_check(arg);
                            let param_ty = Some(runtime_params[i].clone());
                            if arg_ty != param_ty {
                                dpanic!(
                                    info,
                                    "Runtime argument '${}' type should match paramter ${}",
                                    i,
                                    i
                                );
                            }
                        }
                        *ret_ty
                    }
                    None => dpanic!(info, "Undeclared function '{}'", name),
                }
            }
            Node::Empty => None,
            //_ => SymbolType::Void,
        }
    }
}

#[cfg(test)]
#[deny(dead_code)]
mod tests;
