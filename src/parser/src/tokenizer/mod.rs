use std::fmt;
use errors::*;
use diagnostics::*;
use super::input_parser::{InputParser, InputSequence};
use std::collections::{VecDeque, HashMap};
use crate::macros::*;

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

macro_rules! match_str {
    ($s: expr, $($strs: expr),*) => {
        match $s {
            $($strs)|+ => true,
            _ => false
        }
    };
}

#[derive(Clone)]
pub struct Token {
    pub range: Range,
    pub val: TokenType
}

pub struct Tokenizer {
    current: Option<Token>,
    pub filename: String,
    pub errors: Vec<Error>,
    pub input: InputParser,
    pub pre_tokens: VecDeque<Token>,
    pub macros: HashMap<String, Macro>,
    pub is_last_num_as_str: bool,
    pub macro_context: bool,
    pub last_loc: LoC
}

impl Tokenizer {

    pub fn new(code: &str, filename: String) -> Self {
        Tokenizer {
            current: None,
            errors: vec![],
            is_last_num_as_str: false,
            macro_context: false,
            input: InputParser::new(code),
            pre_tokens: VecDeque::new(),
            macros: HashMap::new(),
            last_loc: LoC::default(),
            filename: filename
        }
    }

    fn parse_str(&mut self, end_char: char) -> Token {
        let start = self.input.loc();
        self.input.consume(); // Consume the starting "
        let mut str = String::new();
        loop {
            match self.input.consume() {
                Some(character) => {
                    if character == end_char { break; };
                    str.push(character);
                },
                None => {
                    self.errors.push(err!(END_OF_STR, Range { start, end: self.last_loc }, self.filename));
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
                self.errors.push(err!(EMPTY_CHAR_LITERAL, self.range_here(), self.filename));
                '_'
            }
        };
        let next = self.input.consume();
        if next == None || next.unwrap() != '\'' {
            self.errors.push(err!(ONE_CHAR_ENDPOINT, Range { start, end: self.last_loc }, self.filename));
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
                            self.errors.push(err!(INVALID_DIGIT, self.range_here(), self.filename));
                            num_type = NumberType::None;
                        },
                        NumberType::Octal if ch > '7' => {
                            self.errors.push(err!(INVALID_DIGIT, self.range_here(), self.filename));
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
                        self.errors.push(err!(INVALID_DIGIT, self.range_here(), self.filename));
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
                        self.errors.push(err!(DECIMAL_POINT, Range {start, end: self.input.loc()}, self.filename)); 
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
        match ident.as_str() {
            "true" => Token { val: TokenType::Bool(true), range: Range {start, end: self.input.loc()}},
            "false" => Token { val: TokenType::Bool(false), range: Range {start, end: self.input.loc()}},
            "none" => Token { val: TokenType::None, range: Range { start, end: self.input.loc() }},
            "main" | "let" | "for" | "while" | "if" | "else" | "enum" | "struct" | "fn" | "type" | "const" | "yield" | "match" | "static" | "new" | "private" | "export" | "import" | "as" | "await" | "impl" | "in" | "from" | "macro" => Token { val: TokenType::Kw(ident), range: Range {start, end: self.input.loc() } },
            _ => Token { val: TokenType::Var(ident), range: Range {start, end: self.input.loc() } }
        }
    }

    pub fn parse_text(&mut self) -> String {
        let mut ident = String::new();
        while Some(' ') == self.input.peek(0) {
            self.input.consume();
        }
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
        self.last_loc = self.input.loc();
        return ident;
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
                self.errors.push(err!(UNEXPECTED_OP, Range { start, end: self.input.loc()}, self.filename, &op.clone()));
                break;
            }
            // Operators which cannot be followed by other operators
            if match_str!(op.as_str(), "?", ">") { break; };
            let ch = self.input.peek(0).unwrap();
            if match_str!(ch, '+', '-', '>', '<', '=', '!', '%', '|', '&', '.', '?', '~', '^', '*', '/') { op.push(self.input.consume().unwrap()) }
            else { break; };
        };
        Token {val: TokenType::Op(op), range: Range {start, end: self.input.loc()}}
    }

