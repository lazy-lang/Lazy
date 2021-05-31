use std::fmt;

use super::*;

#[derive(PartialEq)]
#[derive(Clone)]
pub enum TokenType {
    Str(String),
    Float(f32),
    Int(i32),
    Kw(String),
    Bool(bool),
    Var(String),
    Op(String),
    Char(char),
    Punc(char),
    None
}

#[derive(PartialEq)]
pub enum NumberType {
    Binary, // 0b
    Octal, // 0o
    Hex, // 0x
    None
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Str(string) => write!(f, "string {}", string),
            Self::Float(num) => write!(f, "float {}", num),
            Self::Int(num) => write!(f, "integer {}", num),
            Self::Kw(kw) => write!(f, "keyword {}", kw),
            Self::Bool(bo) => write!(f, "boolean {}", bo),
            Self::Var(name) => write!(f, "identifier {}", name),
            Self::Op(op) => write!(f, "operator {}", op),
            Self::Punc(punc) => write!(f, "punctuation {}", punc),
            Self::Char(ch) => write!(f, "char {}", ch),
            Self::None => write!(f, "none")
        }
    }
}

#[derive(Copy)]
pub struct Range {
    pub start: LoC,
    pub end: LoC
}

impl std::clone::Clone for Range {
    fn clone(&self) -> Self {
        *self
    }
}

impl fmt::Display for Range {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}:{} - {}:{})", self.start.line, self.start.col, self.end.line, self.end.col)
    }
}

impl Range {
    #[inline]
    pub fn err(&self, err: ErrorType, tokens: &mut dyn Tokenizer) {
        tokens.error(err, self.start, self.end)
    }

    #[inline]
    pub fn err_start(&self, err: ErrorType, tokens: &mut dyn Tokenizer) {
        tokens.error(err, self.start, tokens.loc())
    }

    #[inline]
    pub fn end(&self, tokens: &dyn Tokenizer) -> Range {
        Range {
            start: self.start,
            end: tokens.loc()
        }
    }
}

pub struct Token {
    pub range: Range,
    pub val: TokenType
}


pub trait Tokenizer {
    fn consume(&mut self) -> Option<Token>;
    fn peek(&mut self) -> Option<&Token>;
    fn error(&mut self, e_type: ErrorType, start: LoC, end: LoC);
    fn error_here(&mut self, e_type: ErrorType);
    fn loc(&self) -> LoC;
    fn last_loc(&self) -> LoC;
    fn is_eof(&self) -> bool;
    fn set_allow_numbers_after_dot(&mut self, toggle: bool);
    fn recorder(&self) -> RangeRecorder;

    fn is_next(&mut self, tok: TokenType) -> bool {
        match self.peek() {
            Some(token) => token.val == tok,
            None => false
        }
    }

    fn skip_or_err(&mut self, tok: TokenType, err: Option<ErrorType>, _loc: Option<Range>) -> bool {
        let location = Range { start: LoC::default(), end: LoC::default() };
        match self.peek() {
            Some(token) => {
                if token.val != tok {
                    let other = token.val.to_string();
                    self.error(err.unwrap_or_else(|| ErrorType::ExpectedFound(tok.to_string(), other)), location.start, location.end);
                    true
                } else {
                    self.consume();
                    false
                }
            },
            None => {
                self.error(err.unwrap_or_else(|| ErrorType::Expected(tok.to_string())), location.start, location.end);
                true
            }
        }
    }

    fn expect_punc(&mut self, puncs: &[char], _loc: Option<Range>) -> Option<char> {
        let location = Range { start: LoC::default(), end: LoC::default() };
        match self.peek() {
            Some(tok) => {
                match tok.val {
                    TokenType::Punc(punc) => {
                        if puncs.contains(&punc) { 
                            self.consume();
                            return Some(punc);
                         };
                        self.error(ErrorType::ExpectedFound(format!("one of {}", puncs.iter().map(|i| format!("({})", i.to_string())).collect::<Vec<_>>().join(", ")), punc.to_string()), location.start, location.end);
                        None
                    },
                    _ => {
                        let tstr = tok.val.to_string();
                        self.error(ErrorType::ExpectedFound(format!("one of {}", puncs.iter().map(|i| format!("({})", i.to_string())).collect::<Vec<_>>().join(", ")), tstr), location.start, location.end);
                        None
                    }
                }
            },
            None => {
                self.error(ErrorType::Expected(format!("one of {}", puncs.iter().map(|i| format!("({})", i.to_string())).collect::<Vec<_>>().join(", "))), location.start, location.end);
                None
            }
        }
    }
}