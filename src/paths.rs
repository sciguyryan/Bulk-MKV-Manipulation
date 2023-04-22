use crate::utils;

use lazy_static::lazy_static;
use serde_derive::{Deserialize, Serialize};

#[derive(Default, Deserialize, Serialize)]
pub struct Paths {
    pub mkvtoolnix: String,
    pub temp: String,
    pub ffmpeg: String,
    pub mediainfo: String,
    pub log: String,
}

lazy_static! {
    pub static ref PATHS: Paths = Paths::from_file();
}

impl Paths {
    fn from_file() -> Paths {
        use std::{env, fs};

        let path = env::current_dir();
        assert!(path.is_ok(), "Failed to get the current directory.");

        let mut path = path.unwrap();
        path.push("paths.json");
        assert!(
            path.exists(),
            "The path to the paths JSON file was invalid."
        );

        let json = fs::read_to_string(path).expect("failed to open paths data file");
        let tools_result = serde_json::from_str::<Paths>(&json);
        assert!(
            tools_result.is_ok(),
            "Error attempting to parse JSON data: {:?}",
            tools_result.err()
        );

        let tools = tools_result.unwrap();
        assert!(tools.check_paths(), "One or more paths were invalid.");

        tools
    }

    pub fn check_paths(&self) -> bool {
        use std::path::Path;

        let mut check: bool = true;

        if !utils::dir_exists(&self.mkvtoolnix) {
            eprintln!("Failed to locate MkvToolNIX at {}", self.mkvtoolnix);
            check = false;
        } else {
            let path = Path::new(&self.mkvtoolnix);
            for exe in ["mkvextract.exe", "mkvmerge.exe"] {
                let temp = path.join(exe);
                if !temp.exists() {
                    eprintln!("Failed to MkvToolNix EXE {exe} at {temp:?}");
                    check = false;
                }
            }
        }

        if !utils::dir_exists(&self.temp) {
            eprintln!("Failed to locate temporary folder at {}", self.temp);
            check = false;
        }

        if !utils::file_exists(&self.ffmpeg) {
            eprintln!("Failed to locate FFMPEG at {}", self.ffmpeg);
            check = false;
        }

        if !utils::file_exists(&self.mediainfo) {
            eprintln!(
                "Failed to locate MediaInfo CLI executable at {}",
                self.mediainfo
            );
            check = false;
        }

        check
    }
}
