
pub use parser::ast::model::*;

pub enum SymbolDeclaration {
   Expression(ASTExpression),
   Statement(ASTStatement),
   Typing(ASTTypings)
}

pub enum SymbolKind {
    Struct{
        properties: Vec<(String, Symbol)>,
        impls: Vec<Symbol>
    },
    Enum{
        members: Vec<(String, Symbol)>,
        impls: Vec<Symbol>
    },
    Fn{
        parameters: Vec<(String, Symbol)>,
        return_type: Symbol
    },
    Module{
        elements: Vec<Symbol>
    }
}

pub struct Symbol {
    pub name: String,
    pub id: u32,
    pub kind: Box<SymbolKind>,
    pub type_params: Vec<Symbol>,
    pub declaration: Option<SymbolDeclaration>
}

pub struct SymbolInstance {
    pub id: u32,
    pub type_params: Vec<Symbol>
}

pub struct SymbolAlias {
    pub name: String,
    pub id: u32
}