    fn _next(&mut self) -> Option<Token> {
        if !self.pre_tokens.is_empty() {
            return self.pre_tokens.pop_front()
        }
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
            ' ' | '\n' | '\t' | '\r' => {
                self.input.consume();
                self._next()
            },
            '+' | '-' | '>' | '<' | '=' | '!' | '%' | '|' | '&' | '.' | '?' | '~' | '^' | '*' | '/' => Some(self.parse_op()),
            ',' | ':' | ';' | '{' | '}' | '[' | ']' | '(' | ')' | '#' => Some(self.parse_punc()),
            'a'..='z' | 'A'..='Z' | '_' => Some(self.parse_ident()),
            ch => {
                let loc = self.input.loc();
                self.input.consume();
                if let Some(confused_err) = Self::is_confusable(ch) {
                    self.errors.push(Error::new(confused_err, loc.to_range(), self.filename.to_string()));
                    return None;
                };
                self.errors.push(err!(INVALID_CHAR, loc.to_range(), self.filename, &ch.to_string()));
                None
            } 
        }
    }

    pub fn consume_input(&mut self) -> Option<char> {
        let val = self.input.consume();
        self.last_loc = self.input.loc();
        self.current = None;
        val
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

    pub fn skip_or_err(&mut self, tok: TokenType, err: Option<Error>) -> LazyResult<()> {
        match self.peek() {
            Some(token) => {
                if token.val != tok {
                    let other = token.val.to_string();
                    Err(err.unwrap_or(err!(EXPECTED_FOUND, self.last_loc.to_range(), self.filename, &tok.to_string(), &other;)))
                } else {
                    self.consume();
                    Ok(())
                }
            },
            None => {
                Err(err.unwrap_or(err!(EXPECTED, self.last_loc.to_range(), self.filename, &tok.to_string())))
            }
        }
    }

    pub fn skip_or_err_here(&mut self, tok: TokenType) -> Option<()> {
        if let Err(err) = self.skip_or_err(tok, None) {
            self.errors.push(err);
            return None;
        };
        Some(())
    }

    pub fn expect_punc(&mut self, puncs: &[char], loc: Option<Range>) -> LazyResult<char> {
        let location = loc.unwrap_or(Range { start: self.last_loc.clone(), end: self.last_loc.clone() });
        match self.peek() {
            Some(tok) => {
                match tok.val {
                    TokenType::Punc(punc) if puncs.contains(&punc) => {
                        self.consume();
                        Ok(punc)
                    },
                    _ => {
                        let tstr = tok.val.to_string();
                        Err(err!(EXPECTED_FOUND, location, self.filename, &format!("one of {}", puncs.iter().map(|i| format!("({})", i.to_string())).collect::<Vec<_>>().join(", ")), &tstr))
                    }
                }
            },
            None => {
                Err(err!(EXPECTED, location, self.filename, &format!("one of {}", puncs.iter().map(|i| format!("({})", i.to_string())).collect::<Vec<_>>().join(", "))))
            }
        }
    }

    pub fn handle_macro_decl(&mut self, name: &str) -> Option<()> {
        self.skip_or_err_here(TokenType::Punc('{'))?;
        let mac = Macro::parse(self)?;
        self.macros.insert(name.to_string(), mac);
        Some(())
    }

    pub fn is_confusable(ch: char) -> Option<String> {
        match ch {
            ';' => Some(dia!(CONFUSABLE, "; (Greek question mark)", "; (semicolon)")),
            '‚' => Some(dia!(CONFUSABLE, "‚ (low-9 quatation mark)", ", (comma)")),
            '٫' => Some(dia!(CONFUSABLE, "‚ (arabic decimal separator)", ", (comma)")),
            '：' => Some(dia!(CONFUSABLE, "： (fullwidth colon)", ": (colon)")),
            '։' => Some(dia!(CONFUSABLE, "： (armenian full stop)", ": (colon)")),
            '∶' => Some(dia!(CONFUSABLE, "∶ (ratio)", ": (colon)")),
            '！' => Some(dia!(CONFUSABLE, "！ (fullwidth exclamation mark)", "! (exclamation mark)")),
            'ǃ' => Some(dia!(CONFUSABLE, "ǃ (latin letter retroflex click)", "! (exclamation mark)")),
            '․' => Some(dia!(CONFUSABLE, "․ (one dot leader)", ". (full stop)")),
            _ => None
        }
    }

}