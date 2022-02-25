use errors::*;
use crate::{module::*, symbol::*};
use rustc_hash::FxHashMap;

pub struct TypeChecker {
    pub symbols: FxHashMap<u32, Symbol>
}

impl SymbolCollector for TypeChecker {
    
    fn get_symbol(&self, name: &u32) -> Option<&Symbol> {
        self.symbols.get(name)
    }

    fn get_mut_symbol(&mut self, name: &u32) -> Option<&mut Symbol> {
        self.symbols.get_mut(name)
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
        let mut props: HashMap<String, SymbolRef> =  HashMap::new();
        for prop in &structure.fields.pairs {
            let sym_id = self.get_sym_from_type(module, prop.value.as_ref().unwrap(), structure.name.value == prop.name)?.to_symbol(self);
            // TODO: If the property has a default value, get the type from it. The property's not guaranteed to have a type.
            props.insert(prop.name.clone(), sym_id.to_ref());
        }
        panic!("TODO")
    }

    //
    // Checks if a type is valid.
    // 
    fn get_sym_from_type(&mut self, module: &mut Module, typing: &ASTTypings, handle_temps: bool) -> LazyResult<SymbolRef> {
        match typing {
            ASTTypings::Var(name) => {
                let sym_id = self.get_sym_from_var(module, &name.value, handle_temps)?;
                let sym = self.symbols.get(&sym_id).unwrap();
                if let Some(generics) = &name.typings {
                    let checked_list = self.check_list(module, generics, handle_temps)?;
                    let sym = self.symbols.get_mut(&sym_id).unwrap();
                    Ok(sym.create_or_get_instance(checked_list, generics)?)
                }
                else {
                    Ok(SymbolRef::new_ref(sym.id))
                }
            },
            ASTTypings::Optional(typ) => {
                let typing = self.get_sym_from_type(module, typ, handle_temps)?;
                Ok(typing.clone().make_optional())
            }
            ASTTypings::Mod(name) => {
                let mut val = self.get_sym_from_var(module, &name.path[0], handle_temps)?.to_symbol(self);
                let mut is_enum = false;
                for i in 1..name.path.len() {
                    let var = &name.path[i];
                    if let Some(typ) = val.get_mod_type(self, &var.value) {
                        if val.kind.is_enum() { is_enum = true };
                        val = typ.to_symbol(self);
                    } else {
                        return Err(err!(NAME_NOT_FOUND, var.range, &var.value))
                    }
                };
                if is_enum {
                    Err(err!(VAL_AS_TYPE, name.range))
                } else {
                    Ok(val.to_ref())
                }
            }
            _ => Err(err!(UNEXPECTED_EOF, Range::default(), &module.filename))
        }
    }

    fn get_sym_from_var(&mut self, module: &mut Module, var: &ASTVar, handle_temps: bool) -> LazyResult<u32> {
        match module.temporary.remove(&var.value) {
            Some(mut val) if handle_temps => {
                val.kind = match &val.declaration {
                    StatementOrExpression::StructStatement(structure) => self.check_struct(module, structure)?,
                    _ => SymbolKind::None
                };
                let sym_id = val.id;
                self.symbols.insert(val.id, val);
                Ok(sym_id)
            },
            _ => {
                if let Some(sym) = module.get_sym(&var.value) {
                    Ok(sym.id)
                } else {
                    Err(err!(NAME_NOT_FOUND, var.range, &module.filename, &var.value))
                }
            }
        }
    }

    fn check_list(&mut self, module: &mut Module, list: &ASTListTyping, handle_temps: bool) -> LazyResult<Vec<SymbolRef>> {
        let mut result: Vec<SymbolRef> = Vec::new();
        for typing in &list.entries {
            result.push(self.get_sym_from_type(module, typing, handle_temps)?);
        }
        Ok(result)
    }

}