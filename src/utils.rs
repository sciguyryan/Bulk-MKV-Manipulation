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

/// Attempt to guess the mimetype of a file from it's file extension.
///
/// # Arguments
///
/// * `ext` - The extension of the file.
///
pub fn guess_mime_from_extension(ext: &str) -> String {
    match ext {
        "otf" => "font/otf".to_string(),
        "ttf" => "font/ttf".to_string(),
        _ => {
            panic!("Unrecognized file extension: {}", ext);
        }
    }
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

/// Swap the extensions of a specified file path.
///
/// # Arguments
///
/// * `fp` - The path to the file.
/// * `ext` - The new extension to be applied.
///
pub fn swap_file_extension(fp: &str, new_ext: &str) -> String {
    use std::path::PathBuf;

    let mut path = PathBuf::from(fp);
    path.set_extension(new_ext);

    path.to_string_lossy().to_string()
}
