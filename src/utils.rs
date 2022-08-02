use std::{fs, path::Path};

/// Convert a boolean value to yes or no.
///
/// # Arguments
///
/// * `b` - The boolean value to be converted.
#[inline]
pub fn bool_to_yes_no(b: bool) -> String {
    if b {
        "yes".to_string()
    } else {
        "no".to_string()
    }
}

/// Return a boolean value indicating whether a given directory exists.
///
/// # Arguments
///
/// * `path` - The path to the directory.
#[inline]
pub fn dir_exists(path: &str) -> bool {
    let path = Path::new(path);
    path.exists() && path.is_dir()
}

/// Delete a directory, and all containing subdirectories and files.
///
/// # Arguments
///
/// * `path` - The path to the directory.
#[inline]
pub fn delete_directory(path: &str) -> bool {
    if !dir_exists(path) {
        return false;
    }

    fs::remove_dir_all(path).is_ok()
}

/// Return a boolean value indicating whether a given file exists.
///
/// # Arguments
///
/// * `path` - The path to the file.
#[inline]
pub fn file_exists(path: &str) -> bool {
    let path = Path::new(path);
    path.exists() && path.is_file()
}

/// Get the extension of a given file path.
///
/// # Arguments
///
/// * `fp` - The path to the file.
#[inline]
pub fn get_file_extension(fp: &str) -> Option<String> {
    let index = fp.rfind('.')?;
    let (_, ext) = fp.split_at(index + 1);

    Some(ext.to_lowercase())
}

/// Join several path segments into a single path.
///
/// # Arguments
///
/// * `base` - The base (root) path.
/// * `paths` - A slice of strings to be appended to the base path.
#[inline]
pub fn join_path_segments<S: AsRef<str>>(base: &str, paths: &[S]) -> String {
    let mut p = Path::new(base).to_path_buf();

    for path in paths {
        p = p.join(path.as_ref());
    }

    p.to_string_lossy().to_string()
}

/// Swap the extensions of a specified file path.
///
/// # Arguments
///
/// * `fp` - The path to the file.
/// * `new_ext` - The new extension to be applied.
///
#[inline]
pub fn swap_file_extension(fp: &str, new_ext: &str) -> String {
    use std::path::PathBuf;

    let mut path = PathBuf::from(fp);
    path.set_extension(new_ext);

    path.to_string_lossy().to_string()
}
