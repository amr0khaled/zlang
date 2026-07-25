use crate::{lexer::Tokenizer, parser::Parser};

mod lexer;
mod parser;
pub mod util;

// TODO: Fix it
// fn syscall<>(num: u64, arg1: A1, arg2: A2, arg3: A3) -> String {
//     let mut _asm = String::new();
//     _asm += &format!("mov   rdi, {}\n", arg1);
//     _asm += &format!("mov   rsi, {}\n", arg2);
//     _asm += &format!("mov   rdx, {}\n", arg3);
//     _asm
// }

fn main() {
    let line = "let x = {5 + 4}; x = y; y = (1 ** 2); let z = 5;";

    let mut parser = Parser::new(line);
    let node = parser.parse();

    println!("Program: {:#?}", node);

    // let mut toknizer = Tokenizer::new(line);
    // println!("token: {:#?}", toknizer.next_token());
    // println!("token: {:#?}", toknizer.next_token());
    // println!("token: {:#?}", toknizer.next_token());
    // println!("token: {:#?}", toknizer.next_token());
    // println!("token: {:#?}", toknizer.next_token());
    // println!("token: {:#?}", toknizer.next_token());
}
