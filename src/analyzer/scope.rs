use std::{
    cell::{Ref, RefCell},
    mem,
    rc::Rc,
    thread::scope,
};

use crate::analyzer::symbol::{Symbol, SymbolTable};
#[derive(Debug)]
pub struct Scope<'a> {
    node: Rc<RefCell<SymbolTable<'a>>>,
    parent: Option<Box<Scope<'a>>>,
    local: bool,
}

pub enum ScopeRes<'a> {
    Local(Rc<RefCell<Symbol<'a>>>),
    Outer(Rc<RefCell<Symbol<'a>>>),
}

impl<'a> Scope<'a> {
    pub fn base(node: Rc<RefCell<SymbolTable<'a>>>) -> Self {
        Self {
            node,
            parent: None,
            local: true,
        }
    }
    pub fn new(node: Rc<RefCell<SymbolTable<'a>>>, parent: Box<Scope<'a>>) -> Self {
        Self {
            node,
            parent: Some(parent),
            local: true,
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
    pub fn is_local(&self) -> bool {
        self.local
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
    pub fn count_live(&self) -> usize {
        let mut count = 1;
        let mut scope = &self.head;
        loop {
            match &scope.parent {
                Some(parent) => {
                    count += 1;
                    println!("Matches: {}", matches!(scope, parent));
                    scope = parent;
                }
                None => return count,
            }
        }
    }
}
