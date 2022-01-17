use std::collections::HashMap;
use crate::{file_host::{FileHost}, symbol::{Symbol, SymbolLike, StatementOrExpression}};
use parser::{ast::{Parser, model::{ASTImportThing, ASTStatement}}};
use errors::*;
use diagnostics::*;
use crate::path::file_dir_and_join;

pub struct Module {
    pub local: HashMap<String, SymbolLike>,
    pub exported: HashMap<String, SymbolLike>,
    pub temporary: HashMap<String, Symbol>,
    pub filename: String
}

impl Module {
    
    pub fn from_str<T: FileHost>(host: &mut T, filename: &str, content: &str) -> LazyMultiResult<Self> {
        let mut temp_syms: HashMap<String, Symbol> = HashMap::new();
        let mut local: HashMap<String, SymbolLike> = HashMap::new();
        let mut exported: HashMap<String, SymbolLike> = HashMap::new();
        let mut parser =  Parser::new(&content, filename.to_string());
        let (ast, mut errs) = parser.parse();
        let mut errors: Vec<Error> = vec![];
        errors.append(&mut parser.tokens.errors);
        errors.append(&mut errs);
        for statement in ast {
            if let Some((name, range, is_exported, decl)) = match statement {
                ASTStatement::Import(decl) => {
                    let path_to_mod = file_dir_and_join(filename, &decl.path.value);
                    let module = if let Some(m) = host.get_or_create(&path_to_mod)? { m } else {
                        errors.push(err!(MOD_NOT_FOUND, decl.path.range, &filename, &decl.path.value));
                        continue;
                    };
                    match decl.thing {
                        ASTImportThing::All => {
                            for (name, id) in module.exported.iter() {
                                local.insert(name.clone(), id.clone());
                            }
                        }
                        ASTImportThing::Items(item_list) => {
                            for item in item_list {
                                let item_name = item.name.clone();
                                let item_id = if let Some(id) = module.exported.get(&item_name) { id } else {
                                    errors.push(err!(TYPE_NOT_FOUND_FROM_MOD, item.range, &filename, &item_name, &decl.path.to_string()));
                                    &SymbolLike::Ref(0)
                                };
                                let name = if let Some(alias) = item.r#as {
                                    alias.value 
                                } else {
                                    item.name
                                };
                                local.insert(name, item_id.clone());
                            }
                        }
                    }
                    None
                }
                ASTStatement::EnumDeclaration(decl) => Some((decl.name.value.clone(), decl.name.range, false, StatementOrExpression::EnumStatement(decl))),
                ASTStatement::Struct(decl) => Some((decl.name.value.clone(), decl.name.range, false, StatementOrExpression::StructStatement(decl))),
                ASTStatement::Type(decl) => Some((decl.name.value.clone(), decl.name.range, false, StatementOrExpression::TypeStatement(decl))),
                ASTStatement::Export(decl) => {
                    match *decl.value {
                        ASTStatement::EnumDeclaration(decl) => Some((decl.name.value.clone(), decl.name.range, true, StatementOrExpression::EnumStatement(decl))),
                        ASTStatement::Struct(decl) => Some((decl.name.value.clone(), decl.name.range, true, StatementOrExpression::StructStatement(decl))),
                        ASTStatement::Type(decl) => Some((decl.name.value.clone(), decl.name.range, true, StatementOrExpression::TypeStatement(decl))),
                        _ => None
                    }
                }
                _ => None
            } {
                if temp_syms.contains_key(&name) || local.contains_key(&name) {
                    errors.push(err!(DUPLICATE_IDENT, range, &filename, &name));
                    continue;
                }
                let id = host.get_unique_id();
                if is_exported { exported.insert(name.to_string(), SymbolLike::Ref(id)); };
                temp_syms.insert(name.to_string(), Symbol::empty(id, name, decl));
            }
        }
        if !errors.is_empty() {
            Err(errors)
        } else {
            Ok(Self { local, exported, filename: filename.to_string(), temporary: temp_syms })
        }
    }

}