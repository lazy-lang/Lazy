
use super::tokenizer::{Tokenizer, TokenType};
pub use errors::{LoC};
pub mod model;
pub mod utils;
use model::*;
use errors::*;
use errors::diagnostics::*;

pub struct Parser {
    pub tokens: Tokenizer,
    is_last_block: bool,
    allow_exp_statements: bool,
    parsed_main: bool
}

impl Parser {

    pub fn new(source: &str, filename: String) -> Self {
        Parser {
            tokens: Tokenizer::new(source, filename),
            parsed_main: false,
            is_last_block: false,
            allow_exp_statements: false
        }
    }

    pub fn reset(&mut self, source: &str, filename: String) {
        self.tokens = Tokenizer::new(source, filename);
        self.parsed_main = false;
        self.is_last_block = false;
        self.allow_exp_statements = false;
    }

    fn get_prec(op: &str) -> i8 {
        match op {
            "=" | "+=" | "-=" | "*=" | "/=" | "%=" => 1,
            "&" | "|" | "^" => 2,
            "<<" => 3,
            "||" => 5,
            "&&" => 7,
            "<" | ">" | "<=" | ">=" | "==" | "!=" => 10,
            "+" | "-" => 15,
            "*" | "/" | "%" => 20,
            _ => -1
        }
    }

    fn parse_binary(&mut self, left_tok: ASTExpression, prec: i8) -> LazyResult<ASTExpression> {
        let start = self.tokens.input.loc();
        let next = self.tokens.peek();
        if next.is_none() { 
            return Ok(left_tok)
         };
        let value = &next.unwrap();
        match &value.val {
            TokenType::Op(val) => {
                let opval = val.to_string();
                let other_prec = Self::get_prec(&val);
                if other_prec == -1 {
                    return Ok(left_tok)
                }
                if other_prec > prec {
                    self.tokens.consume();
                    let exp = if let Some(exp) = self.parse_expression_part(false)? {
                        exp
                    } else {
                        return Ok(left_tok);
                    };
                    let right = self.parse_binary(exp, other_prec)?;
                    return self.parse_binary(ASTExpression::Binary(ASTBinary {
                        op: opval,
                        left: Box::from(left_tok),
                        right: Box::from(right),
                        range: start.end(&self.tokens.last_loc)
                    }), prec);
                }
                Ok(left_tok)
            },
            _ => Ok(left_tok)
        }
    }

    fn parse_suffix(&mut self, token: ASTExpression, parse_generics: bool) -> LazyResult<ASTExpression> {
        let start = self.tokens.input.loc();
        let next_token = if let Some(t) = self.tokens.peek() { t } else {
            return Ok(token);
        };
        match &next_token.val {
            TokenType::Op(val) => {
                let cloned = val.clone();
                match val.as_str() {
                    "." => {
                        self.tokens.consume();
                        let target = self.parse_varname(true, false, !matches!(token, ASTExpression::Int(_) | ASTExpression::Float(_)), true)?.0;
                        self.parse_suffix(ASTExpression::DotAccess(
                            ASTDotAccess {
                                target: target,
                                value: Box::from(token),
                                range: start.end(&self.tokens.last_loc)
                            }
                        ), parse_generics)
                    },
                    "?" => {
                        self.tokens.consume();
                        self.parse_suffix(ASTExpression::Optional(
                            ASTOptional {
                                value: Box::from(token),
                                range: start.end(&self.tokens.last_loc)
                            }
                        ), parse_generics)
                    },
                    ".." | "..=" => {
                        self.tokens.consume();
                        let end = if let Some(end) = self.parse_expression_part(true)? { end } else {
                            return Err(err!(END_OF_ITER, start.end(&self.tokens.last_loc), self.tokens.filename));
                        };
                        Ok(ASTExpression::Iterator(
                            ASTIterator {
                                start: Box::from(token),
                                end: Box::from(end),
                                inclusive: cloned == "..=",
                                range: start.end(&self.tokens.last_loc)
                            }
                        ))
                    },
                    _ => Ok(token)
                }
            },
            TokenType::Punc(punc) => {
                match punc {
                    '(' => {
                        self.tokens.consume();
                        let args = self.parse_expression_list(')')?;
                        self.parse_suffix(ASTExpression::Call(
                            ASTCall {
                                target: Box::from(token),
                                typings: None,
                                args,
                                range: start.end(&self.tokens.last_loc)
                            }
                        ), parse_generics)
                    },
                    '[' => {
                        self.tokens.consume();
                        let target = if let Some(exp) = self.parse_expression()? { Box::from(exp) } else {
                            return Err(err!(EXPECTED, start.end(&self.tokens.last_loc), self.tokens.filename, "expression"));
                        };
                        self.tokens.skip_or_err(TokenType::Punc(']'), None)?;
                        self.parse_suffix(ASTExpression::IndexAccess(
                            ASTIndexAccess {
                            target: Box::from(target),
                            value: Box::from(token),
                            range: start.end(&self.tokens.last_loc)
                            }
                        ), parse_generics)
                    },
                    ':' => {
                        if let ASTExpression::Var(v) = token {
                            match self.parse_mod_access_or_var(v, true, true)? {
                                ASTModAccessValues::ModAccess(mod_access) => Ok(ASTExpression::ModAccess(mod_access)),
                                ASTModAccessValues::Var(v) => Ok(ASTExpression::Var(v.value))
                            }
                        } else {
                            Err(err!(EXPECTED, self.tokens.range_here(), self.tokens.filename, "identifier"))
                        }
                    }
                    _ => Ok(token)
                }
            }
            _ => Ok(token)
        }
    }

    pub fn parse_mod_access_or_var_without_var(&mut self, allow_exp_end: bool, allow_typings: bool) -> LazyResult<ASTModAccessValues> {
        let name = self.parse_varname(false, false, false, false)?.0;
        self.parse_mod_access_or_var(name, allow_exp_end, allow_typings)
    }

