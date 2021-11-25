

pub trait FileHost {
    fn get(&mut self, path: String) -> String;
    fn exists(&mut self, path: String) -> bool;
}
