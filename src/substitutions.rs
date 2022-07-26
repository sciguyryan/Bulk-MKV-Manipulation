use regex::Regex;
use serde_derive::Deserialize;

const BAD_NTFS_CHARS: [char; 9] = ['/', '?', '<', '>', '\\', ':', '*', '|', '"'];

#[derive(Clone, Deserialize)]
pub struct Substitutions {
    /// A list of regex substitutions to be applied when sanitizing a string.
    pub regular_expressions: Vec<[String; 2]>,

    /// A list of regex substitutions to be applied when sanitizing a string.
    pub strings: Vec<[String; 2]>,

    /// This will indicate whether we should strip invalid NTFS characters from the string.
    /// If unspecified the value will default to true.
    #[serde(default = "default_strip_ntfs")]
    pub strip_invalid_ntfs_chars: bool,

    /// This will be lazily initialized upon first use.
    #[serde(skip)]
    regex_internal: Vec<(Regex, String)>,
}

impl Substitutions {
    pub fn apply(&mut self, s: &str) -> String {
        if self.regex_internal.is_empty() {
            assert!(self.initialize_regex());
        }

        // Ensure that there are no stray tabs and spaces at the start
        // and end of the file name.
        let mut line = s.trim().to_string();

        // If the input string is empty, there is nothing to do here.
        if line.is_empty() {
            return "".to_string();
        }

        for (re, sub) in &self.regex_internal {
            line = re.replace(&line, sub).to_string();
        }

        for entry in &self.strings {
            line = line.replace(&entry[0], &entry[1]);
        }

        if self.strip_invalid_ntfs_chars {
            line = line.replace(&BAD_NTFS_CHARS[..], "");
        }

        line
    }

    fn initialize_regex(&mut self) -> bool {
        for entry in &self.regular_expressions {
            let r = Regex::new(&entry[0]);
            if let Ok(re) = r {
                self.regex_internal.push((re, entry[1].clone()));
            } else {
                eprintln!("An error occurred while initializing regex: {:?}", r);
                return false;
            }
        }

        true
    }
}

fn default_strip_ntfs() -> bool {
    true
}
