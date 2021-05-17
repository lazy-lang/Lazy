
use std::fmt;
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
pub struct ASTDeclare {
    pub var: ASTVar,
    pub is_const: bool,
    pub value: Option<Box<ASTExpression>>,
    pub typings: Option<ASTTypings>,
    pub range: Range,
}

pub struct ASTStruct {
    pub name: ASTVar,
    pub fields: ASTPairListTyping,
    pub typings: Option<ASTListTyping>,
    pub range: Range
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
    pub target: ASTVar,
    pub range: Range
}

pub struct ASTOptional {
    pub value: Box<ASTExpression>,
    pub range: Range
}

pub struct ASTEnumDeclaration {
    pub name: String,
    pub values: ASTPairListTyping,
    pub range: Range
}

pub struct ASTFunction {
    pub params: Box<ASTPairListTyping>,
    pub body: Option<Box<ASTExpression>>,
    pub return_type: Option<Box<ASTTypings>>,
    pub range: Range
}

// A block of expressions or statements 
pub struct ASTBlock {
    pub elements: Vec<ASTExpression>,
    pub range: Range
}

pub struct ASTInitializor {
    pub target: ASTVar,
    pub params: ASTPairList,
    pub typings: Option<ASTListTyping>,
    pub range: Range
}

pub struct ASTIterator {
    pub start: Box<ASTExpression>,
    pub end: Box<ASTExpression>,
    pub inclusive: bool,
    pub range: Range
}

pub struct ASTIf {
    pub condition: Box<ASTExpression>,
    pub then: Box<ASTExpression>,
    pub otherwise: Option<Box<ASTExpression>>,
    pub range: Range
}

pub struct ASTChar {
    pub value: char,
    pub range: Range
}

pub struct ASTEnumAccess {
    pub value: ASTVar,
    pub target: ASTVar,
    pub init_value: Option<Box<ASTExpression>>,
    pub range: Range
}

pub struct ASTCall {
    pub target: Box<ASTExpression>,
    pub args: ASTExpressionList,
    pub range: Range
}

pub struct ASTForIn {
    pub var: ASTVar,
    pub iterable: Box<ASTExpression>,
    pub body: Box<ASTExpression>,
    pub range: Range
}

pub struct ASTWhile {
    pub condition: Box<ASTExpression>,
    pub body: Box<ASTExpression>,
    pub range: Range
}

pub struct ASTType {
    pub name: String,
    pub typings: Option<ASTListTyping>,
    pub value: ASTTypings,
    pub range: Range
}

pub struct ASTExpressionList {
    pub expressions: Vec<ASTExpression>,
    pub range: Range
}

pub struct ASTYield {
    pub value: Option<Box<ASTExpression>>,
    pub range: Range
}

pub struct ASTSpread {
    pub value: Box<ASTExpression>,
    pub range: Range
}

// Any expression
pub enum ASTExpression {
    Str(ASTStr),
    Float(ASTFloat),
    Int(ASTInt),
    Bool(ASTBool),
    Var(ASTVar),
    Char(ASTChar),
    Binary(ASTBinary),
    Unary(ASTUnary),
    DotAccess(ASTDotAccess),
    EnumAccess(ASTEnumAccess),
    Optional(ASTOptional),
    Block(ASTBlock),
    Function(ASTFunction),
    Init(ASTInitializor),
    Iterator(ASTIterator),
    Call(ASTCall),
    ForIn(ASTForIn),
    While(ASTWhile),
    If(ASTIf),
    Declare(ASTDeclare),
    Tuple(ASTExpressionList),
    Yield(ASTYield),
    Spread(ASTSpread)
}

// Any statement
pub enum ASTStatement {
    EnumDeclaration(ASTEnumDeclaration),
    Struct(ASTStruct),
    Type(ASTType)
}

pub struct ASTPairTypingItem {
    pub name: String,
    pub value: Option<ASTTypings>,
    pub spread: bool,
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

pub struct ASTVarTyping {
    pub value: String,
    pub typings: Option<ASTListTyping>,
    pub optional: bool,
    pub range: Range
}

pub enum ASTTypings {
    Var(ASTVarTyping),
    PairList(ASTPairListTyping),
    Function(ASTFunction),
    Tuple(ASTListTyping)
}

impl fmt::Display for ASTVarTyping {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}{}", self.value, if self.optional { "?" } else { "" }, if self.typings.is_some() { format!("<{}>", self.typings.as_ref().unwrap().to_string()) } else { String::from("") })
    }
}

impl fmt::Display for ASTPairListTyping {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut string: Vec<String> = vec![];
        for pair in &self.pairs {
            string.push(format!("{}{}{}: {}", if pair.spread { "..." } else {""}, pair.name, if pair.optional {"?"} else {""}, if pair.value.is_some() { pair.value.as_ref().unwrap().to_string() } else { String::from("none")}));
        };
        write!(f, "{}", string.join(", "))
    }
}

impl fmt::Display for ASTFunction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "fn({}) -> {} {}", self.params, if self.return_type.is_some() { self.return_type.as_ref().unwrap().to_string() } else { String::from("none") } ,if self.body.is_some() { format!("{}", self.body.as_ref().unwrap().to_string()) } else { String::from("") })
    }
}

impl fmt::Display for ASTListTyping {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut string: Vec<String> = vec![];
        for entry in &self.entries {
            string.push(entry.to_string());
        };
        write!(f, "{}", string.join(", "))
    }
}

