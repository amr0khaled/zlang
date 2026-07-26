use std::{cell::RefCell, mem, rc::Rc};

use crate::analyzer::symbol::{Symbol, SymbolTable};
#[derive(Debug)]
pub struct Scope<'a> {
    node: Rc<RefCell<SymbolTable<'a>>>,
    parent: Option<Box<Scope<'a>>>,
}

impl<'a> Scope<'a> {
    pub fn base(node: Rc<RefCell<SymbolTable<'a>>>) -> Self {
        Self { node, parent: None }
    }
    pub fn new(node: Rc<RefCell<SymbolTable<'a>>>, parent: Box<Scope<'a>>) -> Self {
        Self {
            node,
            parent: Some(parent),
        }
    }
    pub fn set_parent(&mut self, parent: Box<Scope<'a>>) {
        self.parent = Some(parent);
    }
    pub fn has_parent(&self) -> bool {
        self.parent.is_some()
    }
    pub fn get(&self) -> Rc<RefCell<SymbolTable<'a>>> {
        self.node.clone()
    }
    pub fn lookup(&self, id: &u32) -> Option<Rc<RefCell<Symbol<'a>>>> {
        let table = self.node.borrow_mut();
        match table.get(&id) {
            Some(symbol) => Some(symbol.clone()),
            None => match &self.parent {
                Some(parent) => parent.lookup(id),
                None => None,
            },
        }
    }
}

pub struct ScopeList<'a> {
    head: Box<Scope<'a>>,
}

impl<'a> ScopeList<'a> {
    pub fn new(global: Box<Scope<'a>>) -> Self {
        Self { head: global }
    }
    pub fn push(&mut self, scope: Box<Scope<'a>>) {
        let old_head = mem::replace(&mut self.head, scope);
        self.head.set_parent(old_head);
    }
    pub fn pop(&mut self) -> Option<Box<Scope<'a>>> {
        if let Some(parent) = self.head.parent.take() {
            Some(mem::replace(&mut self.head, parent))
        } else {
            None
        }
    }
    pub fn get(&mut self) -> &Box<Scope<'a>> {
        &self.head
    }
}
