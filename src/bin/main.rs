use lazy::parser::ast::{Parser};
use lazy::parser::ast::utils::{statement_to_string};


fn main() {
    let source = "
    
    /*
    Multi-line
    comment */

    // Single line comment
    struct A {

        A: fn() {
            if false {
                'a' + 1;
            }
        },

    }
    ";
    let vectored: Vec<_> = source.split('\n').collect();
    let mut p = Parser::new(&source);
    let res = p.parse();
    for ast in res {
        println!("{}", statement_to_string(&ast, Some('\n')))
    };
    for error in &p.tokens.errors {
        println!("{}", error.format(&vectored));
    }
}