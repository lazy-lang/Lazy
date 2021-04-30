

pub struct InputParser {
    code: Vec<char>,
    pub line: i32,
    pub col: i32,
    pos: usize
}

pub struct LoC {
    pub pos: usize,
    pub line: i32,
    pub col: i32
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

    pub fn is_eof(&self) -> bool {
        self.pos >= self.code.len()
    }

    pub fn loc(&self) -> LoC {
        LoC { pos: self.pos, line: self.line, col: self.col }
    }
    
}