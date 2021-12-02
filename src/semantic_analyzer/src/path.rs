
use std::{env::current_dir};
use std::path::{PathBuf, Component};

pub fn join_paths(path1: &str, path2: &str, remove_filename: bool) -> String {
    let mut path1buf = PathBuf::from(path1);
    if remove_filename { path1buf.pop(); };
    for comp in PathBuf::from(path2).components() {
        match comp {
            Component::Normal(p) => path1buf.push(p),
            Component::ParentDir => { path1buf.pop(); },
            Component::CurDir => {}
            Component::Prefix(_) => {},
            Component::RootDir => {}
        }
    };
    path1buf.display().to_string()
}

pub fn full_path(path: &str) -> String {
    if PathBuf::from(path).is_absolute() { return path.to_string() };
    join_paths(&current_dir().unwrap().display().to_string(), path, false)
}

pub fn file_dir_and_join(path: &str, path2: &str) -> String {
    join_paths(path, path2, true)
}