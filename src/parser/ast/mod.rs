
use super::tokenizer::{Tokenizer, TokenType, Range};
use super::input_parser::LoC;
pub mod model;
pub mod utils;
use model::{ASTAny, ASTExpression, ASTInt, ASTFloat, ASTStr, ASTBool, ASTVar, ASTBinary};

pub struct Parser<'a> {
    pub tokens: Tokenizer<'a>
}

impl<'a> Parser<'a> {

    pub fn new(source: &'a str) -> Self {
        Parser {
            tokens: Tokenizer::new(source)
        }
    }

    fn get_prec(op: &String) -> i8 {
        match op.as_str() {
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
                    self.tokens.next();
                    let exp = self.parse_expression_part();
                    let right = self.parse_binary(exp, other_prec);
                    if right.is_none() { return Some(left_tok) };
                    return self.parse_binary(Some(ASTExpression::Binary(ASTBinary {
                        op:opval,
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

    fn parse_expression_part(&mut self) -> Option<ASTExpression> {

        let token = self.tokens.next()?;
        match token.val {
            TokenType::Int(value) => Some(ASTExpression::Int(ASTInt { value, range: token.range } )),
            TokenType::Float(value) => Some(ASTExpression::Float(ASTFloat { value, range: token.range })),
            TokenType::Str(value) => Some(ASTExpression::Str(ASTStr { value, range: token.range })),
            TokenType::Var(value) => Some(ASTExpression::Var(ASTVar { value, range: token.range })),
            TokenType::Bool(value) => Some(ASTExpression::Bool(ASTBool { value, range: token.range })),
            TokenType::Punc(val) => {
                match val {
                    // Expression wrapper
                    '(' => {
                        let exp = self.parse_expression();
                        self.tokens.next(); // Skip )
                        exp   
                    },
                    _ => {
                        self.tokens.error(format!("Unexpected punctuation {}", val), self.tokens.input.loc(), self.tokens.input.loc());
                        None
                    }
                }
            },
            _ => None
        }
    }

    fn parse_expression(&mut self) -> Option<ASTExpression> {
        let exp = self.parse_expression_part();
        self.parse_binary(exp, 0)
    }

    pub fn parse(&mut self) -> Vec<ASTAny> {
        let mut res = vec![];
        while !self.tokens.input.is_eof() {
            let parsed_expression = self.parse_expression();
            match parsed_expression {
                Some(exp) => res.push(ASTAny::Expression(exp)),
                None => {}
            }
        }
        res
    }

}