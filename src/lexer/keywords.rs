use std::any::Any;

use crate::lexer::{TokenEnum, TokenType};
#[derive(Debug)]
pub struct Keyword(&'static str);
impl<'a> TokenType<'a> for Keyword {
    fn get(&'a self) -> TokenEnum<'a> {
        TokenEnum::KeywordType(self)
    }
}

pub static LET: Keyword = Keyword("let");
pub static CONST: Keyword = Keyword("const");
pub static INTEGER: Keyword = Keyword("int");
