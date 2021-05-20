use lazy::parser::ast::{Parser};
use std::time::{Instant};

fn main() {
    let source = "

    enum A<A, B> {
        a,
        b,
        c
    }

    struct Smth {
        a: fn<A, B>(a: bool) -> A {
            console?.log(1);
        },
        static const private b?: bool
    }

    static a<B> = [1, 2, 3];

    main {

        some_fn<A, B>();
        let myType = new A<A>{};
        
        let a = [1, 2, 3, 4, 5, 6, none];
        let res = match a {
            1 => {},
            2 | 3 | 4 | 5 | \"str\" => {},
            1..4 => {
                print(\"a is in range 1 - 3\");
            },
            none => 1 + 1,
            true | false => print(1 + 5),
            Option::None => {},
            Option::Some(true) => print(\"Got Some!!!\"),
            Number::Int when 1 > 5 => {},
            3..5 => {},
            [1, 2, 3] => print('c'),
            10..=15 when a == \"hello\" => {},
            _ => {}
        }
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