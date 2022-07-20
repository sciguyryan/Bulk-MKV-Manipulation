use std::{fs, path::Path};

pub fn file_exists(path: &str) -> bool {
    let path = Path::new(path);
    path.exists() && path.is_file()
}

pub fn dir_exists(path: &str) -> bool {
    let path = Path::new(path);
    path.exists() && path.is_dir()
}

pub fn delete_directory(path: &str) -> bool {
    if !dir_exists(path) {
        return false;
    }

    match fs::remove_dir_all(path) {
        Ok(_) => true,
        Err(_) => false,
    }
}
