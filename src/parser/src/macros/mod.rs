use errors::*;
use diagnostics::*;

use crate::ast::Parser;
use crate::tokenizer::*;
use crate::input_parser::InputSequence;
use std::collections::HashMap;
use crate::ast::model::*;

pub enum MacroArgTypes {
    Identifier,
    Expression,
    Statement
}

pub enum MacroRepetitionKinds {
    ZeroOrMore,
    OneOrMore
}

pub enum MacroToken {
    Regular(Token),
    Repetition(MacroRepetitionKinds, Token, Vec<MacroToken>),
    Var(String)
}

pub enum MacroArg {
    Exact(char),
    Typed{
        name: String,
        value: MacroArgTypes
    },
    Repetition{
        kind: MacroRepetitionKinds,
        name: String,
        value: MacroArgTypes,
        separator: char
    }
}

pub enum MacroArgInstance {
    Multiple(Vec<ASTAny>),
    Single(ASTAny)
}

pub struct MacroArm {
    pub args: Vec<MacroArg>,
    pub body: Vec<MacroToken>
}

pub struct Macro(Vec<MacroArm>);

impl Macro {

    fn parse_rep(tokens: &mut Tokenizer) -> Option<Vec<MacroToken>> {
        let mut toks: Vec<MacroToken> = Vec::new();
        let mut punc_stack = 0;
        while let Some(ch) = tokens.input.peek(0) {
            match ch {
                '$' => {
                    tokens.consume_input();
                    toks.push(MacroToken::Var(tokens.parse_text()));
                }
                '(' => {
                    tokens.consume_input();
                    punc_stack += 1;
                }
                ')' => {
                    punc_stack -= 1;
                    if punc_stack < 1 {
                        tokens.consume_input();
                        break;
                    }
                },
                ' ' | '\n' | '\t' | '\r' => {
                    tokens.consume_input();
                },
                _ => toks.push(MacroToken::Regular(tokens.consume()?))
            }
        };
        Some(toks)
    }

    pub fn parse(tokens: &mut Tokenizer) -> Option<Macro> {
        let mut arms: Vec<MacroArm> = vec![];
        loop {
            if tokens.is_next(TokenType::Punc('}')) { break };
            let mut args: Vec<MacroArg> = vec![];
            // Macro parameters
            while let Some(ch) = tokens.consume_input() {
                match ch {
                    ' ' | '\n' | '\t' | '\r' => continue,
                    '=' if tokens.input.peek(0)? == '>' => {
                        tokens.consume_input();
                        break;
                    },
                    '$' => {
                        let name = tokens.parse_text();
                        tokens.skip_or_err_here(TokenType::Punc(':'))?;
                        let val = match tokens.parse_text().as_str() {
                            "ident" => MacroArgTypes::Identifier,
                            "expr" => MacroArgTypes::Expression,
                            "stmt" => MacroArgTypes::Statement,
                            unknown @ _ => {
                                tokens.errors.push(err!(INVALID_MACRO_PARAM, tokens.range_here(), tokens.filename, unknown));
                                return None;
                            }
                        };
                        args.push(MacroArg::Typed{
                            name,
                            value: val
                        });
                    },
                    '+' | '*' => {
                        let name = tokens.parse_text();
                        tokens.skip_or_err_here(TokenType::Punc(':'))?;
                        let val = match tokens.parse_text().as_str() {
                            "ident" => MacroArgTypes::Identifier,
                            "expr" => MacroArgTypes::Expression,
                            "stmt" => MacroArgTypes::Statement,
                            unknown @ _ => {
                                tokens.errors.push(err!(INVALID_MACRO_PARAM, tokens.range_here(), tokens.filename, unknown));
                                return None;
                            }
                        };
                        let kind = match ch {
                            '+' => MacroRepetitionKinds::OneOrMore,
                            _ => MacroRepetitionKinds::ZeroOrMore
                        };
                        let separator = tokens.consume_input()?;
                        args.push(MacroArg::Repetition{
                            kind,
                            name,
                            value: val,
                            separator
                        })
                    },
                    _ => args.push(MacroArg::Exact(ch))
                }
            }
            tokens.skip_or_err_here(TokenType::Punc('{'))?;
            // Macro body
            let mut body_toks: Vec<MacroToken> = Vec::new();
            let mut punc_stack = 0;
            while let Some(ch) = tokens.input.peek(0) {
                match ch {
                    '$' => {
                        tokens.consume_input();
                        body_toks.push(MacroToken::Var(tokens.parse_text()));
                    }
                    '+' | '*' => {
                        tokens.consume_input();
                        let rep_type = match ch {
                            '+' => MacroRepetitionKinds::OneOrMore,
                            _ => MacroRepetitionKinds::ZeroOrMore
                        };
                        let tok = tokens.consume()?;
                        match &tok.val {
                            TokenType::Punc(separator) => {
                                if separator == &'(' {
                                    tokens.errors.push(err!(EXPECTED_FOUND, tokens.range_here(), tokens.filename, "separator", "start of repetition"));
                                    return None;
                                }
                                tokens.skip_or_err_here(TokenType::Punc('('))?;
                                body_toks.push(MacroToken::Repetition(rep_type, tok, Self::parse_rep(tokens)?))
                            },
                            TokenType::Op(_) => {
                                tokens.skip_or_err_here(TokenType::Punc('('))?;
                                body_toks.push(MacroToken::Repetition(rep_type, tok, Self::parse_rep(tokens)?))
                            },
                            _ => body_toks.push(MacroToken::Regular(tok))
                        }
                    },
                    '{' => {
                        punc_stack += 1;
                        body_toks.push(MacroToken::Regular(tokens.consume()?));
                    },
                    '}' => {
                        punc_stack -= 1;
                        tokens.consume();
                        if punc_stack < 1 {
                            break;
                        }
                    },
                    ' ' | '\n' | '\t' | '\r' => {
                        tokens.consume_input();
                    },
                    _ => body_toks.push(MacroToken::Regular(tokens.consume()?))
                }
            }
            arms.push(MacroArm {
                args,
                body: body_toks
            })
        };
        tokens.consume();
        Some(Macro(arms))
    }

    // Assumes the name has already been eaten.
    fn match_arm(&self, parser: &mut Parser) -> Option<HashMap<String, MacroArgInstance>> {
        let mut res: HashMap<String, MacroArgInstance> = HashMap::new();
        let first_arm = &self.0[0];
        for arm_tok in &first_arm.args {
            match arm_tok {
                MacroArg::Exact(ch) => {
                    let p_ch = parser.tokens.consume_input()?;
                    if ch != &p_ch { return None };
                },
                MacroArg::Typed{name, value} => {
                    let exp = match value {
                        MacroArgTypes::Expression => parser.parse_expression()
                    }
                }
            }
        };
        None
    }

    fn replace(&self) -> Vec<Token> {
        Vec::new()
    }
}
