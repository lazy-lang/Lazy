
use std::fmt;
use super::input_parser::{LoC, InputParser};

pub enum TokenType {
    Str(String),
    Float(f32),
    Int(i32),
    Kw(String),
    Var(String),
    Op(String),
    Punc(char)
}

pub struct Range {
    pub start: LoC,
    pub end: LoC
}

impl fmt::Display for Range {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}:{} - {}:{})", self.start.line, self.start.col, self.end.line, self.end.col)
    }
}

pub struct Error {
    pub range: Range,
    pub msg: String
}

impl Error {
    pub fn format(&self, source: &str) -> String {
        let mut col = String::new();
        for x in 0..=self.range.end.col {
            if x >= self.range.start.col { col.push('^'); }
            else { col.push(' '); };
        };
        let line = source.split('\n').nth(self.range.start.line as usize - 1);
        format!("{}\n\n{}\n{} {}", line.unwrap(), col, self.msg, self.range)
    }
}

pub struct Token {
    pub range: Range,
    pub val: TokenType
}

pub struct Tokenizer<'a> {
    keywords: Vec<&'a str>,
    operators: Vec<char>,
    current: Option<Token>,
    pub errors: Vec<Error>,
    pub input: InputParser
}

impl<'a> Tokenizer<'a> {

    pub fn new(code: &'a str) -> Self {
        Tokenizer {
            keywords: vec!["main", "let", "emit", "match", "while", "if", "actor", "enum", "struct"],
            operators: vec!['+', '-', '>', '<', '=', '!', '%', '|', '&'],
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
                    self.error(String::from("Expected end of string"), start, self.input.loc());
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
                        self.error(String::from("Numbers cannot contain more than one decimal point"), start, self.input.loc()); 
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
                        'a'..='z' | '0'..='9' | '_' => ident.push(self.input.consume().unwrap()),
                        _ => break
                    }
                },
                None => break
            }
        };
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
            let ch = self.input.peek(0).unwrap();
            if self.operators.iter().any(|&i| i == ch) { op.push(self.input.consume().unwrap()) }
            else { break; };
        };
        Token {val: TokenType::Op(op), range: Range {start, end: self.input.loc()}}
    }

    fn consume(&mut self) -> Option<Token> {
        if self.input.is_eof() { return None; };
        let tok = self.input.peek(0).unwrap();
        if tok == '/' && self.input.peek(1) == Some('/') {
            self.input.consume();
            self.input.consume();
            return self.consume();
        };
        match tok {
            '"' => Some(self.parse_str()),
            '0'..='9' => Some(self.parse_num()),
            ' ' | '\n' | '\t' => {
                self.input.consume();
                self.consume()
            },
            '+' | '-' | '>' | '<' | '=' | '!' | '%' | '|' | '&' => Some(self.parse_op()),
            ',' | '.' | ':' | ';' | '{' | '}' | '[' | ']' | '(' | ')' => Some(self.parse_punc()),
            'a'..='z' | 'A'..='Z' | '_' => Some(self.parse_ident()),
            ch => {
                self.error(format!("Invalid character {}", ch), self.input.loc(), self.input.loc());
                self.input.consume();
                None
            } 
        }
    }

    pub fn next(&mut self) -> Option<Token> {
        if self.current.is_some() {
            return self.current.take()
        } else {
            return self.consume();
        };
    }

    pub fn peek(&mut self) -> Option<&Token> {
        if self.current.is_some() {
            return self.current.as_ref();
        };
        self.current = self.consume();
        self.current.as_ref()
    }

    pub fn error(&mut self, msg: String, start: LoC, end: LoC) {
        self.errors.push(Error { msg, range: Range {start, end} });
    }


}