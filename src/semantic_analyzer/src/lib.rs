
pub mod type_checker;
pub mod scope;
use type_checker::*;
use type_checker::error::*;
use scope::*;
use std::collections::HashMap;
use errors::*;

pub struct LazyAnalyzer<'a> {
    pub scope: Scope<'a>,
    pub types: HashMap<String, TypeDeclaration<'a>>,
    pub errors: Vec<Error<TypeErrors>>
}


impl<'a> LazyAnalyzer<'a> {

    pub fn new() -> Self {
        LazyAnalyzer {
            scope: Scope::new(None),
            types: HashMap::new(),
            errors: Vec::new()
        }
    }

    pub fn register_type(&mut self, ast: &ASTStatement) {
        match ast {
            ASTStatement::Struct(structure) => {
                if self.types.contains_key(&structure.name.value) {
                    structure.name.range.err(TypeErrors::StructExists(structure.name.value.clone()), self);
                    return;
                }
                self.types.insert(structure.name.value.clone(), TypeDeclaration::Struct(StructStruct {
                    name: structure.name.value.clone(),
                    fields: HashMap::new(),
                    generics: Vec::new()
                }));
            }
            _ => {}
        }
    }
    
    pub fn validate_type(&self, ast: &ASTTypings) -> Result<TypeInstance, Error<TypeErrors>> {
        match ast {
            ASTTypings::Var(var) => {
                if self.scope.has_generic(&var.value.value) {
                    return Ok(TypeInstance::Generic(var.value.value.clone()))
                };
                if let Some(type_dec) = self.types.get(&var.value.value) {
                    match type_dec {
                        TypeDeclaration::Struct(structure) => {

                        }
                        _ => {}
                    }
                    Err(var.range.err_nc(TypeErrors::NotImplemented))
                } else {
                    Err(var.range.err_nc(TypeErrors::TypeDoesntExist(var.value.value.clone())))
                }
            },
            _ => Err(Error::new(TypeErrors::NotImplemented, Range::default()))
        }
    }

    pub fn resolve_generics(&mut self, generics: &ASTListTyping) -> Vec<TypeInstance> {
        let mut res: Vec<TypeInstance> = vec![];
        for typing in &generics.entries {
            let inst = self.validate_type(typing);
            if inst.is_ok() {
                res.push(inst.unwrap());
            }
        }
        res
    }
}

impl<'a> ErrorCollector<TypeErrors> for LazyAnalyzer<'a> {

    fn error(&mut self, err: TypeErrors, range: Range) {
        self.errors.push(Error::new(err, range));
    }

    fn error_lbl(&mut self, err: TypeErrors, range: Range, labels: Vec<ErrorLabel>, highlight: bool) {
        self.errors.push(Error::new_with_labels(err, range, labels, highlight))
    }

}
