
use super::tokenizer::{Tokenizer, TokenType, Range};
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
                            self.tokens.error(String::from("Expected a proper path"), self.tokens.input.loc(), self.tokens.input.loc());
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
                                self.tokens.error(String::from("Expected a proper path"), self.tokens.input.loc(), self.tokens.input.loc());
                                None
                            }
                        }
                    },
                    "->" => {
                        self.tokens.consume();
                        let target = self.tokens.consume();
                        if target.is_none() { 
                            self.tokens.error(String::from("Expected a proper path"), self.tokens.input.loc(), self.tokens.input.loc());
                            return None;
                        };
                        match target.unwrap().val {
                            TokenType::Var(variable) => {
                                Some(ASTExpression::ArrowAccess(
                                    ASTArrowAccess {
                                        target: variable,
                                        value: Box::from(token.unwrap()),
                                        range: Range { start, end: self.tokens.input.loc() }
                                    }
                                ))
                            }
                            _ => {
                                self.tokens.error(String::from("Expected a proper path"), self.tokens.input.loc(), self.tokens.input.loc());
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
                        self.tokens.error(format!("Unexpected operator {}", value), self.tokens.input.loc(), self.tokens.input.loc());
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
                    _ => {
                        self.tokens.error(format!("Unexpected punctuation {}", val), self.tokens.input.loc(), self.tokens.input.loc());
                        None
                    }
                }
            },
            _ => None
        }
        };
        self.parse_suffix(exp)
    }

    fn parse_expression(&mut self) -> Option<ASTExpression> {
        let exp = self.parse_expression_part();
        self.parse_binary(exp, 0)
    }

    pub fn parse(&mut self) -> Vec<ASTAny> {
        let mut res = vec![];
        while !self.tokens.input.is_eof() {
            let parsed_expression = self.parse_expression();
            if let Some(exp) = parsed_expression { res.push(ASTAny::Expression(exp)) }
        }
        res
    }

}