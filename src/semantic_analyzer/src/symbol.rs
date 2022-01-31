use errors::*;
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

pub struct SymbolProperty {
    kind: SymbolLike,
    flags: ASTModifiers
}

pub enum SymbolKind {
    Struct(HashMap<String, SymbolProperty>),
    Enum(HashMap<String, SymbolLike>),
    Fn{
        parameters: HashMap<String, SymbolLike>,
        return_type: SymbolLike
    },
    Module(HashMap<String, SymbolLike>),
    None
}

impl SymbolKind {
    pub fn is_enum(&self) -> bool {
        matches!(self, Self::Enum(_))
    }

    pub fn is_struct(&self) -> bool {
        matches!(self, Self::Struct(_))
    }

    pub fn is_fn(&self) -> bool {
        matches!(self, Self::Fn{parameters: _, return_type: _})
    }

    pub fn is_module(&self) -> bool {
        matches!(self, Self::Module(_))
    }
}

#[derive(Clone)]
pub enum SymbolLike {
    Instance(u32, usize),
    Ref(u32),
    Optional(u32, Option<usize>)
}

impl SymbolLike {

    pub fn get_id(&self) -> u32 {
        match self {
            Self::Instance(inst, _) => *inst,
            Self::Ref(id) => *id,
            Self::Optional(id, _) => *id,
        }
    }

    pub fn get_kind<'a, T: SymbolCollector>(&self, collector: &'a T) -> &'a SymbolKind {
        match self {
            Self::Instance(sym, inst) => &collector.get_symbol(sym).unwrap().instances[*inst].kind,
            Self::Ref(id) => &collector.get_symbol(id).unwrap().kind,
            Self::Optional(id, inst) => {
                if let Some(instance_id) = inst {
                    &collector.get_symbol(id).unwrap().instances[*instance_id].kind
                } else {
                    &collector.get_symbol(id).unwrap().kind
                }
            }
        }
    }

}

pub trait ToSymbol {
    fn to_symbol<'a, C: SymbolCollector>(&self, collector: &'a C) -> &'a Symbol;
}

impl ToSymbol for u32 {
    fn to_symbol<'a, C: SymbolCollector>(&self, collector: &'a C) -> &'a Symbol {
        collector.get_symbol(self).unwrap()
    }
}

impl ToSymbol for SymbolLike {
    fn to_symbol<'a, T: SymbolCollector>(&self, collector: &'a T) -> &'a Symbol {
        match self {
            Self::Instance(sym, _) => collector.get_symbol(sym).unwrap(),
            Self::Ref(r) => collector.get_symbol(r).unwrap(),
            Self::Optional(sym, _) => collector.get_symbol(sym).unwrap()
        }
    }
}

pub struct Symbol {
    pub name: String,
    pub id: u32,
    pub kind: SymbolKind,
    pub type_params: HashMap<String, Option<SymbolLike>>,
    pub instances: Vec<SymbolInstance>,
    pub declaration: StatementOrExpression,
    pub impls: Vec<SymbolLike>
}

pub struct SymbolInstance {
    pub id: usize,
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
            declaration: decl,
            impls: Vec::new()
        }
    }

    pub fn create_or_get_instance(&mut self, params: Vec<SymbolLike>) -> LazyResultDiagnostic<&SymbolInstance> {
        let params_len = params.len();
        if params_len != self.type_params.len() { 
            return Err(dia!(INVALID_AMOUNT_OF_TYPE_PARAMS, &self.type_params.len().to_string(), &params_len.to_string()));
        };
        'outer: for instance in &self.instances {
            for ind in 0..params_len {
                if params[ind].get_id() != instance.type_args[ind].get_id() {
                    continue 'outer;
                }
            }
            return Ok(&instance);
        };
        // TDB: Create an instance, add it to the instances vector and return a ref to it
        Err(dia!(UNEXPECTED_EOF))
    }

    pub fn as_ref(&self) -> SymbolLike {
        SymbolLike::Ref(self.id)
    }

    pub fn optional(&self) -> SymbolLike {
        SymbolLike::Optional(self.id, None)
    }

    pub fn get_mod_type<'a, C: SymbolCollector>(&'a self, collector: &'a C, name: &str) -> Option<&'a SymbolLike> {
        match &self.kind {
            SymbolKind::Struct(props) => {
                let prop = props.get(name)?;
                if prop.flags.contains(ASTModifiers::STATIC) {
                    return Some(&prop.kind);
                };
            },
            SymbolKind::Enum(members) => return members.get(name),
            SymbolKind::Module(exported) => return exported.get(name),
            _ => {}
        };
        for implementation in &self.impls {
            if let SymbolKind::Struct(properties) = implementation.get_kind(collector) {
                let prop = properties.get(name)?;
                if prop.flags.contains(ASTModifiers::STATIC) {
                    return Some(&prop.kind);
                };
            }
        };
        None
    }

}
