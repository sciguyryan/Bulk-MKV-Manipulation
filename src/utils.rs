use std::{ffi::OsStr, fs, path::Path};

pub fn dir_exists(path: &str) -> bool {
    let path = Path::new(path);
    path.exists() && path.is_dir()
}

pub fn delete_directory(path: &str) -> bool {
    if !dir_exists(path) {
        return false;
    }

    fs::remove_dir_all(path).is_ok()
}

pub fn file_exists(path: &str) -> bool {
    let path = Path::new(path);
    path.exists() && path.is_file()
}

pub fn get_file_extension(fp: &str) -> Option<&str> {
    if !file_exists(fp) {
        return None;
    }

    Path::new(fp).extension().and_then(OsStr::to_str)
}
