use lazy::parser::ast::{Parser};
use lazy::parser::ast::model::{ASTAny, ASTExpression};

fn print_ast(ast: &ASTExpression) {
    match ast {
        ASTExpression::Str(string) => {
            print!("{}", string.value);
        },
        ASTExpression::Float(string) => {
            print!("{}", string.value);
        }
        ASTExpression::Int(string) => {
            print!("{}", string.value);
        }
        ASTExpression::Bool(string) => {
            print!("{}", string.value);
        }
        ASTExpression::Var(string) => {
            print!("{}", string.value);
        }
        ASTExpression::Binary(binary) => {
            print_ast(&binary.left);
            print!("{}", binary.op);
            print_ast(&binary.right);
        }
        _ => {}
    }
}

fn main() {
    let source = "\"Hi\" + \"3.1.4\" + (3 + 3)";
    let mut p = Parser::new(&source);
    let res = p.parse();
    for ast in res {
        match ast {
            ASTAny::Expression(val) => {             
                print_ast(&val);
            }
            _ => {}
        }
    };
    for error in &p.tokens.errors {
        println!("{}", error.format(&source));
        break;
    }
}