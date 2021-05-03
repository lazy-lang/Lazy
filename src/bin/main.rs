use lazy::parser::ast::{Parser};
use lazy::parser::ast::model::{ASTAny};
use lazy::parser::ast::utils::{expression_to_string};


fn main() {
    let source = "
    
    /*
    Multi-line
    comment */

    // Single line comment
    // Another comment

    a.b.c->d->a;

    ";
    let mut p = Parser::new(&source);
    let res = p.parse();
    for ast in res {
        if let ASTAny::Expression(val) = ast {
            println!("{}\n", expression_to_string(&val, Some('\n')));
        }
    };
    for error in &p.tokens.errors {
        println!("{}", error.format(&source));
    }
}