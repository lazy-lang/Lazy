
pub use parser::tokenizer::*;
use std::fmt;

pub enum TypeErrors {
    Incompatible(String, String),
    StructExists(String),
    TypeDoesntExist(String),
    NotImplemented
}

impl fmt::Display for TypeErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { 
        match &self {
            Self::Incompatible(a, b) => write!(f, "Types {} and {} are incompatible", a, b),
            Self::StructExists(stname) => write!(f, "The struct with name '{}' already exists", stname),
            Self::TypeDoesntExist(t) => write!(f, "The type {} doesn't exist", t),
            Self::NotImplemented => write!(f, "Not implemented")
        }
    }
}