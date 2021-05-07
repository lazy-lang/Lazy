
use super::tokenizer::{Tokenizer, TokenType, Range};
use super::tokenizer::error::ErrorType;
use super::input_parser::LoC;
pub mod model;
pub mod utils;
use model::*;


pub struct Parser<'a> {
    pub tokens: Tokenizer<'a>
}

impl<'a> Parser<'a> {

    pub fn new(source: &'a str) -> Self {
        Parser {
            tokens: Tokenizer::new(source)
        }
    }

    fn get_prec(op: &str) -> i8 {
        match op {
            "=" => 1,
            "||" => 2,
            "&&" => 3,
            "<" | ">" | "<=" | ">=" | "==" | "!=" => 10,
            "+" | "-" => 15,
            "*" | "/" | "%" => 20,
            _ => 0
        }
    }

    fn parse_binary(&mut self, left: Option<ASTExpression>, prec: i8) -> Option<ASTExpression> {
        let left_tok = left?;
        let start = self.tokens.input.loc();
        let next = self.tokens.peek();
        if next.is_none() { return Some(left_tok) };
        match &next.unwrap().val {
            TokenType::Op(val) => {
                let opval = val.to_string();
                let other_prec = Self::get_prec(&val);
                if other_prec > prec {
                    self.tokens.consume();
                    let exp = self.parse_expression_part();
                    let right = self.parse_binary(exp, other_prec);
                    if right.is_none() { return Some(left_tok) };
                    return self.parse_binary(Some(ASTExpression::Binary(ASTBinary {
                        op: opval,
                        left: Box::from(left_tok),
                        right: Box::from(right.unwrap()),
                        range: Range { start, end: self.tokens.input.loc() }
                    })), prec);
                }
                Some(left_tok)
            },
            _ => Some(left_tok)
        }
    }

    fn parse_suffix(&mut self, token: Option<ASTExpression>) -> Option<ASTExpression> {
        if token.is_none() { return token };
        let start = self.tokens.input.loc();
        let next_token = self.tokens.peek();
        if next_token.is_none() { return token };
        match &next_token.unwrap().val {
            TokenType::Op(val) => {
                match val.as_str() {
                    "." => {
                        self.tokens.consume();
                        let target = self.tokens.consume();
                        if target.is_none() { 
                            self.tokens.error(ErrorType::ProperProperty, start, self.tokens.input.loc());
                            return None;
                        };
                        match target.unwrap().val {
                            TokenType::Var(variable) => {
                                self.parse_suffix(Some(ASTExpression::DotAccess(
                                    ASTDotAccess {
                                        target: variable,
                                        value: Box::from(token.unwrap()),
                                        range: Range { start, end: self.tokens.input.loc() }
                                    }
                                )))
                            }
                            _ => {
                                self.tokens.error(ErrorType::ProperProperty, start, self.tokens.input.loc());
                                None
                            }
                        }
                    },
                    "->" => {
                        self.tokens.consume();
                        let target = self.tokens.consume();
                        if target.is_none() { 
                            self.tokens.error(ErrorType::ProperProperty, start, self.tokens.input.loc());
                            return None;
                        };
                        if self.tokens.is_next(TokenType::Op("->".to_string())) {
                            self.tokens.error(ErrorType::ArrowAccess, self.tokens.input.loc(), self.tokens.input.loc());
                            return None;
                        }
                        match target.unwrap().val {
                            TokenType::Var(variable) => {
                                self.parse_suffix(Some(ASTExpression::ArrowAccess(
                                    ASTArrowAccess {
                                        target: variable,
                                        value: Box::from(token.unwrap()),
                                        range: Range { start, end: self.tokens.input.loc() }
                                    }
                                )))
                            },
                            _ => {
                                self.tokens.error(ErrorType::ProperProperty, start, self.tokens.input.loc());
                                None
                            }
                        }
                    },
                    "?" => {
                        self.tokens.consume();
                        self.parse_suffix(Some(ASTExpression::Optional(
                            ASTOptional {
                                value: Box::from(token.unwrap()),
                                range: Range { start, end: self.tokens.input.loc() }
                            }
                        )))
                    },
                    _ => token
                }
            },
            _ => token
        }
    }

