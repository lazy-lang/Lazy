use lazy::parser::input_parser::InputParser;

fn main() {
    let mut p = InputParser::new("Hello!");
    while !p.is_eof() {
        println!("{} (Line: {}, Col: {})", p.consume().unwrap(), p.line, p.col);
    }
}