use lazy_static::lazy_static;
use regex::Regex;
use serde_derive::Deserialize;
use titlecase::titlecase;

const BAD_NTFS_CHARS: [char; 9] = ['/', '?', '<', '>', '\\', ':', '*', '|', '"'];

lazy_static! {
    static ref UpperRegex: Regex = Regex::new("(-\\s)(\\pL)").unwrap();
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
    pub fn apply(&mut self, s: &str) -> String {
        if self.regex_internal.is_empty() {
            assert!(self.initialize_regex());
        }

        // Ensure that there are no stray tabs and spaces at the start
        // and end of the file name.
        let mut line = s.trim().to_string();

        // If the input string is empty, there is nothing to do here.
        if line.is_empty() {
            return String::new();
        }

        if self.strip_invalid_ntfs_chars {
            line = line.replace(&BAD_NTFS_CHARS[..], "");
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

        if self.fix_case_after_dashes && line.contains('–') {
            let mut replacements: Vec<(String, String)> = Vec::new();

            UpperRegex.captures(&line).into_iter().for_each(|cap| {
                let entire_segment = cap.get(0).unwrap().as_str();
                let first_seg = cap.get(1).unwrap().as_str();
                let second_seg = cap.get(2).unwrap().as_str();
                let replacement = format!("{}{}", first_seg, second_seg.to_uppercase());

                replacements.push((entire_segment.to_string(), replacement))
            });

            for (input, output) in replacements {
                line = line.replace(&input, &output);
            }
        }

        line
    }

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
