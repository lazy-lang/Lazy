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
                ASTExpression::Declare(l) => l.range,
                ASTExpression::Init(init) => init.range,
                ASTExpression::Optional(op) => {
                    let start = full_expression_range(&op.value).start;
                    Range { start, end: op.range.end }
                },
                ASTExpression::Function(fun) => fun.range,
                ASTExpression::Iterator(init) => {
                    let start = full_expression_range(&init.start).start;
                    Range { start, end: init.range.end }
                },
                ASTExpression::Call(call) => {
                    let start = full_expression_range(&call.target).start;
                    Range { start, end: call.range.end }
                },
                ASTExpression::ForIn(for_in) => for_in.range,
                ASTExpression::While(while_loop) => while_loop.range,
                ASTExpression::If(ifexp) => ifexp.range,
                ASTExpression::Char(ch) => ch.range,
                ASTExpression::ModAccess(e) => e.range,
                ASTExpression::Tuple(tup) => tup.range,
                ASTExpression::Yield(y) => y.range,
                ASTExpression::Spread(sp) => sp.range,
                ASTExpression::Match(mtch) => mtch.range,
                ASTExpression::Await(aw) => aw.range,
                ASTExpression::MacroRepeat(r) => r.range,
                ASTExpression::None(range) => *range
        }
}

pub fn is_natural_iter(ast: &ASTIterator) -> bool {
    matches!(*ast.start, ASTExpression::Char(_) | ASTExpression::Int(_)) && matches!(*ast.end, ASTExpression::Char(_) | ASTExpression::Int(_))
}

pub fn is_natural_tuple(ast: &ASTExpressionList) -> bool {
    for value in &ast.expressions {
        if !is_natural_val(value) { return false };
    }
    true
}

pub fn is_natural_val(ast: &ASTExpression) -> bool {
    match ast {
        ASTExpression::Char(_) | ASTExpression::Int(_) | ASTExpression::Float(_) | ASTExpression::Str(_) | ASTExpression::None(_) | ASTExpression::Bool(_) => true,
        ASTExpression::Iterator(iter) => is_natural_iter(iter),
        ASTExpression::Tuple(tup) => is_natural_tuple(tup),
        _ => false
    }
}

 pub fn get_range_or(ast: &Option<ASTExpression>, default: LoC) -> Range {
     match ast {
         Some(exp) => full_expression_range(&exp),
         None => Range { start: default, end: default }
     }
 }