    pub fn parse_mod_access_or_var(&mut self, start: ASTVar, allow_exp_end: bool, allow_typings: bool) -> LazyResult<ASTModAccessValues> {
        if !self.tokens.is_next(TokenType::Punc(':')) {
            let r = start.range;
            let typings = if self.tokens.is_next(TokenType::Op(String::from("<"))) {
                if !allow_typings {
                    return Err(err!(UNEXPECTED, self.tokens.range_here(), self.tokens.filename, "typings"));
                }
                self.tokens.consume();
                Some(self.parse_typing_list(false, false, TokenType::Op(String::from(">")))?)
            } else { None };
            return Ok(ASTModAccessValues::Var(ASTVarTyping { value: start, range: r, typings }));
        };

        let mut path: Vec<ASTVar> = vec![start];
        let start = self.tokens.input.loc();
        while self.tokens.is_next(TokenType::Punc(':')) {
            self.tokens.consume();
            self.tokens.skip_or_err(TokenType::Punc(':'), Some(err!(EXPECTED, self.tokens.range_here(), self.tokens.filename, "Another colon (:)"; ["Add another colon to make the mod access expression (Module::Item)"])))?;
            if let Some(tok) = self.tokens.consume() {
                match tok.val {
                    TokenType::Var(v) => path.push(ASTVar { value: v, range: tok.range }),
                    TokenType::Kw(v) => path.push(ASTVar { value: v, range: tok.range}),
                    _ => { 
                        if !allow_exp_end {
                            return Err(err!(UNEXPECTED, self.tokens.range_here(), self.tokens.filename, "expression"));
                        }
                        break;
                    }
                }
            }
        };
        let typings = if self.tokens.is_next(TokenType::Op(String::from("<"))) {
            if !allow_typings {
                return Err(err!(UNEXPECTED, self.tokens.range_here(), self.tokens.filename, "typings"));
            }
            self.tokens.consume();
            Some(self.parse_typing_list(false, false, TokenType::Op(String::from(">")))?)
        } else { None };
        let init = if self.tokens.is_next(TokenType::Punc('(')) {
            if !allow_exp_end {
                return Err(err!(UNEXPECTED, self.tokens.range_here(), self.tokens.filename, "initializer";));
            }
            self.tokens.consume();
            Some(self.parse_expression_list(')')?)
        } else { None };
        Ok(ASTModAccessValues::ModAccess(
            ASTModAccess {
                path,
                range: start.end(&self.tokens.last_loc),
                typings,
                init
            }
        ))
    }

