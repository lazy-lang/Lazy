use super::{Range, ASTExpression, LoC, ASTAny, ASTStatement, ASTPairList, ASTBlock};

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
                ASTExpression::Let(l) => l.range,
                _ => { Range { start: LoC { col: 0, line: 0 }, end: LoC { col: 0, line: 0 } } }
        }
}

pub fn expression_to_string(ast: &ASTExpression, delimiter: Option<char>) -> String {
    let unwrapped = delimiter.unwrap_or(' ');
    match ast {
        ASTExpression::Str(str) => format!("{}Str ( {} )", unwrapped, str.value),
        ASTExpression::Bool(boolean) => format!("{}Bool ( {} )", unwrapped, boolean.value),
        ASTExpression::Int(i) => format!("{}Int ( {} )", unwrapped, i.value, ),
        ASTExpression::Float(f) => format!("{}Float ( {} )", unwrapped, f.value),
        ASTExpression::Binary(bin) => format!("{}Binary (\n {} {} {} )", unwrapped, expression_to_string(&bin.left, delimiter), bin.op, expression_to_string(&bin.right, delimiter)),
        ASTExpression::Unary(un) => format!("{}Unary ( {} {} )", unwrapped, un.op, expression_to_string(&un.value, delimiter)),
        ASTExpression::Var(variable) => format!("{}Var ( {} )", unwrapped, variable.value),
        ASTExpression::Optional(op) => format!("{}Optional ( {} )", unwrapped, expression_to_string(&op.value, delimiter)),
        ASTExpression::DotAccess(op) => format!("{}DotAccess (\n{} . {} )", unwrapped, expression_to_string(&op.value, delimiter), op.target),
        ASTExpression::ArrowAccess(op) => format!("{}ArrowAccess (\n{} -> {} )", unwrapped, expression_to_string(&op.value, delimiter), op.target),
        ASTExpression::Block(block) => block_to_string(&block, delimiter),
        ASTExpression::Function(func) => format!("{}Function ({}) -> {} {{ {} }}", unwrapped, pair_list_to_string(&func.params, delimiter), if func.return_type.is_none() {String::from("void") } else { expression_to_string(func.return_type.as_ref().unwrap(), delimiter) }, block_to_string(&func.body, delimiter)),
        ASTExpression::Let(st) => format!("{}Let (\n{} = {} )", unwrapped, st.var, { if st.value.is_none() { String::from("None") } else { expression_to_string(st.value.as_ref().unwrap(), delimiter) }}),
        //_ => String::from("Unknown")
    }
}

pub fn block_to_string(block: &ASTBlock, delimiter: Option<char>) -> String {
    let mut strings: Vec<String> = vec![];
    for thing in &block.elements {
        strings.push(expression_to_string(&thing, delimiter))
    };
    format!("{}Block {{ {} }}", delimiter.unwrap_or(' '), strings.join("\n"))
} 

pub fn pair_list_to_string(list: &ASTPairList, delimiter: Option<char>) -> String {
    let mut pairs = String::new();
    for pair in &list.pairs {
        pairs.push_str(&format!("{}: {}{}", pair.0, if pair.1.is_some() { expression_to_string(pair.1.as_ref().unwrap(), delimiter) } else { String::from("None") }, "\n"));   
    };
    format!("{}PairList {{\n {} }}", delimiter.unwrap_or(' '), pairs)
}

pub fn statement_to_string(ast: &ASTStatement, delimiter: Option<char>) -> String {
    let unwrapped = delimiter.unwrap_or(' ');
    match ast {
        ASTStatement::Struct(structure) => format!("{}Struct ( {} = {} )", unwrapped, structure.name, pair_list_to_string(&structure.fields, delimiter)),
        _ => String::from("Unknown")
    } 
}

pub fn any_to_string(ast: &ASTAny, delimiter: Option<char>) -> String {
    match ast {
        ASTAny::Expression(exp) => expression_to_string(&exp, delimiter),
        ASTAny::Statement(st) => statement_to_string(&st, delimiter)
    }
 }

 pub fn get_range_or(ast: &Option<ASTExpression>, default: LoC) -> Range {
     match ast {
         Some(exp) => full_expression_range(&exp),
         None => Range { start: default, end: default }
     }
 }
