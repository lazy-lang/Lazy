
use super::*;

pub enum ErrorType {
    EndOfStr,
    DecimalPoint,
    InvalidCharacter(char),
    ExpectedFound(String, String),
    Expected(String),
    ProperProperty,
    ArrowAccess,
    StartOfBlock,
    EndOfBlock,
    Semicolon,
    UnexpectedOp(String),
    UnexpectedPunc(char),
    Custom(String)
}

pub struct Error {
    pub range: Range,
    pub e_type: ErrorType
}

impl Error {
    pub fn format(&self, source: &str) -> String {
        if self.range.start.line != self.range.end.line {
            let mut line = String::new();
            let lines: Vec<&str> = source.split('\n').collect();
            for x in self.range.start.line..=self.range.end.line {
                let id = x as usize - 1;
                line.push_str(&format!("{} | {}\n", x, lines[id]));
                if x == self.range.start.line {
                    let mut cols = String::new();
                    cols.push_str(&" ".repeat(id.to_string().len() + 3));
                    for col in 0..=lines[id].len() as i32 {
                        if col >= self.range.start.col { cols.push('^'); }
                        else { cols.push(' '); }
                    }
                    cols.push('\n');
                    line.push_str(&cols);
                }
                if x == self.range.end.line {
                    let mut cols = String::new();
                    cols.push_str(&" ".repeat(id.to_string().len() + 3));
                    for col in 0..=lines[id].len() as i32 {
                        if col >= self.range.end.col { cols.push('^'); }
                        else { cols.push(' '); }
                    }
                    cols.push('\n');
                    line.push_str(&cols);
                }
            }
            return format!("\n{}\n\nError: {} {}", line, self.to_string(), self.range);
        };
        let mut col = String::new();
        let start_line = self.range.start.line as usize;
        col.push_str(&" ".repeat(start_line.to_string().len() + 3));
        for x in 0..=self.range.end.col {
            if x >= self.range.start.col { col.push('^'); }
            else { col.push(' '); };
        };
        let line = source.split('\n').nth(start_line - 1).unwrap();
        format!("\n{} | {}\n\n{}\nError: {} {}", start_line, line, col, self.to_string(), self.range)
    }

    pub fn to_string(&self) -> String {
        match &self.e_type {
            ErrorType::EndOfStr => String::from("Expected end of string"),
            ErrorType::DecimalPoint => String::from("Numbers cannot contain more than one decimal point"),
            ErrorType::ProperProperty => String::from("Expected a property name"),
            ErrorType::InvalidCharacter(character) => format!("Invalid character {}", character),
            ErrorType::UnexpectedOp(op) => format!("Unexpected operator {}", op),
            ErrorType::UnexpectedPunc(punc) => format!("Unexpected punctuation {}", punc),
            ErrorType::Semicolon => String::from("Expected semicolon at the end of the expression"),
            ErrorType::EndOfBlock => String::from("Expected end of block"),
            ErrorType::Expected(val) => format!("Expected {}", val),
            ErrorType::ExpectedFound(val, found) => format!("Expected {}, but found {}", val, found),
            ErrorType::StartOfBlock => String::from("Expected start of block"),
            ErrorType::ArrowAccess => String::from("Arrow access cannot be chained"),
            ErrorType::Custom(msg) => msg.to_string()
        }
    }
}