    fn parse_typing(&mut self, allow_fn_keyword: bool, allow_optional_after_var: bool, allow_mod: bool) -> LazyResult<ASTTypings> {
        let range = self.tokens.input.loc();
        let maybe_token = self.tokens.peek();
        let t = match maybe_token {
            Some(token) => {
                match &token.val {
                    TokenType::Punc('{') => {
                        self.tokens.consume();
                        Some(ASTTypings::PairList(self.parse_typing_pair_list(false, false, false, true, false, '}')?))
                    },
                    TokenType::Punc('(') => {
                        self.tokens.consume();
                        let params = Box::from(self.parse_typing_pair_list(false, allow_fn_keyword, true, false, false, ')')?);
                        let return_type = if self.tokens.is_next(TokenType::Op(String::from("->"))) { 
                            self.tokens.consume(); 
                            let typing = self.parse_typing(allow_fn_keyword, true, allow_mod)?;
                            Some(Box::from(typing))
                        } else { None };
                        Some(ASTTypings::Function(ASTFunction {
                            params,
                            return_type,
                            range: range.end(&self.tokens.last_loc),
                            typings: None,
                            body: None
                        }))
                    },
                    TokenType::Punc('[') => {
                        self.tokens.consume();
                        let values = self.parse_typing_list(false, false, TokenType::Punc(']'))?;
                        Some(ASTTypings::Tuple(values))
                    },
                    TokenType::Var(name) => {
                        let tok_range = token.range;
                        let var = ASTVar { value: name.clone(), range: tok_range };
                        self.tokens.consume();
                        if allow_mod {
                        match self.parse_mod_access_or_var(var, false, true)? {
                            ASTModAccessValues::ModAccess(acc) => Some(ASTTypings::Mod(acc)),
                            ASTModAccessValues::Var(v) => Some(ASTTypings::Var(v))
                        }
                    } else {
                        let typings = if self.tokens.is_next(TokenType::Op(String::from("<"))) {
                        self.tokens.consume();
                        Some(self.parse_typing_list(false, false, TokenType::Op(String::from(">")))?)
                        } else { None };
                        Some(ASTTypings::Var(ASTVarTyping { value: var, range: tok_range, typings }))
                    }
                    },
                    TokenType::Kw(kw) => {
                        match kw.as_str() {
                            "fn" => {
                                if !allow_fn_keyword {
                                    return Err(err!(UNEXPECTED, self.tokens.range_here(), self.tokens.filename, "keyword fn"; ["Only function signatures are allowed here. Remove the `fn` and the function body, if there is one."]));
                                }
                                self.tokens.consume();
                                Some(ASTTypings::Function(self.parse_function(true)?))
                            },
                            "impl" => {
                                self.tokens.consume();
                                let val = self.parse_typing(false, false, true)?;
                                match val {
                                    ASTTypings::Var(_) | ASTTypings::Mod(_) => Some(ASTTypings::Impl(ASTImplTyping { 
                                        value: Box::from(val),
                                        range: range.end(&self.tokens.last_loc)
                                    })),
                                    _ => return Err(err!(EXPECTED, range.end(&self.tokens.last_loc), self.tokens.filename, "identifier or module access"; ["Save the typing via the \"type\" keyword."]))
                                }
                            }
                            _ => None
                        }
                    }
                    _ => {
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
                                Ok(ASTTypings::Optional(Box::from(typing)))
                            },
                            "+" => {
                                self.tokens.consume();
                                let right = self.parse_typing(false, false, allow_mod)?;
                                Ok(ASTTypings::Combine(
                                    ASTCombineTyping {
                                        left: Box::from(typing),
                                        right: Box::from(right),
                                        range: range.end(&self.tokens.last_loc)
                                    }
                                ))
                            }
                            _ => { Ok(typing) }
                        }
                    },
                    _ => { Ok(typing) }
                }
            } else {
                Ok(typing)
            }
        } else {
            Ok(t.unwrap())
        }
    }

    fn parse_block(&mut self, allow_statement_as_exp: bool) -> LazyResult<ASTBlock> {
        let range = self.tokens.input.loc();
        let mut res: Vec<ASTExpression> = vec![];
        while !self.tokens.input.is_eof() && !self.tokens.is_next(TokenType::Punc('}')) {
            let exp = if let Some(exp) = if allow_statement_as_exp { self.parse_expression_or_expression_statement()? } else { self.parse_expression()? } { exp } else {
                continue;
            };
            let range = utils::full_expression_range(&exp);
            res.push(exp);
            if !self.is_last_block { 
                self.tokens.skip_or_err(TokenType::Punc(';'), Some(err!(SEMICOLON, range, self.tokens.filename)))?; 
            };
        }
        self.tokens.skip_or_err(TokenType::Punc('}'), Some(err!(END_OF_BLOCK, range.end(&self.tokens.last_loc), self.tokens.filename)))?;
        self.is_last_block = true;
        Ok(ASTBlock {
            elements: res,
            range: range.end(&self.tokens.last_loc)
        })
    }

    fn parse_varname(&mut self, allow_generics: bool, only_varnames_as_generics: bool, allow_ints: bool, allow_keywords: bool) -> LazyResult<(ASTVar, Option<ASTListTyping>)> {
        if allow_ints { self.tokens.is_last_num_as_str = true };
        let next = self.tokens.consume();
        if allow_ints { self.tokens.is_last_num_as_str = false };
        let unwrapped = if let Some(val) = next { 
            val
        } else {
            return Err(err!(EXPECTED, self.tokens.range_here(), self.tokens.filename, "identifier"));
        };
        let var = match unwrapped.val {
            TokenType::Var(v) => ASTVar { value: v, range: unwrapped.range },
            TokenType::Kw(kw) if allow_keywords => ASTVar { value: kw.to_string(), range: unwrapped.range },
            TokenType::Int(i) if allow_ints => ASTVar { value: i.to_string(), range: unwrapped.range },
            _ => {
                return Err(err!(EXPECTED_FOUND, unwrapped.range, self.tokens.filename, "identifier", &unwrapped.val.to_string();));
            }
        };
        if self.tokens.is_next(TokenType::Op(String::from("<"))) {
            if !allow_generics {
                return Ok((var, None));
            }
            self.tokens.consume();
            return Ok((var, Some(self.parse_typing_list(only_varnames_as_generics, false, TokenType::Op(String::from(">")))?)));
        }
        Ok((var, None))
    }

    fn parse_pair_list(&mut self, allow_without_val: bool, closing_punc: char) -> LazyResult<ASTPairList> {
        let range = self.tokens.input.loc();
        let mut res: Vec<(String, Option<ASTExpression>)> = vec![];
        let mut has_consumed_bracket = false;
        while !self.tokens.is_next(TokenType::Punc(closing_punc)) {
            let tok_start = self.tokens.input.loc();
            let key = self.parse_varname(false, false, false, true)?.0;
            match self.tokens.expect_punc(&[',', ':', closing_punc], Some(tok_start.end(&self.tokens.last_loc)))? {
                ',' => {
                    if !allow_without_val {
                        return Err(err!(EXPECTED, tok_start.end(&self.tokens.last_loc), self.tokens.filename, "value"));
                    }
                    res.push((key.value, None));
                },
                ':' => {
                    let exp = if let Some(exp) = self.parse_expression()? { Some(exp) } else {
                        return Err(err!(EXPECTED, tok_start.end(&self.tokens.last_loc), self.tokens.filename, "expression"));
                    };
                    res.push((key.value, exp));
                },
                ch if ch == closing_punc => {
                    if !allow_without_val {
                        return Err(err!(EXPECTED, tok_start.end(&self.tokens.last_loc), self.tokens.filename, "typing"));
                    }
                    has_consumed_bracket = true;
                    res.push((key.value, None));
                    break;
                },
                _ => {}
                }
            if self.tokens.is_next(TokenType::Punc(',')) { self.tokens.consume(); };
        };
        if !has_consumed_bracket { self.tokens.skip_or_err(TokenType::Punc(closing_punc), None)?; };
        Ok(ASTPairList {
            range: range.end(&self.tokens.last_loc),
            pairs: res
        })
    }

    fn parse_expression_list(&mut self, closing_punc: char) -> LazyResult<ASTExpressionList> {
        let range = self.tokens.input.loc();
        let mut expressions: Vec<ASTExpression> = vec![];
        let mut is_first = true;
        while !self.tokens.is_next(TokenType::Punc(closing_punc)) {
            if !is_first {
                self.tokens.skip_or_err(TokenType::Punc(','), None)?;
            };
            let exp = if let Some(exp) = self.parse_expression()? { exp } else {
                return Err(err!(EXPECTED, self.tokens.range_here(), self.tokens.filename, "expression"));
            };
            expressions.push(exp);
            is_first = false;
        };
        self.tokens.skip_or_err(TokenType::Punc(closing_punc), None)?;
        Ok(ASTExpressionList {
            expressions,
            range: range.end(&self.tokens.last_loc)
        })
    }

    fn parse_varname_list(&mut self, closing_punc: char) -> LazyResult<ASTVarList> {
        let range = self.tokens.input.loc();
        let mut values: Vec<ASTVar> = vec![];
        let mut is_first = true;
        while !self.tokens.is_next(TokenType::Punc(closing_punc)) {
            if !is_first {
                self.tokens.skip_or_err(TokenType::Punc(','), None)?;
            };
            let exp = self.parse_varname(false, false, false, true)?.0;
            values.push(exp);
            is_first = false;
        };
        self.tokens.skip_or_err(TokenType::Punc(closing_punc), None)?;
        Ok(ASTVarList {
            values,
            range: range.end(&self.tokens.last_loc)
        })
    }

    fn parse_typing_pair_list(&mut self, allow_without_val: bool, allow_fn_keyword: bool, allow_spread: bool, allow_modifiers: bool, allow_default: bool, closing_punc: char) -> LazyResult<ASTPairListTyping> {
        let range = self.tokens.input.loc();
        let mut res: Vec<ASTPairTypingItem> = vec![];
        let mut has_consumed_bracket = false;
        let mut modifiers = ASTModifiers::empty();
        while !self.tokens.is_next(TokenType::Punc(closing_punc)) {
            let tok_range = self.tokens.input.loc();
            let is_spread = if self.tokens.is_next(TokenType::Op(String::from("..."))) {
                self.tokens.consume();
                if !allow_spread {
                    return Err(err!(DISALLOWED, tok_range.end(&self.tokens.last_loc), self.tokens.filename, "spread operator";));
                }
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
                                    return Err(err!(ALREADY_HAS_MODIFIER, mod_range.end_with(&self.tokens.last_loc), self.tokens.filename, "const";));
                                };
                                modifiers.insert(ASTModifiers::CONST);
                                continue;
                            },
                            "static" => {
                                self.tokens.consume();
                                if modifiers.contains(ASTModifiers::STATIC) {
                                    return Err(err!(ALREADY_HAS_MODIFIER, mod_range.end_with(&self.tokens.last_loc), self.tokens.filename, "static";));
                                };
                                modifiers.insert(ASTModifiers::STATIC);
                                continue;
                            },
                            "private" => {
                                self.tokens.consume();
                                if modifiers.contains(ASTModifiers::PRIVATE) {
                                    return Err(err!(ALREADY_HAS_MODIFIER, mod_range.end_with(&self.tokens.last_loc), self.tokens.filename, "private";));
                                };
                                modifiers.insert(ASTModifiers::PRIVATE);
                                continue;
                            },
                            _ => {}
                        }
                    }
                }
            };
            let key = self.parse_varname(false, false, false, true)?.0;
            if self.tokens.is_next(TokenType::Op(String::from("="))) {
                if !allow_default {
                    return Err(err!(DISALLOWED, self.tokens.range_here(), self.tokens.filename, "default parameter"));
                }
                self.tokens.consume();
                let default_value = if let Some(exp) = self.parse_expression()? { Some(exp) } else {
                    return Err(err!(EXPECTED, self.tokens.range_here(), self.tokens.filename, "expression"));
                };
                res.push(ASTPairTypingItem {name: key.value, value: None, default_value, modifiers, spread: is_spread});
                continue;
            }
            match self.tokens.expect_punc(&[',', ':', closing_punc], None)? {
                ',' => {
                    if !allow_without_val {
                        return Err(err!(EXPECTED, tok_range.end(&self.tokens.last_loc), self.tokens.filename, "type"));
                    }
                    res.push(ASTPairTypingItem {name: key.value, value: None, default_value: None, modifiers, spread: is_spread});
                    modifiers.clear();
                },
                ':' => {
                    let exp = self.parse_typing(allow_fn_keyword, true, true)?;
                    let default_value = if self.tokens.is_next(TokenType::Op(String::from("="))) {
                        if !allow_default {
                            return Err(err!(DISALLOWED, Range { start: self.tokens.last_loc, end: self.tokens.input.loc() }, self.tokens.filename, "default parameter"));
                        }
                        self.tokens.consume();
                        Some(if let Some(exp) = self.parse_expression()? { exp } else {
                            return Err(err!(EXPECTED, self.tokens.range_here(), self.tokens.filename, "expression"));
                        })
                    } else { None };
                    res.push(ASTPairTypingItem { name: key.value, value: Some(exp), default_value, modifiers, spread: is_spread});
                    modifiers.clear();
                },
                ch if ch == closing_punc => {
                    if !allow_without_val {
                        return Err(err!(EXPECTED, tok_range.end(&self.tokens.last_loc), self.tokens.filename, "type"));
                    }
                    has_consumed_bracket = true;
                    res.push(ASTPairTypingItem { name: key.value, value: None, default_value: None, modifiers, spread: is_spread});
                    modifiers.clear();
                    break;
                },
                _ => {}
        };
    }
        if !has_consumed_bracket { self.tokens.skip_or_err(TokenType::Punc(closing_punc), None)?; };
        Ok(ASTPairListTyping {
            range: range.end(&self.tokens.last_loc),
            pairs: res
        })
    }

    fn parse_typing_list(&mut self, only_varnames_and_bounds: bool, allow_fn_keyword: bool, closing_tok: TokenType) -> LazyResult<ASTListTyping> {
        let range = self.tokens.input.loc();
        let mut res: Vec<ASTTypings> = vec![];
        let mut is_first = true;
        while !self.tokens.is_next(closing_tok.clone()) {
            if !is_first {
                self.tokens.skip_or_err(TokenType::Punc(','), None)?;
            };
            let id_range = self.tokens.input.loc();
            let typing = self.parse_typing(allow_fn_keyword, false, !only_varnames_and_bounds)?;
            if only_varnames_and_bounds {
            match &typing {
                ASTTypings::Var(v) => {
                    if v.typings.is_some() {
                        return Err(err!(NO_GENERICS, v.range, self.tokens.filename));
                    }
                    if self.tokens.is_next(TokenType::Punc(':')) {
                        self.tokens.consume();
                        let bound = Box::from(self.parse_typing(false, false, true)?);
                        res.push(ASTTypings::Bound(
                            ASTBoundTyping {
                                name: ASTVar { value: v.value.value.clone(), range: v.range },
                                bound,
                                range: range.end(&self.tokens.last_loc)
                            }
                        ));
                        is_first = false;
                        continue;
                    }
                },
                _ => {
                    return Err(err!(EXPECTED, id_range.end(&self.tokens.last_loc), self.tokens.filename, "generic parameter"));
                }
            }
            }
            res.push(typing);
            is_first = false;
        };
        self.tokens.skip_or_err(closing_tok, None)?;
        Ok(ASTListTyping {
            entries: res,
            range: range.end(&self.tokens.last_loc)
        })
    }


    fn parse_function(&mut self, allow_body: bool) -> LazyResult<ASTFunction> {
        let range = self.tokens.input.loc();
        let typings = if self.tokens.is_next(TokenType::Op(String::from("<"))) {
            self.tokens.consume();
            Some(self.parse_typing_list(true, false, TokenType::Op(String::from(">")))?)
        } else { None };
        self.tokens.skip_or_err(TokenType::Punc('('), Some(err!(EXPECTED, self.tokens.range_here(), self.tokens.filename, "start of function parameters")))?;
        let params = Box::from(self.parse_typing_pair_list(true, false, true, false, true, ')')?);
        let return_type = if self.tokens.is_next(TokenType::Op(String::from("->"))) {
            self.tokens.consume();
            Some(Box::from(self.parse_typing(false, true, true)?))
        } else { None };
        let body = if allow_body {
            if let Some(e) = self.parse_expression()? { Some(Box::from(e)) } else { 
                return Err(err!(EXPECTED, self.tokens.range_here(), self.tokens.filename, "expression"));
            }
        } else { None };
        Ok(ASTFunction {
            range: range.end(&self.tokens.last_loc),
            params,
            typings,
            return_type,
            body
        })
    }

    fn parse_match_arm_exp(&mut self) -> LazyResult<ASTMatchArmExpressions> {
        let range = self.tokens.input.loc();
        let exp = if let Some(exp) = self.parse_expression_part(false)? {
            exp
        } else {
            return Err(err!(EXPECTED, self.tokens.range_here(), self.tokens.filename, "match arm condition"));
        };
        match exp {
            ASTExpression::Str(str_obj) => Ok(ASTMatchArmExpressions::String(str_obj)),
            ASTExpression::Int(int_obj) => Ok(ASTMatchArmExpressions::Int(int_obj)),
            ASTExpression::Float(f_obj) => Ok(ASTMatchArmExpressions::Float(f_obj)),
            ASTExpression::Bool(b_obj) => Ok(ASTMatchArmExpressions::Bool(b_obj)),
            ASTExpression::Tuple(t_obj) => {
                if !utils::is_natural_tuple(&t_obj) {
                    return Err(err!(EXPECTED, range.end(&self.tokens.last_loc), self.tokens.filename, "natural tuple literal"));
                }
                Ok(ASTMatchArmExpressions::Tuple(t_obj))
            },
            ASTExpression::Iterator(i_obj) => {
                if !utils::is_natural_iter(&i_obj) {
                    return Err(err!(EXPECTED, range.end(&self.tokens.last_loc), self.tokens.filename, "natural iterator literal"));
                }
                Ok(ASTMatchArmExpressions::Iterator(i_obj))
            },
            ASTExpression::Var(v) => {
                if v.value != "_" {
                    return Err(err!(UNEXPECTED, range.end(&self.tokens.last_loc), self.tokens.filename, "variable name"));
                };
                Ok(ASTMatchArmExpressions::Rest)
            },
            ASTExpression::None(r) => Ok(ASTMatchArmExpressions::None(r)),
            ASTExpression::ModAccess(acc) => {
                if let Some(init) = &acc.init {
                    if init.expressions.len() == 1 && matches!(&init.expressions[0], ASTExpression::Var(_)) {
                        return Ok(ASTMatchArmExpressions::EnumVar(acc));
                    }
                } 
                if !utils::is_natural_mod_access(&acc) {
                    return Err(err!(EXPECTED, range.end(&self.tokens.last_loc), self.tokens.filename, "natural enum value"));
                }
                Ok(ASTMatchArmExpressions::Enum(acc))
            },
            _ => {
                Err(err!(WRONG_MATCH_ARM_EXP, range.end(&self.tokens.last_loc), self.tokens.filename))
            }
        }
    }

    fn parse_expression_part(&mut self, parse_generics_in_suffix: bool) -> LazyResult<Option<ASTExpression>> {
        self.is_last_block = false;
        let exp = {
        let token = if let Some(t) = self.tokens.consume() {
            t 
        } else {
            return Err(err!(UNEXPECTED_EOF, self.tokens.range_here(), self.tokens.filename));
        };
        match token.val {
            TokenType::Int(value) => ASTExpression::Int(ASTInt { value, range: token.range } ),
            TokenType::Float(value) => ASTExpression::Float(ASTFloat { value, range: token.range }),
            TokenType::Str(value) => ASTExpression::Str(ASTStr { value, range: token.range }),
            TokenType::Char(value) => ASTExpression::Char(ASTChar { value, range: token.range }),
            TokenType::None => ASTExpression::None(token.range),
            TokenType::Var(value) => ASTExpression::Var(ASTVar { value, range: token.range }),
            TokenType::Bool(value) => ASTExpression::Bool(ASTBool { value, range: token.range }),
            TokenType::TempStrStart => {
                let mut string = String::new();
                let mut exps: HashMap<usize, ASTExpression> = HashMap::new();
                let mut is_prev_escape = false;
                loop {
                    match self.tokens.input.consume() {
                        Some(ch) => {
                            match ch {
                                '`' => break,
                                '$' if !is_prev_escape => {
                                    self.tokens.skip_or_err(TokenType::Punc('{'), None)?;
                                    let exp = if let Some(exp) = self.parse_expression()? { exp } else {
                                        return Err(err!(EXPECTED, self.tokens.range_here(), self.tokens.filename, "expression"));
                                    };
                                    self.tokens.skip_or_err(TokenType::Punc('}'), None)?;
                                    exps.insert(string.len(), exp);
                                    string.push(' ');
                                },
                                '\\' => is_prev_escape = true,
                                _ => {
                                    is_prev_escape = false;
                                    string.push(ch);
                                }
                            }
                        },
                        None => {
                            return Err(err!(END_OF_STR, token.range.end_with(&self.tokens.last_loc), self.tokens.filename));
                        }
                    }
                };
                if exps.len() == 0 {
                    return Err(err!(POINTLESS_TEMPLATE, token.range.end_with(&self.tokens.last_loc), self.tokens.filename));
                }
                ASTExpression::TempStr(ASTTempStr {
                    template: string,
                    values: exps,
                    range: token.range.end_with(&self.tokens.last_loc)
                })
            },
            TokenType::Op(value) => {
                // Prefixes
                match value.as_str() {
                    "-" | "!" | "~" => {
                        let val = if let Some(val) = self.parse_expression_part(parse_generics_in_suffix)? { Box::from(val) } else {
                            return Err(err!(EXPECTED, token.range, self.tokens.filename, "expression"));
                        };
                        ASTExpression::Unary(
                            ASTUnary {
                                op: value,
                                value: val,
                                range: token.range
                            }
                        )
                    },
                    ".." | "..=" => ASTExpression::Iterator(ASTIterator {
                            start: Box::from(ASTExpression::Int(ASTInt { value: 0, range: token.range.clone() })),
                            end: if let Some(exp) = self.parse_expression()? { Box::from(exp) } else {
                                return Err(err!(EXPECTED, self.tokens.range_here(), self.tokens.filename, "expression"));
                            },
                            inclusive: value == "..=",
                            range: token.range.end_with(&self.tokens.last_loc)
                    }),
                    "..." => {
                        ASTExpression::Spread(
                            ASTSpread {
                                value: if let Some(exp) = self.parse_expression()? { Box::from(exp) } else {
                                    return Err(err!(EXPECTED, self.tokens.range_here(), self.tokens.filename, "expression"));
                                },
                                range: token.range.end_with(&self.tokens.last_loc)
                            }
                        )
                    }
                    _ => {
                        return Err(err!(UNEXPECTED_OP, token.range.end_with(&self.tokens.last_loc), self.tokens.filename, &value));
                    }
                }
            },
            TokenType::Punc(val) => {
                match val {
                    '(' => {
                        if self.tokens.is_next(TokenType::Punc(')')) {
                            return Err(err!(UNEXPECTED, self.tokens.range_here(), self.tokens.filename, "empty expression"));
                        };
                        let exp = if let Some(exp) = self.parse_expression()? { exp } else {
                            return Err(err!(EXPECTED, self.tokens.range_here(), self.tokens.filename, "expression"));
                        };
                        self.tokens.skip_or_err(TokenType::Punc(')'), Some(err!(EXPECTED, self.tokens.range_here(), self.tokens.filename, "end of wrapped expression")))?;
                        exp   
                    },
                    ';' => return Ok(None),
                    '{' => ASTExpression::Block(self.parse_block(true)?),
                    '[' => {
                        if self.tokens.is_next(TokenType::Punc(']')) {
                            return Err(err!(UNEXPECTED, self.tokens.range_here(), self.tokens.filename, "empty tuple"));
                        };
                        ASTExpression::Tuple(self.parse_expression_list(']')?)
                    },
                    _ => {
                        return Err(err!(UNEXPECTED_PUNC, token.range.end_with(&self.tokens.last_loc), self.tokens.filename, &val.to_string()));
                    }
                }
            },
            TokenType::Kw(val) => {
                match val.as_str() {
                    "let" | "const" => {
                        let is_const = val.as_str() == "const";
                        let to_get_name = if let Some(n) = self.tokens.consume() {
                            n 
                        } else {
                            return Err(err!(EXPECTED, token.range.end_with(&self.tokens.last_loc), self.tokens.filename, "variable name"));
                        };
                        let var = match to_get_name.val {
                            TokenType::Punc('[') => ASTDeclareTypes::TupleDeconstruct(self.parse_varname_list(']')?),
                            TokenType::Punc('{') => ASTDeclareTypes::StructDeconstruct(self.parse_varname_list('}')?),
                            TokenType::Var(v) => ASTDeclareTypes::Var(ASTVar { value: v, range: to_get_name.range  }),
                            _ => {
                                return Err(err!(EXPECTED_FOUND, to_get_name.range, self.tokens.filename, "identifier or deconstruct pattern"));
                            }
                        };
                        let typings = if self.tokens.is_next(TokenType::Punc(':')) {
                            self.tokens.consume();
                            Some(self.parse_typing(false, true, true)?)
                        } else { None };
                        let value = if self.tokens.is_next(TokenType::Op("=".to_string())) {
                            self.tokens.consume(); // Skip =
                            if let Some(exp) = self.parse_expression()? { Some(Box::from(exp)) } else {
                                return Err(err!(EXPECTED, self.tokens.range_here(), self.tokens.filename, "expression"));
                            }
                        } else { 
                            if is_const {
                                return Err(err!(CONST_WITHOUT_INIT, token.range.end_with(&self.tokens.last_loc), self.tokens.filename));
                            }
                            None
                         };
                        return Ok(Some(ASTExpression::Declare(
                            ASTDeclare {
                                var,
                                is_const,
                                typings,
                                value,
                                range: token.range.end_with(&self.tokens.last_loc)
                            }
                        )))
                        },
                    "fn" => ASTExpression::Function(self.parse_function(true)?),
                    "if" => {
                        let condition = if let Some(exp) = self.parse_expression()? { Box::from(exp) } else {
                            return Err(err!(EXPECTED, self.tokens.range_here(), self.tokens.filename, "expression"));
                        };
                        let then = if let Some(exp) = self.parse_expression_or_expression_statement()? { Box::from(exp) } else {
                            return Err(err!(EXPECTED, self.tokens.range_here(), self.tokens.filename, "expression"));
                        };
                        let otherwise = if self.tokens.is_next(TokenType::Kw(String::from("else"))) {
                             self.tokens.consume();
                             if let Some(exp) = self.parse_expression_or_expression_statement()? { Some(Box::from(exp)) } else {
                                return Err(err!(EXPECTED, self.tokens.range_here(), self.tokens.filename, "expression"));
                            }
                        } else { None };
                        return Ok(Some(ASTExpression::If(
                            ASTIf {
                                condition,
                                then,
                                otherwise,
                                range: token.range.end_with(&mut self.tokens.last_loc)
                            }
                        )))
                    },
                    "for" => {
                        let var = self.parse_varname(false, false, false, false)?.0;
                        self.tokens.skip_or_err(TokenType::Kw(String::from("in")), None)?;
                        let iterator = if let Some(exp) = self.parse_expression()? { Box::from(exp) } else {
                            return Err(err!(EXPECTED, self.tokens.range_here(), self.tokens.filename, "expression"));
                        };
                        let turn_off_exp_statements = !self.allow_exp_statements;
                        self.allow_exp_statements = true;
                        let body = if let Some(exp) = self.parse_expression_or_expression_statement()? { Box::from(exp) } else {
                            return Err(err!(EXPECTED, self.tokens.range_here(), self.tokens.filename, "expression"));
                        };
                        if turn_off_exp_statements { self.allow_exp_statements = false; }
                        return Ok(Some(ASTExpression::ForIn(
                            ASTForIn {
                                var,
                                iterable: iterator,
                                body,
                                range: token.range.end_with(&self.tokens.last_loc)
                            }
                        )))
                    },
                    "while" => {
                        let cond = if let Some(exp) = self.parse_expression()? { Box::from(exp) } else {
                            return Err(err!(EXPECTED, self.tokens.range_here(), self.tokens.filename, "expression"));
                        };
                        let turn_off_exp_statements = !self.allow_exp_statements;
                        self.allow_exp_statements = true;
                        let body = if let Some(exp) = self.parse_expression_or_expression_statement()? { Box::from(exp) } else {
                            return Err(err!(EXPECTED, self.tokens.range_here(), self.tokens.filename, "expression"));
                        };
                        if turn_off_exp_statements { self.allow_exp_statements = false; }
                        return Ok(Some(ASTExpression::While(
                            ASTWhile {
                                condition: cond,
                                body,
                                range: token.range.end_with(&self.tokens.last_loc)
                            }
                        )))
                    },
                    "match" => {
                        let to_get_matched = if let Some(exp) = self.parse_expression()? { Box::from(exp) } else {
                            return Err(err!(EXPECTED, self.tokens.range_here(), self.tokens.filename, "expression"));
                        };
                        self.tokens.skip_or_err(TokenType::Punc('{'), None)?;
                        let mut arms: Vec<ASTMatchArm> = vec![];
                        while !self.tokens.is_next(TokenType::Punc('}')) {
                            let match_arm_start = self.tokens.input.loc();
                            let mut possibilities: Vec<ASTMatchArmExpressions> = vec![];
                            possibilities.push(self.parse_match_arm_exp()?);
                            if self.tokens.is_next(TokenType::Op(String::from("|"))) {
                                self.tokens.consume();
                                while !self.tokens.is_next(TokenType::Op(String::from("=>"))) && !self.tokens.is_next(TokenType::Kw(String::from("if")))  {
                                    possibilities.push(self.parse_match_arm_exp()?);
                                    if self.tokens.is_next(TokenType::Op(String::from("|"))) { self.tokens.consume(); };
                                }
                            }
                            let guard = if self.tokens.is_next(TokenType::Kw(String::from("if"))) {
                                self.tokens.consume();
                                if let Some(exp) = self.parse_expression()? { Some(exp) } else {
                                    return Err(err!(EXPECTED, self.tokens.range_here(), self.tokens.filename, "expression"));
                                }
                            } else { None };

                            self.tokens.skip_or_err(TokenType::Op(String::from("=>")), None)?;

                            let body = if let Some(exp) = self.parse_expression()? { exp } else {
                                return Err(err!(EXPECTED, self.tokens.range_here(), self.tokens.filename, "expression"));
                            };
                            if self.tokens.is_next(TokenType::Punc(',')) { self.tokens.consume(); };
                            arms.push(ASTMatchArm {
                                guard,
                                possibilities,
                                body,
                                range: match_arm_start.end(&self.tokens.last_loc)
                            });
                        }
                        self.tokens.skip_or_err(TokenType::Punc('}'), None)?;
                        self.is_last_block = true;
                        return Ok(Some(ASTExpression::Match(ASTMatch {
                            arms,
                            range: token.range.end_with(&self.tokens.last_loc),
                            expression: to_get_matched
                        })))
                    },
                    "new" => {
                        let target = self.parse_mod_access_or_var_without_var(false, true)?;
                        let typings = if self.tokens.is_next(TokenType::Op(String::from("<"))) {
                            self.tokens.consume();
                            Some(self.parse_typing_list(false, false, TokenType::Op(String::from(">")))?)
                        } else { None };
                        self.tokens.skip_or_err(TokenType::Punc('{'), Some(err!(EXPECTED, self.tokens.range_here(), self.tokens.filename, "struct initializor")))?;
                        ASTExpression::Init(
                            ASTInitializor {
                                target,
                                params: self.parse_pair_list(true, '}')?,
                                typings,
                                range: token.range.end_with(&self.tokens.last_loc)
                            }
                        )
                    },
                    "await" => {
                        let optional = if self.tokens.is_next(TokenType::Op(String::from("?"))) {
                            self.tokens.consume();
                            true 
                        } else { false };
                        let expression = if let Some(exp) = self.parse_expression()? { Box::from(exp) } else {
                            return Err(err!(EXPECTED, self.tokens.range_here(), self.tokens.filename, "expression"));
                        };
                        ASTExpression::Await(
                            ASTAwait {
                                optional,
                                expression,
                                range: token.range.end_with(&self.tokens.last_loc)
                            }
                        )
                    }
                    _ => {
                        return Err(err!(EXPECTED_FOUND, token.range, self.tokens.filename, &format!("keyword \"{}\"", val)));
                    }
                }
            }
        }
        };
        Ok(Some(self.parse_suffix(exp, parse_generics_in_suffix)?))
    }

    fn parse_expression(&mut self) -> LazyResult<Option<ASTExpression>> {
        if let Some(exp) = self.parse_expression_part(true)? {
            Ok(Some(self.parse_binary(exp, 0)?))
        } else {
            Ok(None)
        }
    }

    fn parse_expression_or_expression_statement(&mut self) -> LazyResult<Option<ASTExpression>> {
        let range = self.tokens.input.loc();
        let thing = if let Some(t) = self.tokens.peek() {
            t 
        } else {
            return Err(err!(UNEXPECTED_EOF, self.tokens.range_here(), self.tokens.filename));
        };
        match &thing.val {
            TokenType::Kw(kw) => {
                match kw.as_str() {
                    "yield" => { 
                        self.tokens.consume();
                        if !self.allow_exp_statements {
                            return Err(err!(UNEXPECTED, range.end(&self.tokens.last_loc), self.tokens.filename, "yield expression"));
                        }
                        let value = if let Some(exp) = self.parse_expression()? {
                            Some(Box::from(exp))
                        } else { None };
                        Ok(Some(ASTExpression::Yield(ASTYield {
                            value,
                            range: range.end(&self.tokens.last_loc)
                        })))
                     },
                    _ => self.parse_expression()
                }
            },
            _ => self.parse_expression()
        }
    }

    fn parse_statement(&mut self) -> LazyResult<ASTStatement> {
        let range = self.tokens.input.loc();
        let token = if let Some(t) = self.tokens.consume() { t } else {
            return Err(err!(UNEXPECTED_EOF, self.tokens.range_here(), self.tokens.filename));
        };
        match &token.val {
            TokenType::Kw(keyword) => {
                match keyword.as_str() {
                   "struct" => {
                        let name = self.parse_varname(true, true, false, false)?;
                        self.tokens.skip_or_err(TokenType::Punc('{'), Some(err!(EXPECTED, self.tokens.range_here(), self.tokens.filename, "start of struct fields")))?;
                        Ok(ASTStatement::Struct(ASTStruct {
                            name: name.0,
                            typings: name.1,
                            fields: self.parse_typing_pair_list(false, true, false, true, false, '}')?,
                            range: range.end(&self.tokens.last_loc)
                        }))
                   }
                   "enum" => {
                    let name = self.parse_varname(true, true, false, false)?;
                    self.tokens.skip_or_err(TokenType::Punc('{'), Some(err!(EXPECTED, self.tokens.range_here(), self.tokens.filename, "start of enum fields")))?;
                    Ok(ASTStatement::EnumDeclaration(ASTEnumDeclaration {
                    name: name.0,
                    values: self.parse_typing_pair_list(true, false, false, false, true, '}')?,
                    typings: name.1,
                    range: range.end(&self.tokens.last_loc)
                    }))
                   },
                   "type" => {
                       let name = self.parse_varname(true, true, false, false)?;
                       self.tokens.skip_or_err(TokenType::Op(String::from("=")), None)?;
                       let typing = self.parse_typing(false, false, true)?;
                       Ok(ASTStatement::Type(
                           ASTType {
                               name: name.0,
                               typings: name.1,
                               value: typing,
                               range: range.end(&self.tokens.last_loc)
                           }
                       ))
                   },
                   "main" => {
                       if self.parsed_main {
                           return Err(err!(MANY_ENTRIES, range.end(&self.tokens.last_loc), self.tokens.filename));
                       };
                       self.tokens.skip_or_err(TokenType::Punc('{'), None)?;
                       let exp = self.parse_block(false)?;
                       self.parsed_main = true;
                       Ok(ASTStatement::Main(
                           ASTMain {
                               expression: exp,
                               range: range.end(&self.tokens.last_loc)
                           }
                       ))
                   },
                   "static" => {
                       let varname = self.parse_varname(true, false, false, false)?;
                       self.tokens.skip_or_err(TokenType::Op(String::from("=")), None)?;
                       let typings = if let Some(typing) = varname.1 {
                        let len = typing.entries.len();
                        if len == 0 || len > 1 {
                            return Err(err!(EXPECTED, token.range, self.tokens.filename, "only one type"));
                        } else {
                           Some(typing) 
                        }
                    } else { None };
                       let exp = self.parse_expression()?;
                       if exp.is_none() {
                            return Err(err!(EXPECTED, self.tokens.range_here(), self.tokens.filename, "initializor"));
                       }
                       Ok(ASTStatement::Static(
                           Box::from(ASTStatic {
                               typings,
                               var: varname.0,
                               value: exp.unwrap(),
                               range: range.end(&self.tokens.last_loc)
                           })
                       ))
                   },
                   "export" => {
                        let value = self.parse_statement()?;
                        if matches!(value, ASTStatement::Main(_)) {
                            return Err(err!(UNEXPECTED, range.end(&self.tokens.last_loc), self.tokens.filename, "main entry"));
                        }
                       Ok(ASTStatement::Export(
                           ASTExport {
                               value: Box::from(value),
                               range: range.end(&self.tokens.last_loc)
                           }
                       ))
                   },
                   "import" => {
                       let path_start = self.tokens.input.loc();
                       let item = if self.tokens.is_next(TokenType::Punc('{')) {
                            self.tokens.consume();
                            let mut items: Vec<ASTImportItem> = vec![];
                            let mut tok = self.tokens.peek();
                            while matches!(tok, Some(_)) && tok.unwrap().val != TokenType::Punc('}') {
                                let text = self.parse_varname(false, false, false, false)?.0;
                                if self.tokens.is_next(TokenType::Kw(String::from("as"))) {
                                    self.tokens.consume();
                                    let alias = self.parse_varname(false, false, false, false)?.0;
                                    let alias_range = alias.range.end;
                                    items.push(ASTImportItem { name: text.value, r#as: Some(alias), range: Range { start: text.range.start, end: alias_range } });
                                } else {
                                    items.push(ASTImportItem { name: text.value, range: text.range, r#as: None })
                                }
                                if self.tokens.is_next(TokenType::Punc('}')) {
                                    self.tokens.consume();
                                    break;
                                }
                                self.tokens.skip_or_err(TokenType::Punc(','), None)?;
                                tok = self.tokens.peek();
                            }
                            ASTImportThing::Items(items)
                        } else if self.tokens.is_next(TokenType::Op(String::from("*"))) {
                            self.tokens.consume();
                            ASTImportThing::All
                        } else {
                            return Err(err!(EXPECTED, self.tokens.range_here(), self.tokens.filename, "either an import deconstructor or a star (*)"));
                        };
                       self.tokens.skip_or_err(TokenType::Kw(String::from("from")), None)?;
                       let path = if let Some(ASTExpression::Str(string)) = self.parse_expression_part(false)? {
                           string
                       } else {
                        return Err(err!(EXPECTED, range.end(&path_start), self.tokens.filename, "path to module"));
                       };
                       let as_binding = if self.tokens.is_next(TokenType::Kw(String::from("as"))) {
                           self.tokens.consume();
                           Some(self.parse_varname(false, false, false, false)?.0)
                       } else { None };
                       Ok(ASTStatement::Import(
                           ASTImport {
                               path,
                               thing: item,
                               r#as: as_binding,
                               range: range.end(&self.tokens.last_loc)
                           }
                       ))
                   },
                   "impl" => {
                       let typings = if self.tokens.is_next(TokenType::Op(String::from("<"))) {
                           self.tokens.consume();
                           Some(self.parse_typing_list(true, false, TokenType::Op(String::from(">")))?)
                       } else { None };
                       let partial = self.parse_mod_access_or_var_without_var(false, true)?;
                       self.tokens.skip_or_err(TokenType::Kw(String::from("for")), None)?;
                       let target = self.parse_mod_access_or_var_without_var(false, true)?;
                       self.tokens.skip_or_err(TokenType::Punc('{'), None)?;
                       Ok(ASTStatement::Impl(
                           ASTImpl {
                               partial,
                               target,
                               typings,
                               fields: self.parse_typing_pair_list(false, true, false, true, false, '}')?,
                               range: range.end(&self.tokens.last_loc)
                           }
                       ))
                   },
                   _ => {
                    self.tokens.input.skip_line();
                    return Err(err!(EXPECTED_FOUND, token.range, self.tokens.filename, "statement", &token.val.to_string()));
                },
                }
            },
            TokenType::Punc('#') => {
                let name = self.parse_varname(false, false, false, true)?.0;
                let mut args: Vec<TokenType> = vec![];
                if self.tokens.is_next(TokenType::Punc('(')) {
                    self.tokens.consume();
                    let mut is_first = true;
                    while !self.tokens.is_next(TokenType::Punc(')')) {
                        if !is_first {
                            self.tokens.skip_or_err(TokenType::Punc(','), None)?;
                        }
                        if self.tokens.is_next(TokenType::Punc(')')) { break; };

                        args.push(self.tokens.consume().unwrap().val);
                        is_first = false;
                    }
                    self.tokens.skip_or_err(TokenType::Punc(')'), None)?;
                }
                let target = Box::from(self.parse_statement()?);
                Ok(ASTStatement::Meta(
                    ASTMeta {
                    name,
                    args,
                    target,
                    range: token.range.end_with(&self.tokens.last_loc)
                }))
            },
            _ => {
                self.tokens.input.skip_line();
                return Err(err!(EXPECTED_FOUND, token.range, self.tokens.filename, "statement", &token.val.to_string()));
            }
        }
    }

    pub fn parse(&mut self) -> (Vec<ASTStatement>, Vec<Error>) {
        let mut res = vec![];
        let mut errors: Vec<Error> = vec![];
        while !self.tokens.input.is_eof() {
            match self.parse_statement() {
                Ok(stmt) => res.push(stmt),
                Err(error) => errors.push(error)
            }
        }
        (res, errors)
    }

}