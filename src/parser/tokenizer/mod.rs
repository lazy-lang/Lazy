
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

pub enum ConsumeResult {
    Token(Token),
    Invalid(char),
    None
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

pub struct Token {
    pub range: Range,
    pub val: TokenType
}

pub struct Tokenizer {
    keywords: Vec<&'static str>,
    operators: Vec<char>,
    pub input: InputParser
}

impl Tokenizer {

    pub fn new(code: &str) -> Self {
        Tokenizer {
            keywords: vec!["main", "let", "emit", "match", "while", "if", "actor", "enum", "struct"],
            operators: vec!['+', '-', '>', '<', '=', '!', '%', '|', '&'],
            input: InputParser::new(code)
        }
    }

    fn parse_str(&mut self) -> Token {
        self.input.consume(); // Consume the starting "
        let start = self.input.loc();
        let mut str = String::new();
        loop {
            let character = self.input.consume().unwrap();
            if character == '"' { break; };
            str.push(character);
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
                    if dot { break; };
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
            if self.operators.iter().any(|&i| i == ch) { op.push(self.input.consume().unwrap()) };
        };
        Token {val: TokenType::Op(op), range: Range {start, end: self.input.loc()}}
    }

    pub fn consume(&mut self) -> ConsumeResult {
        if self.input.is_eof() { return ConsumeResult::None; };
        let tok = self.input.peek(0).unwrap();
        if tok == '/' && self.input.peek(1) == Some('/') {
            self.input.consume();
            self.input.consume();
            return self.consume();
        };
        match tok {
            '"' => ConsumeResult::Token(self.parse_str()),
            '0'..='9' => ConsumeResult::Token(self.parse_num()),
            ' ' | '\n' | '\t' => {
                self.input.consume();
                self.consume()
            },
            '+' | '-' | '>' | '<' | '=' | '!' | '%' | '|' | '&' => ConsumeResult::Token(self.parse_op()),
            ',' | '.' | ':' | ';' | '{' | '}' | '[' | ']' | '(' | ')' => ConsumeResult::Token(self.parse_punc()),
            'a'..='z' | 'A'..='Z' | '_' | '$' => ConsumeResult::Token(self.parse_ident()),
            ch => ConsumeResult::Invalid(ch)
        }
    }

}