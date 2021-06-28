
use std::collections::HashMap;
use super::type_checker::*;

pub struct Scope<'a> {
    pub parent: Option<Box<Scope<'a>>>,
    pub variables: HashMap<String, TypeInstance<'a>>,
    pub generics: HashMap<String, VarTyping>
}

impl<'a> Scope<'a> {

    pub fn new(parent: Option<Box<Scope<'a>>>) -> Self {
        Scope {
            parent,
            variables: HashMap::new(),
            generics: HashMap::new()
        }
    }

    pub fn extend(self) -> Self {
        Scope {
            variables: HashMap::new(),
            generics: HashMap::new(),
            parent: Some(Box::new(self))
        }
    }

    pub fn clear(mut self) -> Option<Self> {
        self.variables.clear();
        if let Some(parent) = self.parent {
            Some(*parent)
        } else { None }
    }

    pub fn has(&self, key: &str) -> bool {
        if self.variables.contains_key(key) { true }
        else if let Some(parent) = &self.parent {
            parent.has(key)
        } else { false }
    }

    pub fn has_generic(&self, key: &str) -> bool {
        if self.generics.contains_key(key) { true }
        else if let Some(parent) = &self.parent {
            parent.has_generic(key)
        } else { false }
    }


    pub fn get_generic(&self, key: &str) -> Option<&VarTyping> {
        if let Some(gen) = self.generics.get(key) { Some(gen) }
        else if let Some(parent) = &self.parent {
            parent.get_generic(key)
        } else { None }
    }

    pub fn get(&self, key: &str) -> Option<&TypeInstance> {
        if let Some(gen) = self.variables.get(key) { Some(gen) }
        else if let Some(parent) = &self.parent {
            parent.get(key)
        } else { None }
    }
    
}