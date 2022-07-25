use std::{fs, path::Path};

const BAD_NTFS_CHARS: [char; 9] = ['/', '?', '<', '>', '\\', ':', '*', '|', '"'];

#[inline]
pub fn dir_exists(path: &str) -> bool {
    let path = Path::new(path);
    path.exists() && path.is_dir()
}

#[inline]
pub fn delete_directory(path: &str) -> bool {
    if !dir_exists(path) {
        return false;
    }

    fs::remove_dir_all(path).is_ok()
}

#[inline]
pub fn file_exists(path: &str) -> bool {
    let path = Path::new(path);
    path.exists() && path.is_file()
}

#[inline]
pub fn get_file_extension(fp: &str) -> Option<String> {
    let index = fp.rfind('.')?;
    let (_, ext) = fp.split_at(index + 1);

    Some(ext.to_lowercase())
}

#[inline]
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
#[inline]
pub fn sanitize_ntfs_name(str: &str) -> String {
    str.replace(&BAD_NTFS_CHARS[..], "")
}

/// Swap the extensions of a specified file path.
///
/// # Arguments
///
/// * `fp` - The path to the file.
/// * `ext` - The new extension to be applied.
///
#[inline]
pub fn swap_file_extension(fp: &str, new_ext: &str) -> String {
    use std::path::PathBuf;

    let mut path = PathBuf::from(fp);
    path.set_extension(new_ext);

    path.to_string_lossy().to_string()
}
