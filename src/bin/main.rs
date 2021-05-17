use lazy::parser::ast::{Parser};
use std::time::{Instant};

fn main() {
    let source = "

    struct A {

        A: fn(...func: (a: a, b?: b)) -> foo? {
            let a = [1, 2, 3, 4, 5, 6, 7];
            for i in 0..100 {
                if i > 10 {
                    yield for z in 0..5 {
                        yield ...4;
                    }
                }
            }
        },

        b: g
    }
    ";
    let vectored: Vec<_> = source.split('\n').collect();
    let mut p = Parser::new(&source);
    let before = Instant::now();
    let res = p.parse();
    println!("Parsing took {} nanoseconds", before.elapsed().as_nanos());
    for ast in res {
        println!("{}", ast)
    };
    for error in &p.tokens.errors {
        println!("{}", error.format(&vectored));
    }
}