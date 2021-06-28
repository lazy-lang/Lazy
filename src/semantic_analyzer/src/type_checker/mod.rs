
pub mod error;
pub use parser::ast::model::*;
use std::collections::HashMap;

pub enum VarTyping {
    Bound(ASTBoundTyping),
    Unbound(ASTVar)
}

pub struct StructStruct<'a> {
    pub name: String,
    pub fields: HashMap<String, TypeInstance<'a>>,
    pub generics: Vec<VarTyping>
}

impl<'a> StructStruct<'a> {
    fn init(&self, generics: Vec<TypeInstance<'a>>, nullable: bool) -> StructInstance {
        StructInstance {
            generics,
            nullable,
            structure: self
        }
    }
}

pub struct StructInstance<'a> {
    pub structure: &'a StructStruct<'a>,
    pub generics: Vec<TypeInstance<'a>>,
    pub nullable: bool
}

impl<'a> PartialEq for StructInstance<'a> {
    fn eq(&self, other: &Self) -> bool {
        return self.structure.name == other.structure.name && self.generics.iter().enumerate().all(|(ind, entry)| {
            if let Some(t) = other.generics.get(ind) { t == entry } else { false }
        })
    }
}

pub struct FunctionParameter<'a> {
    pub name: String,
    pub typedef: TypeInstance<'a>
}

pub struct FunctionInstance<'a> {
    pub params: Vec<FunctionParameter<'a>>,
    pub return_type: Box<TypeInstance<'a>>,
    pub generics: Vec<VarTyping>,
    pub nullable: bool
}

impl<'a> PartialEq for FunctionInstance<'a> {
    fn eq(&self, other: &Self) -> bool {
        return self.return_type == other.return_type && self.params.iter().enumerate().all(|(ind, entry)| {
            if let Some(t) = other.params.get(ind) { t.typedef == entry.typedef } else { false }
        })
    }
}

pub struct EnumStruct<'a> {
    pub name: String,
    pub variants: HashMap<String, TypeInstance<'a>>
}

pub struct EnumVariant<'a> {
    pub enum_name: String,
    pub variant: String,
    pub variant_type: Box<TypeInstance<'a>>,
    pub nullable: bool
}

impl<'a> PartialEq for EnumVariant<'a> {
    fn eq(&self, other: &Self) -> bool {
        return self.enum_name == other.enum_name && self.variant == other.variant;
    }
}

pub struct TypedefStruct<'a> {
    pub inner: TypeInstance<'a>,
    pub generics: Vec<VarTyping>
}

pub struct PartialInstance<'a> {
    pub fields: HashMap<String, TypeInstance<'a>>
}

impl<'a> PartialEq for PartialInstance<'a> {
    fn eq(&self, other: &Self) -> bool {
        return self.fields.iter().enumerate().all(|(_, (key, val))| {
            if let Some(t) = other.fields.get(key) {
                t == val
            } else { true }
        })
    }
}

impl<'a> PartialEq<StructInstance<'a>> for PartialInstance<'a> {
    fn eq(&self, other: &StructInstance) -> bool {
        return self.fields.iter().enumerate().all(|(_, (key, val))| {
            if let Some(t) = other.structure.fields.get(key) {
                t == val
            } else { true }
        })
    }
}

#[derive(PartialEq)]
pub enum TypeInstance<'a> {
    Struct(StructInstance<'a>),
    Fn(FunctionInstance<'a>),
    Enum(EnumVariant<'a>),
    Partial(PartialInstance<'a>),
    Generic(String)
}


pub enum TypeDeclaration<'a> {
    Struct(StructStruct<'a>),
    Enum(EnumStruct<'a>),
    Typedef(TypedefStruct<'a>)
}
