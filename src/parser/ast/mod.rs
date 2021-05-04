
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

    fn parse_block(&mut self, req_start: bool) -> Option<ASTBlock> {
        let start = self.tokens.input.loc();
        if req_start {
            if !self.tokens.is_next(TokenType::Punc('{')) {
            self.tokens.error(ErrorType::StartOfBlock, self.tokens.input.loc(), self.tokens.input.loc());
            return None;
            } else {
                self.tokens.consume(); // skip {
            }
        };
        let mut res: Vec<ASTAny> = vec![];
        while !self.tokens.input.is_eof() && !self.tokens.is_next(TokenType::Punc('}')) {
            let loc_before = self.tokens.input.loc();
            let exp = self.parse_expression();
            let range = utils::get_range_or(&exp, loc_before);
            match exp {
                Some(expression) => res.push(ASTAny::Expression(expression)),
                None => continue
            };
            if self.tokens.skip_or_err(TokenType::Punc(';'), Some(ErrorType::Semicolon), Some(range)) { continue };
        }
        self.tokens.skip_or_err(TokenType::Punc('}'), Some(ErrorType::EndOfBlock), Some(Range {start, end: self.tokens.input.loc()}));
        Some(ASTBlock {
            elements: res,
            range: Range { start, end: self.tokens.input.loc() }
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
                    // Expression wrapper
                    '(' => {
                        let exp = self.parse_expression();
                        self.tokens.consume(); // Skip )
                        exp   
                    },
                    ';' => None,
                    '{' => Some(ASTExpression::Block(self.parse_block(false)?)),
                    _ => {
                        self.tokens.error(ErrorType::UnexpectedPunc(val), token.range.start, token.range.end);
                        None
                    }
                }
            },
            TokenType::Kw(val) => {
                match val.as_str() {
                    "let" => {
                        let id = self.tokens.consume();
                        if let Some(tok) = id {
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

    /*
    fn parse_statement(&mut self) -> Option<ASTStatement> {
        let token = self.tokens.consume()?;
        match token.val {
            TokenType::Kw(keyword) => {
                match keyword.as_str() {
                    _ => None
                }
            },
            TokenType::Punc(';') => None,
            _ => None
        }
    }
    */

    pub fn parse(&mut self) -> Vec<ASTAny> {
        let mut res = vec![];
        while !self.tokens.input.is_eof() {
            let parsed_expression = self.parse_expression();
            if let Some(exp) = parsed_expression { res.push(ASTAny::Expression(exp)) }
        }
        res
    }

}