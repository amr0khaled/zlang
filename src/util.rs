#[derive(Debug)]
pub struct Iter<'a> {
    src: &'a str,
    pub current: usize,
    len: usize,
}

impl<'a> Iter<'a> {
    pub const fn new(src: &'a str) -> Self {
        Self {
            src,
            current: 0,
            len: src.len(),
        }
    }
    pub fn consume(&mut self) -> char {
        let ch = self.current();
        self.current += ch.len_utf8();
        ch
    }
    pub fn advance_by(&mut self, n: usize) {
        self.current += n;
    }
    fn get_char(&mut self, n: usize) -> char {
        if self.current >= self.len {
            return '\0';
        }
        let byte = self.src.as_bytes()[n];
        if byte.is_ascii() {
            byte as char
        } else {
            let src = &self.src[n..];
            let ch = src.chars().next().unwrap();
            ch
        }
    }
    pub fn current(&mut self) -> char {
        self.get_char(self.current)
    }
    pub fn peek(&mut self) -> char {
        let cur = self.current().len_utf8();
        self.get_char(self.current + cur)
    }
    pub fn peekn(&mut self, i: usize) -> char {
        self.get_char(self.current + i)
    }
}
