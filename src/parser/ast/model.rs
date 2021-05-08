
use super::Range;

// A string literalz
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
    pub var: String,
    pub value: Option<Box<ASTExpression>>,
    pub range: Range,
}

pub struct ASTStruct {
    pub name: String,
    pub fields: ASTPairListTyping
}

// A key value pair list
pub struct ASTPairList {
    pub pairs: Vec<(String, Option<ASTExpression>)>,
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
    pub values: ASTPairListTyping,
    pub range: Range
}

pub struct ASTFunction {
    pub params: Box<ASTPairListTyping>,
    pub body: Option<ASTBlock>,
    pub return_type: Option<Box<ASTTypings>>,
    pub range: Range
}

// A block of expressions or statements 
pub struct ASTBlock {
    pub elements: Vec<ASTExpression>,
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
    VarTyping(ASTVar),
    Binary(ASTBinary),
    Unary(ASTUnary),
    DotAccess(ASTDotAccess),
    ArrowAccess(ASTArrowAccess),
    Optional(ASTOptional),
    Block(ASTBlock),
    Function(ASTFunction),
    Let(ASTLet)
}

// Any statement
pub enum ASTStatement {
    EnumDeclaration(ASTEnumDeclaration),
    Struct(ASTStruct)
}

// Any
pub enum ASTAny {
    Expression(ASTExpression),
    Statement(ASTStatement)
}


// Typings
// typing_name
// TypingName<generics>
// {key: typing_name},
// (param: typing_name) -> return_type
// [type, type] 

// TypingName, TypingName<...>
pub struct ASTVarTyping {
    pub value: String,
    pub generics: Option<ASTListTyping>,
    pub range: Range
}

pub struct ASTPairTypingItem {
    pub name: String,
    pub value: Option<ASTTypings>,
    pub optional: bool
}

// {key: typing_name},
pub struct ASTPairListTyping {
    pub pairs: Vec<ASTPairTypingItem>,
    pub range: Range
}


pub struct ASTListTyping {
    pub entries: Vec<ASTTypings>,
    pub range: Range
}

pub enum ASTTypings {
    Var(ASTVarTyping),
    PairList(ASTPairListTyping),
    Function(ASTFunction),
    Tuple(ASTListTyping)
}
