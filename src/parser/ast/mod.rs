
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
            "&" | "|" | "^" => 2,
            ">>" | "<<" | ">>>" => 3,
            "||" => 5,
            "&&" => 7,
            "<" | ">" | "<=" | ">=" | "==" | "!=" => 10,
            "+" | "-" => 15,
            "*" | "/" | "%" => 20,
            _ => -1
        }
    }

    fn parse_binary(&mut self, left: Option<ASTExpression>, prec: i8) -> Option<ASTExpression> {
        let left_tok = left?;
        let start = self.tokens.recorder();
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
                        range: start.end(&self.tokens)
                    })), prec);
                }
                Some(left_tok)
            },
            _ => Some(left_tok)
        }
    }

    fn parse_suffix(&mut self, token: Option<ASTExpression>, parse_generics: bool) -> Option<ASTExpression> {
        if token.is_none() { return token };
        let recorder = self.tokens.recorder();
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
                            recorder.err(ErrorType::ProperProperty, &mut self.tokens);
                            return None;
                        };
                        self.parse_suffix(Some(ASTExpression::DotAccess(
                            ASTDotAccess {
                                target: target.0.unwrap(),
                                value: Box::from(token.unwrap()),
                                range: recorder.end(&self.tokens)
                            }
                        )), parse_generics)
                    },
                    "?" => {
                        self.tokens.consume();
                        self.parse_suffix(Some(ASTExpression::Optional(
                            ASTOptional {
                                value: Box::from(token.unwrap()),
                                range: recorder.end(&self.tokens)
                            }
                        )), parse_generics)
                    },
                    ".." | "..=" => {
                        self.tokens.consume();
                        let end = self.parse_expression_part(true);
                        if end.is_none() {
                            recorder.err(ErrorType::EndOfIterator, &mut self.tokens);
                            return None;
                        }
                        Some(ASTExpression::Iterator(
                            ASTIterator {
                                start: Box::from(token.unwrap()),
                                end: Box::from(end.unwrap()),
                                inclusive: cloned == "..=",
                                range: recorder.end(&self.tokens)
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
                                    range: recorder.end(&self.tokens)
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
                                range: recorder.end(&self.tokens)
                            }
                        )), parse_generics)
                    },
                    ':' => {
                        if let ASTExpression::Var(v) = token.unwrap() {
                            match self.parse_mod_access_or_var(v, true, true) {
                                ASTModAccessValues::ModAccess(mod_access) => Some(ASTExpression::ModAccess(mod_access)),
                                ASTModAccessValues::Var(v) => Some(ASTExpression::Var(v.value))
                            }
                        } else {
                            recorder.err(ErrorType::Expected(String::from("identifier")), &mut self.tokens);
                            None
                        }
                    }
                    _ => token
                }
            }
            _ => token
        }
    }

    pub fn parse_mod_access_or_var_without_var(&mut self, allow_exp_end: bool, allow_typings: bool) -> Option<ASTModAccessValues> {
        let name = self.parse_varname(false, false, false).0;
        if let Some(v) = name {
            Some(self.parse_mod_access_or_var(v, allow_exp_end, allow_typings))
        } else {
            None
        }
    }

    pub fn parse_mod_access_or_var(&mut self, start: ASTVar, allow_exp_end: bool, allow_typings: bool) -> ASTModAccessValues {
        if !self.tokens.is_next(TokenType::Punc(':')) {
            let r = start.range;
            let typings = if self.tokens.is_next(TokenType::Op(String::from("<"))) {
                if !allow_typings {
                    self.tokens.error_here(ErrorType::Unexpected(String::from("typings")));
                }
                self.tokens.consume();
                Some(self.parse_typing_list(false, false, TokenType::Op(String::from(">"))))
            } else { None };
            return ASTModAccessValues::Var(ASTVarTyping { value: start, range: r, typings });
        };
        let mut path: Vec<ASTVar> = vec![start];
        let start = self.tokens.recorder();
        while self.tokens.is_next(TokenType::Punc(':')) {
            self.tokens.consume();
            self.tokens.skip_or_err(TokenType::Punc(':'), None, None);
            if let Some(tok) = self.tokens.consume() {
                match tok.val {
                    TokenType::Var(v) => {
                        path.push(ASTVar { value: v, range: tok.range });
                    },
                    _ => {
                        if !allow_exp_end {
                            self.tokens.error_here(ErrorType::Unexpected(String::from("expression")));
                            break;
                        }
                        break;
                    }
                }
            }
        };
        let typings = if self.tokens.is_next(TokenType::Op(String::from("<"))) {
            if !allow_typings {
                self.tokens.error_here(ErrorType::Unexpected(String::from("typings")));
            }
            self.tokens.consume();
            Some(self.parse_typing_list(false, false, TokenType::Op(String::from(">"))))
        } else { None };
        let init = if self.tokens.is_next(TokenType::Punc('(')) {
            if !allow_exp_end {
                self.tokens.error_here(ErrorType::Unexpected(String::from("initializer")));
            }
            self.tokens.consume();
            Some(self.parse_expression_list(')'))
        } else { None };
        ASTModAccessValues::ModAccess(
            ASTModAccess {
                path,
                range: start.end(&mut self.tokens),
                typings,
                init
            }
        )
    }

    fn parse_typing(&mut self, allow_fn_keyword: bool, allow_optional_after_var: bool) -> Option<ASTTypings> {
        let range = self.tokens.recorder();
        let maybe_token = self.tokens.peek();
        let t = match maybe_token {
            Some(token) => {
                match &token.val {
                    TokenType::Punc('{') => {
                        self.tokens.consume();
                        Some(ASTTypings::PairList(self.parse_typing_pair_list(false, true, false, true, false, '}')))
                    },
                    TokenType::Punc('(') => {
                        self.tokens.consume();
                        let params = Box::from(self.parse_typing_pair_list(false, allow_fn_keyword, true, false, false, ')'));
                        let return_type = if self.tokens.is_next(TokenType::Op(String::from("->"))) { 
                            self.tokens.consume(); 
                            let typing = self.parse_typing(allow_fn_keyword, true);
                            if typing.is_none() { 
                                range.err(ErrorType::Expected(String::from("return type")), &mut self.tokens);
                                return None
                            };
                            Some(Box::from(typing.unwrap()))
                        } else { None };
                        Some(ASTTypings::Function(ASTFunction {
                            params,
                            return_type,
                            range: range.end(&self.tokens),
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
                        let name_copy = name.clone();
                        let tok_range = token.range;
                        self.tokens.consume();
                        match self.parse_mod_access_or_var(ASTVar { value: name_copy, range: tok_range}, false, true) {
                            ASTModAccessValues::ModAccess(acc) => Some(ASTTypings::Mod(acc)),
                            ASTModAccessValues::Var(v) => Some(ASTTypings::Var(v))
                        }
                    },
                    TokenType::Kw(kw) => {
                        if !allow_fn_keyword {
                            range.err(ErrorType::Unexpected(String::from("keyword fn")), &mut self.tokens);
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
                        //let token_stringed = token.val.to_string();
                        //self.tokens.error(ErrorType::ExpectedFound(String::from("typing"), token_stringed), self.tokens.input.loc(), self.tokens.input.loc());
                        None
                    }
                }
            },
            None => None
        };
        if let Some(typing) = t {
            if let Some(tok) = self.tokens.peek() {
                match &tok.val {
                    TokenType::Op(op) => {
                        match op.as_str() {
                            "?" if allow_optional_after_var => {
                                self.tokens.consume();
                                Some(ASTTypings::Optional(Box::from(typing)))
                            },
                            "+" => {
                                self.tokens.consume();
                                let right = self.parse_typing(false, false);
                                if right.is_none() {
                                    self.tokens.error(ErrorType::Expected(String::from("typing")), self.tokens.input.loc(), self.tokens.input.loc());
                                    return Some(typing);
                                }
                                Some(ASTTypings::Combine(
                                    ASTCombineTyping {
                                        left: Box::from(typing),
                                        right: Box::from(right.unwrap()),
                                        range: range.end(&self.tokens)
                                    }
                                ))
                            },
                            "!" => {
                                self.tokens.consume();
                                Some(ASTTypings::ExplicitImpl(
                                    match typing {
                                        ASTTypings::Var(v) => ASTModAccessValues::Var(v),
                                        ASTTypings::Mod(m) => ASTModAccessValues::ModAccess(m),
                                        _ => {
                                            range.err(ErrorType::Unexpected(String::from("explicit impl operator")), &mut self.tokens);
                                            return None
                                        }
                                    }
                                ))
                            },
                            _ => { Some(typing) }
                        }
                    },
                    _ => { Some(typing) }
                }
            } else {
                Some(typing)
            }
        } else {
            t
        }
    }

    fn parse_block(&mut self, allow_statement_as_exp: bool) -> ASTBlock {
        let range = self.tokens.recorder();
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
        self.tokens.skip_or_err(TokenType::Punc('}'), Some(ErrorType::EndOfBlock), Some(range.end(&self.tokens)));
        self.is_last_block = true;
        ASTBlock {
            elements: res,
            range: range.end(&self.tokens)
        }
    }

    fn parse_varname(&mut self, allow_generics: bool, only_varnames_as_generics: bool, allow_ints: bool) -> (Option<ASTVar>, Option<ASTListTyping>) {
        if allow_ints { self.tokens.is_last_num_as_str = true }; 
        let next = self.tokens.consume();
        if next.is_none() { 
            self.tokens.error_here(ErrorType::Expected(String::from("itentifier")));
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
        let range = self.tokens.recorder();
        let mut res: Vec<(String, Option<ASTExpression>)> = vec![];
        let mut has_consumed_bracket = false;
        while !self.tokens.is_next(TokenType::Punc(closing_punc)) {
            let tok_start = self.tokens.recorder();
            let key = self.parse_varname(false, false, false);
            if key.0.is_none() { continue; };
            match self.tokens.expect_punc(&[',', ':', closing_punc], Some(tok_start.end(&self.tokens))) {
                Some(ch) => {
                    match ch {
                        ',' => {
                            if !allow_without_val {
                                tok_start.err(ErrorType::Expected(String::from("value")), &mut self.tokens);
                                continue;
                            }
                            res.push((key.0.unwrap().value, None));
                        },
                        ':' => {
                            let exp = self.parse_expression();
                            if exp.is_none() { 
                                tok_start.err(ErrorType::Expected(String::from("expression")), &mut self.tokens);
                                continue;
                            }
                            res.push((key.0.unwrap().value, exp));
                        },
                        ch if ch == closing_punc => {
                            if !allow_without_val {
                                tok_start.err(ErrorType::Expected(String::from("typeing")), &mut self.tokens);
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
        if !has_consumed_bracket { self.tokens.skip_or_err(TokenType::Punc(closing_punc), Some(ErrorType::Expected(closing_punc.to_string())), Some(range.end(&self.tokens))); };
        ASTPairList {
            range: range.end(&self.tokens),
            pairs: res
        }
    }

    fn parse_expression_list(&mut self, closing_punc: char) -> ASTExpressionList {
        let range = self.tokens.recorder();
        let mut expressions: Vec<ASTExpression> = vec![];
        let mut is_first = true;
        while !self.tokens.is_next(TokenType::Punc(closing_punc)) {
            if !is_first {
                self.tokens.skip_or_err(TokenType::Punc(','), None, None);
            };
            let exp = self.parse_expression();
            if exp.is_none() { break; };
            expressions.push(exp.unwrap());
            is_first = false;
        };
        self.tokens.skip_or_err(TokenType::Punc(closing_punc), None, None);
        ASTExpressionList {
            expressions,
            range: range.end(&self.tokens)
        }
    }

    fn parse_varname_list(&mut self, closing_punc: char) -> ASTVarList {
        let range = self.tokens.recorder();
        let mut values: Vec<ASTVar> = vec![];
        let mut is_first = true;
        while !self.tokens.is_next(TokenType::Punc(closing_punc)) {
            if !is_first {
                self.tokens.skip_or_err(TokenType::Punc(','), None, None);
            };
            let exp = self.parse_varname(false, false, false).0;
            if exp.is_none() { break; };
            values.push(exp.unwrap());
            is_first = false;
        };
        self.tokens.skip_or_err(TokenType::Punc(closing_punc), None, None);
        ASTVarList {
            values,
            range: range.end(&self.tokens)
        }
    }

    fn parse_typing_pair_list(&mut self, allow_without_val: bool, allow_fn_keyword: bool, allow_spread: bool, allow_modifiers: bool, allow_default: bool, closing_punc: char) -> ASTPairListTyping {
        let range = self.tokens.recorder();
        let mut res: Vec<ASTPairTypingItem> = vec![];
        let mut has_consumed_bracket = false;
        let mut modifiers = ASTModifiers::empty();
        while !self.tokens.is_next(TokenType::Punc(closing_punc)) {
            let tok_range = self.tokens.recorder();
            let is_spread = if self.tokens.is_next(TokenType::Op(String::from("..."))) {
                if !allow_spread {
                    self.tokens.error(ErrorType::Disallowed(String::from("spread operator")), self.tokens.input.loc_inc(-3, 0), self.tokens.input.loc())
                }
                self.tokens.consume();
                true
            } else { false };
            if allow_modifiers {
                if let Some(t) = self.tokens.peek() {
                    let mod_range = t.range;
                    if let TokenType::Kw(kw) = &t.val {
                        match kw.as_str() {
                            "const" => {
                                self.tokens.consume();
                                if modifiers.contains(ASTModifiers::CONST) {
                                    mod_range.err_start(ErrorType::AlreadyHasModifier(String::from("const")), &mut self.tokens);
                                };
                                modifiers.insert(ASTModifiers::CONST);
                                continue;
                            },
                            "static" => {
                                self.tokens.consume();
                                if modifiers.contains(ASTModifiers::STATIC) {
                                    mod_range.err_start(ErrorType::AlreadyHasModifier(String::from("static")), &mut self.tokens);
                                };
                                modifiers.insert(ASTModifiers::STATIC);
                                continue;
                            },
                            "private" => {
                                self.tokens.consume();
                                if modifiers.contains(ASTModifiers::PRIVATE) {
                                    mod_range.err_start(ErrorType::AlreadyHasModifier(String::from("private")), &mut self.tokens);
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
            if self.tokens.is_next(TokenType::Op(String::from("="))) {
                if !allow_default {
                    self.tokens.error_here(ErrorType::NotAllowed(String::from("default parameter")))
                }
                self.tokens.consume();
                let default_value = self.parse_expression();
                if default_value.is_none() { 
                    self.tokens.error_here(ErrorType::Expected(String::from("expression")));
                }
                res.push(ASTPairTypingItem {name: key.0.unwrap().value, value: None, default_value, modifiers, spread: is_spread});
                continue;
            }
            match self.tokens.expect_punc(&[',', ':', '?', closing_punc], Some(tok_range.end(&self.tokens))) {
                Some(ch) => {
                    match ch {
                        ',' => {
                            if !allow_without_val {
                                tok_range.err(ErrorType::Expected(String::from("type")), &mut self.tokens);
                                continue;
                            }
                            res.push(ASTPairTypingItem {name: key.0.unwrap().value, value: None, default_value: None, modifiers, spread: is_spread});
                            modifiers.clear();
                        },
                        ':' => {
                            let exp = self.parse_typing(allow_fn_keyword, true);
                            if exp.is_none() { 
                                tok_range.err(ErrorType::Expected(String::from("expression")), &mut self.tokens);
                                continue;
                            }
                            let default_value = if self.tokens.is_next(TokenType::Op(String::from("="))) {
                                if !allow_default {
                                    self.tokens.error(ErrorType::NotAllowed(String::from("default parameter")), self.tokens.last_loc, self.tokens.input.loc())
                                }
                                self.tokens.consume();
                                self.parse_expression()
                            } else { None };
                            res.push(ASTPairTypingItem { name: key.0.unwrap().value, value: exp, default_value,modifiers, spread: is_spread});
                            modifiers.clear();
                        },
                        ch if ch == closing_punc => {
                            if !allow_without_val {
                                tok_range.err(ErrorType::Expected(String::from("type")), &mut self.tokens);
                                continue;
                            }
                            has_consumed_bracket = true;
                            res.push(ASTPairTypingItem { name: key.0.unwrap().value, value: None, default_value: None, modifiers, spread: is_spread});
                            modifiers.clear();
                            break;
                        },
                        _ => {}
                    }
                },
                None => continue
            };
        };
        if !has_consumed_bracket { self.tokens.skip_or_err(TokenType::Punc(closing_punc), None, None); };
        ASTPairListTyping {
            range: range.end(&self.tokens),
            pairs: res
        }
    }

    fn parse_typing_list(&mut self, only_varnames: bool, allow_fn_keyword: bool, closing_tok: TokenType) -> ASTListTyping {
        let range = self.tokens.recorder();
        let mut res: Vec<ASTTypings> = vec![];
        let mut is_first = true;
        while !self.tokens.is_next(closing_tok.clone()) {
            if !is_first {
                self.tokens.skip_or_err(TokenType::Punc(','), None, None);
            };
            let id_range = self.tokens.recorder();
            let maybe_typing = self.parse_typing(allow_fn_keyword, false);
            if maybe_typing.is_none() { break; };
            let typing = maybe_typing.unwrap();
            if only_varnames {
            match &typing {
                ASTTypings::Var(v) => {
                    if v.typings.is_some() {
                        v.range.err(ErrorType::Unexpected(String::from("token <, generics are not allowed here.")), &mut self.tokens);
                    }
                },
                _ => {
                    id_range.err(ErrorType::Expected(String::from("generic parameter")), &mut self.tokens);
                }
            }
            }
            res.push(typing);
            is_first = false;
        };
        let closing_tok_str = closing_tok.to_string();
        self.tokens.skip_or_err(closing_tok, Some(ErrorType::Expected(closing_tok_str)), Some(range.end(&self.tokens)));
        ASTListTyping {
            entries: res,
            range: range.end(&self.tokens)
        }
    }


    fn parse_function(&mut self, allow_body: bool) -> Option<ASTFunction> {
        let range = self.tokens.recorder();
        let typings = if self.tokens.is_next(TokenType::Op(String::from("<"))) {
            self.tokens.consume();
            Some(self.parse_typing_list(true, false, TokenType::Op(String::from(">"))))
        } else { None };
        if self.tokens.skip_or_err(TokenType::Punc('('), Some(ErrorType::Expected(String::from("start of function params"))), None) { return None };
        let params = Box::from(self.parse_typing_pair_list(true, false, true, false, true, ')'));
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
            range: range.end(&self.tokens),
            params,
            typings,
            return_type,
            body
        })
    }

    fn parse_match_arm_exp(&mut self) -> Option<ASTMatchArmExpressions> {
        let range = self.tokens.recorder();
        if let Some(exp) = self.parse_expression_part(false) {
            match exp {
                ASTExpression::Str(str_obj) => Some(ASTMatchArmExpressions::String(str_obj)),
                ASTExpression::Int(int_obj) => Some(ASTMatchArmExpressions::Int(int_obj)),
                ASTExpression::Float(f_obj) => Some(ASTMatchArmExpressions::Float(f_obj)),
                ASTExpression::Bool(b_obj) => Some(ASTMatchArmExpressions::Bool(b_obj)),
                ASTExpression::Tuple(t_obj) => {
                    if !utils::is_natural_tuple(&t_obj) {
                        range.err(ErrorType::Expected(String::from("natural tuple literal")), &mut self.tokens);
                    }
                    Some(ASTMatchArmExpressions::Tuple(t_obj))
                },
                ASTExpression::Iterator(i_obj) => {
                    if !utils::is_natural_iter(&i_obj) {
                        range.err(ErrorType::Expected(String::from("natural iterator literal")), &mut self.tokens);
                    }
                    Some(ASTMatchArmExpressions::Iterator(i_obj))
                },
                ASTExpression::Var(v) => {
                    if v.value != "_" {
                        range.err(ErrorType::Unexpected(String::from("variable name")), &mut self.tokens);
                    };
                    Some(ASTMatchArmExpressions::Rest)
                },
                ASTExpression::None(r) => Some(ASTMatchArmExpressions::None(r)),
                ASTExpression::ModAccess(acc) => Some(ASTMatchArmExpressions::Enum(acc)),
                _ => {
                    range.err(ErrorType::WrongMatchArmExp, &mut self.tokens);
                    None
                }
            }
        } else {
            range.err(ErrorType::Expected(String::from("match arm expression")), &mut self.tokens);
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
                    "-" | "!" | "~" => {
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
                                range: token.range.end(&self.tokens)
                            }
                        ))
                    }
                    _ => {
                        token.range.err(ErrorType::UnexpectedOp(value), &mut self.tokens);
                        None
                    }
                }
            },
            TokenType::Punc(val) => {
                match val {
                    '(' => {
                        if self.tokens.is_next(TokenType::Punc(')')) {
                            self.tokens.error_here(ErrorType::Unexpected(String::from("empty expression")));
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
                            self.tokens.error_here(ErrorType::Unexpected(String::from("empty tuple")));
                            return None;
                        };
                        let expressions = self.parse_expression_list(']');
                        Some(ASTExpression::Tuple(expressions))
                    },
                    _ => {
                        token.range.err(ErrorType::UnexpectedPunc(val), &mut self.tokens);
                        None
                    }
                }
            },
            TokenType::Kw(val) => {
                match val.as_str() {
                    "let" | "const" => {
                        let is_const = val.as_str() == "const";
                        let to_get_name = self.tokens.consume()?;
                        let var = match to_get_name.val {
                            TokenType::Punc('[') => ASTDeclareTypes::TupleDeconstruct(self.parse_varname_list(']')),
                            TokenType::Punc('{') => ASTDeclareTypes::StructDeconstruct(self.parse_varname_list('}')),
                            TokenType::Var(v) => ASTDeclareTypes::Var(ASTVar { value: v, range: to_get_name.range  }),
                            _ => {
                                to_get_name.range.err(ErrorType::ExpectedFound(String::from("identifier or deconstruct pattern"), to_get_name.val.to_string()), &mut self.tokens);
                                return None;
                            }
                        };
                        let typings = if self.tokens.is_next(TokenType::Punc(':')) {
                            self.tokens.consume();
                            let typing = self.parse_typing(false, true);
                            if typing.is_none() {
                                self.tokens.error_here(ErrorType::Expected(String::from("typing")));
                            }
                            typing
                        } else { None };
                        let value = if self.tokens.is_next(TokenType::Op("=".to_string())) {
                            let equals = self.tokens.consume().unwrap(); // Skip =
                            let exp = self.parse_expression();
                            match exp {
                                Some(e) => Some(Box::from(e)),
                                None => {
                                    self.tokens.error(ErrorType::Expected(String::from("initializor")), token.range.start, equals.range.end);
                                    None
                                }
                            }
                        } else { 
                            if is_const {
                                token.range.err(ErrorType::ConstantWithoutInit, &mut self.tokens)
                            }
                            None
                         };
                        return Some(ASTExpression::Declare(
                            ASTDeclare {
                                var,
                                is_const,
                                typings,
                                value,
                                range: Range { start: token.range.start, end: self.tokens.input.loc() }
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
                            token.range.err(ErrorType::Expected(String::from("condition in if expression")), &mut self.tokens);
                             return None;
                        };
                        let then = if let Some(th) = self.parse_expression_or_expression_statement() {
                             Box::from(th)
                         } else {
                            token.range.err(ErrorType::Expected(String::from("expression that will be executed if the condition is true")), &mut self.tokens);
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
                                range: token.range.end(&mut self.tokens)
                            }
                        ))
                    },
                    "for" => {
                        let var = self.parse_varname(false, false, false).0;
                        if var.is_none() {
                            self.tokens.error_here(ErrorType::Expected(String::from("identifier")));
                            return None;
                        };
                        if self.tokens.skip_or_err(TokenType::Kw(String::from("in")), None, None) { return None; };
                        let iterator = self.parse_expression();
                        if iterator.is_none() {
                            self.tokens.error_here(ErrorType::Expected(String::from("iterator")));
                            return None;
                        }
                        let turn_off_exp_statements = !self.allow_exp_statements;
                        self.allow_exp_statements = true;
                        let body = self.parse_expression_or_expression_statement();
                        if body.is_none() {
                            self.tokens.error_here(ErrorType::Expected(String::from("for...in loop body")));
                            return None;
                        }
                        if turn_off_exp_statements { self.allow_exp_statements = false; }
                        return Some(ASTExpression::ForIn(
                            ASTForIn {
                                var: var.unwrap(),
                                iterable: Box::from(iterator.unwrap()),
                                body: Box::from(body.unwrap()),
                                range: token.range.end(&self.tokens)
                            }
                        ))
                    },
                    "while" => {
                        let cond = self.parse_expression();
                        if cond.is_none() {
                            self.tokens.error_here(ErrorType::Expected(String::from("while condition")));
                            return None;
                        }
                        let turn_off_exp_statements = !self.allow_exp_statements;
                        self.allow_exp_statements = true;
                        let body = self.parse_expression_or_expression_statement();
                        if body.is_none() {
                            self.tokens.error_here(ErrorType::Expected(String::from("while body")));
                            return None;
                        }
                        if turn_off_exp_statements { self.allow_exp_statements = false; }
                        return Some(ASTExpression::While(
                            ASTWhile {
                                condition: Box::from(cond.unwrap()),
                                body: Box::from(body.unwrap()),
                                range: token.range.end(&self.tokens)
                            }
                        ))
                    },
                    "match" => {
                        let to_get_matched = self.parse_expression();
                        if to_get_matched.is_none() {
                            self.tokens.error_here(ErrorType::Expected(String::from("expression to get matched")));
                            return None;
                        };
                        self.tokens.skip_or_err(TokenType::Punc('{'), None, None);
                        let mut arms: Vec<ASTMatchArm> = vec![];
                        while !self.tokens.is_next(TokenType::Punc('}')) {
                            let match_arm_start = self.tokens.recorder();
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
                                match_arm_start.err(ErrorType::Expected(String::from("match arm body")), &mut self.tokens);
                                return None;
                            }
                            if self.tokens.is_next(TokenType::Punc(',')) { self.tokens.consume(); };
                            arms.push(ASTMatchArm {
                                guard,
                                possibilities,
                                body: body.unwrap(),
                                range: match_arm_start.end(&self.tokens)
                            });
                        }
                        self.tokens.skip_or_err(TokenType::Punc('}'), None, None);
                        self.is_last_block = true;
                        Some(ASTExpression::Match(ASTMatch {
                            arms,
                            range: token.range.end(&self.tokens),
                            expression: Box::from(to_get_matched.unwrap())
                        }))
                    },
                    "new" => {
                        let target = if let Some(t) = self.parse_mod_access_or_var_without_var(false, true) {
                            t
                        } else {
                            token.range.err(ErrorType::Expected(String::from("struct identifier")), &mut self.tokens);
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
                                range: token.range.end(&self.tokens)
                            }
                        ))
                    },
                    "await" => {
                        let optional = if self.tokens.is_next(TokenType::Op(String::from("?"))) {
                            self.tokens.consume();
                            true 
                        } else { false };
                        let expression = if let Some(exp) = self.parse_expression() {
                            Box::from(exp)
                        } else {
                            token.range.err(ErrorType::Expected(String::from("expression")), &mut self.tokens);
                            return None;
                        };
                        Some(ASTExpression::Await(
                            ASTAwait {
                                optional,
                                expression,
                                range: token.range.end(&self.tokens)
                            }
                        ))
                    }
                    _ => {
                        token.range.err(ErrorType::ExpectedFound("expression".to_string(), format!("keyword \"{}\"", val)), &mut self.tokens);
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
        let range = self.tokens.recorder();
        let thing = self.tokens.peek()?;
        match &thing.val {
            TokenType::Kw(kw) => {
                match kw.as_str() {
                    "yield" => { 
                        self.tokens.consume();
                        if !self.allow_exp_statements {
                            range.err(ErrorType::Unexpected(String::from("yield expression")), &mut self.tokens);
                            return None;
                        }
                        let value = if let Some(exp) = self.parse_expression() {
                            Some(Box::from(exp))
                        } else { None };
                        Some(ASTExpression::Yield(ASTYield {
                            value,
                            range: range.end(&self.tokens)
                        }))
                     },
                    _ => self.parse_expression()
                }
            },
            _ => self.parse_expression()
        }
    }

    fn parse_statement(&mut self) -> Option<ASTStatement> {
        let range = self.tokens.recorder();
        let token = self.tokens.consume()?;
        match token.val {
            TokenType::Kw(keyword) => {
                match keyword.as_str() {
                   "struct" => {
                        let name = self.parse_varname(true, true, false);
                        if name.0.is_none() { 
                            token.range.err(ErrorType::Expected(String::from("struct name")), &mut self.tokens);
                            return None;
                        }
                        if self.tokens.skip_or_err(TokenType::Punc('{'), Some(ErrorType::Expected(String::from("start of struct fields"))), None) { return None; };
                        Some(ASTStatement::Struct(ASTStruct {
                            name: name.0.unwrap(),
                            typings: name.1,
                            fields: self.parse_typing_pair_list(false, true, false, true, false, '}'),
                            range: range.end(&self.tokens)
                        }))
                   }
                   "enum" => {
                    let name = self.parse_varname(true, true, false);
                    if name.0.is_none() { 
                        token.range.err(ErrorType::Expected(String::from("struct name")), &mut self.tokens);
                        return None;
                    }
                    if self.tokens.skip_or_err(TokenType::Punc('{'), Some(ErrorType::Expected(String::from("start of enum variants"))), None) { return None; };
                    Some(ASTStatement::EnumDeclaration(ASTEnumDeclaration {
                    name: name.0.unwrap(),
                    values: self.parse_typing_pair_list(true, false, false, false, false, '}'),
                    typings: name.1,
                    range: range.end(&self.tokens)
                    }))
                   },
                   "type" => {
                       let name = self.parse_varname(true, true, false);
                       if name.0.is_none() {
                        self.tokens.error_here(ErrorType::Expected(String::from("type name")));
                        return None;
                       }
                       if self.tokens.skip_or_err(TokenType::Op(String::from("=")), None, None) { return None; };
                       let typing = self.parse_typing(false, false);
                       if typing.is_none() {
                        self.tokens.error_here(ErrorType::Expected(String::from("typing")));
                        return None;
                       }
                       Some(ASTStatement::Type(
                           ASTType {
                               name: name.0.unwrap().value,
                               typings: name.1,
                               value: typing.unwrap(),
                               range: range.end(&self.tokens)
                           }
                       ))
                   },
                   "main" => {
                       if self.parsed_main {
                           range.err(ErrorType::ManyEntryPoints, &mut self.tokens);
                       };
                       self.tokens.skip_or_err(TokenType::Punc('{'), None, None);
                       let exp = self.parse_block(false);
                       self.parsed_main = true;
                       Some(ASTStatement::Main(
                           ASTMain {
                               expression: exp,
                               range: range.end(&self.tokens)
                           }
                       ))
                   },
                   "static" => {
                       let varname = self.parse_varname(true, false, false);
                       self.tokens.skip_or_err(TokenType::Op(String::from("=")), None, None);
                       if varname.0.is_none() {
                           range.err(ErrorType::Expected(String::from("identifier")), &mut self.tokens);
                           return None;
                       }
                       let typings = if let Some(typing) = varname.1 {
                        let len = typing.entries.len();
                        if len == 0 {
                            token.range.err(ErrorType::Expected(String::from("at least one type")), &mut self.tokens);
                            None
                        } else {
                           Some(typing) 
                        }
                    } else { None };
                       let exp = self.parse_expression();
                       if exp.is_none() {
                        range.err(ErrorType::Expected(String::from("initializor")), &mut self.tokens);
                        return None;
                       }
                       Some(ASTStatement::Static(
                           Box::from(ASTStatic {
                               typings,
                               var: varname.0.unwrap(),
                               value: exp.unwrap(),
                               range: range.end(&self.tokens)
                           })
                       ))
                   },
                   "export" => {
                       let value = if let Some(stm) = self.parse_statement() {
                           if matches!(stm, ASTStatement::Main(_)) {
                               range.err(ErrorType::Unexpected(String::from("main entry")), &mut self.tokens);
                               return None;
                           }
                           Box::from(stm)
                       } else { return None };
                       Some(ASTStatement::Export(
                           ASTExport {
                               value,
                               range: range.end(&self.tokens)
                           }
                       ))
                   },
                   "import" => {
                       let path_start = self.tokens.input.loc();
                       let path = if let Some(ASTExpression::Str(string)) = self.parse_expression_part(false) {
                           string
                       } else {
                        self.tokens.error(ErrorType::Expected(String::from("path string")), range.start, path_start);
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
                               range: range.end(&self.tokens)
                           }
                       ))
                   },
                   "impl" => {
                       let typings = if self.tokens.is_next(TokenType::Op(String::from("<"))) {
                           self.tokens.consume();
                           Some(self.parse_typing_list(true, false, TokenType::Op(String::from(">"))))
                       } else { None };
                       let partial = if let Some(p) = self.parse_mod_access_or_var_without_var(false, true) {
                           p 
                       } else {
                           range.err(ErrorType::Expected(String::from("Partial identifier")), &mut self.tokens);
                           return None;
                       };
                       self.tokens.skip_or_err(TokenType::Kw(String::from("for")), None, None);
                       let target = if let Some(t) = self.parse_mod_access_or_var_without_var(false, true) {
                           t 
                       } else {
                            range.err(ErrorType::Expected(String::from("Struct or enum identifier")), &mut self.tokens);
                            return None;
                       };
                       self.tokens.skip_or_err(TokenType::Punc('{'), None, None);
                       Some(ASTStatement::Impl(
                           ASTImpl {
                               partial,
                               target,
                               typings,
                               fields: self.parse_typing_pair_list(false, true, false, true, false, '}'),
                               range: range.end(&self.tokens)
                           }
                       ))
                   },
                   _ => {
                    token.range.err(ErrorType::Expected(String::from("statement")), &mut self.tokens);
                    self.tokens.input.skip_line();
                    None
                },
                }
            },
            TokenType::Punc(';') => {
                None
            },
            TokenType::Punc('#') => {
                let name = self.parse_varname(false, false, false).0?.value;
                let mut args: Vec<TokenType> = vec![];
                if self.tokens.is_next(TokenType::Punc('(')) {
                    self.tokens.consume();
                    let mut is_first = true;
                    while !self.tokens.is_next(TokenType::Punc(')')) {
                        if !is_first {
                            if self.tokens.skip_or_err(TokenType::Punc(','), None, None) { return None };
                        }
                        if self.tokens.is_next(TokenType::Punc(')')) { break; };
                        args.push(self.tokens.consume()?.val);
                        is_first = false;
                    }
                    self.tokens.skip_or_err(TokenType::Punc(')'), None, None);
                }
                Some(ASTStatement::Meta(
                    ASTMeta {
                    name,
                    args,
                    range: token.range.end(&self.tokens)
                }))
            },
            _ => {
                token.range.err(ErrorType::Expected(String::from("statement")), &mut self.tokens);
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