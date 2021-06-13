
pub use parser::tokenizer::*;
use std::fmt;

pub enum TypeErrors {
    Incompatible(String, String)
}

impl fmt::Display for TypeErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { 
        match &self {
            Self::Incompatible(a, b) => write!(f, "Types {} and {} are incompatible", a, b)
        }
    }
}