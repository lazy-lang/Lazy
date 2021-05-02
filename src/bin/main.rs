use lazy::parser::ast::{Parser};
use lazy::parser::ast::model::{ASTAny, ASTExpression};
use lazy::parser::ast::utils::{full_expression_range};

fn print_ast(ast: &ASTExpression, main: bool) {
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
            print_ast(&binary.left, false);
            print!(" {} ", binary.op);
            print_ast(&binary.right, false);
            if main {
                print!("(Range: {}", full_expression_range(ast));
            }
        }
        _ => {}
    }
}

fn main() {
    let source = "\"Hi\" + \"3.1.4\" + (3 + 3)\n+3";
    let mut p = Parser::new(&source);
    let res = p.parse();
    for ast in res {
        match ast {
            ASTAny::Expression(val) => {             
                print_ast(&val, true);
            }
            _ => {}
        }
    };
    for error in &p.tokens.errors {
        println!("{}", error.format(&source));
        break;
    }
}