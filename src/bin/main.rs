use lazy::parser::ast::{Parser};
use lazy::parser::ast::model::{ASTAny, ASTExpression};

fn print_ast(ast: &ASTExpression) {
    match ast {
        ASTExpression::Str(string) => {
            println!("String: {}", string.value);
        },
        ASTExpression::Float(string) => {
            println!("Float: {}", string.value);
        }
        ASTExpression::Int(string) => {
            println!("Int: {}", string.value);
        }
        ASTExpression::Bool(string) => {
            println!("Bool: {}", string.value);
        }
        ASTExpression::Var(string) => {
            println!("Variable: {}", string.value);
        },
        ASTExpression::Binary(binary) => {
            print_ast(&binary.left);
            println!("Binary: {}", binary.op);
            print_ast(&binary.right);
        }
        ASTExpression::Unary(un) => {
            println!("Unary: {}, ", un.op);
            print_ast(&un.value);
        },
        ASTExpression::DotAccess(path) => {
            print_ast(&path.value);
            println!("Dot access: {} (optional: {})", path.target, path.optional);
        }
        _ => {}
    }
}

fn main() {
    let source = "hello.world.reee";
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