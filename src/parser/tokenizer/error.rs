
use super::*;
use colored::*;

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
    pub fn format(&self, source: &Vec<&str>) -> String {
        // Multi-line errors
        if self.range.start.line != self.range.end.line {
            let mut line = String::new();
            let end_line = self.range.end.line;
            for x in self.range.start.line..=end_line {
                let id = x as usize - 1;
                line.push_str(&format!("{} {}{} {}\n", x, " ".repeat(end_line.to_string().len() - x.to_string().len()), &"|".cyan(),source[id]));
                if x == self.range.start.line {
                    let mut cols = String::new();
                    cols.push_str(&format!("{} {}", " ".repeat(end_line.to_string().len()), &"|".cyan()));
                    for col in 0..=source[id].len() as i32 {
                        if col >= self.range.start.col { cols.push_str(&format!("{}", "^".red())); }
                        else { cols.push(' '); }
                    }
                    cols.push('\n');
                    line.push_str(&cols);
                }
                if x == self.range.end.line {
                    let mut cols = String::new();
                    cols.push_str(&format!("{} {}", " ".repeat(end_line.to_string().len()), &"|".cyan()));
                    for col in 0..=source[id].len() as i32 {
                        if col >= self.range.end.col { cols.push_str(&format!("{}", "^".red())); }
                        else { cols.push(' '); }
                    }
                    cols.push('\n');
                    line.push_str(&cols);
                }
            }
            return format!("\n{}\n\n{} {}", line, self.to_string().red(), self.range);
        };
        let mut col = String::new();
        let start_line = self.range.start.line as usize;
        col.push_str(&" ".repeat(start_line.to_string().len() + 3));
        for x in 0..=self.range.end.col {
            if x >= self.range.start.col { col.push_str(&format!("{}", "^".red())); }
            else { col.push(' '); };
        };
        format!("\n{} {} {}\n\n{}\n{} {}", start_line, &"|".cyan(), source[start_line - 1], col, self.to_string().red(), self.range)
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

/*
pub fn damage_control(tokenizer: &mut Tokenizer, err: &Error) {
    match err.e_type {
        ErrorType::Semicolon
    }
}
*/