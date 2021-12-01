use std::collections::HashMap;
use crate::{file_host::{FileHost}, symbol::{Symbol}};
use parser::{ast::{Parser, model::{ASTImportThing, ASTStatement}}};
use errors::*;
use diagnostics::*;

pub struct Module {
    pub local: HashMap<String, u32>,
    pub exported: HashMap<String, u32>,
    pub filename: String
}

impl Module {
    
    pub fn from_str<T: FileHost>(host: &mut T, filename: &str, content: &str) -> LazyMultiResult<Self> {
        let mut temp_syms: HashMap<String, Symbol> = HashMap::new();
        let mut local: HashMap<String, u32> = HashMap::new();
        let mut exported: HashMap<String, u32> = HashMap::new();
        let mut parser =  Parser::new(&content, filename.to_string());
        let (ast, mut errs) = parser.parse();
        if !errs.is_empty() || !parser.tokens.errors.is_empty() {
            let mut new_vec: Vec<Error> = vec![];
            new_vec.append(&mut parser.tokens.errors);
            new_vec.append(&mut errs);
            return Err(new_vec);
        }
        if !parser.tokens.errors.is_empty() {
            return Err(parser.tokens.errors);
        }
        let mut errors: Vec<Error> = vec![];
        for statement in ast {
            match statement {
                ASTStatement::Import(decl) => {
                    let module = if let Some(m) = host.get_or_create(&decl.path.value)? { m } else {
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
                                    errors.push(err!(TYPE_NOT_FOUND_FROM_MOD, item.range, &item_name, &filename, &decl.path.to_string()));
                                    &0
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
                }
                ASTStatement::EnumDeclaration(decl) => {
                    let name = &decl.name.value;
                    if temp_syms.contains_key(name) || local.contains_key(name) {
                        errors.push(err!(DUPLICATE_IDENT, decl.name.range, &filename, name));
                        continue;
                    }
                    temp_syms.insert(name.to_string(), Symbol::empty(host.get_unique_id(), name.to_string(), ASTStatement::EnumDeclaration(decl)));
                },
                ASTStatement::Struct(decl) => {
                    let name = &decl.name.value;
                    if temp_syms.contains_key(name) || local.contains_key(name) {
                        errors.push(err!(DUPLICATE_IDENT, decl.name.range, &filename, name));
                        continue;
                    }
                    temp_syms.insert(name.to_string(), Symbol::empty(host.get_unique_id(), name.to_string(), ASTStatement::Struct(decl)));
                },
                ASTStatement::Type(decl) => {
                    let name = &decl.name.value;
                    if temp_syms.contains_key(name) || local.contains_key(name) {
                        errors.push(err!(DUPLICATE_IDENT, decl.name.range, &filename, name));
                        continue;
                    }
                    temp_syms.insert(name.to_string(), Symbol::empty(host.get_unique_id(), name.to_string(), ASTStatement::Type(decl)));
                },
                ASTStatement::Export(decl) => {
                    match *decl.value {
                        ASTStatement::EnumDeclaration(decl) => {
                            let name = &decl.name.value;
                            if temp_syms.contains_key(name) || local.contains_key(name) {
                                errors.push(err!(DUPLICATE_IDENT, decl.name.range, &filename, name));
                                continue;
                            }
                            let id = host.get_unique_id();
                            exported.insert(name.to_string(), id);
                            temp_syms.insert(name.to_string(), Symbol::empty(id, name.to_string(), ASTStatement::EnumDeclaration(decl)));
                        },
                        ASTStatement::Struct(decl) => {
                            let id = host.get_unique_id();
                            let name = decl.name.value.to_string();
                            exported.insert(name.to_string(), id);
                            temp_syms.insert(name.to_string(), Symbol::empty(id, name, ASTStatement::Struct(decl)));
                        },
                        ASTStatement::Type(decl) => {
                            let name = &decl.name.value;
                            if temp_syms.contains_key(name) || local.contains_key(name) {
                                errors.push(err!(DUPLICATE_IDENT, decl.name.range, name));
                                continue;
                            }
                            let id = host.get_unique_id();
                            exported.insert(name.to_string(), id);
                            temp_syms.insert(name.to_string(), Symbol::empty(id, name.to_string(), ASTStatement::Type(decl)));
                        },
                        _ => {}
                    }
                }
                _ => {}
            };
        }
        if !errors.is_empty() {
            Err(errors)
        } else {
            Ok(Self { local, exported, filename: filename.to_string() })
        }
    }

}