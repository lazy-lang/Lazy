use std::collections::HashMap;
use crate::{file_host::{FileHost}, symbol::{Symbol}};
use parser::{ast::{Parser, model::{ASTImportThing, ASTStatement}}, tokenizer::error::ParserErrorType};
use errors::Error;

pub struct Module {
    pub local: HashMap<String, u32>,
    pub exported: HashMap<String, u32>,
    pub filename: String,
    pub content: String
}

impl Module {
    
    pub fn from_str<T: FileHost>(host: &mut T, filename: &str, content: String) -> Result<Self, Vec<Error<ParserErrorType>>> {
        let mut temp_syms: HashMap<String, Symbol> = HashMap::new();
        let mut local: HashMap<String, u32> = HashMap::new();
        let mut exported: HashMap<String, u32> = HashMap::new();
        let mut parser =  Parser::new(&content);
        let ast = parser.parse();
        if !parser.tokens.errors.is_empty() {
            return Err(parser.tokens.errors);
        }
        for statement in ast {
            match statement {
                ASTStatement::Import(decl) => {
                    let imported_from = host.get_or_create(&decl.path.value);
                    if let Some(module_res) = imported_from {
                        let module = module_res;
                        match decl.thing {
                            ASTImportThing::All => {
                                for (name, id) in module.exported.iter() {
                                    local.insert(name.clone(), id.clone());
                                }
                            }
                            ASTImportThing::Items(item_list) => {
                                for item in item_list {
                                    let item_name = item.name.clone();
                                    let item_id = if let Some(id) = module.exported.get(&item_name) { id } 
                                    else {
                                        host.error(format!("Type {} not found in module \"{}\"", item_name, decl.path), item.range);
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
                }
                ASTStatement::EnumDeclaration(decl) => {
                    if let Some(_) = temp_syms.insert(decl.name.value.to_string(), Symbol::empty(host.get_unique_id(), decl.name.value.to_string(), ASTStatement::EnumDeclaration(decl))) {

                    }
                },
                ASTStatement::Struct(decl) => {
                    temp_syms.insert(decl.name.value.to_string(), Symbol::empty(host.get_unique_id(), decl.name.value.to_string(), ASTStatement::Struct(decl)));
                },
                ASTStatement::Type(decl) => {
                    temp_syms.insert(decl.name.to_string(), Symbol::empty(host.get_unique_id(), decl.name.to_string(), ASTStatement::Type(decl)));
                },
                ASTStatement::Export(decl) => {
                    match *decl.value {
                        ASTStatement::EnumDeclaration(decl) => {
                            let id = host.get_unique_id();
                            let name = decl.name.value.to_string();
                            exported.insert(name.to_string(), id);
                            temp_syms.insert(name.to_string(), Symbol::empty(id, name, ASTStatement::EnumDeclaration(decl)));
                        },
                        ASTStatement::Struct(decl) => {
                            let id = host.get_unique_id();
                            let name = decl.name.value.to_string();
                            exported.insert(name.to_string(), id);
                            temp_syms.insert(name.to_string(), Symbol::empty(id, name, ASTStatement::Struct(decl)));
                        },
                        ASTStatement::Type(decl) => {
                            let id = host.get_unique_id();
                            let name = decl.name.to_string();
                            exported.insert(name.to_string(), id);
                            temp_syms.insert(name.to_string(), Symbol::empty(id, name, ASTStatement::Type(decl)));
                        },
                        _ => {}
                    }
                }
                _ => {}
            };
        }
        Ok(Self { local, exported, filename: filename.to_string(), content })
    }

}