
use super::*;
use std::fmt;

pub enum MacroExpressionTypes {
    Expr,
    Pat,
    Ident
}

pub enum MacroExpressionTypesWithValues {
    Expr(ASTExpression),
    Pat(ASTMatchArmExpressions),
    Ident(ASTVar)
}

pub enum MacroRepeatingTypes {
    None,
    ZeroOrMore,
    OneOrMore
}

pub struct MacroParameter {
    pub exp_type: MacroExpressionTypes,
    pub followed_by: Option<char>,
    pub repeat_type: MacroRepeatingTypes,
    pub name: String
}

pub struct Macro {
    pub parameters: Vec<MacroParameter>,
    pub body: ASTBlock,
    pub range: Range
}

fn resolve_exp_type(t: &str) -> Option<MacroExpressionTypes> {
    match t {
        "expr" => Some(MacroExpressionTypes::Expr),
        "pat" => Some(MacroExpressionTypes::Pat),
        "ident" => Some(MacroExpressionTypes::Ident),
        _ => None
    }
}

pub fn parse_macro(parser: &mut Parser) -> Option<(String, Macro)> {
    let range = parser.tokens.recorder();
    let macro_name = if let Some(var) = parser.parse_varname(false, false, false).0 {
        var.value
    } else {
        parser.tokens.error_here(ErrorType::Expected(String::from("macro name")));
        return None;
    };
    parser.tokens.skip_or_err(TokenType::Punc('('), None, None);
    let mut parameters: Vec<MacroParameter> = vec![];
    let mut has_repeater = false;
    while !parser.tokens.is_next(TokenType::Punc(')')) {
        let name = if let Some(var) = parser.parse_varname(false, false, false).0 {
            var.value
        } else { break; };
        parser.tokens.skip_or_err(TokenType::Punc(':'), None, None);
        let param_type = if let Some(var) = parser.parse_varname(false, false, false).0 {
            let t = resolve_exp_type(&var.value);
            match t {
                None => {
                    var.range.err(ErrorType::Invalid(String::from("expression type")), &mut parser.tokens);
                    break;
                },
                Some(v) => v
            }
        } else {
            break;
        };
        let followed_by = {
            let p = parser.tokens.peek();
            if p.is_none() { return None };
            match &p.unwrap().val {
                TokenType::Punc(ch) => {
                    match ch {
                    ',' | ';' | '@' | '#' | ':' => {
                        let ch_clone = ch.clone();
                        parser.tokens.consume();
                        Some(ch_clone)
                    },
                    ')' => {
                        None
                    }
                    _ => {
                        let chstr = ch.to_string();
                        parser.tokens.error_here(ErrorType::ExpectedFound(String::from("separator"), chstr)); 
                        None
                    }
                    }
                },
                TokenType::Op(op) if op == "+" || op == "*" => {
                    parser.tokens.error_here(ErrorType::ExpectedFound(String::from("separator"), String::from("repeat operator")));
                    None
                },
                _ => {
                    parser.tokens.error_here(ErrorType::Expected(String::from("separator")));
                    None
                }
            }
        };
        let repeat_type = {
            let p = parser.tokens.peek();
            if p.is_none() { return None };
            match &p.unwrap().val {
                TokenType::Op(op) => {
                    match op.as_str() {
                        "+" => {
                            if has_repeater {
                                parser.tokens.error_here(ErrorType::Unexpected(String::from("repeat operator")))
                            }
                            parser.tokens.consume();
                            has_repeater = true;
                            MacroRepeatingTypes::OneOrMore
                        },
                        "*" => {
                            if has_repeater {
                                parser.tokens.error_here(ErrorType::Unexpected(String::from("repeat operator")))
                            }
                            parser.tokens.consume();
                            has_repeater = true;
                            MacroRepeatingTypes::ZeroOrMore
                        },
                        _ => MacroRepeatingTypes::None
                    }
                },
                _ => MacroRepeatingTypes::None
            }
        };
        parameters.push(MacroParameter { name, exp_type: param_type, followed_by, repeat_type });
    }
    parser.tokens.skip_or_err(TokenType::Punc(')'), None, None);
    parser.tokens.skip_or_err(TokenType::Punc('{'), None, None);
    parser.allow_macro_exps = true;
    let body = parser.parse_block(false);
    parser.allow_macro_exps = false;
    Some((macro_name, Macro {
            parameters,
            body,
            range: range.end(&parser.tokens)
    }))
}

pub fn run_macro(args: Vec<ASTExpression>, mac: &Macro) -> Option<ASTExpression> {
    None
}

impl fmt::Display for MacroExpressionTypes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Expr => write!(f, "expr"),
            Self::Pat => write!(f, "pat"),
            Self::Ident => write!(f, "ident")
        }
   }
}


impl fmt::Display for MacroRepeatingTypes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::OneOrMore => write!(f, "+"),
            Self::ZeroOrMore => write!(f, "*"),
            Self::None => write!(f, "")
        }
   }
}


impl fmt::Display for Macro {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut params = String::new();
        for param in &self.parameters {
            params += &format!("{}: {}{}{}", param.name, param.exp_type, param.repeat_type, if param.followed_by.is_some() { param.followed_by.unwrap() } else { ' ' });
        };
        writeln!(f, "macro ({}) {}", params, self.body)
   }
}