
use super::input_parser;

pub struct Tokenizer {
    pub input: input_parser::InputParser
}

pub enum TokenType {
    Str(String),
    Float(f32),
    Int(i32),
    Kw(String),
    Var(String),
    Op(String),
    Invalid(char),
    None
}

impl Tokenizer {

    pub fn new(code: &str) -> Self {
        Tokenizer {
            input: input_parser::InputParser::new(code)
        }
    }

    fn parse_str(&mut self) -> TokenType {
        self.input.consume(); // Consume the starting "
        let mut str = String::new();
        while !self.input.is_eof() {
            match self.input.consume() {
                Some(character) => {
                    if character == '"' { break; };
                    str.push(character);
                },
                None => panic!("Expected end of string")
            }
        };
        TokenType::Str(str)
    }

    fn parse_num(&mut self) -> TokenType {
        let mut dot = false;
        let mut num = String::new();
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
        if dot { return TokenType::Float(num.parse().unwrap()) }
        else { return TokenType::Int(num.parse().unwrap()) }
    }

    pub fn consume(&mut self) -> TokenType {
        if self.input.is_eof() { return TokenType::None; };
        let tok = self.input.peek(0).unwrap();
        if tok == '/' && self.input.peek(1) == Some('/') {
            self.input.consume();
            self.input.consume();
            return self.consume();
        };
        match tok {
            '"' => self.parse_str(),
            '0'..='9' => self.parse_num(),
            ' ' => {
                self.input.consume();
                self.consume()
            },
            ch => {
                TokenType::Invalid(ch)
            }
        }
    }

}