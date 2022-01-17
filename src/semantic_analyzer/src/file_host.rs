
use crate::{module::Module};
use std::collections::HashMap;
use errors::{builder::ErrorFormatter, LazyMultiResult};
use std::fs;
use crate::path::full_path;

pub trait FileHost: ErrorFormatter {
    fn create(&mut self, path: &str) -> LazyMultiResult<Option<&Module>>;
    fn get(&self, path: &str) -> Option<&Module>;
    fn get_or_create(&mut self, path: &str) -> LazyMultiResult<Option<&Module>>;
    fn get_unique_id(&mut self) -> u32;
}

pub struct VirtualFileHost {
    pub id_counter: u32,
    pub files: HashMap<String, Module>,
    pub file_contents: HashMap<String, String>,
    pub file_cache: HashMap<String, String>,
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

}

impl VirtualFileHost {

    pub fn new() -> Self {
        Self {
            id_counter: 0,

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

pub struct FSFileHost {
    pub id_counter: u32,
    pub files: HashMap<String, Module>,
    pub file_contents: HashMap<String, String>,
}

impl ErrorFormatter for FSFileHost {

    fn get_file_contents(&self, file: &str) -> Option<&str> {
        Some(&self.file_contents.get(file)?)
    }
}

impl FileHost for FSFileHost {

    fn get_unique_id(&mut self) -> u32 {
        self.id_counter += 1;
        self.id_counter
    }

    fn get(&self, path: &str) -> Option<&Module> {
        self.files.get(path)
    }

    fn create(&mut self, path: &str) -> LazyMultiResult<Option<&Module>> {
        let mut full_path = full_path(path);
        if !full_path.ends_with(".lazy") { full_path += ".lazy" };
        if let Ok(text) = fs::read_to_string(&full_path) {
            let module = Module::from_str(self, &full_path, &text);
            self.file_contents.insert(full_path.to_string(), text);
            let module = module?;
            self.files.insert(full_path.clone(), module);
            Ok(self.files.get(&full_path))
        } else {
            Ok(None)
        }
    }

    fn get_or_create(&mut self, path: &str) -> LazyMultiResult<Option<&Module>> {
        if self.files.contains_key(path) {
            Ok(self.files.get(path))
        } else {
            self.create(path)
        }
    }

}

impl FSFileHost {

    pub fn new() -> Self {
        Self {
            id_counter: 0,
            files: HashMap::new(),
            file_contents: HashMap::new()
        }
    }
}