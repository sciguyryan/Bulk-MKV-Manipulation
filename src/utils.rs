use std::{fs, path::Path};

const BAD_NTFS_CHARS: [char; 9] = ['/', '?', '<', '>', '\\', ':', '*', '|', '"'];

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

pub fn get_file_extension(fp: &str) -> Option<String> {
    let index = fp.rfind('.')?;
    let (_, ext) = fp.split_at(index + 1);

    Some(ext.to_lowercase())
}

pub fn join_paths_to_string(base: &str, paths: &[&str]) -> String {
    let mut p = Path::new(base).to_path_buf();

    for path in paths {
        p = p.join(path);
    }

    p.to_string_lossy().to_string()
}

/// Strip invalid NTFS characters from a path string.
///
/// # Arguments
///
/// * `str` - The string to be sanitized.
///
pub fn sanitize_ntfs_name(str: &str) -> String {
    str.replace(&BAD_NTFS_CHARS[..], "")
}
