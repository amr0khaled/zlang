use std::any::Any;

use crate::lexer::Tokenizer;

mod lexer;
mod util;

// TODO: Fix it
// fn syscall<>(num: u64, arg1: A1, arg2: A2, arg3: A3) -> String {
//     let mut _asm = String::new();
//     _asm += &format!("mov   rdi, {}\n", arg1);
//     _asm += &format!("mov   rsi, {}\n", arg2);
//     _asm += &format!("mov   rdx, {}\n", arg3);
//     _asm
// }

fn main() {
    let line = "ي <- 5.a;";

    let mut toknizer = Tokenizer::new(line);
    println!("token: {:#?}", toknizer.next_token());
    println!("token: {:#?}", toknizer.next_token());
    println!("token: {:#?}", toknizer.next_token());
    println!("token: {:#?}", toknizer.next_token());
    println!("token: {:#?}", toknizer.next_token());
    println!("token: {:#?}", toknizer.next_token());
}
