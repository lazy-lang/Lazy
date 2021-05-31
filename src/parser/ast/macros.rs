
use super::*;
use std::collections::HashMap;

pub enum RepeatTypes {
    ZeroOrMore,
    OneOrMore,
    None
}

pub struct MacroParameter {
    pub name: String,
    pub followed_by: Option<char>,
    pub repeat_type: RepeatTypes
}

pub struct Macro {
    pub params: Vec<MacroParameter>,
    pub body: Vec<Token>
}

pub struct MacroParser {
    macros: HashMap<String, Macro>
}

impl MacroParser {

    pub fn new() -> Self {
        MacroParser {
            macros: HashMap::new()
        }
    }

    pub fn parse_macro(&mut self, tokens: &mut dyn Tokenizer) -> Option<bool> {
        let macro_name = if let TokenType::Var(v) = tokens.consume()?.val {
            v
        } else {
            tokens.error_here(ErrorType::Expected(String::from("Macro name")));
            return None;
        };
        tokens.skip_or_err(TokenType::Punc('('), None, None);
        let mut params: Vec<MacroParameter> = vec![];
        let mut has_repeat = false;
        while !tokens.is_next(TokenType::Punc(')')) {
            let param_name = if let TokenType::Var(v) = tokens.consume()?.val {
                v
            } else {
                tokens.error_here(ErrorType::Expected(String::from("Macro parameter name")));
                return None;
            };
            let followed_by = match &tokens.peek()?.val {
                TokenType::Op(op) => {
                    match op.as_str() {
                        "+" | "*" => {
                            tokens.error_here(ErrorType::Expected(String::from("Separator before repeat operator")));
                            None
                        },
                        "|" => { tokens.consume(); Some('|') },
                        "." => { tokens.consume(); Some('.') },
                        _ => {
                            tokens.error_here(ErrorType::Expected(String::from("Invalid separator")));
                            None
                        }
                    }
                },
                TokenType::Punc(p) => {
                    match p {
                        ';' | ',' | ':' => { 
                            let clone_of_the_char = p.clone();
                            tokens.consume(); 
                            Some(clone_of_the_char) 
                        },
                        ')' => { tokens.consume(); None },
                        _ => {
                            tokens.error_here(ErrorType::Expected(String::from("Invalid separator")));
                            None
                        }
                    }
                },
                _ => {
                    tokens.error_here(ErrorType::Expected(String::from("Invalid separator")));
                    None
                }
            };
            let repeat_type = match &tokens.peek()?.val {
                TokenType::Op(op) => {
                    match op.as_str() {
                        "+" => {
                            if has_repeat {
                                tokens.error_here(ErrorType::Expected(String::from("Multiple repeat parameters are not allowed.")));
                            }
                            tokens.consume();
                            has_repeat = true;
                            RepeatTypes::OneOrMore
                        },
                        "*" => {
                            if has_repeat {
                                tokens.error_here(ErrorType::Expected(String::from("Multiple repeat parameters are not allowed.")));
                            }
                            tokens.consume();
                            has_repeat = true;
                            RepeatTypes::ZeroOrMore
                        }
                        _ => RepeatTypes::None
                    }
                },
                _ => RepeatTypes::None
            };
            params.push(MacroParameter { name: param_name, followed_by, repeat_type });
        }; 
        tokens.skip_or_err(TokenType::Punc(')'), None, None);
        tokens.skip_or_err(TokenType::Punc('{'), err: Option<ErrorType>, _loc: Option<Range>)
        None
    }

}