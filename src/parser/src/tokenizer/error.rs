
use super::*;
use colored::*;

pub enum ParserErrorType {
    EndOfStr,
    DecimalPoint,
    InvalidCharacter(char),
    ExpectedFound(String, String),
    Expected(&'static str),
    ExpectedString(String),
    ExpectedDelimiter(char),
    ProperProperty,
    ArrowAccess,
    StartOfBlock,
    EndOfBlock,
    Semicolon,
    EmptyCharLiteral,
    ConstantWithoutInit,
    NoGenerics,
    TooMuchTypes(i8),
    UnexpectedOp(String),
    UnexpectedPunc(char),
    Unexpected(&'static str),
    EndOfIterator,
    ManyEntryPoints,
    WrongMatchArmExp,
    AlreadyHasModifier(&'static str),
    Disallowed(&'static str),
    Custom(&'static str),
    Confusable(&'static str, &'static str),
    InvalidDigit,
    PointlessTemplate
}

impl ParserErrorType {
    pub fn expected_found<T: Into<String>, R: Into<String>>(a: T, b: R) -> ParserErrorType {
        Self::ExpectedFound(a.into(), b.into())
    }

}

pub trait ErrorCollector<T> where T: fmt::Display {
    fn error(&mut self, e_type: T, start: LoC, end: LoC);
}

pub struct Error<T> where T: fmt::Display {
    pub range: Range,
    pub e_type: T 
}

impl<T> Error<T> where T: fmt::Display {
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
            return format!("\n{}\n\n{} {}", line, self.e_type.to_string().red(), self.range);
        };
        let mut col = String::new();
        let start_line = self.range.start.line as usize;
        col.push_str(&" ".repeat(start_line.to_string().len() + 3));
        for x in 0..=self.range.end.col {
            if x >= self.range.start.col { col.push_str(&format!("{}", "^".red())); }
            else { col.push(' '); };
        };
        format!("\n{} {} {}\n\n{}\n{} {}", start_line, &"|".cyan(), source[start_line - 1], col, self.e_type.to_string().red(), self.range)
    }

}

impl fmt::Display for ParserErrorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Self::EndOfStr => write!(f, "Expected end of string"),
            Self::DecimalPoint =>  write!(f, "Numbers cannot contain more than one decimal point"),
            Self::ProperProperty =>  write!(f, "Expected a property name"),
            Self::InvalidCharacter(character) =>  write!(f, "Invalid character {}", character),
            Self::UnexpectedOp(op) =>  write!(f, "Unexpected operator {}", op),
            Self::UnexpectedPunc(punc) =>  write!(f, "Unexpected punctuation {}", punc),
            Self::Semicolon =>  write!(f, "Expected semicolon at the end of the expression"),
            Self::EndOfBlock =>  write!(f, "Expected end of block"),
            Self::Expected(val) =>  write!(f, "Expected {}", val),
            Self::ExpectedString(val) => write!(f, "Expected {}", val),
            Self::ExpectedFound(val, found) =>  write!(f, "Expected {}, but found {}", val, found),
            Self::StartOfBlock =>  write!(f, "Expected start of block"),
            Self::ArrowAccess =>  write!(f, "Arrow access cannot be chained"),
            Self::ExpectedDelimiter(val) =>  write!(f, "Expected delimiter {}", val),
            Self::Custom(msg) =>  write!(f, "{}", msg.to_string()),
            Self::Unexpected(msg) => write!(f, "Unexpected {}", msg.to_string()),
            Self::TooMuchTypes(amount) => write!(f, "Too much typings provided, expected only {}", amount),
            Self::EmptyCharLiteral => write!(f, "Empty char literal"),
            Self::ConstantWithoutInit => write!(f, "Constant variables must have an initializor"),
            Self::NoGenerics => write!(f, "Generics are not allowed here"),
            Self::EndOfIterator => write!(f, "Expected end of iterator"),
            Self::Disallowed(string) => write!(f, "{} is not allowed here", string),
            Self::ManyEntryPoints => write!(f, "Too many entry points"),
            Self::WrongMatchArmExp => write!(f, "Incorrect match arm expression. Match arms only accept enum variants or literals."),
            Self::AlreadyHasModifier(string) => write!(f, "The field is already {}, unnecessary {} modifier", string, string),
            Self::Confusable(confused_with, expected) => write!(f, "Found {}, which is similar to {}", confused_with, expected),
            Self::InvalidDigit => write!(f, "Invalid digit"),
            Self::PointlessTemplate => write!(f, "Pointless template literal")
        }
    }
}