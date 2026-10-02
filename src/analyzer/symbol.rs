use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{analyzer::gen_ident_id, lexer::types::TokenType};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SymbolType {
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
    Uninitialized,
    Void,
}

impl SymbolType {
    pub fn is_literal(&self) -> bool {
        match self {
            SymbolType::Uninitialized | SymbolType::Void => false,
            _ => true,
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
    is_const: bool,
    ty: SymbolType,
    depth: usize,
}

impl<'a> Symbol<'a> {
    pub fn new(name: &'a str, ty: SymbolType, depth: usize) -> Self {
        let id = gen_ident_id(name);
        Self {
            id,
            name,
            is_const: false,
            ty,
            depth,
        }
    }
    pub fn new_const(name: &'a str, ty: SymbolType, depth: usize) -> Self {
        let id = gen_ident_id(name);
        Self {
            id,
            name,
            is_const: true,
            ty,
            depth,
        }
    }
    pub fn get_type(&self) -> SymbolType {
        self.ty
    }
    pub fn is_const(&self) -> bool {
        self.is_const
    }
    pub fn get_id(&self) -> u32 {
        self.id
    }
    pub fn depth(&self) -> usize {
        self.depth
    }
}
