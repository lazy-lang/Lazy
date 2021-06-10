
use std::fmt;
pub use errors::{Range};
use super::{TokenType};
pub use std::collections::hash_map::HashMap;

// A string literalz
pub struct ASTStr {
    pub value: String,
    pub range: Range
}

pub struct ASTTempStr {
    pub template: String,
    pub values: HashMap<usize, ASTExpression>,
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

pub struct ASTVarList {
    pub values: Vec<ASTVar>,
    pub range: Range
}

pub enum ASTDeclareTypes {
    TupleDeconstruct(ASTVarList),
    StructDeconstruct(ASTVarList),
    Var(ASTVar)
}

pub struct ASTDeclare {
    pub var: ASTDeclareTypes,
    pub is_const: bool,
    pub value: Option<Box<ASTExpression>>,
    pub typings: Option<ASTTypings>,
    pub range: Range,
}

pub struct ASTStatic {
    pub var: ASTVar,
    pub typings: Option<ASTListTyping>,
    pub value: ASTExpression,
    pub range: Range
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

pub struct ASTIndexAccess {
    pub value: Box<ASTExpression>,
    pub target: Box<ASTExpression>,
    pub range: Range
}

pub struct ASTOptional {
    pub value: Box<ASTExpression>,
    pub range: Range
}

pub struct ASTEnumDeclaration {
    pub name: ASTVar,
    pub values: ASTPairListTyping,
    pub typings: Option<ASTListTyping>,
    pub range: Range
}

pub struct ASTFunction {
    pub params: Box<ASTPairListTyping>,
    pub body: Option<Box<ASTExpression>>,
    pub return_type: Option<Box<ASTTypings>>,
    pub typings: Option<ASTListTyping>,
    pub range: Range
}

pub struct ASTBlock {
    pub elements: Vec<ASTExpression>,
    pub range: Range
}

pub struct ASTInitializor {
    pub target: ASTModAccessValues,
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

pub enum ASTModAccessValues {
    ModAccess(ASTModAccess),
    Var(ASTVarTyping)
}

pub struct ASTModAccess {
    pub path: Vec<ASTVar>,
    pub init: Option<ASTExpressionList>,
    pub typings: Option<ASTListTyping>,
    pub range: Range
}

pub struct ASTCall {
    pub target: Box<ASTExpression>,
    pub args: ASTExpressionList,
    pub typings: Option<ASTListTyping>,
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

pub struct ASTMain {
    pub expression: ASTBlock,
    pub range: Range
}

pub enum ASTMatchArmExpressions {
    String(ASTStr),
    Int(ASTInt),
    Float(ASTFloat),
    Iterator(ASTIterator),
    Char(ASTChar),
    Bool(ASTBool),
    Tuple(ASTExpressionList),
    None(Range),
    Rest,
    Enum(ASTModAccess)
}

pub struct ASTMatchArm {
    pub possibilities: Vec<ASTMatchArmExpressions>,
    pub guard: Option<ASTExpression>,
    pub body: ASTExpression,
    pub range: Range
}

pub struct ASTMatch {
    pub arms: Vec<ASTMatchArm>,
    pub expression: Box<ASTExpression>,
    pub range: Range
}

pub struct ASTExport {
    pub value: Box<ASTStatement>,
    pub range: Range
}

pub struct ASTImport {
    pub path: ASTStr,
    pub _as: Option<ASTVar>,
    pub range: Range
}

pub struct ASTAwait {
    pub optional: bool,
    pub expression: Box<ASTExpression>,
    pub range: Range
}

pub struct ASTImpl {
    pub partial: ASTModAccessValues,
    pub target: ASTModAccessValues,
    pub typings: Option<ASTListTyping>,
    pub fields: ASTPairListTyping,
    pub range: Range
}

pub struct ASTMeta {
    pub name: String,
    pub args: Vec<TokenType>,
    pub target: Box<ASTStatement>,
    pub range: Range
}

// Any expression
pub enum ASTExpression {
    Str(ASTStr),
    TempStr(ASTTempStr),
    Float(ASTFloat),
    Int(ASTInt),
    Bool(ASTBool),
    Var(ASTVar),
    Char(ASTChar),
    Binary(ASTBinary),
    Unary(ASTUnary),
    DotAccess(ASTDotAccess),
    IndexAccess(ASTIndexAccess),
    ModAccess(ASTModAccess),
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
    Spread(ASTSpread),
    None(Range),
    Match(ASTMatch),
    Await(ASTAwait)
}

// Any statement
pub enum ASTStatement {
    EnumDeclaration(ASTEnumDeclaration),
    Struct(ASTStruct),
    Static(Box<ASTStatic>),
    Type(ASTType),
    Main(ASTMain),
    Export(ASTExport),
    Import(ASTImport),
    Meta(ASTMeta),
    Impl(ASTImpl)
}

bitflags! {
    pub struct ASTModifiers: u32 {
        const PRIVATE = 1 << 0;
        const STATIC = 1 << 1;
        const CONST = 1 << 2;
    }
}

impl ASTModifiers {
    pub fn clear(&mut self) {
        self.bits = 0;
    }
}

pub struct ASTPairTypingItem {
    pub name: String,
    pub value: Option<ASTTypings>,
    pub spread: bool,
    pub default_value: Option<ASTExpression>,
    pub modifiers: ASTModifiers
}

pub struct ASTPairListTyping {
    pub pairs: Vec<ASTPairTypingItem>,
    pub range: Range
}

pub struct ASTListTyping {
    pub entries: Vec<ASTTypings>,
    pub range: Range
}

pub struct ASTVarTyping {
    pub value: ASTVar,
    pub typings: Option<ASTListTyping>,
    pub range: Range
}

pub struct ASTCombineTyping {
    pub left: Box<ASTTypings>,
    pub right: Box<ASTTypings>,
    pub range: Range
}

pub struct ASTBoundTyping {
    pub name: ASTVar,
    pub bound: Box<ASTTypings>,
    pub range: Range
}

pub enum ASTTypings {
    Var(ASTVarTyping),
    Mod(ASTModAccess),
    PairList(ASTPairListTyping),
    Function(ASTFunction),
    Optional(Box<ASTTypings>),
    Tuple(ASTListTyping),
    Combine(ASTCombineTyping),
    ExplicitImpl(ASTModAccessValues),
    Bound(ASTBoundTyping)
}

impl fmt::Display for ASTVarTyping {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.value, if self.typings.is_some() { format!("<{}>", self.typings.as_ref().unwrap().to_string()) } else { String::from("") })
    }
}

impl fmt::Display for ASTPairListTyping {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut string: Vec<String> = vec![];
        for pair in &self.pairs {
            let modifiers = {
                let mut mods = String::new();
                if pair.modifiers.contains(ASTModifiers::CONST) { mods += "const " };
                if pair.modifiers.contains(ASTModifiers::STATIC) { mods += "static "};
                if pair.modifiers.contains(ASTModifiers::PRIVATE) { mods += "private " };
                mods
            };
            string.push(format!("{}{}{}{}{}", modifiers, if pair.spread { "..." } else {""}, pair.name, if pair.value.is_some() { format!(": {}", pair.value.as_ref().unwrap()) } else { String::from("")}, if pair.default_value.is_some() { format!(" = {}", pair.default_value.as_ref().unwrap()) } else { String::from("") }));
        };
        write!(f, "{}", string.join(", "))
    }
}

impl fmt::Display for ASTFunction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "fn<{}>({}) -> {} {}", if self.typings.is_some() { self.typings.as_ref().unwrap().to_string() } else { String::from("none") }, self.params, if self.return_type.is_some() { self.return_type.as_ref().unwrap().to_string() } else { String::from("none") } ,if self.body.is_some() {  self.body.as_ref().unwrap().to_string() } else { String::from("") })
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
            Self::Str(str) => str.fmt(f),
            Self::Bool(boolean) => boolean.fmt(f),
            Self::Int(i) => i.fmt(f),
            Self::Float(fl) => fl.fmt(f),
            Self::Binary(bin) => bin.fmt(f),
            Self::Unary(un) => un.fmt(f),
            Self::Var(variable) => variable.fmt(f),
            Self::Optional(op) => op.fmt(f),
            Self::DotAccess(op) => op.fmt(f),
            Self::IndexAccess(op) => op.fmt(f),
            Self::Block(block) => block.fmt(f),
            Self::Function(func) => func.fmt(f),
            Self::Declare(st) => st.fmt(f),
            Self::Init(initializor) => initializor.fmt(f),
            Self::Iterator(it) => it.fmt(f),
            Self::If(exp) => exp.fmt(f),
            Self::Char(ch) => ch.fmt(f),
            Self::ModAccess(e) => e.fmt(f),
            Self::Call(call) => call.fmt(f),
            Self::ForIn(for_loop) => for_loop.fmt(f),
            Self::While(while_loop) => while_loop.fmt(f),
            Self::Tuple(tup) => write!(f, "[{}]", tup.to_string()),
            Self::Yield(y) => y.fmt(f),
            Self::Spread(sp) => write!(f, "...{}", sp.value.to_string()),
            Self::Match(mtch) => mtch.fmt(f),
            Self::Await(aw) => aw.fmt(f),
            Self::TempStr(tmp) => tmp.fmt(f),
            Self::None(_) => write!(f, "none")
        }
    }
}


