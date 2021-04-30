use lazy::parser::tokenizer::{Tokenizer, TokenType};

fn main() {
    let mut p = Tokenizer::new("\"Hi\" 3.14");
    while !p.input.is_eof() {
        let tok = p.consume();
        match tok {
            TokenType::Str(val) => {
                println!("Found string: {}", val)
            },
            TokenType::Int(val) => {
                println!("Found integer: {}", val);
            },
            TokenType::Float(val) => {
                println!("Found float: {}", val);
            },
            TokenType::Invalid(val) => {
                println!("Invalid character {}", val);
            }
            _ => {}
        };
    }
}