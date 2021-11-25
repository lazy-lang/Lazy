
pub struct Module {
    
}

pub trait FileHost {
    fn get(&mut self, path: String) -> String;
    fn exists(&mut self, path: String) -> bool;
    fn create(&mut self, path: String, content: String);
    fn get_mod(&self, path: String) -> Module;
}
