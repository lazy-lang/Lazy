
use super::*;
pub use errors::*;

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