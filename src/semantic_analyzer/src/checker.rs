use errors::*;
use diagnostics::*;

use crate::{module::*, symbol::*};
use rustc_hash::FxHashMap;

pub struct TypeChecker {
    pub symbols: FxHashMap<u32, Symbol>,
    pub temp_symbols: FxHashMap<u32, Symbol>
}

impl SymbolCollector for TypeChecker {
    
    fn get_symbol(&self, name: &u32) -> Option<&Symbol> {
        self.symbols.get(name).or_else(|| self.temp_symbols.get(name))
    }

    fn insert_symbol(&mut self, sym: Symbol) {
        self.symbols.insert(sym.id, sym);
    }

}

impl TypeChecker {

    pub fn insert_temp_symbol(&mut self, sym: Symbol) {
        self.temp_symbols.insert(sym.id, sym);
    }

    pub fn check_module(&mut self, module: &mut Module) -> LazyResult<()> {
        Ok(())
    }

    fn check_struct(&mut self, module: &mut Module, structure: ASTStruct) {

    }

    fn check_type(&mut self, module: &mut Module, typing: ASTTypings) -> LazyResult<SymbolLike> {
        match typing {
            ASTTypings::Var(name) => {
                if let Some(sym) = module.temporary.get(&name.value.value) {
                    if let Some(typings) = name.typings {
                        //TBD
                        Ok(SymbolLike::Ref(1))
                    } else {
                        Ok(SymbolLike::Ref(sym.id))
                    }
                } else {
                    Err(err!(NAME_NOT_FOUND, name.value.range, &module.filename, &name.value.value))
                }
            }
            _ => Err(err!(UNEXPECTED_EOF, Range::default(), &module.filename))
        }
    }

}