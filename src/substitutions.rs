use hashbrown::HashMap;
use lazy_static::lazy_static;
use regex::Regex;
use serde_derive::Deserialize;
use titlecase::titlecase;

const BAD_NTFS_CHARS: [char; 9] = ['/', '?', '<', '>', '\\', ':', '*', '|', '"'];

lazy_static! {
    static ref UPPER_REGEX: Regex = Regex::new("(\\s[–-]\\s)(\\p{Ll})").unwrap();
}

#[derive(Clone, Deserialize)]
pub struct Substitutions {
    /// This will indicate whether we should convert titles into correct title case.
    /// If unspecified the value will default to true.
    #[serde(default = "default_title_case")]
    pub convert_to_proper_title_case: bool,

    /// A list of regex substitutions to be applied when sanitizing a string.
    pub regular_expressions: Vec<[String; 2]>,

    /// A list of regex substitutions to be applied when sanitizing a string.
    pub strings: Vec<[String; 2]>,

    /// This will indicate whether we should strip invalid NTFS characters from the string.
    /// If unspecified the value will default to true.
    #[serde(default = "default_strip_ntfs")]
    pub strip_invalid_ntfs_chars: bool,

    /// This will fix proper case after dashes when sanitizing a string.
    #[serde(default = "default_fix_dashes")]
    pub fix_case_after_dashes: bool,

    /// This will be lazily initialized upon first use.
    #[serde(skip)]
    regex_internal: Vec<(Regex, String)>,
}

impl Substitutions {
    /// Apply any substitutions as specified by the substitution profile.
    ///
    /// # Arguments
    ///
    /// * `in_string` - The input string.
    ///
    /// # Returns
    ///
    /// A string with any specified substitutions applied.
    pub fn apply(&mut self, in_string: &str) -> String {
        if self.regex_internal.is_empty() {
            assert!(self.initialize_regex());
        }

        // Ensure that there are no stray tabs and spaces at the start
        // and end of the file name.
        let mut line = in_string.trim().to_string();

        // If the input string is empty, there is nothing to do here.
        if line.is_empty() {
            return String::new();
        }

        // This should be the last action to be performed.
        if self.convert_to_proper_title_case {
            line = titlecase(&line);
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

        if self.fix_case_after_dashes && line.contains('–') {
            let mut replacements = HashMap::new();

            for cap in UPPER_REGEX.captures_iter(&line) {
                let entire_match = cap.get(0).unwrap().as_str();
                let first_group = cap.get(1).unwrap().as_str();
                let second_group = cap.get(2).unwrap().as_str().to_uppercase();

                replacements.insert(
                    entire_match.to_string(),
                    format!("{first_group}{second_group}"),
                );
            }

            for (input, output) in replacements {
                line = line.replace(&input, &output);
            }
        }

        line
    }

    /// Initialize any regular expression objects as specified by the substitution profile.
    ///
    /// # Returns
    ///
    /// True if the regular expressions were successfully initialized, false otherwise.
    fn initialize_regex(&mut self) -> bool {
        for entry in &self.regular_expressions {
            let r = Regex::new(&entry[0]);
            if let Ok(re) = r {
                self.regex_internal.push((re, entry[1].clone()));
            } else {
                eprintln!("An error occurred while initializing regex: {r:?}");
                return false;
            }
        }

        true
    }
}

fn default_fix_dashes() -> bool {
    true
}

fn default_strip_ntfs() -> bool {
    true
}

fn default_title_case() -> bool {
    true
}