    fn parse_block(&mut self) -> ASTBlock {
        let start = self.tokens.input.loc();
        let mut res: Vec<ASTExpression> = vec![];
        while !self.tokens.input.is_eof() && !self.tokens.is_next(TokenType::Punc('}')) {
            let loc_before = self.tokens.input.loc();
            let exp = self.parse_expression();
            let range = utils::get_range_or(&exp, loc_before);
            match exp {
                Some(expression) => res.push(expression),
                None => continue
            };
            if self.tokens.skip_or_err(TokenType::Punc(';'), Some(ErrorType::Semicolon), Some(range)) { continue };
        }
        self.tokens.skip_or_err(TokenType::Punc('}'), Some(ErrorType::EndOfBlock), Some(Range {start, end: self.tokens.input.loc()}));
        ASTBlock {
            elements: res,
            range: Range { start, end: self.tokens.input.loc() }
        }
    }

    fn parse_varname(&mut self) -> Option<ASTVar> {
        let next = self.tokens.consume();
        if next.is_none() { 
            self.tokens.error(ErrorType::Expected(String::from("identifier")), self.tokens.input.loc(), self.tokens.input.loc());
            return None;
         };
        let unwrapped = next.unwrap();
        match unwrapped.val {
            TokenType::Var(v) => Some(ASTVar { value: v, range: unwrapped.range}),
            v => {
                self.tokens.error(ErrorType::ExpectedFound(String::from("identifier"), v.to_string()), unwrapped.range.start, unwrapped.range.end);
                None
            }
        }
    }

    fn parse_pair_list(&mut self, allow_without_val: bool, closing_punc: char) -> ASTPairList {
        let start = self.tokens.input.loc();
        let mut res: Vec<(String, Option<ASTExpression>)> = vec![];
        while !self.tokens.input.is_eof() && !self.tokens.is_next(TokenType::Punc(closing_punc)) {
            let tok_start = self.tokens.input.loc();
            let key = self.parse_varname();
            if key.is_none() { continue; };
            match self.tokens.expect_punc(&[',', ':'], Some(Range { start: tok_start, end: self.tokens.input.loc()})) {
                Some(ch) => {
                    match ch {
                        ',' => {
                            if !allow_without_val {
                                self.tokens.error(ErrorType::Expected(String::from("value")), tok_start, self.tokens.input.loc());
                                continue;
                            }
                            res.push((key.unwrap().value, None));
                        },
                        ':' => {
                            let exp = self.parse_expression();
                            if exp.is_none() { 
                                self.tokens.error(ErrorType::Expected(String::from("expression")), tok_start, self.tokens.input.loc());
                                continue;
                            }
                            res.push((key.unwrap().value, exp));
                        },
                        _ => {}
                    }
                },
                None => continue
            };
            if self.tokens.is_next(TokenType::Punc(',')) { self.tokens.consume(); };
        };
        self.tokens.skip_or_err(TokenType::Punc(closing_punc), Some(ErrorType::Expected(closing_punc.to_string())), Some(Range { start, end: self.tokens.input.loc()}));
        ASTPairList {
            range: Range { start, end: self.tokens.input.loc() },
            pairs: res
        }
    }

    fn parse_function(&mut self) -> Option<ASTFunction> {
        let start = self.tokens.input.loc();
        let params = self.parse_pair_list(false, ')');
        let mut return_type = None;
        if self.tokens.is_next(TokenType::Op(String::from("->"))) {
            self.tokens.consume();
            // Todo: Parse typing instead of expression
            let exp = self.parse_expression();
            if exp.is_none() { 
                self.tokens.error(ErrorType::Expected(String::from("return type")), self.tokens.input.loc(), self.tokens.input.loc()); 
                return None; 
            };
            return_type = Some(Box::from(exp.unwrap()));
        }
        if self.tokens.skip_or_err(TokenType::Punc('{'), Some(ErrorType::StartOfBlock), None) { return None };
        Some(ASTFunction {
            range: Range { start, end: self.tokens.input.loc() },
            params,
            return_type,
            body: self.parse_block()
        })
    }

