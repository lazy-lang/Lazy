
use std::collections::HashMap;
use super::type_checker::*;

pub struct Scope<'a> {
    pub parent: Option<Box<Scope<'a>>>,
    pub variables: HashMap<String, TypeInstance<'a>>
}

impl<'a> Scope<'a> {

    pub fn new(parent: Option<Box<Scope<'a>>>) -> Self {
        Scope {
            parent,
            variables: HashMap::new()
        }
    }

    pub fn extend(self) -> Self {
        Scope {
            variables: HashMap::new(),
            parent: Some(Box::new(self))
        }
    }

    pub fn clear(mut self) -> Option<Self> {
        self.variables.clear();
        if let Some(parent) = self.parent {
            Some(*parent)
        } else { None }
    }

    pub fn has(&self, key: &String) -> bool {
        if self.variables.contains_key(key) { true }
        else if let Some(parent) = &self.parent {
            parent.has(key)
        } else { false }
    }

    pub fn set(&mut self, key: String, val: TypeInstance<'a>) {
        self.variables.insert(key, val);
    }
    
}