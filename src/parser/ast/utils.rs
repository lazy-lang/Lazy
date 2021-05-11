use super::*;

pub fn full_expression_range(ast: &ASTExpression) -> Range {
            match ast {
                ASTExpression::Var(v) => v.range,
                ASTExpression::Str(v) => v.range,
                ASTExpression::Bool(v) => v.range,
                ASTExpression::Float(v) => v.range,
                ASTExpression::Int(v) => v.range,
                ASTExpression::Binary(bin) => Range{start: full_expression_range(&bin.left).start, end: full_expression_range(&bin.right).end},
                ASTExpression::Unary(un) => Range { start: un.range.start, end: full_expression_range(&un.value).end },
                ASTExpression::DotAccess(access) => Range { start:  full_expression_range(&access.value).start, end: access.range.end },
                ASTExpression::Block(block) => block.range,
                ASTExpression::Let(l) => l.range,
                ASTExpression::Init(init) => Range { start: init.target.range.start, end: init.params.range.end},
                ASTExpression::Optional(op) => op.range,
                ASTExpression::Function(fun) => fun.range,
                ASTExpression::Iterator(init) => {
                    let start = full_expression_range(&init.start).start;
                    Range { start, end: init.range.end }
                },
                ASTExpression::Call(call) => {
                    let start = full_expression_range(&call.target).start;
                    Range { start, end: call.range.end }
                },
                ASTExpression::If(ifexp) => ifexp.range,
                ASTExpression::Char(ch) => ch.range,
                ASTExpression::EnumAccess(e) => e.range
                //_ => { Range { start: LoC { col: 0, line: 0 }, end: LoC { col: 0, line: 0 } } }
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
        ASTExpression::DotAccess(op) => format!("{}DotAccess (\n{} . {} )", unwrapped, expression_to_string(&op.value, delimiter), op.target.value),
        ASTExpression::Block(block) => block_to_string(&block, delimiter),
        ASTExpression::Function(func) => format!("{}Function ({}) -> {} {{ {} }}", unwrapped, pair_list_typing_to_string(&func.params, delimiter), if func.return_type.is_none() {String::from("void") } else { typing_to_string(func.return_type.as_ref().unwrap(), delimiter) }, if func.body.is_some() { expression_to_string(func.body.as_ref().unwrap(), delimiter) } else { String::from("{}")}),
        ASTExpression::Let(st) => format!("{}Let<{}> (\n{} = {} )", unwrapped, if st.typings.is_some() { typing_to_string(st.typings.as_ref().unwrap(), delimiter)} else { String::from("none") }, st.var.value, { if st.value.is_none() { String::from("None") } else { expression_to_string(st.value.as_ref().unwrap(), delimiter) }}),
        ASTExpression::Init(initializor) => format!("{}Init<{}> ( {} )", unwrapped, if let Some(typing) = &initializor.typings { list_typing_to_string(&typing, delimiter)} else { String::from("none") }, pair_list_to_string(&initializor.params, delimiter)),
        ASTExpression::Iterator(it) => format!("{}Iterator ({} .. {})", unwrapped, expression_to_string(&it.start, delimiter), expression_to_string(&it.end, delimiter)),
        ASTExpression::If(exp) => format!("{}If ({}) {} {}", unwrapped, expression_to_string(&exp.condition, delimiter), expression_to_string(&exp.then, delimiter), if exp.otherwise.is_some() { format!("else {}", expression_to_string(exp.otherwise.as_ref().unwrap(), delimiter)) } else { String::from("") } ),
        ASTExpression::Char(ch) => format!("{}Char ({})", unwrapped, ch.value),
        ASTExpression::EnumAccess(e) => format!("{}EnumAccess ( {}:{}({}) )", unwrapped, e.value.value, e.target.value, if e.init_value.is_some() { expression_to_string(e.init_value.as_ref().unwrap(), delimiter)} else { String::from("") }),
        ASTExpression::Call(call) => format!("{}Call {}({})", unwrapped, expression_to_string(&call.target, delimiter), pair_list_to_string(&call.args, delimiter))
        //_ => String::from("Unknown")
    }
}

pub fn typing_to_string(ast: &ASTTypings, delimiter: Option<char>) -> String {
    let unwrapped = delimiter.unwrap_or(' ');
    match ast {
        ASTTypings::Tuple(tup) => list_typing_to_string(tup, delimiter),
        ASTTypings::Var(var) => format!("{}Var<{}> ( {} )", unwrapped, if var.typings.is_some() { list_typing_to_string(var.typings.as_ref().unwrap(), delimiter) } else { String::from("none") },var.value),
        ASTTypings::PairList(list) => pair_list_typing_to_string(&list, delimiter),
        ASTTypings::Function(func) => format!("{}FunctionTyping ({}) -> {} {{ {} }}", unwrapped, pair_list_typing_to_string(&func.params, delimiter), if func.return_type.is_some() { typing_to_string(func.return_type.as_ref().unwrap(), delimiter) } else { String::from("void") }, if func.body.is_some() { expression_to_string(func.body.as_ref().unwrap(), delimiter) } else { String::from("{}")})
    }
}

pub fn list_typing_to_string(ast: &ASTListTyping, delimiter: Option<char>) -> String {
    let mut strings: Vec<String> = vec![];
    for typing in &ast.entries {
        strings.push(typing_to_string(&typing, delimiter));
    };
    strings.join(" ")
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
    format!("{{\n {} }}", pairs)
}

pub fn pair_list_typing_to_string(list: &ASTPairListTyping, delimiter: Option<char>) -> String {
    let mut pairs = String::new();
    for pair in &list.pairs {
        pairs.push_str(&format!("{}: {}{}{}", pair.name, if pair.value.is_some() { typing_to_string(pair.value.as_ref().unwrap(), delimiter) } else { String::from("None") }, if pair.optional {"?"} else { "" }, "\n"));   
    };
    format!("{}PairList {{\n {} }}", delimiter.unwrap_or(' '), pairs)
}

pub fn statement_to_string(ast: &ASTStatement, delimiter: Option<char>) -> String {
    let unwrapped = delimiter.unwrap_or(' ');
    match ast {
        ASTStatement::Struct(structure) => format!("{}Struct<{}> {} ( {} )", unwrapped, if structure.typings.is_some() { list_typing_to_string(structure.typings.as_ref().unwrap(), delimiter) } else { String::from("none") }, structure.name.value, pair_list_typing_to_string(&structure.fields, delimiter)),
        ASTStatement::EnumDeclaration(en) => format!("{}Enum {} ( {} )", unwrapped, en.name, pair_list_typing_to_string(&en.values, delimiter)),
        //_ => String::from("Unknown")
    } 
}

 pub fn get_range_or(ast: &Option<ASTExpression>, default: LoC) -> Range {
     match ast {
         Some(exp) => full_expression_range(&exp),
         None => Range { start: default, end: default }
     }
 }