impl fmt::Display for ASTTypings {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Self::Tuple(tup) => write!(f, "[{}]", tup),
            Self::Var(var) => var.fmt(f),
            Self::PairList(list) => list.fmt(f),
            Self::Optional(typing) => write!(f, "{}?", typing),
            Self::Function(func) => func.fmt(f),
            Self::Combine(c) => c.fmt(f),
            Self::Mod(m) => m.fmt(f),
            Self::ExplicitImpl(im) => write!(f, "{}!", im),
            Self::Bound(b) => b.fmt(f)
        }
    }
}

impl fmt::Display for ASTStatement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Self::Struct(structure) => structure.fmt(f),
            Self::EnumDeclaration(en) => en.fmt(f),
            Self::Type(typing) => typing.fmt(f),
            Self::Static(st) => st.fmt(f),
            Self::Main(m) => m.fmt(f),
            Self::Export(ex) => ex.fmt(f),
            Self::Import(imp) => imp.fmt(f),
            Self::Impl(imp) => imp.fmt(f),
            Self::Meta(m) => m.fmt(f)
        } 
    }
}

impl fmt::Display for ASTMain {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "main {}", self.expression)
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
         write!(f, "{}{}({})", self.target, if self.typings.is_some() { format!("<{}>", self.typings.as_ref().unwrap()) } else { String::from("") }, self.args)
    }
}

