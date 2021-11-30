
pub use parser::ast::{ParserResult, model::*};

pub enum SymbolKind {
    Struct{
        properties: Vec<(String, SymbolLike)>,
        impls: Vec<SymbolLike>
    },
    Enum{
        members: Vec<(String, SymbolLike)>,
        impls: Vec<SymbolLike>
    },
    Fn{
        parameters: Vec<(String, SymbolLike)>,
        return_type: SymbolLike
    },
    Module{
        elements: Vec<SymbolLike>
    },
    None
}

pub enum SymbolLike {
    Instance(SymbolInstance),
    Ref(u32)
}

pub struct Symbol {
    pub name: String,
    pub id: u32,
    pub kind: SymbolKind,
    pub type_params: Vec<SymbolLike>,
    pub declaration: ASTStatement
}

pub struct SymbolInstance {
    pub id: u32,
    pub type_params: Vec<SymbolLike>
}

pub struct SymbolAlias {
    pub name: String,
    pub id: u32
}

impl Symbol {

    pub fn empty(id: u32, name: String, decl: ASTStatement) -> Self {
        Self {
            id,
            name,
            kind: SymbolKind::None,
            type_params: Vec::new(),
            declaration: decl
        }
    }

    pub fn instance(&self, type_params: Vec<SymbolLike>) -> SymbolLike {
        SymbolLike::Instance(SymbolInstance {
            id: self.id,
            type_params
        })
    }

}
