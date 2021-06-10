
use errors::{LoC};

pub struct InputParser {
    code: Vec<char>,
    pub line: usize,
    pub col: usize,
    pos: usize
}

impl InputParser {

    pub fn new(code: &str) -> Self {
        InputParser {
            line: 1,
            col: 0,
            pos: 0,
            code: code.chars().collect()
        }
    }

    pub fn consume(&mut self) -> Option<char> {
        if self.is_eof() { return None };
        let char = self.code[self.pos];
        self.pos += 1;
        if char == '\n' {
            self.line += 1;
            self.col = 0;
        } else { self.col += 1; };
        Some(char)
    }

    pub fn peek(&self, am: usize) -> Option<char> {
        if (self.pos + am) >= self.code.len() { return None; };
        Some(self.code[self.pos + am])
    }

    pub fn unpeek(&mut self, am: usize) {
        self.pos -= am;
    }

    pub fn is_eof(&self) -> bool {
        self.pos >= self.code.len()
    }

    pub fn skip_line(&mut self) {
        if self.is_eof() { return; };
        while self.code[self.pos] != '\n' {
            self.pos += 1;
        }
        self.pos += 1;
        self.col = 0;
        self.line += 1;
    }

    pub fn loc(&self) -> LoC {
        LoC { line: self.line, col: self.col, pos: self.pos }
    }
    
}