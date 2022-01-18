pub use parser::ast::{model::*};

pub trait SymbolCollector {
    fn insert_symbol(&mut self, sym: Symbol);
    fn get_symbol(&self, name: &u32) -> Option<&Symbol>;
}

pub enum StatementOrExpression {
    EnumStatement(ASTEnumDeclaration),
    StructStatement(ASTStruct),
    TypeStatement(ASTType),
    None
}

pub enum SymbolKind {
    Struct{
        properties: HashMap<String, SymbolLike>,
        impls: Vec<SymbolLike>
    },
    Enum{
        members: HashMap<String, SymbolLike>,
        impls: Vec<SymbolLike>
    },
    Fn{
        parameters: HashMap<String, SymbolLike>,
        return_type: SymbolLike
    },
    Module{
        elements: Vec<SymbolLike>
    },
    None
}

#[derive(Clone)]
pub enum SymbolLike {
    Instance(u32, usize),
    Ref(u32),
    Optional(Box<SymbolLike>)
}

impl SymbolLike {

    pub fn to_symbol<'a, T: SymbolCollector>(&self, collector: &'a T) -> &'a Symbol {
        match self {
            Self::Instance(sym, _) => collector.get_symbol(sym).unwrap(),
            Self::Ref(r) => collector.get_symbol(r).unwrap(),
            Self::Optional(sym) => sym.to_symbol(collector)
        }
    }

    pub fn get_id(&self) -> u32 {
        match self {
            Self::Instance(inst, _) => *inst,
            Self::Ref(id) => *id,
            Self::Optional(sym) => sym.get_id()
        }
    }

    pub fn get_kind<'a, T: SymbolCollector>(&self, collector: &'a T) -> &'a SymbolKind {
        match self {
            Self::Instance(sym, inst) => &collector.get_symbol(sym).unwrap().instances[*inst].kind,
            Self::Ref(id) => &collector.get_symbol(id).unwrap().kind,
            Self::Optional(sym) =>sym.get_kind(collector)
        }
    }

}

pub struct Symbol {
    pub name: String,
    pub id: u32,
    pub kind: SymbolKind,
    pub type_params: HashMap<String, Option<SymbolLike>>,
    pub instances: Vec<SymbolInstance>,
    pub declaration: StatementOrExpression
}

pub struct SymbolInstance {
    pub kind: SymbolKind,
    pub type_args: Vec<SymbolLike>
}

impl Symbol {

    pub fn empty(id: u32, name: String, decl: StatementOrExpression) -> Self {
        Self {
            id,
            name,
            kind: SymbolKind::None,
            type_params: HashMap::new(),
            instances: Vec::new(),
            declaration: decl
        }
    }

    pub fn create_or_get_instance(&mut self, params: Vec<SymbolLike>) -> Option<&SymbolInstance> {
        let params_len = params.len();
        if params_len != self.type_params.len() { 
            return None;
        };
        'outer: for instance in &self.instances {
            for ind in 0..params_len {
                if params[ind].get_id() != instance.type_args[ind].get_id() {
                    continue 'outer;
                }
            }
            return Some(&instance);
        };
        // TDB: Create an instance, add it to the instances vector and return a ref to it
        None
    }


    pub fn reference(&self) -> SymbolLike {
        SymbolLike::Ref(self.id)
    }

    pub fn optional(&self) -> SymbolLike {
        SymbolLike::Optional(Box::from(SymbolLike::Ref(self.id)))
    }

}
