use std::fs;
use std::io::Write;

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
#[macro_export]
macro_rules! iterative_num {
    (
        $(#[$meta:meta])*
        pub enum $name:ident : $underlying:ty {
            $($variant:ident),* $(,)?
        }
    ) => {
        $(#[$meta])*
        #[repr($underlying)]
        #[derive(Copy, Clone)]
        pub enum $name {
            $($variant,)*
        }

        impl std::convert::TryFrom<$underlying> for $name {
            type Error = ();

            fn try_from(v: $underlying) -> Result<Self, Self::Error> {
                // Loop through all variants and check against Rust's auto-assigned values
                $(
                    if v == $name::$variant as $underlying {
                        return Ok($name::$variant);
                    }
                )*
                Err(())
            }
        }
    };
}

pub fn read_file(filename: &str) -> String {
    fs::read_to_string(filename).expect("Cannot find this file")
}

fn write_file(overwrite: bool, filename: &str, content: String) -> Result<(), std::io::Error> {
    let file = fs::File::options()
        .create(true)
        .append(!overwrite)
        .write(overwrite)
        .truncate(overwrite)
        .open(filename);
    match file {
        Ok(mut f) => write!(f, "{}", content),
        Err(e) => Err(e),
    }
}

pub fn overwrite_file(filename: &str, content: String) -> Result<(), std::io::Error> {
    write_file(true, filename, content)
}

pub fn append_file(filename: &str, content: String) -> Result<(), std::io::Error> {
    write_file(false, filename, content)
}