    fn parse_expression_part(&mut self) -> Option<ASTExpression> {
        let exp = {
        let token = self.tokens.consume()?;
        match token.val {
            TokenType::Int(value) => Some(ASTExpression::Int(ASTInt { value, range: token.range } )),
            TokenType::Float(value) => Some(ASTExpression::Float(ASTFloat { value, range: token.range })),
            TokenType::Str(value) => Some(ASTExpression::Str(ASTStr { value, range: token.range })),
            TokenType::Var(value) => Some(ASTExpression::Var(ASTVar { value, range: token.range })),
            TokenType::Bool(value) => Some(ASTExpression::Bool(ASTBool { value, range: token.range })),
            TokenType::Op(value) => {
                // Prefixes
                match value.as_str() {
                    "-" | "!" => {
                        Some(ASTExpression::Unary(
                            ASTUnary {
                                op: value,
                                value: Box::from(self.parse_expression()?),
                                range: token.range
                            }
                        ))
                    },
                    _ => {
                        self.tokens.error(ErrorType::UnexpectedOp(value), token.range.start, token.range.end);
                        None
                    }
                }
            },
            TokenType::Punc(val) => {
                match val {
                    '(' => {
                        let exp = self.parse_expression();
                        self.tokens.consume(); // Skip )
                        exp   
                    },
                    ';' => None,
                    '{' => Some(ASTExpression::Block(self.parse_block())),
                    _ => {
                        self.tokens.error(ErrorType::UnexpectedPunc(val), token.range.start, token.range.end);
                        None
                    }
                }
            },
            TokenType::Kw(val) => {
                match val.as_str() {
                    "let" => {
                        if let Some(tok) = self.tokens.consume() {
                            if let TokenType::Var(name) = tok.val {
                                if self.tokens.is_next(TokenType::Op("=".to_string())) {
                                    let equals = self.tokens.consume().unwrap(); // Skip =
                                    let exp = self.parse_expression();
                                    return match exp {
                                        Some(expression) => {
                                            let exp_end = utils::full_expression_range(&expression).end;
                                            Some(ASTExpression::Let(
                                                ASTLet {
                                                    var: name,
                                                    value: Some(Box::from(expression)),
                                                    range: Range { start: token.range.start, end: exp_end }
                                                }
                                            )) 
                                        },
                                        None => {
                                            self.tokens.error(ErrorType::Expected("initializer".to_string()), equals.range.end, equals.range.end);
                                            println!("{}", self.tokens.peek().unwrap().val);
                                            return None;
                                        }
                                    }
                                };
                                return Some(ASTExpression::Let(
                                    ASTLet {
                                        var: name,
                                        value: None,
                                        range: Range { start: token.range.start, end: tok.range.end }
                                    }
                                ))
                            }
                        }
                    self.tokens.error(ErrorType::Expected("variable name".to_string()), token.range.start, token.range.end);
                    None
                    },
                    "f" => {
                        if self.tokens.skip_or_err(TokenType::Punc('('), Some(ErrorType::Expected(String::from("start of function params"))), None) { return None };
                        Some(ASTExpression::Function(self.parse_function()?))
                    },
                    _ => {
                        self.tokens.error(ErrorType::ExpectedFound("expression".to_string(), format!("keyword {}", val)), token.range.start, token.range.end);
                        None
                    }
                }
            }
        }
        };
        self.parse_suffix(exp)
    }

    fn parse_expression(&mut self) -> Option<ASTExpression> {
        let exp = self.parse_expression_part();
        self.parse_binary(exp, 0)
    }

    fn parse_statement(&mut self) -> Option<ASTStatement> {
        let token = self.tokens.consume()?;
        match token.val {
            TokenType::Kw(keyword) => {
                match keyword.as_str() {
                   "struct" => {
                    if let Some(tok) = self.tokens.consume() {
                        if let TokenType::Var(name) = tok.val {
                            if self.tokens.skip_or_err(TokenType::Punc('{'), Some(ErrorType::Expected(String::from("start of struct fields"))), None) { return None; };
                            return Some(ASTStatement::Struct(ASTStruct {
                                name,
                                fields: self.parse_pair_list(false, '}')
                            }));
                        };
                    };
                    self.tokens.error(ErrorType::Expected("struct name".to_string()), token.range.start, token.range.end);
                    None
                   }
                   "enum" => {
                    if let Some(tok) = self.tokens.consume() {
                        if let TokenType::Var(name) = tok.val {
                            if self.tokens.skip_or_err(TokenType::Punc('{'), Some(ErrorType::Expected(String::from("start of enum fields"))), None) { return None; };
                            return Some(ASTStatement::Struct(ASTStruct {
                                name,
                                fields: self.parse_pair_list(true, '}')
                            }));
                        };
                    };
                    self.tokens.error(ErrorType::Expected("enum name".to_string()), token.range.start, token.range.end);
                    None
                   }
                   _ => {
                    self.tokens.error(ErrorType::Expected(String::from("statement")), token.range.start, self.tokens.input.loc());
                    self.tokens.input.skip_line();
                    None
                },
                }
            },
            TokenType::Punc(';') => None,
            _ => {
                self.tokens.error(ErrorType::Expected(String::from("statement")), token.range.start, self.tokens.input.loc());
                self.tokens.input.skip_line();
                None
            }
        }
    }

    pub fn parse(&mut self) -> Vec<ASTAny> {
        let mut res = vec![];
        while !self.tokens.input.is_eof() {
            let parsed_statement = self.parse_statement();
            if let Some(exp) = parsed_statement { res.push(ASTAny::Statement(exp)) }
        }
        res
    }

}