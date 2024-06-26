use std::{fs::File, io::Write, path::PathBuf};

use crate::paths;

pub struct ScriptFile {
    path: PathBuf,
}

impl ScriptFile {
    pub fn new(file_id: usize, suffix: &str) -> Self {
        let path = ScriptFile::generate_temp_file_path(file_id, suffix);

        _ = File::create(&path).expect("failed to create temporary file");

        Self { path }
    }

    /// Generate a temporary file path.
    ///
    /// # Arguments
    ///
    /// * `file_id` - The ID of the media file.
    /// * `postfix` - The postfix notation for the purpose of this file.
    fn generate_temp_file_path(file_id: usize, postfix: &str) -> PathBuf {
        let mut temp_path = PathBuf::from(&paths::PATHS.temp);
        temp_path.push(file_id.to_string());
        temp_path.push("scripts");
        temp_path.push(ScriptFile::generate_random_file_name(postfix));

        temp_path
    }

    /// Generate a random file name.
    ///
    /// # Arguments
    ///
    /// * `postfix` - The postfix notation for the purpose of this file.
    fn generate_random_file_name(postfix: &str) -> String {
        use rand::{self, distributions::Alphanumeric, Rng};

        let file_name: String = Rng::sample_iter(rand::thread_rng(), &Alphanumeric)
            .take(16)
            .map(char::from)
            .collect();

        format!("{file_name}-{postfix}.bat")
    }

    /// Get the temporary file's path.
    pub fn get_path(&self) -> &PathBuf {
        &self.path
    }

    /// Write a string to the temporary file.
    pub fn write_string(&self, str: String) {
        let mut file = File::options()
            .write(true)
            .open(&self.path)
            .expect("failed to open temporary file");

        writeln!(&mut file, "{str}").expect("failed to write to temporary file");

        file.flush()
            .expect("failed to flush data to temporary file");
    }
}
