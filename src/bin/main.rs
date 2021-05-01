use lazy::parser::tokenizer::{Tokenizer, TokenType};

fn main() {
    let source = "\"Hi\" \"3.1.4\";
    
    emit .|| ... . . \"Hello there";
    let mut p = Tokenizer::new(&source);
    while !p.input.is_eof() {
        let tok = p.next();
        match tok {
            Some(token) => {
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
    _ => {}
    }
    }
    for error in &p.errors {
        println!("{}", error.format(&source));
        break;
    }
}