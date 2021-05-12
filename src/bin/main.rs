use lazy::parser::ast::{Parser};


fn main() {
    let source = "

    struct A<B> {

        A?: fn(func: (a: a, b: b)) {
            1 + 1;
        },

        b: g
    }
    ";
    let vectored: Vec<_> = source.split('\n').collect();
    let mut p = Parser::new(&source);
    let res = p.parse();
    for ast in res {
        println!("{}", ast)
    };
    for error in &p.tokens.errors {
        println!("{}", error.format(&vectored));
    }
}