impl fmt::Display for ASTModAccess {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut path = String::new();
        for path_part in 0..self.path.len() {
            if path_part != 0 { path += "::" };
            path += &self.path[path_part].to_string();
        }
        write!(f, "{}{}{}", path, if self.typings.is_some() { format!("<{}>", self.typings.as_ref().unwrap().to_string()) } else { String::from("") }, if self.init.is_some() { format!("({})", self.init.as_ref().unwrap().to_string()) } else { String::from("") })
   }
}

impl fmt::Display for ASTModAccessValues {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ModAccess(m) => m.fmt(f),
            Self::Var(v) => v.fmt(f)
        }
   }
}

impl fmt::Display for ASTInitializor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "new {}{}{}", self.target, if self.typings.is_some() { format!("<{}>", self.typings.as_ref().unwrap()) } else { String::from("") }, self.params)
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
        writeln!(f, "{} {}{} = {}", if self.is_const { "const" } else { "let" },self.var, if self.typings.is_some() { format!(": {}", self.typings.as_ref().unwrap().to_string()) } else { String::from("") }, if self.value.is_some() { self.value.as_ref().unwrap().to_string()} else { String::from("none") })
   }
}

impl fmt::Display for ASTType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "type {}{} = {}", self.name, if self.typings.is_some() { format!("<{}>", self.typings.as_ref().unwrap().to_string()) } else { String::from("") }, self.value)
   }
}

