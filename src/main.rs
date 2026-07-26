use std::env;

use crate::{
    analyzer::Analyzer,
    parser::Parser,
    util::{overwrite_file, read_file},
};

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
    let args: Vec<String> = env::args().collect();
    let line = "let x = {return 5 + 4;};
    x = 5;
    let y = x;
    {
        let y = x + 5;
        y;
    }
    const z = {y};
    ";
    let filename = args[1].as_str();
    let content = read_file(filename);

    let mut parser = Parser::new(&content);
    let node = parser.parse();
    let node_struct = format!("Node: {:#?}", node);
    let mut analyzer = Analyzer::new();
    analyzer.scope_check(&Box::new(node));

    let file_splits = filename.split('.');
    let path = String::from(
        file_splits
            .clone()
            .take(file_splits.count() - 1)
            .collect::<Vec<&str>>()
            .concat(),
    );
    let mut name = String::from(".");
    name.push_str(&path);
    name.push_str(".z.node");
    match overwrite_file(&name, node_struct) {
        Ok(()) => {}
        Err(e) => panic!("FS Error: {:?}", e),
    }

    // let mut toknizer = Tokenizer::new(line);
    // println!("token: {:#?}", toknizer.next_token());
    // println!("token: {:#?}", toknizer.next_token());
    // println!("token: {:#?}", toknizer.next_token());
    // println!("token: {:#?}", toknizer.next_token());
    // println!("token: {:#?}", toknizer.next_token());
    // println!("token: {:#?}", toknizer.next_token());
}
