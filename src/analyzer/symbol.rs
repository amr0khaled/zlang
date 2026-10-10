use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{analyzer::gen_ident_id, parser::node::DataType};

#[derive(Debug, Clone, PartialEq)]
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
    Fn {
        compile_params: Vec<SymbolType>,
        runtime_params: Vec<SymbolType>,
        ret_ty: Box<Option<SymbolType>>,
    },
}

impl SymbolType {
    pub fn is_literal(&self) -> bool {
        match self {
            SymbolType::Uninitialized => false,
            _ => true,
        }
    }
    pub fn is_bool(&self) -> bool {
        match self {
            SymbolType::Boolean => true,
            _ => false,
        }
    }
    pub fn is_initialized(&self) -> bool {
        match self {
            SymbolType::Uninitialized => false,
            _ => true,
        }
    }
    pub fn from_datatype(dt: Option<DataType>) -> Option<Self> {
        if dt.is_none() {
            return None;
        }
        match dt.unwrap() {
            DataType::Unsigned8 => Some(Self::Unsigned8),
            DataType::Signed8 => Some(Self::Signed8),
            DataType::Unsigned16 => Some(Self::Unsigned16),
            DataType::Signed16 => Some(Self::Signed16),
            DataType::Unsigned32 => Some(Self::Unsigned32),
            DataType::Signed32 => Some(Self::Signed32),
            DataType::Unsigned64 => Some(Self::Unsigned64),
            DataType::Signed64 => Some(Self::Signed64),
            DataType::Float32 => Some(Self::Float32),
            DataType::Float64 => Some(Self::Float64),
            DataType::Boolean => Some(Self::Boolean),
            DataType::Dynamic => Some(Self::Uninitialized),
            DataType::Void => None,
        }
    }
}
pub type SymbolTable<'a> = HashMap<u32, Rc<RefCell<Symbol<'a>>>>;

#[derive(Debug)]
#[allow(dead_code)]
pub struct Symbol<'a> {
    id: u32,
    name: &'a str,
    is_const: bool,
    ty: Option<SymbolType>,
    depth: usize,
}

impl<'a> Symbol<'a> {
    pub fn new(name: &'a str, ty: Option<SymbolType>, depth: usize) -> Self {
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
            ty: Some(ty),
            depth,
        }
    }
    pub fn get_type(&self) -> &Option<SymbolType> {
        &self.ty
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
