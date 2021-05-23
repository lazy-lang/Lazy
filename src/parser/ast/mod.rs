
use super::tokenizer::{Tokenizer, TokenType, Range};
use super::tokenizer::error::ErrorType;
use super::input_parser::LoC;
pub mod model;
pub mod utils;
use model::*;


pub struct Parser {
    pub tokens: Tokenizer,
    is_last_block: bool,
    allow_exp_statements: bool,
    parsed_main: bool
}

impl Parser {

    pub fn new(source: &str) -> Self {
        Parser {
            tokens: Tokenizer::new(source),
            parsed_main: false,
            is_last_block: false,
            allow_exp_statements: false
        }
    }

    pub fn reset(&mut self, source: &str) {
        self.tokens = Tokenizer::new(source);
        self.parsed_main = false;
        self.is_last_block = false;
        self.allow_exp_statements = false;
    }

    fn get_prec(op: &str) -> i8 {
        match op {
            "=" | "+=" | "-=" | "*=" | "/=" | "%=" => 1,
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
                    return Some(left_tok)
                }
                if other_prec > prec {
                    let err_start = value.range.start;
                    let err_end = value.range.end;
                    self.tokens.consume();
                    let exp = self.parse_expression_part(false);
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

    fn parse_suffix(&mut self, token: Option<ASTExpression>, parse_generics: bool) -> Option<ASTExpression> {
        if token.is_none() { return token };
        let start = self.tokens.input.loc();
        let next_token = self.tokens.peek();
        if next_token.is_none() { return token };
        match &next_token.unwrap().val {
            TokenType::Op(val) => {
                let cloned = val.clone();
                match val.as_str() {
                    "." => {
                        self.tokens.consume();
                        let target = self.parse_varname(true, false, true);
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
                        )), parse_generics)
                    },
                    "?" => {
                        self.tokens.consume();
                        self.parse_suffix(Some(ASTExpression::Optional(
                            ASTOptional {
                                value: Box::from(token.unwrap()),
                                range: Range { start, end: self.tokens.input.loc() }
                            }
                        )), parse_generics)
                    },
                    ".." | "..=" => {
                        self.tokens.consume();
                        let end = self.parse_expression_part(true);
                        if end.is_none() {
                            self.tokens.error(ErrorType::EndOfIterator, start, self.tokens.input.loc());
                            return None;
                        }
                        Some(ASTExpression::Iterator(
                            ASTIterator {
                                start: Box::from(token.unwrap()),
                                end: Box::from(end.unwrap()),
                                inclusive: cloned == "..=",
                                range: Range { start, end: self.tokens.input.loc() }
                            }
                        ))
                    },
                    "<" if parse_generics => {
                        self.tokens.consume();
                        let type_list = self.parse_typing_list(false, false, TokenType::Op(String::from(">")));
                        if self.tokens.is_next(TokenType::Punc('(')) {
                            self.tokens.consume();
                            let args = self.parse_expression_list(')');
                            self.parse_suffix(Some(ASTExpression::Call(
                                ASTCall {
                                    target: Box::from(token.unwrap()),
                                    typings: Some(type_list),
                                    args,
                                    range: Range { start, end: self.tokens.input.loc() }
                                }
                            )), parse_generics)
                        } else {
                            None
                        }
                    },
                    _ => token
                }
            },
            TokenType::Punc(punc) => {
                match punc {
                    '(' => {
                        self.tokens.consume();
                        let args = self.parse_expression_list(')');
                        self.parse_suffix(Some(ASTExpression::Call(
                            ASTCall {
                                target: Box::from(token.unwrap()),
                                typings: None,
                                args,
                                range: Range { start, end: self.tokens.input.loc() }
                            }
                        )), parse_generics)
                    },
                    ':' => {
                        self.tokens.consume();
                        self.tokens.skip_or_err(TokenType::Punc(':'), None, None);
                        let val = match token.unwrap() {
                            ASTExpression::Var(v) => ASTModAccessValues::Var(Box::from(v)),
                            ASTExpression::ModAccess(ma) => ASTModAccessValues::ModAccess(Box::from(ma)),
                            _ => {
                                self.tokens.error(ErrorType::Expected(String::from("identifier")), start, self.tokens.input.loc());
                                return None;
                            }
                        };
                        let variant_name = self.parse_varname(false, false, false);
                        if variant_name.0.is_none() {
                            self.tokens.error(ErrorType::Expected(String::from("identifier")), start, self.tokens.input.loc());
                            return None;
                        }
                        self.parse_suffix(Some(ASTExpression::ModAccess(
                            ASTModAccess {
                                value: val,
                                target: variant_name.0.unwrap(),
                                range: Range { start, end: self.tokens.input.loc() }
                                }
                            )), parse_generics)
                    }
                    _ => token
                }
            }
            _ => token
        }
    }

    fn parse_typing(&mut self, allow_fn_keyword: bool, allow_optional_after_var: bool) -> Option<ASTTypings> {
        let start = self.tokens.input.loc();
        let maybe_token = self.tokens.peek();
        let t = match maybe_token {
            Some(token) => {
                match &token.val {
                    TokenType::Punc('{') => {
                        self.tokens.consume();
                        Some(ASTTypings::PairList(self.parse_typing_pair_list(false, allow_fn_keyword, false, false, '}')))
                    },
                    TokenType::Punc('(') => {
                        self.tokens.consume();
                        let params = Box::from(self.parse_typing_pair_list(false, allow_fn_keyword, true, false, ')'));
                        let return_type = if self.tokens.is_next(TokenType::Op(String::from("->"))) { 
                            self.tokens.consume(); 
                            let typing = self.parse_typing(allow_fn_keyword, true);
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
                            typings: None,
                            body: None
                        }))
                    },
                    TokenType::Punc('[') => {
                        self.tokens.consume();
                        let values = self.parse_typing_list(false, false, TokenType::Punc(']'));
                        Some(ASTTypings::Tuple(values))
                    },
                    TokenType::Var(name) => {
                        let value = name.to_string();
                        self.tokens.consume();
                        let typings = if self.tokens.is_next(TokenType::Op(String::from("<"))) {
                            self.tokens.consume(); // Skip <
                            Some(self.parse_typing_list(false, allow_fn_keyword, TokenType::Op(String::from(">"))))
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
            None => None
        };
        if allow_optional_after_var && t.is_some() {
            if self.tokens.is_next(TokenType::Op(String::from("?"))) {
                self.tokens.consume();
                Some(ASTTypings::Optional(Box::from(t.unwrap())))
            } else { t }
        } else { t }
    }

    fn parse_block(&mut self, allow_statement_as_exp: bool) -> ASTBlock {
        let start = self.tokens.input.loc();
        let mut res: Vec<ASTExpression> = vec![];
        while !self.tokens.input.is_eof() && !self.tokens.is_next(TokenType::Punc('}')) {
            let exp = if allow_statement_as_exp { self.parse_expression_or_expression_statement() } else { self.parse_expression() };
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

    fn parse_varname(&mut self, allow_generics: bool, only_varnames_as_generics: bool, allow_ints: bool) -> (Option<ASTVar>, Option<ASTListTyping>) {
        if allow_ints { self.tokens.is_last_num_as_str = true }; 
        let next = self.tokens.consume();
        if next.is_none() { 
            self.tokens.error(ErrorType::Expected(String::from("itentifier")), self.tokens.input.loc(), self.tokens.input.loc());
            return (None, None);
        };
        let unwrapped = next.unwrap();
        let var = match unwrapped.val {
            TokenType::Var(v) => ASTVar { value: v, range: unwrapped.range },
            TokenType::Int(i) if allow_ints => ASTVar { value: i.to_string(), range: unwrapped.range },
            _ => {
                self.tokens.error(ErrorType::ExpectedFound(String::from("identifier"), unwrapped.val.to_string()), unwrapped.range.start, unwrapped.range.end);
                return (None, None);
            }
        };
        if self.tokens.is_next(TokenType::Op(String::from("<"))) {
            if !allow_generics {
                return (Some(var), None);
            }
            self.tokens.consume();
            return (Some(var), Some(self.parse_typing_list(only_varnames_as_generics, false, TokenType::Op(String::from(">")))));
        }
        (Some(var), None)
    }

    fn parse_pair_list(&mut self, allow_without_val: bool, closing_punc: char) -> ASTPairList {
        let start = self.tokens.input.loc();
        let mut res: Vec<(String, Option<ASTExpression>)> = vec![];
        let mut has_consumed_bracket = false;
        while !self.tokens.is_next(TokenType::Punc(closing_punc)) {
            let tok_start = self.tokens.input.loc();
            let key = self.parse_varname(false, false, false);
            if key.0.is_none() { continue; };
            match self.tokens.expect_punc(&[',', ':', closing_punc], Some(Range { start: tok_start, end: self.tokens.input.loc()})) {
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
                        ch if ch == closing_punc => {
                            if !allow_without_val {
                                self.tokens.error(ErrorType::Expected(String::from("typeing")), tok_start, self.tokens.input.loc());
                                continue;
                            }
                            has_consumed_bracket = true;
                            res.push((key.0.unwrap().value, None));
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

    fn parse_expression_list(&mut self, closing_punc: char) -> ASTExpressionList {
        let start = self.tokens.input.loc();
        let mut expressions: Vec<ASTExpression> = vec![];
        while !self.tokens.is_next(TokenType::Punc(closing_punc)) {
            let exp = self.parse_expression();
            if exp.is_none() { break; };
            expressions.push(exp.unwrap());
            if !self.tokens.is_next(TokenType::Punc(closing_punc)) { self.tokens.skip_or_err(TokenType::Punc(','), None, None); };
        };
        self.tokens.skip_or_err(TokenType::Punc(closing_punc), None, None);
        ASTExpressionList {
            expressions,
            range: Range { start, end: self.tokens.input.loc() }
        }
    }

    fn parse_typing_pair_list(&mut self, allow_without_val: bool, allow_fn_keyword: bool, allow_spread: bool, allow_modifiers: bool, closing_punc: char) -> ASTPairListTyping {
        let start = self.tokens.input.loc();
        let mut res: Vec<ASTPairTypingItem> = vec![];
        let mut has_consumed_bracket = false;
        let mut modifiers = ASTModifiers::empty();
        while !self.tokens.is_next(TokenType::Punc(closing_punc)) {
            let tok_start = self.tokens.input.loc();
            let is_spread = if self.tokens.is_next(TokenType::Op(String::from("..."))) {
                if !allow_spread {
                    self.tokens.error(ErrorType::Disallowed(String::from("spread operator")), self.tokens.input.loc_inc(-3, 0), self.tokens.input.loc())
                }
                self.tokens.consume();
                true
            } else { false };
            if allow_modifiers {
                if let Some(t) = self.tokens.peek() {
                    let range_start = t.range.start;
                    if let TokenType::Kw(kw) = &t.val {
                        match kw.as_str() {
                            "const" => {
                                self.tokens.consume();
                                if modifiers.contains(ASTModifiers::CONST) {
                                    self.tokens.error(ErrorType::AlreadyHasModifier(String::from("const")), range_start, self.tokens.input.loc());
                                };
                                modifiers.insert(ASTModifiers::CONST);
                                continue;
                            },
                            "static" => {
                                self.tokens.consume();
                                if modifiers.contains(ASTModifiers::STATIC) {
                                    self.tokens.error(ErrorType::AlreadyHasModifier(String::from("static")), range_start, self.tokens.input.loc());
                                };
                                modifiers.insert(ASTModifiers::STATIC);
                                continue;
                            },
                            "private" => {
                                self.tokens.consume();
                                if modifiers.contains(ASTModifiers::PRIVATE) {
                                    self.tokens.error(ErrorType::AlreadyHasModifier(String::from("private")), range_start, self.tokens.input.loc());
                                };
                                modifiers.insert(ASTModifiers::PRIVATE);
                                continue;
                            },
                            _ => {}
                        }
                    }
                }
            };
            let key = self.parse_varname(false, false, false);
            if key.0.is_none() { continue };
            let is_optional = if self.tokens.is_next(TokenType::Op(String::from("?"))) {
                self.tokens.consume();
                true
            } else { false };
            match self.tokens.expect_punc(&[',', ':', '?', closing_punc], Some(Range { start: tok_start, end: self.tokens.input.loc()})) {
                Some(ch) => {
                    match ch {
                        ',' => {
                            if !allow_without_val {
                                self.tokens.error(ErrorType::Expected(String::from("type")), tok_start, self.tokens.input.loc());
                                continue;
                            }
                            res.push(ASTPairTypingItem {name: key.0.unwrap().value, value: None, optional: is_optional, modifiers, spread: is_spread});
                            modifiers.clear();
                        },
                        ':' => {
                            let exp = self.parse_typing(allow_fn_keyword, false);
                            if exp.is_none() { 
                                self.tokens.error(ErrorType::Expected(String::from("expression")), tok_start, self.tokens.input.loc());
                                continue;
                            }
                            res.push(ASTPairTypingItem { name: key.0.unwrap().value, value: exp, optional: is_optional, modifiers, spread: is_spread});
                            modifiers.clear();
                        },
                        ch if ch == closing_punc => {
                            if !allow_without_val {
                                self.tokens.error(ErrorType::Expected(String::from("type")), tok_start, self.tokens.input.loc());
                                continue;
                            }
                            has_consumed_bracket = true;
                            res.push(ASTPairTypingItem { name: key.0.unwrap().value, value: None, optional: is_optional, modifiers, spread: is_spread});
                            modifiers.clear();
                            break;
                        },
                        _ => {}
                    }
                },
                None => continue
            };
            if self.tokens.is_next(TokenType::Punc(',')) { self.tokens.consume(); };
        };
        if !has_consumed_bracket { self.tokens.skip_or_err(TokenType::Punc(closing_punc), None, None); };
        ASTPairListTyping {
            range: Range { start, end: self.tokens.input.loc() },
            pairs: res
        }
    }

    fn parse_typing_list(&mut self, only_varnames: bool, allow_fn_keyword: bool, closing_tok: TokenType) -> ASTListTyping {
        let start = self.tokens.input.loc();
        let mut res: Vec<ASTTypings> = vec![];
        while !self.tokens.is_next(closing_tok.clone()) {
            let id_start = self.tokens.input.loc();
            let maybe_typing = self.parse_typing(allow_fn_keyword, false);
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
        let closing_tok_str = closing_tok.to_string();
        self.tokens.skip_or_err(closing_tok, Some(ErrorType::Expected(closing_tok_str)), Some(Range { start, end: self.tokens.input.loc()}));
        ASTListTyping {
            entries: res,
            range: Range { start, end: self.tokens.input.loc() }
        }
    }


    fn parse_function(&mut self, allow_body: bool) -> Option<ASTFunction> {
        let start = self.tokens.input.loc();
        let typings = if self.tokens.is_next(TokenType::Op(String::from("<"))) {
            self.tokens.consume();
            Some(self.parse_typing_list(true, false, TokenType::Op(String::from(">"))))
        } else { None };
        if self.tokens.skip_or_err(TokenType::Punc('('), Some(ErrorType::Expected(String::from("start of function params"))), None) { return None };
        let params = Box::from(self.parse_typing_pair_list(true, false, true, false, ')'));
        let return_type = if self.tokens.is_next(TokenType::Op(String::from("->"))) {
            self.tokens.consume();
            let exp = self.parse_typing(false, true);
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
            typings,
            return_type,
            body
        })
    }

    fn parse_match_arm_exp(&mut self) -> Option<ASTMatchArmExpressions> {
        let start = self.tokens.input.loc();
        if let Some(exp) = self.parse_expression_part(false) {
            match exp {
                ASTExpression::Str(str_obj) => Some(ASTMatchArmExpressions::String(str_obj)),
                ASTExpression::Int(int_obj) => Some(ASTMatchArmExpressions::Int(int_obj)),
                ASTExpression::Float(f_obj) => Some(ASTMatchArmExpressions::Float(f_obj)),
                ASTExpression::Bool(b_obj) => Some(ASTMatchArmExpressions::Bool(b_obj)),
                ASTExpression::Tuple(t_obj) => {
                    if !utils::is_natural_tuple(&t_obj) {
                        self.tokens.error(ErrorType::Expected(String::from("natural tuple literal")), start, self.tokens.input.loc());
                    }
                    Some(ASTMatchArmExpressions::Tuple(t_obj))
                },
                ASTExpression::Iterator(i_obj) => {
                    if !utils::is_natural_iter(&i_obj) {
                        self.tokens.error(ErrorType::Expected(String::from("natural iterator literal")), start, self.tokens.input.loc());
                    }
                    Some(ASTMatchArmExpressions::Iterator(i_obj))
                },
                ASTExpression::None(r) => Some(ASTMatchArmExpressions::None(r)),
                ASTExpression::Var(v) => {
                    if v.value != "_" {
                        self.tokens.error(ErrorType::Unexpected(String::from("variable name")), start, self.tokens.input.loc());
                    };
                    Some(ASTMatchArmExpressions::Rest)
                },
                ASTExpression::ModAccess(acc) => Some(ASTMatchArmExpressions::Enum(acc)),
                _ => {
                    self.tokens.error(ErrorType::WrongMatchArmExp, start, self.tokens.input.loc());
                    None
                }
            }
        } else {
            self.tokens.error(ErrorType::Expected(String::from("match arm expression")), start, self.tokens.input.loc());
            None
        }
    }

    fn parse_expression_part(&mut self, parse_generics_in_suffix: bool) -> Option<ASTExpression> {
        self.is_last_block = false;
        let exp = {
        let token = self.tokens.consume()?;
        match token.val {
            TokenType::Int(value) => Some(ASTExpression::Int(ASTInt { value, range: token.range } )),
            TokenType::Float(value) => Some(ASTExpression::Float(ASTFloat { value, range: token.range })),
            TokenType::Str(value) => Some(ASTExpression::Str(ASTStr { value, range: token.range })),
            TokenType::Char(value) => Some(ASTExpression::Char(ASTChar { value, range: token.range })),
            TokenType::None => Some(ASTExpression::None(token.range)),
            TokenType::Var(value) => Some(ASTExpression::Var(ASTVar { value, range: token.range })),
            TokenType::Bool(value) => Some(ASTExpression::Bool(ASTBool { value, range: token.range })),
            TokenType::Op(value) => {
                // Prefixes
                match value.as_str() {
                    "-" | "!" => {
                        Some(ASTExpression::Unary(
                            ASTUnary {
                                op: value,
                                value: Box::from(self.parse_expression_part(parse_generics_in_suffix)?),
                                range: token.range
                            }
                        ))
                    },
                    "..." => {
                        Some(ASTExpression::Spread(
                            ASTSpread {
                                value: Box::from(self.parse_expression()?),
                                range: Range { start: token.range.start, end: self.tokens.input.loc() }
                            }
                        ))
                    }
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
                    '{' => Some(ASTExpression::Block(self.parse_block(true))),
                    '[' => {
                        if self.tokens.is_next(TokenType::Punc(']')) {
                            self.tokens.error(ErrorType::Unexpected(String::from("empty tuple")), self.tokens.input.loc(), self.tokens.input.loc());
                            return None;
                        };
                        let expressions = self.parse_expression_list(']');
                        Some(ASTExpression::Tuple(expressions))
                    },
                    _ => {
                        self.tokens.error(ErrorType::UnexpectedPunc(val), token.range.start, token.range.end);
                        None
                    }
                }
            },
            TokenType::Kw(val) => {
                match val.as_str() {
                    "let" | "const" => {
                        let name = self.parse_varname(true, false, false);
                        if name.0.is_none() {
                            self.tokens.error(ErrorType::Expected("variable name".to_string()), token.range.start, token.range.end);
                            return None;
                        }
                        let varname = name.0.unwrap();
                        let mut end = varname.range.end;
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
                        let is_const = val.as_str() == "const";
                        let value = if self.tokens.is_next(TokenType::Op("=".to_string())) {
                            let equals = self.tokens.consume().unwrap(); // Skip =
                            let exp = self.parse_expression();
                            match exp {
                                Some(e) => {
                                    end = utils::full_expression_range(&e).end;
                                    Some(Box::from(e))
                                },
                                None => {
                                    self.tokens.error(ErrorType::Expected(String::from("initializor")), token.range.start, equals.range.end);
                                    None
                                }
                            }
                        } else { 
                            if is_const {
                                self.tokens.error(ErrorType::ConstantWithoutInit, token.range.start, end);
                            }
                            None
                         };
                        return Some(ASTExpression::Declare(
                            ASTDeclare {
                                var: varname,
                                is_const,
                                typings,
                                value,
                                range: Range { start: token.range.start, end }
                            }
                        ))
                        },
                    "fn" => {
                        Some(ASTExpression::Function(self.parse_function(true)?))
                    },
                    "if" => {
                        let condition = if let Some(cond) = self.parse_expression() {
                             Box::from(cond)
                        } else {
                            self.tokens.error(ErrorType::Expected(String::from("condition in if expression")), token.range.start, self.tokens.input.loc());
                             return None;
                        };
                        let then = if let Some(th) = self.parse_expression_or_expression_statement() {
                             Box::from(th)
                         } else {
                            self.tokens.error(ErrorType::Expected(String::from("expression that will be executed if the condition is true")), token.range.start, self.tokens.input.loc());
                            return None;
                         };
                         let otherwise = if self.tokens.is_next(TokenType::Kw(String::from("else"))) {
                             self.tokens.consume();
                             if let Some(exp) = self.parse_expression_or_expression_statement() {
                                 Some(Box::from(exp))
                             } else { None }
                         } else { None };
                        return Some(ASTExpression::If(
                            ASTIf {
                                condition,
                                then,
                                otherwise,
                                range: Range { start: token.range.start, end: self.tokens.input.loc() }
                            }
                        ))
                    },
                    "for" => {
                        let var = self.parse_varname(false, false, false).0;
                        if var.is_none() {
                            self.tokens.error(ErrorType::Expected(String::from("identifier")), self.tokens.input.loc(), self.tokens.input.loc());
                            return None;
                        };
                        if self.tokens.skip_or_err(TokenType::Kw(String::from("in")), None, None) { return None; };
                        let iterator = self.parse_expression();
                        if iterator.is_none() {
                            self.tokens.error(ErrorType::Expected(String::from("iterator")), self.tokens.input.loc(), self.tokens.input.loc());
                            return None;
                        }
                        let turn_off_exp_statements = !self.allow_exp_statements;
                        self.allow_exp_statements = true;
                        let body = self.parse_expression_or_expression_statement();
                        if body.is_none() {
                            self.tokens.error(ErrorType::Expected(String::from("for...in loop body")), self.tokens.input.loc(), self.tokens.input.loc());
                            return None;
                        }
                        if turn_off_exp_statements { self.allow_exp_statements = false; }
                        return Some(ASTExpression::ForIn(
                            ASTForIn {
                                var: var.unwrap(),
                                iterable: Box::from(iterator.unwrap()),
                                body: Box::from(body.unwrap()),
                                range: Range { start: token.range.start, end: self.tokens.input.loc() }
                            }
                        ))
                    },
                    "while" => {
                        let cond = self.parse_expression();
                        if cond.is_none() {
                            self.tokens.error(ErrorType::Expected(String::from("while condition")), self.tokens.input.loc(), self.tokens.input.loc());
                            return None;
                        }
                        let turn_off_exp_statements = !self.allow_exp_statements;
                        self.allow_exp_statements = true;
                        let body = self.parse_expression_or_expression_statement();
                        if body.is_none() {
                            self.tokens.error(ErrorType::Expected(String::from("while body")), self.tokens.input.loc(), self.tokens.input.loc());
                            return None;
                        }
                        if turn_off_exp_statements { self.allow_exp_statements = false; }
                        return Some(ASTExpression::While(
                            ASTWhile {
                                condition: Box::from(cond.unwrap()),
                                body: Box::from(body.unwrap()),
                                range: Range { start: token.range.start, end: self.tokens.input.loc() }
                            }
                        ))
                    },
                    "match" => {
                        let to_get_matched = self.parse_expression();
                        if to_get_matched.is_none() {
                            self.tokens.error(ErrorType::Expected(String::from("expression to get matched")), self.tokens.input.loc(), self.tokens.input.loc());
                            return None;
                        };
                        self.tokens.skip_or_err(TokenType::Punc('{'), None, None);
                        let mut arms: Vec<ASTMatchArm> = vec![];
                        while !self.tokens.is_next(TokenType::Punc('}')) {
                            let match_arm_start = self.tokens.input.loc();
                            let mut possibilities: Vec<ASTMatchArmExpressions> = vec![];
                            possibilities.push(self.parse_match_arm_exp()?);
                            if self.tokens.is_next(TokenType::Op(String::from("|"))) {
                                self.tokens.consume();
                                while !self.tokens.is_next(TokenType::Op(String::from("=>"))) && !self.tokens.is_next(TokenType::Kw(String::from("when")))  {
                                    possibilities.push(self.parse_match_arm_exp()?);
                                    if self.tokens.is_next(TokenType::Op(String::from("|"))) { self.tokens.consume(); };
                                }
                            }
                            let guard = if self.tokens.is_next(TokenType::Kw(String::from("when"))) {
                                self.tokens.consume();
                                self.parse_expression()
                            } else { None };

                            self.tokens.skip_or_err(TokenType::Op(String::from("=>")), None, None);

                            let body = self.parse_expression();
                            if body.is_none() {
                                self.tokens.error(ErrorType::Expected(String::from("match arm body")), match_arm_start, self.tokens.input.loc());
                                return None;
                            }
                            if self.tokens.is_next(TokenType::Punc(',')) { self.tokens.consume(); };
                            arms.push(ASTMatchArm {
                                guard,
                                possibilities,
                                body: body.unwrap(),
                                range: Range { start: match_arm_start, end: self.tokens.input.loc() }
                            });
                        }
                        self.tokens.skip_or_err(TokenType::Punc('}'), None, None);
                        self.is_last_block = true;
                        Some(ASTExpression::Match(ASTMatch {
                            arms,
                            range: Range { start: token.range.start, end: self.tokens.input.loc() },
                            expression: Box::from(to_get_matched.unwrap())
                        }))
                    },
                    "new" => {
                        let target = if let Some(t) = self.parse_expression_part(false) {
                            match t {
                                ASTExpression::Var(v) => ASTModAccessValues::Var(Box::from(v)),
                                ASTExpression::ModAccess(v) => ASTModAccessValues::ModAccess(Box::from(v)),
                                _ => {
                                    self.tokens.error(ErrorType::ExpectedFound(String::from("struct identifier"), t.to_string()), token.range.start, self.tokens.input.loc());
                                    return None;
                                }
                            }
                        } else {
                            self.tokens.error(ErrorType::Expected(String::from("struct identifier")), token.range.start, self.tokens.input.loc());
                            return None;
                        };
                        let typings = if self.tokens.is_next(TokenType::Op(String::from("<"))) {
                            self.tokens.consume();
                            Some(self.parse_typing_list(false, false, TokenType::Op(String::from(">"))))
                        } else { None };
                        self.tokens.skip_or_err(TokenType::Punc('{'), Some(ErrorType::Expected(String::from("struct initializor"))), None);
                        Some(ASTExpression::Init(
                            ASTInitializor {
                                target,
                                params: self.parse_pair_list(true, '}'),
                                typings,
                                range: Range { start: token.range.start, end: self.tokens.input.loc() }
                            }
                        ))
                    }
                    _ => {
                        self.tokens.error(ErrorType::ExpectedFound("expression".to_string(), format!("keyword \"{}\"", val)), token.range.start, token.range.end);
                        None
                    }
                }
            }
        }
        };
        self.parse_suffix(exp, parse_generics_in_suffix)
    }

    fn parse_expression(&mut self) -> Option<ASTExpression> {
        let exp = self.parse_expression_part(true);
        self.parse_binary(exp, 0)
    }

    fn parse_expression_or_expression_statement(&mut self) -> Option<ASTExpression> {
        let start = self.tokens.input.loc();
        let thing = self.tokens.peek()?;
        match &thing.val {
            TokenType::Kw(kw) => {
                match kw.as_str() {
                    "yield" => { 
                        self.tokens.consume();
                        if !self.allow_exp_statements {
                            self.tokens.error(ErrorType::Unexpected(String::from("yield expression")), start, self.tokens.input.loc());
                            return None;
                        }
                        let value = if let Some(exp) = self.parse_expression() {
                            Some(Box::from(exp))
                        } else { None };
                        Some(ASTExpression::Yield(ASTYield {
                            value,
                            range: Range { start, end: self.tokens.input.loc() }
                        }))
                     },
                    _ => self.parse_expression()
                }
            },
            _ => self.parse_expression()
        }
    }

    fn parse_statement(&mut self) -> Option<ASTStatement> {
        let start = self.tokens.input.loc();
        let token = self.tokens.consume()?;
        match token.val {
            TokenType::Kw(keyword) => {
                match keyword.as_str() {
                   "struct" => {
                        let name = self.parse_varname(true, true, false);
                        if name.0.is_none() { 
                            self.tokens.error(ErrorType::Expected(String::from("struct name")), token.range.start, token.range.end);
                            return None;
                        }
                        if self.tokens.skip_or_err(TokenType::Punc('{'), Some(ErrorType::Expected(String::from("start of struct fields"))), None) { return None; };
                        Some(ASTStatement::Struct(ASTStruct {
                            name: name.0.unwrap(),
                            typings: name.1,
                            fields: self.parse_typing_pair_list(false, true, false, true, '}'),
                            range: Range { start, end: self.tokens.input.loc() }
                        }))
                   }
                   "enum" => {
                    let name = self.parse_varname(true, true, false);
                    if name.0.is_none() { 
                        self.tokens.error(ErrorType::Expected(String::from("struct name")), token.range.start, token.range.end);
                        return None;
                    }
                    if self.tokens.skip_or_err(TokenType::Punc('{'), Some(ErrorType::Expected(String::from("start of enum variants"))), None) { return None; };
                    Some(ASTStatement::EnumDeclaration(ASTEnumDeclaration {
                    name: name.0.unwrap(),
                    values: self.parse_typing_pair_list(true, false, false, false, '}'),
                    typings: name.1,
                    range: Range { start, end: self.tokens.input.loc() }
                    }))
                   },
                   "type" => {
                       let name = self.parse_varname(true, true, false);
                       if name.0.is_none() {
                        self.tokens.error(ErrorType::Expected(String::from("type name")), self.tokens.input.loc(), self.tokens.input.loc());
                        return None;
                       }
                       if self.tokens.skip_or_err(TokenType::Op(String::from("=")), None, None) { return None; };
                       let typing = self.parse_typing(false, false);
                       if typing.is_none() {
                        self.tokens.error(ErrorType::Expected(String::from("typing")), self.tokens.input.loc(), self.tokens.input.loc());
                        return None;
                       }
                       Some(ASTStatement::Type(
                           ASTType {
                               name: name.0.unwrap().value,
                               typings: name.1,
                               value: typing.unwrap(),
                               range: Range { start, end: self.tokens.input.loc() }
                           }
                       ))
                   },
                   "main" => {
                       if self.parsed_main {
                           self.tokens.error(ErrorType::ManyEntryPoints, start, self.tokens.input.loc());
                       };
                       self.tokens.skip_or_err(TokenType::Punc('{'), None, None);
                       let exp = self.parse_block(false);
                       self.parsed_main = true;
                       Some(ASTStatement::Main(
                           ASTMain {
                               expression: exp,
                               range: Range { start, end: self.tokens.input.loc() }
                           }
                       ))
                   },
                   "static" => {
                       let varname = self.parse_varname(true, false, false);
                       self.tokens.skip_or_err(TokenType::Op(String::from("=")), None, None);
                       if varname.0.is_none() {
                           self.tokens.error(ErrorType::Expected(String::from("identifier")), start, self.tokens.input.loc());
                           return None;
                       }
                       let typings = if let Some(mut typing) = varname.1 {
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
                       let exp = self.parse_expression();
                       if exp.is_none() {
                        self.tokens.error(ErrorType::Expected(String::from("initializor")), start, self.tokens.input.loc());
                        return None;
                       }
                       Some(ASTStatement::Static(
                           ASTStatic {
                               typings,
                               var: varname.0.unwrap(),
                               value: exp.unwrap(),
                               range: Range { start, end: self.tokens.input.loc() } 
                           }
                       ))
                   },
                   "export" => {
                       let value = if let Some(stm) = self.parse_statement() {
                           if matches!(stm, ASTStatement::Main(_)) {
                               self.tokens.error(ErrorType::Unexpected(String::from("main entry")), start, self.tokens.input.loc());
                               return None;
                           }
                           Box::from(stm)
                       } else { return None };
                       Some(ASTStatement::Export(
                           ASTExport {
                               value,
                               range: Range { start, end: self.tokens.input.loc() }
                           }
                       ))
                   },
                   "import" => {
                       let path_start = self.tokens.input.loc();
                       let path = if let Some(ASTExpression::Str(string)) = self.parse_expression_part(false) {
                           string
                       } else {
                        self.tokens.error(ErrorType::Expected(String::from("path string")), start, path_start);
                        return None;
                       };
                       let as_binding = if self.tokens.is_next(TokenType::Kw(String::from("as"))) {
                           self.tokens.consume();
                           self.parse_varname(false, false, false).0
                       } else { None };
                       Some(ASTStatement::Import(
                           ASTImport {
                               path,
                               _as: as_binding,
                               range: Range { start, end: self.tokens.input.loc() }
                           }
                       ))
                   },
                   _ => {
                    self.tokens.error(ErrorType::Expected(String::from("statement")), token.range.start, self.tokens.input.loc());
                    self.tokens.input.skip_line();
                    None
                },
                }
            },
            TokenType::Punc(';') => {
                self.tokens.error(ErrorType::UnexpectedPunc(';'), token.range.start, self.tokens.input.loc());
                None
            },
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