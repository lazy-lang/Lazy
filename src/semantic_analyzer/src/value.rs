use crate::symbol::SymbolLike;

bitflags::bitflags! {
    pub struct ValueFlags: u32 {
        const STATIC = 1 << 0;
        const CONST = 1 << 1;
    }
}

pub trait ValueCollector {
    fn get_value(&self, name: &str) -> Option<&Value>;
    fn set_value(&mut self, name: &str, value: Value);
}

pub struct Value {
    pub kind: SymbolLike,
    pub flags: ValueFlags
}

impl Value {

    pub fn new(kind: SymbolLike) -> Self {
        Self {
            kind,
            flags: ValueFlags::empty()
        }
    }

}