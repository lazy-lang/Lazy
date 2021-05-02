

// A string literal
pub struct ASTStr {
    pub value: String
}

// A floating point literal 
pub struct ASTFloat {
    pub value: f32
}

// An integer literal
pub struct ASTInt {
    pub value: i32
}

//  A boolean literal
pub struct ASTBool {
    pub value: bool
}

// A variable / typing name  
pub struct ASTVar {
    pub value: String
}

// let statement
pub struct ASTLet {
    pub id: String,
    pub value: ASTExpression
}

// A pair list {key: value}
pub struct ASTPairList {
    pub pairs: std::collections::HashMap::<String, Box<ASTExpression>>
}

// A binary expression
pub struct ASTBinary {
    pub op: String,
    pub left: Box<ASTExpression>,
    pub right: Box<ASTExpression>
}

pub struct ASTEnumDeclaration {
    pub name: ASTVar,
    pub values: ASTPairList
}

// A block of expressions or statements 
pub struct ASTBlock {
    pub elements: Vec<ASTAny>
}

// Event typing (param: value_type)
pub struct ASTEventType {
    pub pairs: std::collections::HashMap::<String, ASTVar>
}

// Typings list <..., ... ,...>
pub struct ASTTypeList {
    pub types: Vec<ASTTypings>
}

// Any expression
pub enum ASTExpression {
    Str(ASTStr),
    Float(ASTFloat),
    Int(ASTInt),
    Bool(ASTBool),
    Var(ASTVar),
    PairList(ASTPairList),
    Binary(ASTBinary)
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