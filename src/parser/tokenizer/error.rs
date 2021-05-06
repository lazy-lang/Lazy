
use super::*;
use colored::*;

pub enum ErrorType {
    EndOfStr,
    DecimalPoint,
    InvalidCharacter(char),
    ExpectedFound(String, String),
    Expected(String),
    ExpectedDelimiter(char),
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
    pub fn format(&self, source: &[&str]) -> String {
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

}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.e_type {
            ErrorType::EndOfStr => write!(f, "Expected end of string"),
            ErrorType::DecimalPoint =>  write!(f, "Numbers cannot contain more than one decimal point"),
            ErrorType::ProperProperty =>  write!(f, "Expected a property name"),
            ErrorType::InvalidCharacter(character) =>  write!(f, "Invalid character {}", character),
            ErrorType::UnexpectedOp(op) =>  write!(f, "Unexpected operator {}", op),
            ErrorType::UnexpectedPunc(punc) =>  write!(f, "Unexpected punctuation {}", punc),
            ErrorType::Semicolon =>  write!(f, "Expected semicolon at the end of the expression"),
            ErrorType::EndOfBlock =>  write!(f, "Expected end of block"),
            ErrorType::Expected(val) =>  write!(f, "Expected {}", val),
            ErrorType::ExpectedFound(val, found) =>  write!(f, "Expected {}, but found {}", val, found),
            ErrorType::StartOfBlock =>  write!(f, "Expected start of block"),
            ErrorType::ArrowAccess =>  write!(f, "Arrow access cannot be chained"),
            ErrorType::ExpectedDelimiter(val) =>  write!(f, "Expected delimiter {}", val),
            ErrorType::Custom(msg) =>  write!(f, "{}", msg.to_string())
        }
    }
}