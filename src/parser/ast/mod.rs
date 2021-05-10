
use super::tokenizer::{Tokenizer, TokenType, Range};
use super::tokenizer::error::ErrorType;
use super::input_parser::LoC;
pub mod model;
pub mod utils;
use model::*;


pub struct Parser<'a> {
    pub tokens: Tokenizer<'a>,
    is_last_block: bool
}

impl<'a> Parser<'a> {

    pub fn new(source: &'a str) -> Self {
        Parser {
            tokens: Tokenizer::new(source),
            is_last_block: false
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
            _ => -1
        }
    }

    fn parse_binary(&mut self, left: Option<ASTExpression>, prec: i8) -> Option<ASTExpression> {
        let left_tok = left?;
        let start = self.tokens.input.loc();
        let next = self.tokens.peek();
        if next.is_none() { 
            return Some(left_tok)
         };
        let value = &next.unwrap();
        match &value.val {
            TokenType::Op(val) => {
                let opval = val.to_string();
                let other_prec = Self::get_prec(&val);
                if other_prec == -1 {
                    self.tokens.consume();
                    self.tokens.error(ErrorType::UnexpectedOp(opval), start, start);
                    return Some(left_tok)
                }
                if other_prec > prec {
                    let err_start = value.range.start;
                    let err_end = value.range.end;
                    self.tokens.consume();
                    let exp = self.parse_expression_part();
                    let right = self.parse_binary(exp, other_prec);
                    if right.is_none() { 
                        self.tokens.error(ErrorType::UnexpectedOp(opval), err_start, err_end);
                        return Some(left_tok)
                     };
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
                        let target = self.parse_varname(true, false);
                        if target.0.is_none() { 
                            self.tokens.error(ErrorType::ProperProperty, start, self.tokens.input.loc());
                            return None;
                        };
                        self.parse_suffix(Some(ASTExpression::DotAccess(
                            ASTDotAccess {
                                target: target.0.unwrap(),
                                value: Box::from(token.unwrap()),
                                range: Range { start, end: self.tokens.input.loc() }
                            }
                        )))
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
                    ".." => {
                        self.tokens.consume();
                        let end = self.parse_expression();
                        if end.is_none() {
                            self.tokens.error(ErrorType::Expected(String::from("end of iterator")), start, self.tokens.input.loc());
                            return None;
                        }
                        Some(ASTExpression::Iterator(
                            ASTIterator {
                                start: Box::from(token.unwrap()),
                                end: Box::from(end.unwrap()),
                                range: Range { start, end: self.tokens.input.loc() }
                            }
                        ))
                    },
                    _ => token
                }
                // TBD: Add ( for calling functions
            },
            _ => token
        }
    }

    fn parse_typing(&mut self, allow_fn_keyword: bool) -> Option<ASTTypings> {
        let start = self.tokens.input.loc();
        let maybe_token = self.tokens.peek();
        match maybe_token {
            Some(token) => {
                match &token.val {
                    TokenType::Punc('{') => {
                        self.tokens.consume();
                        Some(ASTTypings::PairList(self.parse_typing_pair_list(false, allow_fn_keyword, '}')))
                    },
                    TokenType::Punc('(') => {
                        self.tokens.consume();
                        let params = Box::from(self.parse_typing_pair_list(false, allow_fn_keyword, ')'));
                        let return_type = if self.tokens.is_next(TokenType::Op(String::from("->"))) { 
                            self.tokens.consume(); 
                            let typing = self.parse_typing(allow_fn_keyword);
                            if typing.is_none() { 
                                self.tokens.error(ErrorType::Expected(String::from("return type")), start, self.tokens.input.loc());
                                return None
                            };
                            Some(Box::from(typing.unwrap()))
                        } else { None };
                        Some(ASTTypings::Function(ASTFunction {
                            params,
                            return_type,
                            range: Range { start, end: self.tokens.input.loc() },
                            body: None
                        }))
                    },
                    TokenType::Var(name) => {
                        let value = name.to_string();
                        self.tokens.consume();
                        let typings = if self.tokens.is_next(TokenType::Op(String::from("<"))) {
                            self.tokens.consume(); // Skip <
                            Some(self.parse_typing_list(false, allow_fn_keyword))
                        } else { None };
                        Some(ASTTypings::Var(ASTVarTyping {
                            value,
                            typings,
                            range: Range { start, end: self.tokens.input.loc() }
                        }))
                    },
                    TokenType::Kw(kw) => {
                        if !allow_fn_keyword {
                            self.tokens.error(ErrorType::Unexpected(String::from("keyword fn")), start, self.tokens.input.loc());
                            return None;
                        }
                        match kw.as_str() {
                            "fn" => {
                                self.tokens.consume();
                                self.tokens.consume(); // Skip (
                                Some(ASTTypings::Function(self.parse_function(true)?))
                            },
                            _ => None
                        }
                    }
                    _ => {
                        let token_stringed = token.val.to_string();
                        self.tokens.error(ErrorType::ExpectedFound(String::from("typing"), token_stringed), self.tokens.input.loc(), self.tokens.input.loc());
                        None
                    }
                }
            },
            None => {
                self.tokens.error(ErrorType::Expected(String::from("typing")), self.tokens.input.loc(), self.tokens.input.loc());
                None
            }
        }
    }

