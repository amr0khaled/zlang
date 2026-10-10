use std::{cell::RefCell, mem, rc::Rc};

use crate::{
    analyzer::{
        gen_ident_id,
        symbol::{Symbol, SymbolTable, SymbolType},
    },
    parser::node::SymbolRef,
};
#[derive(Debug)]
pub struct Scope<'a> {
    node: Rc<RefCell<SymbolTable<'a>>>,
    parent: Option<Box<Scope<'a>>>,
}

pub enum ScopeRes<'a> {
    Local(Rc<RefCell<Symbol<'a>>>),
    Outer(Rc<RefCell<Symbol<'a>>>),
}

impl<'a> Scope<'a> {
    pub fn base(node: Rc<RefCell<SymbolTable<'a>>>) -> Self {
        Self { node, parent: None }
    }
    #[allow(dead_code)]
    pub fn new(node: Rc<RefCell<SymbolTable<'a>>>, parent: Box<Scope<'a>>) -> Self {
        Self {
            node,
            parent: Some(parent),
        }
    }
    pub fn set_parent(&mut self, parent: Box<Scope<'a>>) {
        self.parent = Some(parent);
    }
    pub fn get(&self) -> Rc<RefCell<SymbolTable<'a>>> {
        self.node.clone()
    }
    pub fn lookup(&mut self, id: &u32) -> Option<ScopeRes<'a>> {
        let table = self.node.borrow();
        if let Some(symbol) = table.get(&id) {
            return Some(ScopeRes::Local(symbol.clone()));
        }
        match &mut self.parent {
            Some(parent) => match parent.lookup(&id) {
                None => None,
                Some(ScopeRes::Local(symbol)) | Some(ScopeRes::Outer(symbol)) => {
                    Some(ScopeRes::Outer(symbol))
                }
            },
            None => None,
        }
    }
}

pub struct ScopeList<'a> {
    head: Box<Scope<'a>>,
    pub depth: usize,
}

impl<'a> ScopeList<'a> {
    pub fn new(global: Box<Scope<'a>>) -> Self {
        Self {
            head: global,
            depth: 0,
        }
    }
    pub fn push(&mut self, scope: Box<Scope<'a>>) {
        let old_head = mem::replace(&mut self.head, scope);
        self.depth += 1;
        self.head.set_parent(old_head);
    }
    pub fn pop(&mut self) -> Option<Box<Scope<'a>>> {
        if let Some(parent) = self.head.parent.take() {
            self.depth -= 1;
            Some(mem::replace(&mut self.head, parent))
        } else {
            None
        }
    }
    pub fn get(&mut self) -> &mut Box<Scope<'a>> {
        &mut self.head
    }
    pub fn declare(
        &mut self,
        name: &'a str,
        ty: Option<SymbolType>,
        depth: usize,
        is_const: bool,
    ) -> SymbolRef {
        let id = gen_ident_id(name);
        let sym;
        if is_const {
            sym = Symbol::new_const(
                name,
                ty.unwrap_or_else(|| panic!("Constant shall explicity have type")),
                depth,
            );
        } else {
            sym = Symbol::new(name, ty, depth);
        }
        self.get()
            .get()
            .borrow_mut()
            .insert(id, Rc::new(RefCell::new(sym)));

        SymbolRef::new(id, depth)
    }
    #[allow(dead_code)]
    pub fn count_live(&self) -> usize {
        let mut count = 1;
        let mut scope = &self.head;
        loop {
            match &scope.parent {
                Some(parent) => {
                    count += 1;
                    scope = parent;
                }
                None => return count,
            }
        }
    }
}
