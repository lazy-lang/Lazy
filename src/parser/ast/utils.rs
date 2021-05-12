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
                ASTExpression::ForIn(for_in) => for_in.range,
                ASTExpression::While(while_loop) => while_loop.range,
                ASTExpression::If(ifexp) => ifexp.range,
                ASTExpression::Char(ch) => ch.range,
                ASTExpression::EnumAccess(e) => e.range
        }
}

 pub fn get_range_or(ast: &Option<ASTExpression>, default: LoC) -> Range {
     match ast {
         Some(exp) => full_expression_range(&exp),
         None => Range { start: default, end: default }
     }
 }
