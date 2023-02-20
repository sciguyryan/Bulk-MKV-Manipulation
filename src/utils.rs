use std::{fmt::Display, fs, path::Path};

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

struct DurationUnit<'a> {
    amount: u64,
    unit: &'a str,
}

impl<'a> DurationUnit<'a> {
    pub fn new(amount: u64, unit: &'a str) -> Self {
        Self { amount, unit }
    }
}

impl<'a> Display for DurationUnit<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut r = write!(f, "{} {}", self.amount, self.unit);
        if self.amount != 1 {
            r = write!(f, "s");
        }
        r
    }
}

/// Convert a duration (in seconds) into days, hours, minutes and seconds.
///
/// # Arguments
///
/// * `seconds` - The duration, in seconds.
pub fn format_duration(seconds: u64) -> String {
    let mut units = vec![];

    let mut seconds = seconds;

    let seconds_in_minute = 60;
    let seconds_in_hour = seconds_in_minute * 60;
    let seconds_in_day = seconds_in_hour * 24;

    if seconds >= seconds_in_day {
        let d = seconds / seconds_in_day;
        seconds -= d * seconds_in_day;

        units.push(DurationUnit::new(d, "day"));
    }

    if seconds >= seconds_in_hour {
        let h = seconds / seconds_in_hour;
        seconds -= h * seconds_in_hour;

        units.push(DurationUnit::new(h, "hour"));
    }

    if seconds >= seconds_in_minute {
        let m = seconds / seconds_in_minute;
        seconds -= m * seconds_in_minute;

        units.push(DurationUnit::new(m, "minute"));
    }

    if seconds > 0 {
        units.push(DurationUnit::new(seconds, "second"));
    }

    let mut formatted = String::new();
    let is_single = units.len() == 1;
    let last = units.len() - 1;
    for (i, unit) in units.into_iter().enumerate() {
        if i == last && !is_single {
            formatted.push_str("and ");
        }

        formatted.push_str(&format!("{unit}"));

        if i < last {
            formatted.push_str(", ");
        }
    }

    formatted
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
