use super::{Range, ASTExpression, LoC, ASTAny, ASTStatement};

pub fn full_expression_range(ast: &ASTExpression) -> Range {
            match ast {
                ASTExpression::Var(v) => v.range,
                ASTExpression::Str(v) => v.range,
                ASTExpression::Bool(v) => v.range,
                ASTExpression::Float(v) => v.range,
                ASTExpression::Int(v) => v.range,
                ASTExpression::Binary(bin) => {
                    let start = full_expression_range(&bin.left);
                    let end = full_expression_range(&bin.right);
                    Range{start: start.start, end: end.end}
                },
                ASTExpression::Unary(un) => {
                    let start = un.range;
                    let end = full_expression_range(&un.value);
                    Range { start: start.start, end: end.end }  
                },
                ASTExpression::DotAccess(access) => {
                    let start = full_expression_range(&access.value);
                    Range { start: start.start, end: access.range.end }  
                },
                ASTExpression::ArrowAccess(access) => {
                    let start = full_expression_range(&access.value);
                    Range { start: start.start, end: access.range.end }  
                },
                ASTExpression::Block(block) => block.range,
                _ => { Range { start: LoC { col: 0, pos: 0, line: 0 }, end: LoC { col: 0, pos: 0, line: 0 } } }
        }
}

pub fn expression_to_string(ast: &ASTExpression, delimiter: Option<char>) -> String {
    let unwrapped = delimiter.unwrap_or(' ');
    match ast {
        ASTExpression::Str(str) => format!("{}Str ( {} )", unwrapped, str.value),
        ASTExpression::Bool(boolean) => format!("{}Bool ( {} )", unwrapped, boolean.value),
        ASTExpression::Int(i) => format!("{}Int ( {} )", unwrapped, i.value, ),
        ASTExpression::Float(f) => format!("{}Float ( {} )", unwrapped, f.value),
        ASTExpression::Binary(bin) => format!("{}Binary ( {} {} {} )", unwrapped, expression_to_string(&bin.left, delimiter), bin.op, expression_to_string(&bin.right, delimiter)),
        ASTExpression::Unary(un) => format!("{}Unary ( {} {} )", unwrapped, un.op, expression_to_string(&un.value, delimiter)),
        ASTExpression::Var(variable) => format!("{}Var ( {} )", unwrapped, variable.value),
        ASTExpression::Optional(op) => format!("{}Optional ( {} )", unwrapped, expression_to_string(&op.value, delimiter)),
        ASTExpression::DotAccess(op) => format!("{}DotAccess ( {} . {} )", unwrapped, expression_to_string(&op.value, delimiter), op.target),
        ASTExpression::ArrowAccess(op) => format!("{}ArrowAccess ( {} -> {} )", unwrapped, expression_to_string(&op.value, delimiter), op.target),
        ASTExpression::Block(block) => {
            let mut strings: Vec<String> = vec![];
            for thing in &block.elements {
                match thing {
                    ASTAny::Expression(exp) => {
                        strings.push(expression_to_string(&exp, delimiter))
                    },
                    ASTAny::Statement(st) => {
                        strings.push(statement_to_string(&st, delimiter));
                    }
                };
            };
            format!("{}Block {{ {} }}", unwrapped, strings.join("\n"))
        },
        ASTExpression::Let(st) => format!("{}Let ( {} = {} )", unwrapped, st.var, { if st.value.is_none() { String::from("None") } else { expression_to_string(st.value.as_ref().unwrap(), delimiter) }}),
        _ => String::from("Unknown")
    }
}

pub fn statement_to_string(ast: &ASTStatement, delimiter: Option<char>) -> String {
    let _unwrapped = delimiter.unwrap_or(' ');
    match ast {
        _ => String::from("Unknown")
    } 
}

pub fn any_to_string(ast: &ASTAny, delimiter: Option<char>) -> String {
    match ast {
        ASTAny::Expression(exp) => expression_to_string(&exp, delimiter),
        ASTAny::Statement(st) => statement_to_string(&st, delimiter)
    }
 }