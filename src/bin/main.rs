use lazy::parser::ast::{Parser};
use lazy::parser::ast::utils::{any_to_string};


fn main() {
    let source = "
    
    /*
    Multi-line
    comment */

    // Single line comment
    {
       let a = {};
       let b =
       let c;
       let a = {};
       let a = {};
       let a = {};
    ";
    let vectored = source.split('\n').collect();
    let mut p = Parser::new(&source);
    let res = p.parse();
    for ast in res {
        println!("{}", any_to_string(&ast, Some('\n')))
    };
    for error in &p.tokens.errors {
        println!("{}", error.format(&vectored));
    }
}