    fn parse_block(&mut self) -> ASTBlock {
        let start = self.tokens.input.loc();
        let mut res: Vec<ASTExpression> = vec![];
        while !self.tokens.input.is_eof() && !self.tokens.is_next(TokenType::Punc('}')) {
            let exp = self.parse_expression();
            let range = utils::get_range_or(&exp, self.tokens.input.loc());
            match exp {
                Some(expression) => res.push(expression),
                None => continue
            };
           if !self.is_last_block { self.tokens.skip_or_err(TokenType::Punc(';'), Some(ErrorType::Semicolon), Some(range)); };
        }
        self.tokens.skip_or_err(TokenType::Punc('}'), Some(ErrorType::EndOfBlock), Some(Range {start, end: self.tokens.input.loc()}));
        self.is_last_block = true;
        ASTBlock {
            elements: res,
            range: Range { start, end: self.tokens.input.loc() }
        }
    }

    fn parse_varname(&mut self, allow_generics: bool, only_varnames_as_generics: bool) -> (Option<ASTVar>, Option<ASTListTyping>) {
        let next = self.tokens.consume();
        if next.is_none() { 
            self.tokens.error(ErrorType::Expected(String::from("identifier")), self.tokens.input.loc(), self.tokens.input.loc());
            return (None, None);
        };
        let unwrapped = next.unwrap();
        let var = match unwrapped.val {
            TokenType::Var(v) => ASTVar { value: v, range: unwrapped.range },
            v => {
                self.tokens.error(ErrorType::ExpectedFound(String::from("identifier"), v.to_string()), unwrapped.range.start, unwrapped.range.end);
                return (None, None);
            }
        };
        if self.tokens.is_next(TokenType::Op(String::from("<"))) {
            if !allow_generics {
                self.tokens.error(ErrorType::Unexpected(String::from("token <, generics are not allowed here.")), self.tokens.input.loc(), self.tokens.input.loc());
            }
            self.tokens.consume();
            return (Some(var), Some(self.parse_typing_list(only_varnames_as_generics, false)));
        }
        (Some(var), None)
    }

    fn parse_pair_list(&mut self, allow_without_val: bool, closing_punc: char) -> ASTPairList {
        let start = self.tokens.input.loc();
        let mut res: Vec<(String, Option<ASTExpression>)> = vec![];
        let mut has_consumed_bracket = false;
        while !self.tokens.input.is_eof() && !self.tokens.is_next(TokenType::Punc(closing_punc)) {
            let tok_start = self.tokens.input.loc();
            let key = self.parse_varname(false, false);
            if key.0.is_none() { continue; };
            match self.tokens.expect_punc(&[',', ':', '}'], Some(Range { start: tok_start, end: self.tokens.input.loc()})) {
                Some(ch) => {
                    match ch {
                        ',' => {
                            if !allow_without_val {
                                self.tokens.error(ErrorType::Expected(String::from("value")), tok_start, self.tokens.input.loc());
                                continue;
                            }
                            res.push((key.0.unwrap().value, None));
                        },
                        ':' => {
                            let exp = self.parse_expression();
                            if exp.is_none() { 
                                self.tokens.error(ErrorType::Expected(String::from("expression")), tok_start, self.tokens.input.loc());
                                continue;
                            }
                            res.push((key.0.unwrap().value, exp));
                        },
                        '}' => {
                            if !allow_without_val {
                                self.tokens.error(ErrorType::Expected(String::from("type")), tok_start, self.tokens.input.loc());
                                continue;
                            }
                            has_consumed_bracket = true;
                            break;
                        },
                        _ => {}
                    }
                },
                None => continue
            };
            if self.tokens.is_next(TokenType::Punc(',')) { self.tokens.consume(); };
        };
        if !has_consumed_bracket { self.tokens.skip_or_err(TokenType::Punc(closing_punc), Some(ErrorType::Expected(closing_punc.to_string())), Some(Range { start, end: self.tokens.input.loc()})); };
        ASTPairList {
            range: Range { start, end: self.tokens.input.loc() },
            pairs: res
        }
    }

