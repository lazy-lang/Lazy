
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
}

impl<'a> ErrorCollector<TypeErrors> for LazyAnalyzer<'a> {

    fn error(&mut self, err: TypeErrors, start: LoC, end: LoC) {
        self.errors.push(Error::new(err, Range { start, end }));
    }

    fn error_lbl(&mut self, err: TypeErrors, start: LoC, end: LoC, labels: Vec<ErrorLabel>, highlight: bool) {
        self.errors.push(Error::new_with_labels(err, Range { start, end }, labels, highlight))
    }

}
