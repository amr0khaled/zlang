use crate::cg::opcode::OpCode;

pub struct OpCodeIter {
    opcodes: Vec<OpCode>,
    i: usize,
}

impl OpCodeIter {
    pub fn new() -> Self {
        Self {
            opcodes: vec![],
            i: 0,
        }
    }
    pub fn from(opcodes: Vec<OpCode>) -> Self {
        Self { opcodes, i: 0 }
    }
    pub fn current(&self) -> &OpCode {
        &self.opcodes[self.i]
    }
    pub fn peek(&self) -> &OpCode {
        &self.opcodes[self.i + 1]
    }
    pub fn peekn(&self, n: usize) -> &OpCode {
        &self.opcodes[self.i + n]
    }
    pub fn rpeek(&self) -> &OpCode {
        &self.opcodes[self.i - 1]
    }
    pub fn rpeekn(&self, n: usize) -> &OpCode {
        &self.opcodes[self.i - n]
    }
    pub fn next(&mut self) -> OpCode {
        let cur = *self.current();
        self.i += 1;
        cur
    }
    pub fn advance_by(&mut self, n: usize) {
        self.i += n;
    }
    pub fn index(&self) -> usize {
        self.i
    }
}

pub struct AsmGen {
    ins: Vec<String>,
    iter: OpCodeIter,
}

impl AsmGen {
    pub fn new() -> Self {
        Self {
            ins: vec![],
            iter: OpCodeIter::new(),
        }
    }
    fn emit(&mut self, line: String) {
        self.ins.push(line);
    }
    fn operands(&mut self, opcode: &str, oper1: &str, oper2: &str) {
        self.emit(format!("   {}  {}, {}", opcode, oper1, oper2));
    }
    fn operand(&mut self, opcode: &str, oper: &str) {
        self.emit(format!("   {}  {}", opcode, oper));
    }
    fn mov(&mut self, oper1: &str, oper2: &str) {
        self.operands("mov", oper1, oper2);
    }
    pub fn gen_asm(&mut self, opcodes: Vec<OpCode>) -> String {
        self.emit(".intel_syntax noprefix".to_string());
        self.emit(".global main".to_string());
        self.emit("main:".to_string());
        self.iter = OpCodeIter::from(opcodes);

        self.ins.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        analyzer::Analyzer,
        cg::{Memory, asm::AsmGen, opcode::OpCode},
        parser::{Parser, node::Node},
    };

    fn exec_opcodes(code: &'static str) -> Vec<OpCode> {
        let mut parser: Parser<'static> = Parser::new(code);
        let mut node = Box::new(parser.parse());
        let mut analyzer = Analyzer::new();
        analyzer.scope_check(&mut node);
        let mut memory = Memory::new(analyzer.tables);
        memory.emit(&node);
        let opcodes = memory.opcodes;
        opcodes
    }

    #[test]
    fn test_initial_header() {
        let code: &'static str = "";
        let opcodes = exec_opcodes(code);
        let mut asm_gen = AsmGen::new();
        let asm = asm_gen.gen_asm(opcodes);

        assert_eq!(
            String::from(
                ".intel_syntax noprefix
.global main
main:"
            ),
            asm
        )
    }
}