    fn parse_typing_pair_list(&mut self, allow_without_val: bool, allow_fn_keyword: bool, closing_punc: char) -> ASTPairListTyping {
        let start = self.tokens.input.loc();
        let mut res: Vec<ASTPairTypingItem> = vec![];
        let mut is_optional = false;
        let mut has_consumed_bracket = false;
        while !self.tokens.input.is_eof() && !self.tokens.is_next(TokenType::Punc(closing_punc)) {
            let tok_start = self.tokens.input.loc();
            let key = self.parse_varname(false, false);
            if key.0.is_none() { continue; };
            if self.tokens.is_next(TokenType::Op(String::from("?"))) {
                self.tokens.consume();
                is_optional = true;
            }
            match self.tokens.expect_punc(&[',', ':', '?', '}'], Some(Range { start: tok_start, end: self.tokens.input.loc()})) {
                Some(ch) => {
                    match ch {
                        ',' => {
                            if !allow_without_val {
                                self.tokens.error(ErrorType::Expected(String::from("type")), tok_start, self.tokens.input.loc());
                                continue;
                            }
                            res.push(ASTPairTypingItem {name: key.0.unwrap().value, value: None, optional: is_optional});
                            is_optional = false;
                        },
                        ':' => {
                            let exp = self.parse_typing(allow_fn_keyword);
                            if exp.is_none() { 
                                self.tokens.error(ErrorType::Expected(String::from("expression")), tok_start, self.tokens.input.loc());
                                continue;
                            }
                            res.push(ASTPairTypingItem { name: key.0.unwrap().value, value: exp, optional: is_optional});
                            is_optional = false;
                        },
                        '}' => {
                            if !allow_without_val {
                                self.tokens.error(ErrorType::Expected(String::from("type")), tok_start, self.tokens.input.loc());
                                continue;
                            }
                            has_consumed_bracket = true;
                            break;
                        },
                        _ => {}
                    }
                },
                None => continue
            };
            if self.tokens.is_next(TokenType::Punc(',')) { self.tokens.consume(); };
        };
        if !has_consumed_bracket { self.tokens.skip_or_err(TokenType::Punc(closing_punc), Some(ErrorType::Expected(closing_punc.to_string())), Some(Range { start, end: self.tokens.input.loc()})); };
        ASTPairListTyping {
            range: Range { start, end: self.tokens.input.loc() },
            pairs: res
        }
    }

    fn parse_typing_list(&mut self, only_varnames: bool, allow_fn_keyword: bool) -> ASTListTyping {
        let start = self.tokens.input.loc();
        let mut res: Vec<ASTTypings> = vec![];
        while !self.tokens.input.is_eof() && !self.tokens.is_next(TokenType::Op(String::from(">"))) {
            let id_start = self.tokens.input.loc();
            let maybe_typing = self.parse_typing(allow_fn_keyword);
            if maybe_typing.is_none() { break; };
            let typing = maybe_typing.unwrap();
            if only_varnames {
            match &typing {
                ASTTypings::Var(v) => {
                    if v.typings.is_some() {
                        self.tokens.error(ErrorType::Unexpected(String::from("token <, generics are not allowed here.")), v.range.start, self.tokens.input.loc());
                    }
                },
                _ => {
                    self.tokens.error(ErrorType::Expected(String::from("generic parameter")), id_start, self.tokens.input.loc());
                }
            }
            }
            res.push(typing);
            if self.tokens.is_next(TokenType::Punc(',')) { self.tokens.consume(); };
        };
        self.tokens.skip_or_err(TokenType::Op(String::from(">")), Some(ErrorType::Expected(String::from(">"))), Some(Range { start, end: self.tokens.input.loc()}));
        ASTListTyping {
            entries: res,
            range: Range { start, end: self.tokens.input.loc() }
        }
    }

