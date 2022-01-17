
pub use parser::ast::{model::*};

pub trait SymbolCollector {
    fn insert_symbol(&mut self, sym: Symbol);
    fn get_symbol(&self, name: &u32) -> Option<&Symbol>;
}

pub enum StatementOrExpression {
    EnumStatement(ASTEnumDeclaration),
    StructStatement(ASTStruct),
    TypeStatement(ASTType),
    FnExp(ASTFunction),
    None
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

#[derive(Clone)]
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

    pub fn get_id(&self) -> u32 {
        match self {
            Self::Instance(inst) => inst.id,
            Self::Ref(id) => *id
        }
    }

}

pub struct Symbol {
    pub name: String,
    pub id: u32,
    pub kind: SymbolKind,
    pub type_params: HashMap<String, Option<SymbolLike>>,
    pub declaration: StatementOrExpression
}

#[derive(Clone)]
pub struct SymbolInstance {
    pub id: u32,
    pub type_args: Vec<SymbolLike>
}

impl SymbolInstance {

    pub fn to_symbol<'a, T: SymbolCollector>(&self, collector: &'a T) -> &'a Symbol {
        return collector.get_symbol(&self.id).unwrap()
    }

}

impl Symbol {

    pub fn empty(id: u32, name: String, decl: StatementOrExpression) -> Self {
        Self {
            id,
            name,
            kind: SymbolKind::None,
            type_params: HashMap::new(),
            declaration: decl
        }
    }

    pub fn instance(&self, type_args: Vec<SymbolLike>) -> SymbolLike {
        SymbolLike::Instance(SymbolInstance {
            id: self.id,
            type_args
        })
    }

}
