use crate::{analyzer::Analyzer, parser::Parser};

mod analyzer;
mod lexer;
pub mod parser;
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
    let line = "let x = {return 5 + 4;};
    x = 5;
    let y = x;
    {
        let y = x + 5;
        y;
    }
    const z = {y};
    ";

    let mut parser = Parser::new(line);
    let node = parser.parse();
    println!("Node: {:#?}", node);
    let mut analyzer = Analyzer::new();
    analyzer.scope_check(&Box::new(node));

    // let mut toknizer = Tokenizer::new(line);
    // println!("token: {:#?}", toknizer.next_token());
    // println!("token: {:#?}", toknizer.next_token());
    // println!("token: {:#?}", toknizer.next_token());
    // println!("token: {:#?}", toknizer.next_token());
    // println!("token: {:#?}", toknizer.next_token());
    // println!("token: {:#?}", toknizer.next_token());
}
