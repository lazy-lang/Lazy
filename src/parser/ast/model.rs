
use super::Range;

// A string literal
pub struct ASTStr {
    pub value: String,
    pub range: Range
}

// A floating point literal 
pub struct ASTFloat {
    pub value: f32,
    pub range: Range
}

// An integer literal
pub struct ASTInt {
    pub value: i32,
    pub range: Range
}

//  A boolean literal
pub struct ASTBool {
    pub value: bool,
    pub range: Range
}

// A variable / typing name  
pub struct ASTVar {
    pub value: String,
    pub range: Range
}

// let statement
pub struct ASTLet {
    pub id: String,
    pub value: ASTExpression,
    pub range: Range,
}

// A pair list {key: value}
pub struct ASTPairList {
    pub pairs: std::collections::HashMap::<String, Box<ASTExpression>>,
    pub range: Range
}

// A binary expression
pub struct ASTBinary {
    pub op: String,
    pub left: Box<ASTExpression>,
    pub right: Box<ASTExpression>,
    pub range: Range
}

pub struct ASTUnary {
    pub op: String,
    pub value: Box<ASTExpression>,
    pub range: Range
}

pub struct ASTDotAccess {
    pub value: Box<ASTExpression>,
    pub target: String,
    pub range: Range
}

pub struct ASTArrowAccess {
    pub value: Box<ASTExpression>,
    pub target: String,
    pub range: Range 
}

pub struct ASTOptional {
    pub value: Box<ASTExpression>,
    pub range: Range
}

pub struct ASTEnumDeclaration {
    pub name: ASTVar,
    pub values: ASTPairList,
    pub range: Range
}

// A block of expressions or statements 
pub struct ASTBlock {
    pub elements: Vec<ASTAny>,
    pub range: Range
}

// Event typing (param: value_type)
pub struct ASTEventType {
    pub pairs: std::collections::HashMap::<String, ASTVar>,
    pub range: Range
}

// Typings list <..., ... ,...>
pub struct ASTTypeList {
    pub types: Vec<ASTTypings>,
    pub range: Range
}

// Any expression
pub enum ASTExpression {
    Str(ASTStr),
    Float(ASTFloat),
    Int(ASTInt),
    Bool(ASTBool),
    Var(ASTVar),
    PairList(ASTPairList),
    Binary(ASTBinary),
    Unary(ASTUnary),
    DotAccess(ASTDotAccess),
    ArrowAccess(ASTArrowAccess),
    Optional(ASTOptional)
}

// Any statement
pub enum ASTStatement {
    Let(ASTLet),
    EnumDeclaration(ASTEnumDeclaration)
}

// Any
pub enum ASTAny {
    Expression(ASTExpression),
    Statement(ASTStatement)
}

// Typings are only allowed:
// - After the 'with' keyword
// - After an identifier, which is after either the 'enum' or 'struct' keyword
pub enum ASTTypings {
    Var(ASTVar),
    PairList(ASTPairList),
    EventType(ASTEventType)
}
