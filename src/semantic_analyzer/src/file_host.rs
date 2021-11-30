
use crate::{module::Module, symbol::Symbol};
use std::collections::HashMap;
use errors::{ErrorCollector, Error, builder::ErrorFormatter};


pub trait FileHost: ErrorCollector<String> + ErrorFormatter {
    fn create(&mut self, path: &str) -> Option<&Module>;
    fn get(&self, path: &str) -> Option<&Module>;
    fn get_or_create(&mut self, path: &str) -> Option<&Module>;
    fn get_unique_id(&mut self) -> u32;
    fn insert_symbol(&mut self, sym: Symbol);
    fn get_symbol(&self, name: &u32) -> Option<&Symbol>;
}

pub struct VirtualFileHost {
    pub errors: Vec<Error<String>>,
    pub id_counter: u32,
    pub symbols: HashMap<u32, Symbol>,
    pub files: HashMap<String, Module>,
    pub file_cache: HashMap<String, String>
}

impl ErrorCollector<String> for VirtualFileHost {

    fn error(&mut self, e_type: String, range: errors::Range) {
        self.errors.push(Error {
            range,
            msg: e_type,
            highlighted: true,
            labels: None
        })
    }

    fn error_lbl(&mut self, e_type: String, range: errors::Range, labels: Vec<errors::ErrorLabel>) {
        self.errors.push(Error {
            range,
            msg: e_type,
            highlighted: true,
            labels: Some(labels)
        })
    }

}

impl ErrorFormatter for VirtualFileHost {

    fn get_file_contents(&self, file: &str) -> Option<&str> {
        Some(&self.files.get(file)?.content)
    }
}

impl FileHost for VirtualFileHost {

    fn get(&self, path: &str) -> Option<&Module> {
        self.files.get(path)
    }

    fn get_or_create(&mut self, path: &str) -> Option<&Module> {
        if let Some(file_contents) = self.file_cache.remove(path) {
            self.create_virtual(path, file_contents)
        } else {
            self.files.get(path)
        }
    }

    fn get_unique_id(&mut self) -> u32 {
        self.id_counter += 1;
        self.id_counter
    }

    fn create(&mut self, _path: &str) -> Option<&Module> {
        panic!("'create' method doesn't exist for virtual file hosts! Use the 'create_virtual' method instead.")
    }

    fn insert_symbol(&mut self, sym: Symbol) {
        self.symbols.insert(sym.id, sym);
    }

    fn get_symbol(&self, sym: &u32) -> Option<&Symbol> {
        self.symbols.get(sym)
    }

}

impl VirtualFileHost {

    pub fn new() -> Self {
        VirtualFileHost {
            id_counter: 0,
            errors: Vec::new(),
            symbols: HashMap::new(),
            files: HashMap::new(),
            file_cache: HashMap::new()
        }
    }

    pub fn add_to_cache(&mut self, path: &str, content: String) {
        self.file_cache.insert(path.to_string(), content);
    }

    pub fn create_virtual(&mut self, path: &str, content: String) -> Option<&Module> {
        let module = if let Ok(module) = Module::from_str(self, path, content) {
            module
        } else {
            self.error
        };
        let module = Module::from_str(self, path, content);
        self.files.insert(path.to_string(), module);
        self.files.get(path)
    }
}