    fn parse_function(&mut self, allow_body: bool) -> Option<ASTFunction> {
        let start = self.tokens.input.loc();
        let params = Box::from(self.parse_typing_pair_list(true, false, ')'));
        let return_type = if self.tokens.is_next(TokenType::Op(String::from("->"))) {
            self.tokens.consume();
            let exp = self.parse_typing(false);
            if exp.is_none() { 
                self.tokens.error(ErrorType::Expected(String::from("return type")), self.tokens.input.loc(), self.tokens.input.loc()); 
                return None; 
            };
           Some(Box::from(exp.unwrap()))
        } else { None };
        let body = if allow_body {
            if let Some(e) = self.parse_expression() {
                Some(Box::from(e))
            } else { None }
        } else { None };
        Some(ASTFunction {
            range: Range { start, end: self.tokens.input.loc() },
            params,
            return_type,
            body
        })
    }

    fn parse_expression_part(&mut self) -> Option<ASTExpression> {
        self.is_last_block = false;
        let exp = {
        let token = self.tokens.consume()?;
        match token.val {
            TokenType::Int(value) => Some(ASTExpression::Int(ASTInt { value, range: token.range } )),
            TokenType::Float(value) => Some(ASTExpression::Float(ASTFloat { value, range: token.range })),
            TokenType::Str(value) => Some(ASTExpression::Str(ASTStr { value, range: token.range })),
            TokenType::Char(value) => Some(ASTExpression::Char(ASTChar { value, range: token.range })),
            TokenType::Var(value) => {
                if self.tokens.is_next(TokenType::Op(String::from("<"))) {
                    self.tokens.consume();
                    let typings = Some(self.parse_typing_list(false, false));
                    if self.tokens.skip_or_err(TokenType::Punc('{'), Some(ErrorType::Expected(String::from("initializor"))), None) { return None };
                    return Some(ASTExpression::Init(ASTInitializor{
                        target: ASTVar { value, range: token.range },
                        typings,
                        params: self.parse_pair_list(true, '}'),
                        range: Range { start: token.range.start, end: self.tokens.input.loc() }
                    }));
                } else if self.tokens.is_next(TokenType::Punc(':')) {
                    self.tokens.consume();
                    let target = self.parse_varname(false, true).0;
                    if target.is_none() {
                        self.tokens.error(ErrorType::Expected(String::from("enum identifier")), self.tokens.input.loc(), self.tokens.input.loc());
                        return None;
                    }
                    let init = if self.tokens.is_next(TokenType::Punc('(')) {
                        let exp = self.parse_expression();
                        if let Some(t) = exp {
                            Some(Box::from(t))
                        } else { None }
                    } else { None };
                    return Some(ASTExpression::EnumAccess(
                        ASTEnumAccess {
                            value: ASTVar { value, range: token.range },
                            target: target.unwrap(),
                            init_value: init,
                            range: Range { start: token.range.start, end: self.tokens.input.loc() }
                        }
                    ))
                }
                Some(ASTExpression::Var(ASTVar { value, range: token.range }))
            },
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
                        if self.tokens.is_next(TokenType::Punc(')')) {
                            self.tokens.error(ErrorType::Unexpected(String::from("empty expression")), self.tokens.input.loc(), self.tokens.input.loc());
                            return None;
                        };
                        let exp = self.parse_expression();
                        self.tokens.skip_or_err(TokenType::Punc(')'), Some(ErrorType::Expected(String::from("end of wrapped expression"))), None);
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
                        let name = self.parse_varname(true, false);
                        if name.0.is_none() {
                            self.tokens.error(ErrorType::Expected("variable name".to_string()), token.range.start, token.range.end);
                            return None;
                        }
                        let typings = if let Some(mut typing) = name.1 {
                            let len = typing.entries.len();
                            if len == 0 {
                                self.tokens.error(ErrorType::Expected(String::from("at least one type")), token.range.start, self.tokens.input.loc());
                                None
                            }
                            else if len > 1 {
                                self.tokens.error(ErrorType::TooMuchTypes(1), token.range.start, self.tokens.input.loc());
                                None
                            } else { Some(typing.entries.remove(0)) }
                        } else { None };
                        if self.tokens.is_next(TokenType::Op("=".to_string())) {
                            let equals = self.tokens.consume().unwrap(); // Skip =
                            let exp = self.parse_expression();
                            return match exp {
                                    Some(expression) => {
                                        let exp_end = utils::full_expression_range(&expression).end;
                                        Some(ASTExpression::Let(
                                            ASTLet {
                                                var: name.0.unwrap(),
                                                typings,
                                                value: Some(Box::from(expression)),
                                                range: Range { start: token.range.start, end: exp_end }
                                            }
                                        ))
                                    },
                                    None => {
                                        self.tokens.error(ErrorType::Expected("initializer".to_string()), equals.range.end, equals.range.end);
                                        return None;
                                        }
                                    }
                            };
                                let varname = name.0.unwrap();
                                let end = varname.range.end;
                                return Some(ASTExpression::Let(
                                    ASTLet {
                                        var: varname,
                                        typings,
                                        value: None,
                                        range: Range { start: token.range.start, end }
                                    }
                            ))
                        },
                    "fn" => {
                        if self.tokens.skip_or_err(TokenType::Punc('('), Some(ErrorType::Expected(String::from("start of function params"))), None) { return None };
                        Some(ASTExpression::Function(self.parse_function(true)?))
                    },
                    "if" => {
                        let start = self.tokens.input.loc();
                        let condition = if let Some(cond) = self.parse_expression() {
                             Box::from(cond)
                        } else {
                            self.tokens.error(ErrorType::Expected(String::from("condition in if expression")), start, self.tokens.input.loc());
                             return None;
                        };
                        let then = if let Some(th) = self.parse_expression() {
                             Box::from(th)
                         } else {
                            self.tokens.error(ErrorType::Expected(String::from("expression that will be executed if the condition is true")), start, self.tokens.input.loc());
                            return None;
                         };
                         let otherwise = if self.tokens.is_next(TokenType::Kw(String::from("else"))) {
                             self.tokens.consume();
                             if let Some(exp) = self.parse_expression() {
                                 Some(Box::from(exp))
                             } else { None }
                         } else { None };
                        return Some(ASTExpression::If(
                            ASTIf {
                                condition,
                                then,
                                otherwise,
                                range: Range { start, end: self.tokens.input.loc() }
                            }
                        ))
                    },
                    _ => {
                        self.tokens.error(ErrorType::ExpectedFound("expression".to_string(), format!("keyword \"{}\"", val)), token.range.start, token.range.end);
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
        let start = self.tokens.input.loc();
        let token = self.tokens.consume()?;
        match token.val {
            TokenType::Kw(keyword) => {
                match keyword.as_str() {
                   "struct" => {
                        let name = self.parse_varname(true, true);
                        if name.0.is_none() { 
                            self.tokens.error(ErrorType::Expected("struct name".to_string()), token.range.start, token.range.end);
                            return None;
                        }
                        if self.tokens.skip_or_err(TokenType::Punc('{'), Some(ErrorType::Expected(String::from("start of struct fields"))), None) { return None; };
                        Some(ASTStatement::Struct(ASTStruct {
                            name: name.0.unwrap(),
                            typings: name.1,
                            fields: self.parse_typing_pair_list(false, true, '}'),
                            range: Range { start, end: self.tokens.input.loc() }
                        }))
                   }
                   "enum" => {
                    if let Some(tok) = self.tokens.consume() {
                        if let TokenType::Var(name) = tok.val {
                            if self.tokens.skip_or_err(TokenType::Punc('{'), Some(ErrorType::Expected(String::from("start of enum fields"))), None) { return None; };
                            return Some(ASTStatement::EnumDeclaration(ASTEnumDeclaration {
                                name,
                                values: self.parse_typing_pair_list(true, false, '}'),
                                range: Range { start, end: self.tokens.input.loc() }
                            }));
                        };
                    };
                    self.tokens.error(ErrorType::Expected("enum name".to_string()), token.range.start, token.range.end);
                    None
                   },
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

    pub fn parse(&mut self) -> Vec<ASTStatement> {
        let mut res = vec![];
        while !self.tokens.input.is_eof() {
            let parsed_statement = self.parse_statement();
            if let Some(stm) = parsed_statement { res.push(stm) }
        }
        res
    }

}