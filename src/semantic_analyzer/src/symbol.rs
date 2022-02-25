use errors::*;
pub use parser::ast::{model::*};

bitflags::bitflags! {
    pub struct SymbolFlags: u32 {
        const STATIC = 1 << 0;
        const CONST = 1 << 1;
        const OPTIONAL = 1 << 2;
        const TYPE_PARAM = 1 << 3;
    }
}

pub trait SymbolCollector {
    fn insert_symbol(&mut self, sym: Symbol);
    fn get_symbol(&self, name: &u32) -> Option<&Symbol>;
    fn get_mut_symbol(&mut self, name: &u32) -> Option<&mut Symbol>;
}

pub enum StatementOrExpression {
    EnumStatement(ASTEnumDeclaration),
    StructStatement(ASTStruct),
    TypeStatement(ASTType),
    None
}

pub struct SymbolProperty {
    kind: SymbolRef,
    flags: ASTModifiers
}

pub enum SymbolKind {
    Struct(HashMap<String, SymbolProperty>),
    Enum(HashMap<String, SymbolRef>),
    Fn{
        parameters: HashMap<String, SymbolRef>,
        return_type: SymbolRef
    },
    Module(HashMap<String, SymbolRef>),
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
pub struct SymbolRef {
    pub id: u32,
    pub instance_id: Option<usize>,
    flags: SymbolFlags
}

impl SymbolRef {

    pub fn new_ref(id: u32) -> Self {
        SymbolRef { id, instance_id: None, flags: SymbolFlags::empty() }
    }

    pub fn new_instance(id: u32, instance_id: usize) -> Self {
        SymbolRef { id: id, instance_id: Some(instance_id), flags: SymbolFlags::empty() }
    }

    pub fn get_kind<'a, T: SymbolCollector>(&self, collector: &'a T) -> &'a SymbolKind {
        let sym = collector.get_symbol(&self.id).unwrap();
        match self.instance_id {
            Some(id ) => &sym.instances[id].kind,
            None => &sym.kind
        }
    }

    pub fn is_value(&self) -> bool {
        self.flags.contains(SymbolFlags::CONST) | self.flags.contains(SymbolFlags::STATIC)
    }

    pub fn is_optional(&self) -> bool {
        self.flags.contains(SymbolFlags::OPTIONAL)
    }

    pub fn is_const(&self) -> bool {
        self.flags.contains(SymbolFlags::CONST)
    }

    pub fn is_static(&self) -> bool {
        self.flags.contains(SymbolFlags::STATIC)
    }

    pub fn is_type_param(&self) -> bool {
        self.flags.contains(SymbolFlags::TYPE_PARAM)
    }

    pub fn make_optional(mut self) -> Self {
        self.flags.set(SymbolFlags::OPTIONAL, true);
        self
    }

    pub fn make_const(mut self) -> Self {
        self.flags.set(SymbolFlags::CONST, true);
        self
    }

    pub fn make_static(mut self) -> Self {
        self.flags.set(SymbolFlags::STATIC, true);
        self
    }

    pub fn make_type_param(mut self) -> Self {
        self.flags.set(SymbolFlags::TYPE_PARAM, true);
        self
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

impl ToSymbol for SymbolRef {
    fn to_symbol<'a, T: SymbolCollector>(&self, collector: &'a T) -> &'a Symbol {
        collector.get_symbol(&self.id).unwrap()
    }
}

pub struct Symbol {
    pub name: String,
    pub id: u32,
    pub kind: SymbolKind,
    pub type_params: HashMap<String, Option<SymbolRef>>,
    pub instances: Vec<SymbolInstance>,
    pub declaration: StatementOrExpression,
    pub impls: Vec<SymbolRef>
}

pub struct SymbolInstance {
    pub id: usize,
    pub kind: SymbolKind,
    pub type_args: Vec<SymbolRef>
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

    pub fn create_or_get_instance(&mut self, params: Vec<SymbolRef>, ast: &ASTListTyping) -> LazyResult<SymbolRef> {
        let params_len = params.len();
        if params_len != self.type_params.len() { 
            return Err(err!(INVALID_AMOUNT_OF_TYPE_PARAMS, ast.range, &self.type_params.len().to_string(), &params_len.to_string()));
        };
        'outer: for instance in &self.instances {
            for ind in 0..params_len {
                if params[ind].id != instance.type_args[ind].id {
                    continue 'outer;
                }
            }
            return Ok(SymbolRef::new_instance(self.id, instance.id));
        };
        let instance_id = self.instances.len() + 1;
        self.instances.push(SymbolInstance {
            id: instance_id,
            // Create a new kind from this symbol's kind, but replace type arguments with actual values
            kind: SymbolKind::None,
            type_args: params
        });
        Ok(SymbolRef::new_instance(self.id, instance_id))
    }

    pub fn to_ref(&self) -> SymbolRef {
        SymbolRef { id: self.id, instance_id: None, flags: SymbolFlags::empty() }
    }

    pub fn get_mod_type<'a, C: SymbolCollector>(&'a self, collector: &'a C, name: &str) -> Option<&'a SymbolRef> {
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