impl fmt::Display for ASTExpression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            ASTExpression::Str(str) => str.fmt(f),
            ASTExpression::Bool(boolean) => boolean.fmt(f),
            ASTExpression::Int(i) => i.fmt(f),
            ASTExpression::Float(fl) => fl.fmt(f),
            ASTExpression::Binary(bin) => bin.fmt(f),
            ASTExpression::Unary(un) => un.fmt(f),
            ASTExpression::Var(variable) => variable.fmt(f),
            ASTExpression::Optional(op) => op.fmt(f),
            ASTExpression::DotAccess(op) => op.fmt(f),
            ASTExpression::Block(block) => block.fmt(f),
            ASTExpression::Function(func) => func.fmt(f),
            ASTExpression::Declare(st) => st.fmt(f),
            ASTExpression::Init(initializor) => initializor.fmt(f),
            ASTExpression::Iterator(it) => it.fmt(f),
            ASTExpression::If(exp) => exp.fmt(f),
            ASTExpression::Char(ch) => ch.fmt(f),
            ASTExpression::EnumAccess(e) => e.fmt(f),
            ASTExpression::Call(call) => call.fmt(f),
            ASTExpression::ForIn(for_loop) => for_loop.fmt(f),
            ASTExpression::While(while_loop) => while_loop.fmt(f),
            ASTExpression::Tuple(tup) => write!(f, "[{}]", tup.to_string()),
            ASTExpression::Yield(y) => y.fmt(f),
            ASTExpression::Spread(sp) => write!(f, "...{}", sp.value.to_string())
        }
    }
}


impl fmt::Display for ASTTypings {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            ASTTypings::Tuple(tup) => tup.fmt(f),
            ASTTypings::Var(var) => var.fmt(f),
            ASTTypings::PairList(list) => list.fmt(f),
            ASTTypings::Function(func) => func.fmt(f)
        }
    }
}

impl fmt::Display for ASTStatement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            ASTStatement::Struct(structure) => structure.fmt(f),
            ASTStatement::EnumDeclaration(en) => en.fmt(f),
            ASTStatement::Type(typing) => typing.fmt(f)
        } 
    }
}

impl fmt::Display for ASTStr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\"{}\"", self.value)
    }
}

impl fmt::Display for ASTInt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl fmt::Display for ASTFloat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl fmt::Display for ASTVar {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl fmt::Display for ASTBool {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl fmt::Display for ASTChar {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "'{}'", self.value)
    }
}

impl fmt::Display for ASTBinary {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {} {}", self.left, self.op, self.right)
    }
}

impl fmt::Display for ASTUnary {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.op, self.value)
    }
}

impl fmt::Display for ASTIterator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}..{}{}", self.start, if self.inclusive {"="} else {""}, self.end)
    }
}


impl fmt::Display for ASTDotAccess {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
         write!(f, "{}.{}", self.value, self.target)
    }
}

impl fmt::Display for ASTPairList {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut string = String::new();
        for pair in &self.pairs {
            string.push_str(&format!("{}: {}", pair.0, if pair.1.is_some() { pair.1.as_ref().unwrap().to_string() } else { String::from("{}") }));
        };
        write!(f, "{{ {} }}", string)
   }
}

impl fmt::Display for ASTCall {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
         write!(f, "{}({})", self.target, self.args)
    }
}

impl fmt::Display for ASTEnumAccess {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}({})", self.value, self.target, if self.init_value.is_some() { self.init_value.as_ref().unwrap().to_string() } else { String::from("none") })
   }
}

impl fmt::Display for ASTInitializor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.target, self.params)
   }
}

impl fmt::Display for ASTForIn {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "for {} in {} {{\n {} \n}}", self.var, self.iterable, self.body)
   }
}

impl fmt::Display for ASTWhile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "while {} {{\n {} \n}}",self.condition, self.body)
   }
}

impl fmt::Display for ASTBlock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut string = String::new();
        for exp in &self.elements {
            string.push_str(&format!("\n{}", exp));
        }
        write!(f, "{{{} \n}}", string)
   }
}

impl fmt::Display for ASTDeclare {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}{} = {}\n", if self.is_const { "const" } else { "let" },self.var, if self.typings.is_some() { format!("<{}>", self.typings.as_ref().unwrap().to_string()) } else { String::from("") }, if self.value.is_some() { self.value.as_ref().unwrap().to_string()} else { String::from("none") })
   }
}

impl fmt::Display for ASTType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "type {}{} = {}\n", self.name, if self.typings.is_some() { format!("<{}>", self.typings.as_ref().unwrap().to_string()) } else { String::from("") }, self.value)
   }
}

impl fmt::Display for ASTEnumDeclaration {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "enum {} {{\n {} }}\n", self.name, self.values)
   }
}

impl fmt::Display for ASTStruct {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "struct {}{} {{\n {} }}\n", self.name, if self.typings.is_some() { format!("<{}>", self.typings.as_ref().unwrap().to_string()) } else { String::from("") }, self.fields)
   }
}

impl fmt::Display for ASTOptional {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}?", self.value)
   }
}

impl fmt::Display for ASTIf {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "if {} {} {}", self.condition, self.then, if self.otherwise.is_some() { format!("else {}", self.otherwise.as_ref().unwrap()) } else {String::from("")})
   }
}

impl fmt::Display for ASTExpressionList {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut string: Vec<String> = vec![];
        for exp in &self.expressions {
            string.push(exp.to_string());
        }
        write!(f, "{}", string.join(", "))
   }
}

impl fmt::Display for ASTYield {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "yield {}", if self.value.is_some() { self.value.as_ref().unwrap().to_string() } else { String::from(";") })
   }
}