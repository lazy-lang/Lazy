use errors::*;
use crate::{module::*, symbol::*, value::{Value, ValueCollector}};
use rustc_hash::FxHashMap;

pub struct TypeChecker {
    pub symbols: FxHashMap<u32, Symbol>,
    pub global_values: FxHashMap<String, Value>
}

impl SymbolCollector for TypeChecker {
    
    fn get_symbol(&self, name: &u32) -> Option<&Symbol> {
        self.symbols.get(name)
    }

    fn insert_symbol(&mut self, sym: Symbol) {
        self.symbols.insert(sym.id, sym);
    }

}

impl ValueCollector for TypeChecker {
    fn get_value(&self, name: &str) -> Option<&Value> {
        self.global_values.get(name)
    }

    fn set_value(&mut self, name: &str, value: Value) {
        self.global_values.insert(name.to_string(), value);
    }
}

impl TypeChecker {

    pub fn check_module(&mut self, module: &mut Module) -> LazyResult<()> {
        Ok(())
    }

    fn check_struct(&mut self, module: &mut Module, structure: &ASTStruct) -> LazyResult<SymbolKind> {
        let mut props: HashMap<String, SymbolLike> =  HashMap::new();
        for prop in &structure.fields.pairs {
            let sym_id = self.get_sym_from_type(module, prop.value.as_ref().unwrap(), structure.name.value == prop.name)?;
            // TODO: If the property has a default value, get the type from it. The property's not guaranteed to have a type.
            props.insert(prop.name.clone(), sym_id);
        }
        panic!("TODO")
    }

    //
    // Checks if a type is valid.
    // 
    fn get_sym_from_type(&mut self, module: &mut Module, typing: &ASTTypings, handle_temps: bool) -> LazyResult<SymbolLike> {
        match typing {
            ASTTypings::Var(name) => {
                let sym_id = self.get_sym_from_var(module, &name.value, handle_temps)?;
                // TODO: Check for type parameters by getting the symbol
                //let sym = self.symbols.get(&self.get_sym_from_var(module, &name.value)?).unwrap();
                Ok(SymbolLike::Ref(sym_id))
            },
            ASTTypings::Mod(name) => {
                let mut val = self.get_sym_from_var(module, &name.path[0], handle_temps)?.to_symbol(self);
                let mut is_enum = false;
                for i in 1..name.path.len() {
                    let var = &name.path[i];
                    if let Some(typ) = val.get_mod_type(self, &var.value) {
                        if val.kind.is_enum() { is_enum = true };
                        val = typ.to_symbol(self);
                    } else {
                        return Err(err!(NAME_NOT_FOUND, var.range, module.filename, &var.value))
                    }
                };
                if is_enum {
                    Err(err!(VAL_AS_TYPE, name.range, module.filename))
                } else {
                    Ok(val.as_ref())
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
                    Ok(sym.get_id())
                } else {
                    Err(err!(NAME_NOT_FOUND, var.range, &module.filename, &var.value))
                }
            }
        }
    }

}