
use std::fmt;
pub mod error;
pub mod range_recorder;
use error::*;
use range_recorder::*;
use super::input_parser::{InputParser};

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

#[derive(PartialEq, fmt::Debug)]
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

pub trait RangeErrors {
    fn err<T: fmt::Display>(&self, err: T, tokens: &mut dyn ErrorCollector<T>);
    fn err_nc<T: fmt::Display>(&self, err: T) -> Error<T>;
    fn err_start(&self, err: ParserErrorType, tokens: &mut Tokenizer);
    fn end(&self, tokens: &Tokenizer) -> Range;
}

impl RangeErrors for Range {
    #[inline]
    fn err<T: fmt::Display>(&self, err: T, tokens: &mut dyn ErrorCollector<T>) {
        tokens.error(err, self.start, self.end)
    }

    #[inline]
    fn err_start<'a>(&self, err: ParserErrorType, tokens: &mut Tokenizer) {
        tokens.error(err, self.start, tokens.input.loc())
    }

    #[inline]
    fn end(&self, tokens: &Tokenizer) -> Range {
        Range {
            start: self.start,
            end: tokens.last_loc
        }
    }

    fn err_nc<T: fmt::Display>(&self, err: T) -> Error<T> {
        Error {
            msg: err,
            range: Range { start: self.start, end: self.end },
            labels: None,
            highlighted: false
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
    pub errors: Vec<Error<ParserErrorType>>,
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
                    self.error(ParserErrorType::EndOfStr, start, loc);
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
                self.error(ParserErrorType::EmptyCharLiteral, self.input.loc(), self.input.loc());
                '_'
            }
        };
        let next = self.input.consume();
        if next == None || next.unwrap() != '\'' {
            self.error(ParserErrorType::Expected("character literal may only contain one codepoint"), start, self.input.loc());
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
                            self.error(ParserErrorType::InvalidDigit, self.input.loc(), self.input.loc());
                            num_type = NumberType::None;
                        },
                        NumberType::Octal if ch > '7' => {
                            self.error(ParserErrorType::InvalidDigit, self.input.loc(), self.input.loc());
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
                        self.error(ParserErrorType::InvalidDigit, self.input.loc(), self.input.loc());
                        break;
                    }
                },
                '.' => {
                    if self.is_last_num_as_str {
                        break;
                    }
                    if self.input.peek(1) == Some('.') {
                        return Token { val: TokenType::Int(num.parse().unwrap()), range: Range {start, end: self.input.loc()} }
                    }
                    if dot {
                        self.input.consume();
                        self.error(ParserErrorType::DecimalPoint, start, self.input.loc()); 
                        break;
                     };
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
        let token_type = match num_type {
            NumberType::Hex => TokenType::Int((isize::from_str_radix(&num, 16).unwrap() * multiply_val) as i32),
            NumberType::Octal => TokenType::Int((isize::from_str_radix(&num, 8).unwrap() * multiply_val) as i32),
            NumberType::Binary => TokenType::Int((isize::from_str_radix(&num, 2).unwrap() * multiply_val) as i32),
            NumberType::None => {
                if dot { TokenType::Float(num.parse::<f32>().unwrap() * (multiply_val as f32)) }
                else { TokenType::Int(num.parse::<i32>().unwrap() * (multiply_val as i32)) }
            }
        };
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

        let token_type = if match_str!(ident.as_str(), "main", "let", "for", "while", "if", "else", "enum", "struct", "fn", "type", "const", "yield", "match", "static", "new", "private", "export", "import", "as", "await", "impl", "in") { TokenType::Kw(ident) } else { TokenType::Var(ident) };
        Token { val: token_type, range: Range {start, end: self.input.loc() } }
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
                self.error(ParserErrorType::UnexpectedOp(op.clone()), start, self.input.loc());
                break;
            }
            // Operators which cannot be followed by other operators
            if match_str!(op.as_str(), "?", ">") { break; };
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
                range: self.range_here()
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
                self.error(ParserErrorType::InvalidCharacter(ch), loc, loc);
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
    pub fn error_here(&mut self, e_type: ParserErrorType) {
        self.errors.push(Error::new(e_type, self.range_here()));
    }

    #[inline]
    pub fn range_here(&self) -> Range {
        Range {start: self.last_loc, end: self.last_loc }
    }
 
    pub fn is_next(&mut self, tok: TokenType) -> bool {
        let next = self.peek();
        match next {
            Some(token) => token.val == tok,
            None => true
        }
    }

    pub fn skip_or_err(&mut self, tok: TokenType, err: Option<Error<ParserErrorType>>) -> bool {
        match self.peek() {
            Some(token) => {
                if token.val != tok {
                    let other = token.val.to_string();
                    self.errors.push(err.unwrap_or(Error::new(ParserErrorType::expected_found(tok.to_string(), other), Range { start: self.last_loc, end: self.last_loc })));
                    true
                } else {
                    self.consume();
                    false
                }
            },
            None => {
                self.errors.push(err.unwrap_or(Error::new(ParserErrorType::ExpectedString(tok.to_string()), Range { start: self.last_loc, end: self.last_loc })));
                true
            }
        }
    }

    pub fn expect_punc(&mut self, puncs: &[char], loc: Option<Range>) -> Option<char> {
        let location = loc.unwrap_or(Range { start: self.input.loc(), end: self.input.loc()});
        match self.peek() {
            Some(tok) => {
                match tok.val {
                    TokenType::Punc(punc) if puncs.contains(&punc) => {
                        self.consume();
                        Some(punc)
                    },
                    _ => {
                        let tstr = tok.val.to_string();
                        self.error(ParserErrorType::expected_found(format!("one of {}", puncs.iter().map(|i| format!("({})", i.to_string())).collect::<Vec<_>>().join(", ")), tstr), location.start, location.end);
                        None
                    }
                }
            },
            None => {
                self.error(ParserErrorType::ExpectedString(format!("one of {}", puncs.iter().map(|i| format!("({})", i.to_string())).collect::<Vec<_>>().join(", "))), location.start, location.end);
                None
            }
        }
    }

    pub fn is_confusable(ch: char) -> Option<ParserErrorType> {
        match ch {
            ';' => Some(ParserErrorType::Confusable("; (Greek question mark)", "; (semicolon)")),
            '‚' => Some(ParserErrorType::Confusable("‚ (low-9 quatation mark)", ", (comma)")),
            '٫' => Some(ParserErrorType::Confusable("‚ (arabic decimal separator)", ", (comma)")),
            '：' => Some(ParserErrorType::Confusable("： (fullwidth colon)", ": (colon)")),
            '։' => Some(ParserErrorType::Confusable("： (armenian full stop)", ": (colon)")),
            '∶' => Some(ParserErrorType::Confusable("∶ (ratio)", ": (colon)")),
            '！' => Some(ParserErrorType::Confusable("！ (fullwidth exclamation mark)", "! (exclamation mark)")),
            'ǃ' => Some(ParserErrorType::Confusable("ǃ (latin letter retroflex click)", "! (exclamation mark)")),
            '․' => Some(ParserErrorType::Confusable("․ (one dot leader)", ". (full stop)")),
            _ => None
        }‎
    }

    pub fn recorder(&self) -> RangeRecorder {
        RangeRecorder::new(self)
    }

}

impl error::ErrorCollector<ParserErrorType> for Tokenizer {

    #[inline]
    fn error(&mut self, e_type: ParserErrorType, start: LoC, end: LoC) {
        self.errors.push(Error::new(e_type, Range {start, end}));
    }

    fn error_lbl(&mut self, e_type: ParserErrorType, start: LoC, end: LoC, labels: Vec<ErrorLabel>, highlight: bool) {
        self.errors.push(Error::new_with_labels(e_type, Range { start, end }, labels, highlight));
    }
}