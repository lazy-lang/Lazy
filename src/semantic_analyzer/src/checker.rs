use errors::*;
use diagnostics::*;

use crate::{module::*, symbol::*};
use rustc_hash::FxHashMap;

pub struct TypeChecker {
    pub symbols: FxHashMap<u32, Symbol>
}

impl SymbolCollector for TypeChecker {
    
    fn get_symbol(&self, name: &u32) -> Option<&Symbol> {
        self.symbols.get(name)
    }

    fn insert_symbol(&mut self, sym: Symbol) {
        self.symbols.insert(sym.id, sym);
    }

}

impl TypeChecker {

    pub fn check_module(&mut self, module: &mut Module) -> LazyResult<()> {
        Ok(())
    }

    fn check_struct(&mut self, module: &mut Module, structure: &ASTStruct) -> LazyResult<SymbolKind> {
        let mut props: HashMap<String, SymbolLike> =  HashMap::new();
        for prop in &structure.fields.pairs {
            // TODO: If the property has a default value, get the type from it. The property's not guaranteed to have a type.
            props.insert(prop.name.clone(), self.get_sym_from_type(module, prop.value.as_ref().unwrap())?);
        }
        panic!("TODO")
    }

    //
    // Checks if a type is valid.
    // 
    fn get_sym_from_type(&mut self, module: &mut Module, typing: &ASTTypings) -> LazyResult<SymbolLike> {
        match typing {
            ASTTypings::Var(name) => {
                let sym_id = self.get_sym_from_var(module, &name.value)?;
                // TODO: Check for type parameters by getting the symbol
                //let sym = self.symbols.get(&self.get_sym_from_var(module, &name.value)?).unwrap();
                Ok(SymbolLike::Ref(sym_id))
            },
            _ => Err(err!(UNEXPECTED_EOF, Range::default(), &module.filename))
        }
    }

    fn get_sym_from_var(&mut self, module: &mut Module, var: &ASTVar) -> LazyResult<u32> {
        if let Some(mut val) = module.temporary.remove(&var.value) {
            val.kind = match &val.declaration {
                StatementOrExpression::StructStatement(structure) => self.check_struct(module, structure)?,
                _ => SymbolKind::None
            };
            let sym_id = val.id;
            self.symbols.insert(val.id, val);
            Ok(sym_id)
        } else if let Some(sym) = module.get_sym(&var.value) {
            Ok(sym.get_id())
        } else {
            Err(err!(NAME_NOT_FOUND, var.range, &module.filename, &var.value))
        }
    }

}