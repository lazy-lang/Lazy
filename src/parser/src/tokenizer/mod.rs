
use std::fmt;
pub mod error;
pub mod range_recorder;
use error::*;
use range_recorder::*;
use super::input_parser::{LoC, InputParser};

#[derive(PartialEq)]
#[derive(Clone)]
pub enum TokenType {
    Str(String),
    TempStrStart,
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
            Self::TempStrStart => write!(f, "beginning of template literal"),
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
    pub fn err(&self, err: ErrorType, tokens: &mut Tokenizer) {
        tokens.error(err, self.start, self.end)
    }

    #[inline]
    pub fn err_start(&self, err: ErrorType, tokens: &mut Tokenizer) {
        tokens.error(err, self.start, tokens.input.loc())
    }

    #[inline]
    pub fn end(&self, tokens: &Tokenizer) -> Range {
        Range {
            start: self.start,
            end: tokens.input.loc()
        }
    }
}

macro_rules! match_str {
    ($s: expr, $($strs: expr),*) => {
        match $s {
            $($strs)|+ => true,
            _ => false
        }
    };
}

pub struct Token {
    pub range: Range,
    pub val: TokenType
}


pub struct Tokenizer {
    current: Option<Token>,
    pub errors: Vec<Error<ErrorType>>,
    pub input: InputParser,
    pub is_last_num_as_str: bool,
    pub last_loc: LoC
}

impl Tokenizer {

    pub fn new(code: &str) -> Self {
        Tokenizer {
            current: None,
            errors: vec![],
            is_last_num_as_str: false,
            input: InputParser::new(code),
            last_loc: LoC::default()
        }
    }

