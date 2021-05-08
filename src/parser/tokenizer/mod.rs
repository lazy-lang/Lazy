
use std::fmt;
pub mod error;
use error::*;
use super::input_parser::{LoC, InputParser};

#[derive(PartialEq)]
pub enum TokenType {
    Str(String),
    Float(f32),
    Int(i32),
    Kw(String),
    Bool(bool),
    Var(String),
    Op(String),
    Punc(char)
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
            Self::Punc(punc) => write!(f, "punctuation {}", punc)
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


pub struct Token {
    pub range: Range,
    pub val: TokenType
}


pub struct Tokenizer<'a> {
    keywords: Vec<&'a str>,
    operators: Vec<char>,
    standalone_operators: Vec<char>,
    current: Option<Token>,
    pub errors: Vec<Error>,
    pub input: InputParser
}

impl<'a> Tokenizer<'a> {

    pub fn new(code: &'a str) -> Self {
        Tokenizer {
            keywords: vec!["main", "let", "emit", "match", "while", "if", "actor", "enum", "struct", "true", "false", "on", "single", "f"],
            operators: vec!['+', '-', '>', '<', '=', '!', '%', '|', '&', '.', '?'],
            standalone_operators: vec!['?', '>'], // Operators which cannot be combined, but other separate operators can follow them
            current: None,
            errors: vec![],
            input: InputParser::new(code)
        }
    }

    fn parse_str(&mut self) -> Token {
        self.input.consume(); // Consume the starting "
        let start = self.input.loc();
        let mut str = String::new();
        loop {
            match self.input.consume() {
                Some(character) => {
                    if character == '"' { break; };
                    str.push(character);
                },
                None => {
                    let loc = self.input.loc();
                    self.error(ErrorType::EndOfStr, start, loc);
                    break;
                }
            }
        };
        Token { val: TokenType::Str(str), range: Range {start, end: self.input.loc()} }
    }

    fn parse_num(&mut self) -> Token {
        let mut dot = false;
        let mut num = String::new();
        let start = self.input.loc();
        while !self.input.is_eof() {
            match self.input.peek(0) {
            Some(ch) => {
            match ch {
                '0'..='9' => num.push(self.input.consume().unwrap()),
                '.' => {
                    self.input.consume();
                    if dot {
                        let loc = self.input.loc();
                        self.error(ErrorType::DecimalPoint, start, loc); 
                        break;
                     };
                    dot = true;
                    num.push(ch);
                },
                '_' => {
                    self.input.consume();
                    continue;
                },
                _ => break
            }
        },
        None => break
        }
        };

        let token_type = if dot { TokenType::Float(num.parse().unwrap()) } else {TokenType::Int(num.parse().unwrap()) };
        Token { val: token_type, range: Range {start, end: self.input.loc()} }
    }

    fn parse_ident(&mut self) -> Token {
        let mut ident = String::new();
        let start = self.input.loc();
        while !self.input.is_eof() {
            match self.input.peek(0) {
                Some(ch) => {
                    match ch {
                        'a'..='z' | 'A'..='Z' | '0'..='9' | '_' => ident.push(self.input.consume().unwrap()),
                        _ => break
                    }
                },
                None => break
            }
        };
        if ident == "true" { return Token { val: TokenType::Bool(true), range: Range {start, end: self.input.loc()} } }
        else if ident == "false" { return Token { val: TokenType::Bool(false), range: Range {start, end: self.input.loc()} } }
        let token_type = if self.keywords.iter().any(|&i| i == ident) { TokenType::Kw(ident) } else { TokenType::Var(ident) };
        Token { val: token_type, range: Range {start, end: self.input.loc()} }
    }

    fn parse_punc(&mut self) -> Token {
        let range = Range { start: self.input.loc(), end: self.input.loc() };
        Token { val: TokenType::Punc(self.input.consume().unwrap()), range }
    }

    fn parse_op(&mut self) -> Token {
        let start = self.input.loc();
        let mut op = String::new();
        while !self.input.is_eof() {
            if self.standalone_operators.iter().any(|&i| i.to_string() == op) { break; };
            let ch = self.input.peek(0).unwrap();
            if self.operators.iter().any(|&i| i == ch) { op.push(self.input.consume().unwrap()) }
            else { break; };
        };
        Token {val: TokenType::Op(op), range: Range {start, end: self.input.loc()}}
    }

    fn _next(&mut self) -> Option<Token> {
        if self.input.is_eof() { return None; };
        let tok = self.input.peek(0)?;
        if tok == '/' && self.input.peek(1)? == '/' {
            self.input.consume();
            self.input.consume();
            while !self.input.is_eof() {
                if self.input.consume()? == '\n' { break; };
            }
            return self._next();
        }
        if tok == '/' && self.input.peek(1)? == '*' {
            self.input.consume();
            while !self.input.is_eof() {
                if self.input.consume()? == '*' && self.input.peek(0)? =='/' { break; };
            }
            self.input.consume();
            return self._next();
        }
        match tok {
            '"' => Some(self.parse_str()),
            '0'..='9' => Some(self.parse_num()),
            ' ' | '\n' | '\t' => {
                self.input.consume();
                self._next()
            },
            '+' | '-' | '>' | '<' | '=' | '!' | '%' | '|' | '&' | '.' | '?' => Some(self.parse_op()),
            ',' | ':' | ';' | '{' | '}' | '[' | ']' | '(' | ')' => Some(self.parse_punc()),
            'a'..='z' | 'A'..='Z' | '_' => Some(self.parse_ident()),
            ch => {
                let loc = self.input.loc();
                self.error(ErrorType::InvalidCharacter(ch), loc, loc);
                self.input.consume();
                None
            } 
        }
    }

    pub fn consume(&mut self) -> Option<Token> {
        if self.current.is_some() {
            self.current.take()
        } else {
            self._next()
        }
    }

    pub fn peek(&mut self) -> Option<&Token> {
        if self.current.is_some() {
            return self.current.as_ref();
        }
        self.current = self._next();
        self.current.as_ref()
    }

    #[inline]
    pub fn error(&mut self, e_type: ErrorType, start: LoC, end: LoC) {
        self.errors.push(Error { e_type, range: Range {start, end} });
    }

    pub fn is_next(&mut self, tok: TokenType) -> bool {
        let next = self.peek();
        match next {
            Some(token) => token.val == tok,
            None => false
        }
    }

    pub fn skip_or_err(&mut self, tok: TokenType, err: Option<ErrorType>, loc: Option<Range>) -> bool {
        let location = loc.unwrap_or(Range { start: self.input.loc(), end: self.input.loc()});
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

    pub fn expect_punc(&mut self, puncs: &[char], loc: Option<Range>) -> Option<char> {
        let location = loc.unwrap_or(Range { start: self.input.loc(), end: self.input.loc()});
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
                        self.error(ErrorType::Expected(format!("one of {}", puncs.iter().map(|i| format!("({})", i.to_string())).collect::<Vec<_>>().join(", "))), location.start, location.end);
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