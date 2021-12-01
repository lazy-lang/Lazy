
use crate::{module::Module, symbol::Symbol};
use std::collections::HashMap;
use errors::{builder::ErrorFormatter, LazyMultiResult};


pub trait FileHost: ErrorFormatter {
    fn create(&mut self, path: &str) -> LazyMultiResult<Option<&Module>>;
    fn get(&self, path: &str) -> Option<&Module>;
    fn get_or_create(&mut self, path: &str) -> LazyMultiResult<Option<&Module>>;
    fn get_unique_id(&mut self) -> u32;
    fn insert_symbol(&mut self, sym: Symbol);
    fn get_symbol(&self, name: &u32) -> Option<&Symbol>;
}

pub struct VirtualFileHost {
    pub id_counter: u32,
    pub symbols: HashMap<u32, Symbol>,
    pub files: HashMap<String, Module>,
    pub file_contents: HashMap<String, String>,
    pub file_cache: HashMap<String, String>
}

impl ErrorFormatter for VirtualFileHost {

    fn get_file_contents(&self, file: &str) -> Option<&str> {
        Some(&self.file_contents.get(file)?)
    }
}

impl FileHost for VirtualFileHost {

    fn get(&self, path: &str) -> Option<&Module> {
        self.files.get(path)
    }

    fn get_or_create(&mut self, path: &str) -> LazyMultiResult<Option<&Module>> {
        if let Some(file_contents) = self.file_cache.remove(path) {
            self.create_virtual(path, file_contents)
        } else {
            Ok(self.files.get(path))
        }
    }

    fn get_unique_id(&mut self) -> u32 {
        self.id_counter += 1;
        self.id_counter
    }

    fn create(&mut self, _path: &str) -> LazyMultiResult<Option<&Module>> {
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
            symbols: HashMap::new(),
            files: HashMap::new(),
            file_contents: HashMap::new(),
            file_cache: HashMap::new()
        }
    }

    pub fn add_to_cache(&mut self, path: &str, content: String) {
        self.file_cache.insert(path.to_string(), content);
    }

    pub fn create_virtual(&mut self, path: &str, content: String) -> LazyMultiResult<Option<&Module>> {
        self.file_contents.insert(path.to_string(), content.clone());
        let module = Module::from_str(self, path, &content)?;
        self.files.insert(path.to_string(), module);
        Ok(self.files.get(path))
    }
}