    fn parse_str(&mut self, end_char: char) -> Token {
        self.input.consume(); // Consume the starting "
        let start = self.input.loc();
        let mut str = String::new();
        loop {
            match self.input.consume() {
                Some(character) => {
                    if character == end_char { break; };
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

    fn parse_char(&mut self) -> Token {
        let start = self.input.loc();
        self.input.consume(); // Consume the starting '
        let maybe_ch = self.input.consume();
        let val = match maybe_ch {
            Some(ch) => ch,
            None => {
                self.error(ErrorType::EmptyCharLiteral, self.input.loc(), self.input.loc());
                '_'
            }
        };
        let next = self.input.consume();
        if next == None || next.unwrap() != '\'' {
            self.error(ErrorType::Expected(String::from("character literal may only contain one codepoint")), start, self.input.loc());
        }
        Token { val: TokenType::Char(val), range: Range { start, end: self.input.loc() }}
    }

    fn parse_num(&mut self) -> Token {
        let mut dot = false;
        let mut num = String::new();
        let start = self.input.loc();
        let mut num_type = NumberType::None; 
        while !self.input.is_eof() {
            match self.input.peek(0) {
            Some(ch) => {
            match ch {
                '0' => {
                    let next = self.input.peek(1);
                    if next.is_none() {
                        break;
                    }
                    match next.unwrap() {
                        'o' => {
                            num_type = NumberType::Octal;
                            self.input.consume();
                            self.input.consume();
                        },
                        'x' => {
                            num_type = NumberType::Hex;
                            self.input.consume();
                            self.input.consume();
                        },
                        'b' => {
                            num_type = NumberType::Binary;
                            self.input.consume();
                            self.input.consume();
                        },
                        _ => {
                            num.push('0');
                            self.input.consume();
                        }
                    }
                },
                '1'..='9' => {
                    match num_type {
                        NumberType::Binary if ch > '1' => {
                            self.error(ErrorType::InvalidDigit, self.input.loc(), self.input.loc());
                            num_type = NumberType::None;
                        },
                        NumberType::Octal if ch > '7' => {
                            self.error(ErrorType::InvalidDigit, self.input.loc(), self.input.loc());
                            num_type = NumberType::None;
                        },
                        _ => {}
                    };
                    num.push(self.input.consume().unwrap())
                },
                'A'..='F' | 'a'..='f' => {
                    if num_type == NumberType::Hex {
                        num.push(self.input.consume().unwrap())
                    } else {
                        self.error(ErrorType::InvalidDigit, self.input.loc(), self.input.loc());
                        break;
                    }
                },
                '.' => {
                    if self.is_last_num_as_str {
                        self.is_last_num_as_str = false;
                        break;
                    }
                    if dot {
                        self.input.consume();
                        let loc = self.input.loc();
                        self.error(ErrorType::DecimalPoint, start, loc); 
                        break;
                     };
                     if self.input.peek(1) == Some('.') {
                        return Token { val: TokenType::Int(num.parse().unwrap()), range: Range {start, end: self.input.loc()} }
                    }
                    self.input.consume();
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

        let multiply_val = match self.input.peek(0) {
            Some('s') => {
                self.input.consume();
                1000
            },
            Some('m') => {
                self.input.consume();
                60 * 1000
            },
            Some('h') => {
                self.input.consume();
                60 * 60 * 1000
            },
            Some('d') => {
                self.input.consume();
                24 * 60 * 60 * 1000
            },
            _ => 1
        } as isize;
        let actual_num = match num_type {
            NumberType::Hex => isize::from_str_radix(&num, 16).unwrap(),
            NumberType::Octal => isize::from_str_radix(&num, 8).unwrap(),
            NumberType::Binary => isize::from_str_radix(&num, 2).unwrap(),
            NumberType::None => num.parse::<isize>().unwrap()
        };
        let token_type = if dot { TokenType::Float((actual_num * multiply_val) as f32) } else { TokenType::Int((actual_num * multiply_val) as i32) };
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
        else if ident == "none" { return Token { val: TokenType::None, range: Range { start, end: self.input.loc() } } }

        let token_type = if match_str!(ident.as_str(), "main", "let", "for", "while", "if", "else", "enum", "struct", "fn", "type", "const", "yield", "when", "match", "static", "new", "private", "export", "import", "as", "await", "impl", "in") { TokenType::Kw(ident) } else { TokenType::Var(ident) };
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
            // Invalid operators
            if match_str!(op.as_str(), "<>") {
                self.error(ErrorType::UnexpectedOp(op.clone()), start, self.input.loc());
                break;
            }
            // Operators which cannot be folled by other operators
            if match_str!(op.as_str(), "?") { break; };
            let ch = self.input.peek(0).unwrap();
            if match_str!(ch, '+', '-', '>', '<', '=', '!', '%', '|', '&', '.', '?', '~', '^') { op.push(self.input.consume().unwrap()) }
            else { break; };
        };
        Token {val: TokenType::Op(op), range: Range {start, end: self.input.loc()}}
    }

    fn _next(&mut self) -> Option<Token> {
        if self.input.is_eof() { return None; };
        self.last_loc = self.input.loc();
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
            '\'' => Some(self.parse_char()),
            '"' => Some(self.parse_str('"')),
            '`' => {
                self.input.consume();
                Some(Token {
                val: TokenType::TempStrStart,
                range: Range { start: self.input.loc(), end: self.input.loc() }
                })
            },
            '0'..='9' => Some(self.parse_num()),
            ' ' | '\n' | '\t' => {
                self.input.consume();
                self._next()
            },
            '+' | '-' | '>' | '<' | '=' | '!' | '%' | '|' | '&' | '.' | '?' | '~' | '^' => Some(self.parse_op()),
            ',' | ':' | ';' | '{' | '}' | '[' | ']' | '(' | ')' | '#' => Some(self.parse_punc()),
            'a'..='z' | 'A'..='Z' | '_' => Some(self.parse_ident()),
            ch => {
                let loc = self.input.loc();
                self.input.consume();
                if let Some(confused_err) = Self::is_confusable(ch) {
                    self.error(confused_err, loc, loc);
                    return None;
                };
                self.error(ErrorType::InvalidCharacter(ch), loc, loc);
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

    #[inline]
    pub fn error_here(&mut self, e_type: ErrorType) {
        self.errors.push(Error { e_type, range: Range {start: self.input.loc(), end: self.input.loc() } });
    }

    pub fn is_next(&mut self, tok: TokenType) -> bool {
        let next = self.peek();
        match next {
            Some(token) => token.val == tok,
            None => true
        }
    }

    pub fn skip_or_err(&mut self, tok: TokenType, err: Option<ErrorType>, loc: Option<Range>) -> bool {
        let location = loc.unwrap_or(Range { start: self.last_loc, end: self.last_loc});
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

    pub fn is_confusable(ch: char) -> Option<ErrorType> {
        match ch {
            ';' => Some(ErrorType::Confusable("; (Greek question mark)".to_string(), "; (semicolon)".to_string())),
            '‚' => Some(ErrorType::Confusable("‚ (low-9 quatation mark)".to_string(), ", (comma)".to_string())),
            '٫' => Some(ErrorType::Confusable("‚ (arabic decimal separator)".to_string(), ", (comma)".to_string())),
            '：' => Some(ErrorType::Confusable("： (fullwidth colon)".to_string(), ": (colon)".to_string())),
            '։' => Some(ErrorType::Confusable("： (armenian full stop)".to_string(), ": (colon)".to_string())),
            '∶' => Some(ErrorType::Confusable("∶ (ratio)".to_string(), ": (colon)".to_string())),
            '！' => Some(ErrorType::Confusable("！ (fullwidth exclamation mark)".to_string(), "! (exclamation mark)".to_string())),
            'ǃ' => Some(ErrorType::Confusable("ǃ (latin letter retroflex click)".to_string(), "! (exclamation mark)".to_string())),
            '․' => Some(ErrorType::Confusable("․ (one dot leader)".to_string(), ". (full stop)".to_string())),
            _ => None
        }‎
    }

    pub fn recorder(&self) -> RangeRecorder {
        RangeRecorder::new(self)
    }

}