impl fmt::Display for ASTEnumDeclaration {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "enum {}{} {{\n {} }}", self.name, if self.typings.is_some() { format!("<{}>", self.typings.as_ref().unwrap() )} else { String::from("") }, self.values)
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

impl fmt::Display for ASTMatch {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut str = String::new();
        for arm in &self.arms {
            str.push_str(&format!("{}\n", arm));
        };
        write!(f, "match {} {{\n{}}}", self.expression, str)
   }
}

impl fmt::Display for ASTMatchArm {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {} => {}", self.possibilities.iter().map(|i| i.to_string()).collect::<Vec<String>>().join(" | "), if self.guard.is_some() { format!("when {}", self.guard.as_ref().unwrap()) } else { String::from("")}, self.body)
   }
}

impl fmt::Display for ASTMatchArmExpressions {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Char(ch) => ch.fmt(f),
            Self::String(st) => st.fmt(f),
            Self::Int(int) => int.fmt(f),
            Self::Float(fl) => fl.fmt(f),
            Self::Iterator(iter) => iter.fmt(f),
            Self::Enum(en) => en.fmt(f),
            Self::Tuple(t) => write!(f, "[{}]", t),
            Self::Bool(b) => b.fmt(f),
            Self::None(_) => write!(f, "none"),
            Self::Rest => write!(f, "_")
        }
   }
}


impl fmt::Display for ASTStatic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "static {}{} = {}", self.var, if self.typings.is_some() { format!("<{}>", self.typings.as_ref().unwrap().to_string()) } else { String::from("") }, self.value)
   }
}

impl fmt::Display for ASTExport {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "export {}", self.value)
   }
}

impl fmt::Display for ASTImport {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "import {}{}", self.path, if self._as.is_some() { format!(" as {}", self._as.as_ref().unwrap() )} else { String::from("") })
   }
}

impl fmt::Display for ASTAwait {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "await{} {}", if self.optional { "?" } else { "" }, self.expression)
   }
}

impl fmt::Display for ASTCombineTyping {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} + {}", self.left, self.right)
   }
}

impl fmt::Display for ASTImpl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "impl{} {} for {} {{\n{}\n}}", if let Some(t) = &self.typings { format!("<{}>", t) } else { String::from("") }, self.partial, self.target, self.fields)
   }
}

impl fmt::Display for ASTVarList {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.values.iter().map(|v| v.to_string()).collect::<Vec<String>>().join(", "))
   }
}

impl fmt::Display for ASTDeclareTypes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Var(v) => v.fmt(f),
            Self::TupleDeconstruct(vars) => write!(f, "[{}]", vars),
            Self::StructDeconstruct(vars) => write!(f, "{{{}}}", vars)
        }
   }
}

impl fmt::Display for ASTMeta {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "#{}({})\n{}", self.name, self.args.iter().map(|v| v.to_string()).collect::<Vec<String>>().join(", "), self.target)
   }
}

impl fmt::Display for ASTIndexAccess {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}[{}]", self.value, self.target)
   }
}

impl fmt::Display for ASTTempStr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut new_str = String::new();
        for (ind, ch) in self.template.chars().enumerate() {
            if let Some(k) = self.values.get(&ind) {
                new_str.push_str(&format!("${{{}}}", k));
            } else {
                new_str.push(ch);
            };
        }
        write!(f, "`{}`", new_str)
   }
}

impl fmt::Display for ASTBoundTyping {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.bound)
   }
}