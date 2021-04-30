use lazy::parser::tokenizer::{Tokenizer, TokenType, ConsumeResult};

fn main() {
    let mut p = Tokenizer::new("\"Hi\" 3.14 hello
    
    emit .||");
    while !p.input.is_eof() {
        let tok = p.consume();
        match tok {
            ConsumeResult::Token(token) => {
            match token.val {
            TokenType::Str(val) => {
                println!("Found string: {} {}", val, token.range)
            },
            TokenType::Int(val) => {
                println!("Found integer: {} {}", val, token.range);
            },
            TokenType::Float(val) => {
                println!("Found float: {} {}", val, token.range);
            },
            TokenType::Var(val) => {
                println!("Found variable: {} {}", val, token.range);
            },
            TokenType::Kw(val) => {
                println!("Found keyword: {} {}", val, token.range);
            },
            TokenType::Op(val) => {
                println!("Found operator: {} {}", val, token.range);
            },
            TokenType::Punc(val) => {
                println!("Found punctuation: {} {}", val, token.range);
            }
        }
    },
    ConsumeResult::Invalid(chr) => {
        println!("Invalid character {}", chr);
    },
    _ => {}
    }
    }
}