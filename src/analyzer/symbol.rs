use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::analyzer::gen_ident_id;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SymbolType {
    Integer,
    Float,
    Boolean,
    Uninitialized,
    Void,
}

impl SymbolType {
    pub fn is_literal(&self) -> bool {
        match self {
            SymbolType::Integer | SymbolType::Float | SymbolType::Boolean => true,
            _ => false,
        }
    }
    pub fn is_initialized(&self) -> bool {
        match self {
            SymbolType::Uninitialized | SymbolType::Void => false,
            _ => true,
        }
    }
}
pub type SymbolTable<'a> = HashMap<u32, Rc<RefCell<Symbol<'a>>>>;

#[derive(Debug)]
pub struct Symbol<'a> {
    id: u32,
    name: &'a str,
    pub is_const: bool,
    pub ty: SymbolType,
}

impl<'a> Symbol<'a> {
    pub fn new(name: &'a str, ty: SymbolType) -> Self {
        let id = gen_ident_id(name);
        Self {
            id,
            name,
            is_const: false,
            ty,
        }
    }
    pub fn new_const(name: &'a str, ty: SymbolType) -> Self {
        let id = gen_ident_id(name);
        Self {
            id,
            name,
            is_const: true,
            ty,
        }
    }
}
