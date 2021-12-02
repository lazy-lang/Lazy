
pub use parser::ast::{model::*};

pub trait SymbolCollector {
    fn insert_symbol(&mut self, sym: Symbol);
    fn get_symbol(&self, name: &u32) -> Option<&Symbol>;
}

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

impl SymbolLike {

    pub fn to_symbol<'a, T: SymbolCollector>(&self, collector: &'a T) -> &'a Symbol {
        match self {
            Self::Instance(inst) => collector.get_symbol(&inst.id).unwrap(),
            Self::Ref(r) => collector.get_symbol(r).unwrap()
        }
    }
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

impl SymbolInstance {

    pub fn to_symbol<'a, T: SymbolCollector>(&self, collector: &'a T) -> &'a Symbol {
        return collector.get_symbol(&self.id).unwrap()
    }

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
