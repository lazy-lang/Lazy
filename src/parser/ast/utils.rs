use super::{Range, ASTExpression, LoC};

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
                }
            _ => { Range { start: LoC { col: 0, pos: 0, line: 0 }, end: LoC { col: 0, pos: 0, line: 0 } } }
        }
}