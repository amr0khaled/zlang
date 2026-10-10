use std::env;

use crate::{
    analyzer::Analyzer,
    // cg::Memory,
    parser::Parser,
    util::{overwrite_file, read_file},
};

pub mod analyzer;
// mod cg;
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
    let filename = args[1].as_str();
    let content = read_file(filename);

    let mut parser = Parser::new(&content);
    let node = &mut Box::new(parser.parse());
    let mut analyzer = Analyzer::new();
    analyzer.scope_check(node);
    // let mut cg = Memory::new(analyzer.tables);
    // cg.emit(node);
    // let ins = cg.get_instructions();
    let node_struct = format!("Node: {:#?}", node);

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
    let mut node_file = name.clone();
    node_file.push_str(".z.node");
    let mut opcode_file = name.clone();
    opcode_file.push_str(".z.opcodes");
    match overwrite_file(&node_file, node_struct) {
        Ok(()) => {}
        Err(e) => panic!("FS Error: {:?}", e),
    }
    // match overwrite_file(&opcode_file, format!("{:#?}", ins)) {
    //     Ok(()) => {}
    //     Err(e) => panic!("FS Error: {:?}", e),
    // }

    // let mut toknizer = Tokenizer::new(line);
    // println!("token: {:#?}", toknizer.next_token());
    // println!("token: {:#?}", toknizer.next_token());
    // println!("token: {:#?}", toknizer.next_token());
    // println!("token: {:#?}", toknizer.next_token());
    // println!("token: {:#?}", toknizer.next_token());
    // println!("token: {:#?}", toknizer.next_token());
}
